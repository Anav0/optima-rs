use rand::{prelude::ThreadRng, thread_rng};

pub mod crossover;

use crate::{
    analysis::Saver,
    base::{Criterion, OptAlgorithm, Problem, Solution},
};

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
    pub breed: &'a dyn Fn(u32, &Vec<S>, &mut ThreadRng) -> [S; 2],
    pub generations: u32,
    initial_population: Vec<S>,
}

impl<'a, S> GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub fn new(
        population: Vec<S>,
        mutate: &'a dyn Fn(&mut S),
        breed: &'a dyn Fn(u32, &Vec<S>, &mut ThreadRng) -> [S; 2],
        generations: u32,
        print_rate: Option<u32>,
    ) -> Self {
        Self {
            generations,
            initial_population: population.clone(),
            population,
            mutate,
            breed,
            print_rate,
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
            if self.print_rate.is_some() && generation % self.print_rate.unwrap() == 0 {
                println!("Generation {}", generation);
            }
        }

        self.evaluate_population(&problem, criterion);
        self.population
            .sort_by(|a, b| b.get_value().partial_cmp(&a.get_value()).unwrap());

        self.population[0].clone()
    }

    fn add_saver(&mut self, _saver: &mut dyn Saver<S>) {
        todo!()
    }

    fn clear_savers(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        self.population = self.initial_population.clone();
    }
}
