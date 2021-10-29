use optima_rust::genetic::{
    evaluator::DefaultEvaluator, Crosser, GeneticAlgorithm, Mutator, Speciment,
};
use rand::{
    prelude::{SliceRandom, ThreadRng},
    thread_rng, Rng,
};

fn main() {
    let mut crosser = KnapsackCrosser::default();
    let mut mutator = KnapsackMutator::default();
    let mut evaluator = DefaultEvaluator::new(300);
    let mut genetic = GeneticAlgorithm::new(&mut crosser, &mut mutator, &mut evaluator);

    let population_size = 100;
    let values = vec![5.0, 6.0, 12.0, 4.0, 33.5];
    let weights = vec![2.0, 3.0, 1.0, 9.0, 20.0];
    let capacity: f64 = 23.0;
    let cycles = 200;
    let mut population = Vec::with_capacity(population_size);

    let mut rng = thread_rng();
    for _ in 0..population_size {
        let mut picked_items = Vec::with_capacity(values.len());
        for _ in 0..values.len() {
            picked_items.push(rng.gen::<f32>() > 0.5);
        }
        population.push(KnapsackSpeciment::new(
            &values,
            &weights,
            picked_items,
            &capacity,
        ));
    }

    let best = genetic.evolve(&mut population, cycles, 4);

    println!("=======================================================");
    println!("==== Knapsack problem solved via genetic algorithm ====");
    println!("=======================================================");
    println!("Values: {:?}", values);
    println!("Weights: {:?}", weights);
    println!("Capacity: {:?}", capacity);
    println!("Population size: {:?}", population_size);
    println!("Cycles: {:?}", cycles);
    println!("Best solutions:");
    for item in best {
        println!("  {:?}, total value: {}", item.picked_items, item.score());
    }
}

#[derive(Clone)]
struct KnapsackSpeciment<'a> {
    values: &'a Vec<f64>,
    weights: &'a Vec<f64>,
    picked_items: Vec<bool>,
    capacity: &'a f64,
}

impl<'a> KnapsackSpeciment<'a> {
    fn new(
        values: &'a Vec<f64>,
        weights: &'a Vec<f64>,
        picked_items: Vec<bool>,
        capacity: &'a f64,
    ) -> Self {
        Self {
            values,
            weights,
            picked_items,
            capacity,
        }
    }
}

struct KnapsackCrosser {
    rng: ThreadRng,
}

impl Default for KnapsackCrosser {
    fn default() -> Self {
        Self { rng: thread_rng() }
    }
}

struct KnapsackMutator {
    rng: ThreadRng,
}

impl Default for KnapsackMutator {
    fn default() -> Self {
        Self { rng: thread_rng() }
    }
}

impl<'a> Speciment for KnapsackSpeciment<'a> {
    fn score(&self) -> f64 {
        let mut value: f64 = 0.0;
        let mut weight: f64 = 0.0;
        for i in 0..self.picked_items.len() {
            if self.picked_items[i] {
                value += self.values[i];
                weight += self.weights[i];
            }
        }
        if weight > *self.capacity {
            self.capacity - weight
        } else {
            value
        }
    }
}

impl Crosser<KnapsackSpeciment<'_>> for KnapsackCrosser {
    fn cross(&mut self, population: &mut Vec<KnapsackSpeciment>, best: &Vec<KnapsackSpeciment>) {
        for speciment in population {
            if self.rng.gen::<f32>() > 0.5 {
                let random_best_speciment = best.choose(&mut self.rng).unwrap();
                for i in 0..speciment.picked_items.len() {
                    if self.rng.gen::<f32>() > 0.5 {
                        speciment.picked_items[i] = random_best_speciment.picked_items[i];
                    }
                }
            }
        }
    }
}

impl Mutator<KnapsackSpeciment<'_>> for KnapsackMutator {
    fn mutate(&mut self, population: &mut Vec<KnapsackSpeciment>) {
        for speciment in population {
            if self.rng.gen::<f32>() > 0.5 {
                for i in 0..speciment.picked_items.len() {
                    if self.rng.gen::<f32>() > 0.5 {
                        speciment.picked_items[i] = !speciment.picked_items[i];
                    }
                }
            }
        }
    }
}
