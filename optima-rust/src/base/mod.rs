use std::{collections::HashMap, fmt::Display, hash::Hash};

use crate::{
    annealing::{coolers::Cooler, stop::StopCriteria, ChangeFn, SimulatedAnnealing},
    genetic::{ChangePopFn, GeneticAlgorithm, SelectionFn},
};

pub use self::criterion::Criterion;
mod criterion;

pub use optima_macros::{solution_attr, DerivedSolution};

#[derive(Clone, Copy, Debug)]
pub struct Evaluation {
    pub value: f64,
    pub is_feasible: bool,
}

impl Default for Evaluation {
    fn default() -> Self {
        Self {
            value: f64::NAN,
            is_feasible: false,
        }
    }
}

pub trait OptAlgorithm<'a, P, S>: Display
where
    S: Solution,
    P: Problem,
{
    fn solve(&mut self, problem: P, criterion: &mut Criterion<P, S>) -> Vec<S>;
    fn reset(&mut self);
}

pub trait Solution: Clone {
    fn get_value(&self) -> f64;
    fn get_eval(&self) -> &Evaluation;
    fn get_eval_mut(&mut self) -> &mut Evaluation;
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum State {
    Best,
    Current,
    BeforeChange,
}

pub struct Solver<'a, P, S>
where
    S: Solution,
    P: Problem,
{
    problems_soo_far: Vec<u32>,
    registry: HashMap<u32, (&'a P, Vec<Criterion<'a, P, S>>)>,
    algorithms: HashMap<u32, Vec<Box<dyn OptAlgorithm<'a, P, S> + 'a>>>,
}
impl<'a, P, S> Solver<'a, P, S>
where
    S: Solution,
    P: Problem,
{
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            problems_soo_far: vec![],
            algorithms: HashMap::new(),
        }
    }

    pub fn solve(&mut self, problems: &[&'a P]) -> &mut Self {
        self.problems_soo_far.clear();
        for problem in problems {
            if self.registry.contains_key(&problem.get_id()) {
                panic!("Cannot add two problems with same id");
            }

            self.problems_soo_far.push(problem.get_id());
            self.registry.insert(problem.get_id(), (problem, vec![]));
            self.algorithms.insert(problem.get_id(), vec![]);
        }

        self
    }

    pub fn use_criteria(&mut self, criterion: Criterion<'a, P, S>) -> &mut Self {
        for id in &self.problems_soo_far {
            self.registry.get_mut(id).unwrap().1.push(criterion.clone());
        }
        self
    }

    pub fn with_annealing<C: Cooler + 'a, SC: StopCriteria + 'a>(
        &mut self,
        initial_sol: &'a S,
        cooler: C,
        stop_criteria: SC,
        change_sol: &'a ChangeFn<S, P>,
    ) -> &mut Self {
        for id in &self.problems_soo_far {
            let annealing = SimulatedAnnealing::new(
                initial_sol,
                stop_criteria.clone(),
                cooler.clone(),
                change_sol,
            );
            self.algorithms
                .get_mut(id)
                .unwrap()
                .push(Box::new(annealing));
        }
        self
    }

    pub fn with_genetic(
        &mut self,
        population_cap: usize,
        population: Vec<S>,
        select: &'a SelectionFn<S>,
        change: &'a ChangePopFn<S>,
        generations: u32,
    ) -> &mut Self {
        for id in &self.problems_soo_far {
            let genetic =
                //TODO: get rid of clone()
                GeneticAlgorithm::new(population_cap,population.clone(), change, select, generations);
            self.algorithms.get_mut(id).unwrap().push(Box::new(genetic));
        }
        self
    }

    pub fn run(&mut self) -> Vec<SolverResult<P, S>> {
        let mut results: Vec<SolverResult<P, S>> = Vec::new();
        for (problem_id, (problem, criterions)) in self.registry.iter_mut() {
            let algorithms = self.algorithms.get_mut(&problem_id).unwrap();
            for alg in algorithms.iter_mut() {
                for criterion in criterions.iter_mut() {
                    let result = SolverResult {
                        algorithm: alg.to_string(),
                        problem: *problem,
                        solutions: alg.solve(**problem, criterion),
                    };
                    results.push(result)
                }
            }
        }
        results
    }
}
pub trait Problem: Clone + Copy {
    fn get_id(&self) -> u32;
}

pub struct SolverResult<'a, P: Problem, S: Solution> {
    pub algorithm: String,
    pub solutions: Vec<S>,
    pub problem: &'a P,
}
