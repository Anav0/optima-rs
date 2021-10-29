use crate::base::InfoHolder;

pub trait StopCriteria<T> {
    fn should_stop(&mut self, value: f64) -> bool;
}

pub struct NotGettingBetter {
    max_steps: u64,
    best_value: f64,
    found_at: u64,
    steps: u64,
    not_getting_better: u64,
    is_minimazation: bool,
}
impl NotGettingBetter {
    pub fn new(max_steps: u64, not_getting_better: u64, is_minimazation: bool) -> Self {
        let best_value = match is_minimazation {
            true => f64::MAX,
            false => f64::MIN,
        };
        Self {
            steps: 0,
            found_at: 0,
            best_value,
            max_steps,
            not_getting_better,
            is_minimazation,
        }
    }
}
impl<T> StopCriteria<T> for NotGettingBetter
where
    T: Clone,
    T: InfoHolder,
{
    fn should_stop(&mut self, value: f64) -> bool {
        self.steps += 1;
        if self.steps > self.max_steps {
            return true;
        };

        let is_better = match self.is_minimazation {
            true => value > self.best_value,
            false => value < self.best_value,
        };

        if is_better {
            self.best_value = value;
            self.found_at = self.steps;
        }

        if self.steps - self.found_at > self.not_getting_better {
            return true;
        };

        return false;
    }
}