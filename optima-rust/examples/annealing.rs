use std::thread::Thread;

use optima_rust::{
    analysis::{AsCsvRow, CsvSaver},
    annealing::{coolers::QuadraticCooler, stop::MaxSteps, SimulatedAnnealing},
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
    },
};
use rand::{prelude::ThreadRng, random, thread_rng, Rng};

pub type CrossFn<S> = dyn Fn(&S, &S, &mut ThreadRng) -> Vec<S>;

#[solution_attr]
#[derive(Clone, Debug, DerivedSolution)]
pub struct KnapsackSolution {
    pub picked_items: Vec<bool>,
}

impl KnapsackSolution {
    pub fn new(picked_items: Vec<bool>) -> Self {
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
    pub fn random_init(_id: u32, num_items: usize) -> Self {
        let mut picked_items = Vec::with_capacity(num_items);
        for _ in 0..picked_items.capacity() {
            picked_items.push(random::<bool>());
        }
        Self {
            picked_items,
            eval: Evaluation::default(),
        }
    }
}

pub fn value(problem: &KnapsackProblem, current: &KnapsackSolution) -> f64 {
    let mut total_value = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_value += bool_as_number as f64 * problem.values[i];
    }
    total_value
}

pub fn penalty(problem: &KnapsackProblem, current: &KnapsackSolution) -> f64 {
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

fn change_solution(
    solution: &mut KnapsackSolution,
    _problem: &KnapsackProblem,
    rng: &mut ThreadRng,
) {
    let random_index: usize = rng.gen_range(0..solution.picked_items.len());
    solution.picked_items[random_index] = !solution.picked_items[random_index];
}

#[derive(Clone, Copy)]
pub struct KnapsackProblem<'a> {
    id: u32,
    pub weights: &'a Vec<f64>,
    pub values: &'a Vec<f64>,
    pub capacity: f64,
}
impl<'a> KnapsackProblem<'a> {
    pub fn new(id: u32, weights: &'a Vec<f64>, values: &'a Vec<f64>, capacity: f64) -> Self {
        Self {
            id,
            weights,
            values,
            capacity,
        }
    }
}
impl<'a> Problem for KnapsackProblem<'a> {}
impl AsCsvRow for KnapsackSolution {
    fn as_row(&self, i: usize) -> String {
        format!("{},{}", i, self.get_value())
    }
}

fn main() {
    let weights = vec![1.0, 2.0, 3.0, 8.0, 12.0, 20.0, 30.0];
    let values = vec![4.0, 5.0, 1.0, 2.0, 8.0, 5.0, 6.0];
    let capacity = 6.0;

    let initial_solution = KnapsackSolution::random_init(0, weights.len());
    let problem1 = KnapsackProblem::new(0, &weights, &values, capacity);
    let problem2 = KnapsackProblem::new(1, &weights, &values, capacity);

    let mut criterion = Criterion::new(&penalty, &value, false);
    let cooler = QuadraticCooler::new(1000.0, 0.997);
    let max_steps = MaxSteps::new(20000);

    let header = String::from("iter,value");
    let mut prev_problem_id = u32::MAX;
    let mut csv = CsvSaver::new(String::from("./0.csv"), header);
    let mut insight = move |_: u32,
                            problem: &KnapsackProblem,
                            best: &KnapsackSolution,
                            _: &KnapsackSolution,
                            last_call: bool| {
        if last_call {
            csv.flush();
            return;
        }

        if problem.id != prev_problem_id {
            csv.flush();
            let file = format!("./problem{}.csv", problem.id);
            csv.reset(file, None);
        }
        prev_problem_id = problem.id;
        csv.save_element(best);
    };

    let mut annealing =
        SimulatedAnnealing::new(&initial_solution, max_steps, cooler, &change_solution);

    annealing.register_insight(&mut insight);

    let mut solutions = vec![];

    solutions.push(annealing.solve(problem1, &mut criterion));
    println!();
    solutions.push(annealing.solve(problem2, &mut criterion));
}
