use crate::{
    base::{Solution, State},
    constraints::ConstraintsAggr,
    objectives::ObjectivesAggr,
};

pub struct Criterion<'a, S> {
    constraints: &'a mut dyn ConstraintsAggr<S>,
    objectives: &'a mut dyn ObjectivesAggr<S>,
    is_minimalization_problem: bool,
}

impl<'a, S> Criterion<'a, S>
where
    S: Solution,
{
    pub fn new(
        constraints: &'a mut dyn ConstraintsAggr<S>,
        objectives: &'a mut dyn ObjectivesAggr<S>,
        is_minimalization_problem: bool,
    ) -> Self {
        Self {
            constraints,
            objectives,
            is_minimalization_problem,
        }
    }

    fn evaluate(&mut self, solution: &mut S) {
        self.constraints.penalty(solution);
        self.objectives.value(solution);
    }

    pub fn initial(&mut self, solution: &mut S) {
        self.evaluate(solution);
        solution.replace_before();
        solution.replace_best();
    }

    pub fn is_first_better(&mut self, solution: &mut S, first: State, second: State) -> bool {
        self.evaluate(solution);
        let first_info = solution.get_info(first);
        let second_info = solution.get_info(second);

        if first_info.is_feasible && !second_info.is_feasible {
            return true;
        };

        if first_info.is_feasible {
            return match self.is_minimalization_problem {
                true => first_info.value < second_info.value,
                false => first_info.value > second_info.value,
            };
        };

        if !second_info.is_feasible {
            return first_info.value < second_info.value;
        };

        false
    }
}
