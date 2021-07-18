use crate::base::{Solution, State};

pub trait Objective {
    type SolutionType;

    fn value(&mut self, solution: &mut Self::SolutionType) -> f64;
}
pub trait ObjectivesAggr<S>
where
    S: Solution,
{
    fn value(&mut self, solution: &mut S) {
        if !solution.get_info(State::Current).check_penalty
            && solution.get_info(State::Current).value == 0.0
        {
            let value = self.calculate_value(solution);
            solution.set_info(value, true, false)
        }
    }
    fn calculate_value(&mut self, solution: &mut S) -> f64;
}
pub struct WeightedObjectives<'a, S> {
    objectives: Vec<Box<dyn Objective<SolutionType = S> + 'a>>,
    weights: &'a [f64],
}

impl<S> ObjectivesAggr<S> for WeightedObjectives<'_, S>
where
    S: Solution,
{
    fn calculate_value(&mut self, solution: &mut S) -> f64 {
        let mut value: f64 = 0.0;
        for i in 0..self.objectives.len() {
            value += self.weights[i] * self.objectives[i].value(solution);
        }
        value
    }
}

impl<'a, S> WeightedObjectives<'a, S>
where
    S: Solution,
{
    pub fn new(
        objectives: Vec<Box<dyn Objective<SolutionType = S> + 'a>>,
        weights: &'a [f64],
    ) -> Self {
        if objectives.len() != weights.len() {
            panic!("Length of objectives and weights needs to be the same");
        }
        Self {
            objectives,
            weights,
        }
    }
}
