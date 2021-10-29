use crate::{
    base::{InfoHolder, Solution},
    criterion::Criterion,
};

pub trait OptAlghorithm<'a, T>
where
    T: Clone,
    T: InfoHolder,
{
    fn solve(
        &mut self,
        solution: &'a mut Solution<T>,
        criterion: &mut Criterion<T>,
        change: &dyn Fn(&mut T),
    );
}
