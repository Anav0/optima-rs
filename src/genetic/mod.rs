use rand::{prelude::ThreadRng, thread_rng};

pub mod crossover;

use crate::base::{Criterion, OptAlgorithm, Solution};

pub trait Crosser<S: Solution> {
    fn cross(&mut self, solution: &mut S);
}

pub trait Mutator<S: Solution> {
    fn mutate(&mut self, solution: &mut S);
}

pub struct GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub print_rate: Option<u32>,
    pub population: Vec<S>,
    pub mutate: &'a dyn Fn(&mut S),
    pub breed: &'a dyn Fn(&Vec<S>, &mut ThreadRng) -> [S; 2],
    pub generations: u32,
}

impl<'a, S> GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub fn new(
        population: Vec<S>,
        mutate: &'a dyn Fn(&mut S),
        breed: &'a dyn Fn(&Vec<S>, &mut ThreadRng) -> [S; 2],
        cycles: u32,
        print_rate: Option<u32>,
    ) -> Self {
        Self {
            generations: cycles,
            population,
            mutate,
            breed,
            print_rate,
        }
    }
    fn evaluate_population(&mut self, criterion: &Criterion<S>) {
        for specimen in self.population.iter_mut() {
            criterion.evaluate(specimen);
        }
    }
}

impl<S> OptAlgorithm<'_, S> for GeneticAlgorithm<'_, S>
where
    S: Solution + Clone,
{
    fn solve(&mut self, criterion: &mut crate::base::Criterion<S>) -> S {
        let mutate = self.mutate;
        let mut rng = thread_rng();

        for generation in 0..self.generations {
            let mut new_pop: Vec<S> = Vec::with_capacity(self.population.len());

            self.evaluate_population(criterion);

            while new_pop.len() < new_pop.capacity() {
                let children = (self.breed)(&self.population, &mut rng);
                for mut child in children {
                    mutate(&mut child);
                    new_pop.push(child);
                }
            }
            self.population = new_pop;
            if self.print_rate.is_some() && generation % self.print_rate.unwrap() == 0 {
                println!("Generation {}", generation);
            }
        }

        self.evaluate_population(criterion);
        self.population
            .sort_by(|a, b| b.get_eval().value.partial_cmp(&a.get_eval().value).unwrap());

        self.population[0].clone()
    }
}
