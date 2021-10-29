use optima_rust::genetic::{
    evaluator::DefaultEvaluator, Crosser, GeneticAlgorithm, Mutator, Speciment,
};
use rand::prelude::*;
use std::fs;

fn main() {
    const POPULATION_SIZE: usize = 300;

    let original_bytes = fs::read("./examples/test_image.raw").expect("Failed to read test file");

    let mut crosser = DrawerCrosser::default();
    let mut mutator = DrawerMutator::new(256, 144);
    let mut evaluator = DefaultEvaluator::new(POPULATION_SIZE);
    let mut genetic = GeneticAlgorithm::new(&mut crosser, &mut mutator, &mut evaluator);

    let mut population = Vec::with_capacity(POPULATION_SIZE);

    for _ in 0..POPULATION_SIZE {
        population.push(DrawerSpeciment::new(
            vec![255; original_bytes.len()],
            &original_bytes,
        ));
    }

    let best = genetic.evolve(&mut population, 1000, 10);

    print_best(1000, &best);
}

fn print_best(i: u32, best: &Vec<DrawerSpeciment>) {
    fs::write(format!("./examples/out/{}_best.raw", i), &best[0].bytes)
        .expect(&format!("Failed to write best in cycle: {}", i));
}

#[derive(Clone)]
struct DrawerSpeciment<'a> {
    bytes: Vec<u8>,
    original_bytes: &'a Vec<u8>,
}

impl<'a> DrawerSpeciment<'a> {
    fn new(bytes: Vec<u8>, original_bytes: &'a Vec<u8>) -> Self {
        Self {
            bytes,
            original_bytes,
        }
    }
}

impl<'a> Speciment for DrawerSpeciment<'a> {
    fn score(&self) -> f64 {
        let mut value = 0.0;
        for i in 0..self.bytes.len() {
            value += match self.bytes[i] < self.original_bytes[i] {
                true => (self.original_bytes[i] - self.bytes[i]) as f64,
                false => (self.bytes[i] - self.original_bytes[i]) as f64,
            };
        }
        value
    }
}

struct DrawerMutator {
    rng: ThreadRng,
    HEIGHT: u32,
    WIDTH: u32,
}

impl DrawerMutator {
    fn new(HEIGHT: u32, WIDTH: u32) -> Self {
        Self {
            rng: thread_rng(),
            HEIGHT,
            WIDTH,
        }
    }
}

impl<'a> Mutator<DrawerSpeciment<'a>> for DrawerMutator {
    fn mutate(&mut self, population: &mut Vec<DrawerSpeciment>) {
        for speciment in population {
            let end_x: u32 = self.rng.gen_range(1..self.WIDTH - 1);
            let end_y: u32 = self.rng.gen_range(1..self.HEIGHT - 1);

            let start_x: u32 = self.rng.gen_range(1..self.WIDTH - end_x);
            let start_y: u32 = self.rng.gen_range(1..self.HEIGHT - end_y);

            let color: u8 = self.rng.gen_range(0..255);

            for x in end_y..end_y + start_y {
                for y in end_x..end_x + start_x {
                    let index = (x * self.WIDTH + y) as usize;
                    speciment.bytes[index] = (speciment.bytes[index] / 2) + (color / 2);
                }
            }
        }
    }
}

struct DrawerCrosser {
    rng: ThreadRng,
}

impl Default for DrawerCrosser {
    fn default() -> Self {
        Self { rng: thread_rng() }
    }
}

impl Crosser<DrawerSpeciment<'_>> for DrawerCrosser {
    fn cross(&mut self, population: &mut Vec<DrawerSpeciment>, best: &Vec<DrawerSpeciment>) {
        for speciment in population {
            let random_best_speciment = best.choose(&mut self.rng).unwrap();

            for i in 0..speciment.bytes.len() {
                speciment.bytes[i] = random_best_speciment.bytes[i];
            }
        }
    }
}
