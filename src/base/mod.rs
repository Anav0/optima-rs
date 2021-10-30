pub use self::criterion::Criterion;
mod criterion;

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

pub trait Solution {
    fn get_eval(&self) -> &Evaluation;
    fn get_eval_mut(&mut self) -> &mut Evaluation;
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum State {
    Best,
    Current,
    BeforeChange,
}

pub trait OptAlgorithm<'a, S>
where
    S: Solution,
    S: Clone,
{
    fn solve(&mut self, solution: S, criterion: &mut Criterion<S>, change: &dyn Fn(&mut S)) -> S;
}
