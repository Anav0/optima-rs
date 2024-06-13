use std::{fmt::Display, hash::Hash};

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

pub trait Problem {}

// pub type AnnealingInsightFn<S, P, C> = dyn FnMut(&C, u32, &P, &S, &S, bool);
// pub type SwarmInsightFn              = dyn FnMut(&FnProblem<RangeInclusive<f64>>, &Vec<Particle>, usize) -> bool;
// pub type GeneticInsightFn<S>         = dyn Fn(u32, &Vec<S>);

