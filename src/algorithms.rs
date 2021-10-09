use crate::{
    base::{Solution, StateChanger},
    criterion::Criterion,
};

pub trait OptAlghorithm<'a, S>
where
    S: Solution,
{
    type SolutionType;

    fn solve(
        &mut self,
        solution: &mut S,
        criterion: &mut Criterion<S>,
        mover: &'a mut dyn StateChanger<SolutionType = Self::SolutionType>,
    );
}
