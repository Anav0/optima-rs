use optima_rust::{
    analysis::AsCsvRow,
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
    },
    genetic::{selection::tournament, GeneticAlgorithm},
};
use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    random, Rng,
};

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

fn change_population(population: &mut Vec<KnapsackSolution>, rng: &mut ThreadRng) {
    let uniform = Uniform::new(0, population.len());
    let mut children = Vec::with_capacity(population.len());

    while children.len() < population.len() {
        let father = &population[uniform.sample(rng)];
        let mather = &population[uniform.sample(rng)];

        let cross_point = rng.gen_range(1..father.picked_items.len());

        let from_father = father.picked_items[..cross_point].to_vec();
        let from_mather = mather.picked_items[cross_point..].to_vec();

        let child_a_picked_items = vec![from_father, from_mather].concat();

        let from_mather = mather.picked_items[..cross_point].to_vec();
        let from_father = father.picked_items[cross_point..].to_vec();

        let child_b_picked_items = vec![from_mather, from_father].concat();

        let child_a = KnapsackSolution::new(child_a_picked_items);
        let child_b = KnapsackSolution::new(child_b_picked_items);

        children.push(child_a);
        children.push(child_b);
    }

    let mutate_rate = 0.5;
    for i in 0..population.len() {
        let child = &mut children[i];
        for j in 0..child.picked_items.len() {
            if random::<f64>() < mutate_rate {
                child.picked_items[j] = !child.picked_items[j];
            }
        }
        population[i] = child.clone();
    }
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
impl AsCsvRow for KnapsackSolution {
    fn as_row(&self, i: usize) -> String {
        format!("{},{}", i, self.get_value())
    }
}

fn main() {
    let weights = vec![1.0, 2.0, 3.0, 8.0, 12.0, 20.0, 30.0];
    let values = vec![4.0, 5.0, 1.0, 2.0, 8.0, 5.0, 6.0];
    let capacity = 6.0;

    let problem = KnapsackProblem::new(0, &weights, &values, capacity);

    let mut criterion = Criterion::new(&penalty, &value, false);

    let pop_size = 20;

    let population = random_population(pop_size, values.len());

    let mut genetic = GeneticAlgorithm::new(
        pop_size,
        population,
        &change_population,
        &|_: usize, population: &Vec<KnapsackSolution>, rng: &mut ThreadRng| {
            tournament(4, population, false, rng, 0)
        },
        100,
        None,
    );

    let solutions = genetic.solve(problem, &mut criterion);

    for sol in solutions {
        print!("{} ", sol.get_value());
    }
}
