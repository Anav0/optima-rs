use crate::analysis::Saver;

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

pub trait OptAlgorithm<'a, S>
where
    S: Solution,
    S: Clone,
{
    fn solve(&mut self, criterion: &mut Criterion<S>) -> S;
    fn add_saver(&mut self, saver: &'a mut dyn Saver<S>);
    fn clear_savers(&mut self);
}

pub trait Solution {
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
