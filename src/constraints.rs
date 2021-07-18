use crate::base::{Solution, State};

pub trait Constraint<S>
where
    S: Solution,
{
    fn penalty(&mut self, solution: &S) -> f64;
}

pub trait ConstraintsAggr<S>
where
    S: Solution,
{
    fn penalty(&mut self, solution: &mut S) {
        if solution.get_info(State::Current).check_penalty {
            let penalty = self.calculate_penalty(solution);
            solution.set_info(penalty, false, false);
        }
    }

    fn calculate_penalty(&mut self, solution: &mut S) -> f64;
}

pub struct WeightedConstraints<'a, S> {
    constraints: Vec<Box<dyn Constraint<S> + 'a>>,
    weights: &'a [f64],
}

impl<'a, S> WeightedConstraints<'a, S> {
    pub fn new(constraints: Vec<Box<dyn Constraint<S> + 'a>>, weights: &'a [f64]) -> Self {
        if constraints.len() != weights.len() {
            panic!("Length of constraints and weights needs to be the same");
        }
        Self {
            constraints,
            weights,
        }
    }
}

impl<S> ConstraintsAggr<S> for WeightedConstraints<'_, S>
where
    S: Solution,
{
    fn calculate_penalty(&mut self, solution: &mut S) -> f64 {
        let mut penalty: f64 = 0.0;
        for i in 0..self.constraints.len() {
            penalty += self.weights[i] * self.constraints[i].penalty(solution);
        }
        penalty
    }
}
