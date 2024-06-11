use std::{thread::Thread, time::Instant};

mod misc;

use chrono::{DateTime, Local};
use optima_rust::{
    analysis::{AsCsvRow, CsvSaver},
    annealing::{
        coolers::{Cooler, QuadraticCooler},
        stop::MaxSteps,
        SimulatedAnnealing,
    },
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
    },
};
use rand::{prelude::ThreadRng, random, thread_rng, Rng};

use crate::misc::KnapsackInstanceFactory;

pub type CrossFn<S> = dyn Fn(&S, &S, &mut ThreadRng) -> Vec<S>;

#[solution_attr]
#[derive(Clone, Copy, Debug)]
pub struct KnapsackSolution<const LENGTH: usize> {
    pub picked_items: [bool; LENGTH],
}
//TODO: Make generics work in our macro
impl<const LENGTH: usize> Solution for KnapsackSolution<LENGTH> {
    fn get_value(&self) -> f64 {
        self.eval.value
    }

    fn get_eval(&self) -> &Evaluation {
        &self.eval
    }

    fn get_eval_mut(&mut self) -> &mut Evaluation {
        &mut self.eval
    }
}

impl<const LENGTH: usize> KnapsackSolution<LENGTH> {
    pub fn new(picked_items: [bool; LENGTH]) -> Self {
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
    pub fn random_init(problem: &KnapsackProblem<LENGTH>) -> Self {
        let mut picked_items = [false; LENGTH];
        for i in 0..picked_items.len() {
            picked_items[i] = random::<bool>();
        }
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
}

pub fn value<const LENGTH: usize>(
    problem: &KnapsackProblem<LENGTH>,
    current: &KnapsackSolution<LENGTH>,
) -> f64 {
    let mut total_value = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_value += bool_as_number as f64 * problem.values[i];
    }
    total_value
}

pub fn penalty<const LENGTH: usize>(
    problem: &KnapsackProblem<LENGTH>,
    current: &KnapsackSolution<LENGTH>,
) -> f64 {
    let mut total_weight = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_weight += bool_as_number as f64 * problem.weights[i];
    }
    if total_weight > problem.capacity {
        problem.capacity - total_weight
    } else {
        0.0
    }
}

fn change_solution<const LENGTH: usize>(
    solution: &mut KnapsackSolution<LENGTH>,
    _problem: &KnapsackProblem<LENGTH>,
    rng: &mut ThreadRng,
) {
    let random_index: usize = rng.gen_range(0..solution.picked_items.len());
    solution.picked_items[random_index] = !solution.picked_items[random_index];
}

#[derive(Clone)]
pub struct KnapsackProblem<const LENGTH: usize> {
    name: String,
    pub weights: [f64; LENGTH],
    pub values: [f64; LENGTH],
    pub capacity: f64,
    pub instance: u32,
    pub run: u8,
}
impl<const LENGTH: usize> KnapsackProblem<LENGTH> {
    pub fn new(name: String, weights: [f64; LENGTH], values: [f64; LENGTH], capacity: f64) -> Self {
        Self {
            weights,
            values,
            capacity,
            name,
            instance: 0,
            run: 0,
        }
    }
}
impl<'a, const LENGTH: usize> Problem for KnapsackProblem<LENGTH> {}
impl<const LENGTH: usize> AsCsvRow for KnapsackSolution<LENGTH> {
    fn as_row(&self, i: usize) -> String {
        format!("{},{}", i, self.get_value())
    }
}

use misc::Generator::{SimilarWeight, StronglyCorrelated, Uncorrelated};

fn main() {
    const HOW_MANY_RUNS: u8 = 100;
    let mut factory = KnapsackInstanceFactory::new(25, 250.0, 2);

    let mut problems = factory
        .generate_distribution_problem(Uncorrelated, 3.0)
        .generate_distribution_problem(StronglyCorrelated, 3.0)
        .collect();

    let problems_len = problems.len();

    let local: DateTime<Local> = Local::now();
    let time_str = local.format("%Y-%m-%d_%H-%M-%S");

    let header = String::from("Iter,Value,Temp,InstanceName,WhichInstance,WhichRun\n");

    let n = 20000;
    let mut criterion = Criterion::new(&penalty, &value, false);
    let cooler                              = QuadraticCooler::new(1000.0, 0.997);
    let max_steps                                  = MaxSteps::new(n);
    let mut csv = CsvSaver::new(format!("D:\\Projects\\optima-rust\\optima-rust\\csv\\{}.csv", time_str), header);
    let mut call_count = 0;
    let mut insight = move |cooler: &QuadraticCooler,
                            _: u32,
                            problem: &KnapsackProblem<25>,
                            best: &KnapsackSolution<25>,
                            _: &KnapsackSolution<25>,
                            last_call: bool| {
        let additional = format!(
            "{},{},{},{}",
            cooler.get_temp(),
            &problem.name,
            problem.instance,
            problem.run
        );
        csv.save_element(best, Some(&additional));
        call_count += 1;

        //TODO: oh my god
        if call_count == n * HOW_MANY_RUNS as usize {
            println!("FLUSH!");
            csv.flush();
        }
    };

    let mut which_instance = 0;
    for problem in &mut problems {
        problem.instance = which_instance;
        println!("\n{}\n", problem.name);

        let initial_solution = KnapsackSolution::random_init(&problem);
        let mut annealing = SimulatedAnnealing::new(&initial_solution, max_steps, cooler, &change_solution);

        annealing.register_insight(&mut insight);

        for run in 0..HOW_MANY_RUNS {
            problem.run = run;
            annealing.solve(problem.clone(), &mut criterion);
        }
        which_instance += 1;
    }
}
