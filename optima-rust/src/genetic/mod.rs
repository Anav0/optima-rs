use std::fmt::Display;

use rand::{prelude::ThreadRng, thread_rng};

pub mod selection;

use crate::base::{OptAlgorithm, Problem, Solution};

pub type SelectionFn<S> = dyn Fn(usize, &Vec<S>, &mut ThreadRng) -> Vec<S>;
pub type ChangePopFn<S> = dyn Fn(&mut Vec<S>, &mut ThreadRng);
pub type GeneticInsightFn<S> = dyn Fn(u32, &Vec<S>);

pub struct GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub population: Vec<S>,
    pub select: &'a SelectionFn<S>,
    pub change: &'a ChangePopFn<S>,
    pub generations: u32,
    initial_population: Vec<S>,
    population_cap: usize,
    insight: Option<&'a GeneticInsightFn<S>>,
}

impl<'a, S> GeneticAlgorithm<'a, S>
where
    S: Solution,
{
    pub fn new(
        population_cap: usize,
        population: Vec<S>,
        change: &'a ChangePopFn<S>,
        select: &'a SelectionFn<S>,
        generations: u32,
        insight: Option<&'a GeneticInsightFn<S>>,
    ) -> Self {
        Self {
            generations,
            initial_population: population.clone(),
            population,
            select,
            change,
            population_cap,
            insight,
        }
    }

    pub fn register_insight(&mut self, insight: &'a GeneticInsightFn<S>) {
        self.insight = Some(insight);
    }
}

impl<S, P> OptAlgorithm<'_, P, S> for GeneticAlgorithm<'_, S>
where
    S: Solution,
    P: Problem,
{
    fn solve(&mut self, problem: P, criterion: &mut crate::base::Criterion<P, S>) -> Vec<S> {
        let mut rng = thread_rng();

        for generation in 0..self.generations {
            //Select new population form the previous one
            self.population = (self.select)(self.population_cap, &self.population, &mut rng);

            (self.change)(&mut self.population, &mut rng);

            for specimen in self.population.iter_mut() {
                criterion.evaluate(&problem, specimen);
            }

            match self.insight {
                Some(f) => f(generation, &self.population),
                _ => {}
            }
        }

        for specimen in self.population.iter_mut() {
            criterion.evaluate(&problem, specimen);
        }
        self.population
            .sort_by(|a, b| b.get_value().partial_cmp(&a.get_value()).unwrap());

        self.population.clone()
    }

    fn reset(&mut self) {
        self.population = self.initial_population.clone();
    }
}

impl<'a, S: Solution> Display for GeneticAlgorithm<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Genetic algorithm\nInitial pop size: {}",
            self.initial_population.len()
        )
    }
}
