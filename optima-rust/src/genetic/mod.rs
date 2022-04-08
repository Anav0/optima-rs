use rand::{prelude::ThreadRng, thread_rng};

pub mod crossover;

use crate::base::{Criterion, OptAlgorithm, Problem, Solution};

pub trait Crosser<S: Solution> {
    fn cross(&mut self, solution: &mut S);
}

pub trait Mutator<S: Solution> {
    fn mutate(&mut self, solution: &mut S);
}

pub type MutationFn<S> = dyn Fn(&mut S);
pub type BreedingFn<S> = dyn Fn(u32, &Vec<S>, &mut ThreadRng) -> [S; 2];

pub struct GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub population: Vec<S>,
    pub mutate: &'a MutationFn<S>,
    pub breed: &'a BreedingFn<S>,
    pub generations: u32,
    initial_population: Vec<S>,
}

impl<'a, S> GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub fn new(
        population: Vec<S>,
        mutate: &'a MutationFn<S>,
        breed: &'a BreedingFn<S>,
        generations: u32,
    ) -> Self {
        Self {
            generations,
            initial_population: population.clone(),
            population,
            mutate,
            breed,
        }
    }
    fn evaluate_population<P: Problem>(&mut self, problem: &P, criterion: &Criterion<P, S>) {
        for specimen in self.population.iter_mut() {
            criterion.evaluate(problem, specimen);
        }
    }
}

impl<S, P> OptAlgorithm<'_, P, S> for GeneticAlgorithm<'_, S>
where
    S: Solution,
    P: Problem,
{
    fn solve(&mut self, problem: P, criterion: &mut crate::base::Criterion<P, S>) -> S {
        let mutate = self.mutate;
        let mut rng = thread_rng();

        let mut counter = 0;

        for generation in 0..self.generations {
            let mut new_pop: Vec<S> = Vec::with_capacity(self.population.len());

            self.evaluate_population(&problem, criterion);

            while new_pop.len() < new_pop.capacity() {
                let children = (self.breed)(counter, &self.population, &mut rng);
                counter += 2;
                for mut child in children {
                    mutate(&mut child);
                    new_pop.push(child);
                }
            }
            self.population = new_pop;
        }

        self.evaluate_population(&problem, criterion);
        self.population
            .sort_by(|a, b| b.get_value().partial_cmp(&a.get_value()).unwrap());

        self.population[0].clone()
    }

    fn reset(&mut self) {
        self.population = self.initial_population.clone();
    }
}
