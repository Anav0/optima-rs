use std::marker::PhantomData;

use crate::base::{InfoHolder, Solution, SolutionInfo, State};

pub struct Criterion<'a, T>
where
    T: Clone,
    T: InfoHolder,
{
    penalty: &'a dyn Fn(&T) -> f64,
    value: &'a dyn Fn(&T) -> f64,
    is_minimalization_problem: bool,
    phantom: PhantomData<T>,
}

impl<'a, T> Criterion<'a, T>
where
    T: Clone,
    T: InfoHolder,
{
    pub fn new(
        penalty: &'a dyn Fn(&T) -> f64,
        value: &'a dyn Fn(&T) -> f64,
        is_minimalization_problem: bool,
    ) -> Self {
        Self {
            penalty,
            value,
            is_minimalization_problem,
            phantom: PhantomData,
        }
    }

    pub fn is_first_better(
        &mut self,
        first_info: &SolutionInfo,
        second_info: &SolutionInfo,
    ) -> bool {
        if first_info.is_feasible && !second_info.is_feasible {
            return true;
        };

        if !first_info.is_feasible && second_info.is_feasible {
            return false;
        }

        return match self.is_minimalization_problem {
            true => first_info.value < second_info.value,
            false => first_info.value > second_info.value,
        };
    }

    pub fn evaluate(&self, solution: &mut Solution<T>) {
        let holder = solution.get_state_mut(State::Current);
        let info = holder.get_info();
        let mut value: f64 = info.value;
        let mut is_feasible: bool = info.is_feasible;
        if info.check_penalty {
            value = (self.penalty)(holder);
        }
        is_feasible = value == 0.0;
        if is_feasible {
            value = (self.value)(holder);
        }
        solution.set_state_info(State::Current, value, is_feasible, false);
    }
}