use optima_rust::{
    annealing::{
        coolers::QuadraticCooler,
        stop::{MaxSteps, NotGettingBetter},
    },
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
        Solver,
    },
    genetic::crossover::tournament,
};
use rand::{prelude::ThreadRng, random, thread_rng, Rng};

#[solution_attr]
#[derive(Clone, Debug, DerivedSolution)]
pub struct KnapsackSolution {
    pub picked_items: Vec<bool>,
}

impl KnapsackSolution {
    pub fn new(_id: u32, picked_items: Vec<bool>) -> Self {
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

fn change(solution: &mut KnapsackSolution, _problem: &KnapsackProblem) {
    let mut rng = thread_rng();
    let random_index: usize = rng.gen_range(0..solution.picked_items.len());
    solution.picked_items[random_index] = !solution.picked_items[random_index];
}

pub fn mutate(specimen: &mut KnapsackSolution, mutate_rate: f64) {
    for i in 0..specimen.picked_items.len() {
        if random::<f64>() < mutate_rate {
            specimen.picked_items[i] = !specimen.picked_items[i];
        }
    }
}

fn point_crossover(
    id: u32,
    father: &KnapsackSolution,
    mather: &KnapsackSolution,
    rng: &mut ThreadRng,
) -> [KnapsackSolution; 2] {
    let cross_point = rng.gen_range(0..father.picked_items.len()); //TODO: Guard from picking point at 0 or at max length

    let from_father = father.picked_items[..cross_point].to_vec();
    let from_mather = mather.picked_items[cross_point..].to_vec();

    let child_a_picked_items = vec![from_father, from_mather].concat();

    let from_mather = mather.picked_items[..cross_point].to_vec();
    let from_father = father.picked_items[cross_point..].to_vec();

    let child_b_picked_items = vec![from_mather, from_father].concat();

    let child_a = KnapsackSolution::new(id + 1, child_a_picked_items);
    let child_b = KnapsackSolution::new(id + 2, child_b_picked_items);

    [child_a, child_b]
}

fn tournament_wrapper(
    id: u32,
    population: &Vec<KnapsackSolution>,
    rng: &mut ThreadRng,
) -> [KnapsackSolution; 2] {
    tournament(id, population, false, &point_crossover, rng)
}

fn mutate_wrapper(solution: &mut KnapsackSolution) {
    mutate(solution, 0.25);
}

fn random_population(size: usize, num_items: usize) -> Vec<KnapsackSolution> {
    let mut population = Vec::with_capacity(size);
    for i in 0..size {
        let specimen = KnapsackSolution::random_init(i as u32, num_items);
        population.push(specimen);
    }

    population
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
impl<'a> Problem for KnapsackProblem<'a> {
    fn get_id(&self) -> u32 {
        self.id
    }
}

fn main() {
    let weights = vec![1.0, 2.0, 3.0, 8.0, 12.0, 20.0, 30.0];
    let values = vec![4.0, 5.0, 1.0, 2.0, 8.0, 5.0, 6.0];
    let capacity = 6.0;

    let initial_solution = KnapsackSolution::random_init(0, weights.len());
    let problem1 = KnapsackProblem::new(0, &weights, &values, capacity);
    let problem2 = KnapsackProblem::new(1, &weights, &values, capacity);
    let problem3 = KnapsackProblem::new(3, &weights, &values, capacity);

    let criterion = Criterion::new(&penalty, &value, false);
    let cooler = QuadraticCooler::new(1000.0, 0.997);
    let max_steps = MaxSteps::new(10000);
    let not_getting_better = NotGettingBetter::new(100000, 100, false);

    let population = random_population(100, weights.len());

    let best = Solver::new()
        .solve(&[&problem1, &problem2])
        .use_criteria(criterion.clone())
        .with_annealing(&initial_solution, cooler, max_steps, &change)
        .solve(&[&problem3])
        .use_criteria(criterion)
        .with_genetic(population, &mutate_wrapper, &tournament_wrapper, 100)
        .with_annealing(&initial_solution, cooler, not_getting_better, &change)
        .run();

    for sol in &best {
        println!("{:?}", sol);
    }
}
