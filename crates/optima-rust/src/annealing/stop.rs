use std::fmt::Display;

pub trait StopCriteria: Clone + Display {
    fn should_stop(&mut self, value: f64) -> bool;
    fn reset(&mut self);
}
#[derive(Clone, Copy)]
pub struct MaxSteps {
    max_steps: usize,
    steps: usize,
}
impl MaxSteps {
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            steps: 0,
        }
    }
}
impl Display for MaxSteps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Max steps: {}", self.max_steps)
    }
}
impl StopCriteria for MaxSteps {
    fn should_stop(&mut self, _value: f64) -> bool {
        if self.steps > self.max_steps {
            return true;
        }
        self.steps += 1;
        false
    }

    fn reset(&mut self) {
        self.steps = 0;
    }
}

#[derive(Clone, Copy)]
pub struct NotGettingBetter {
    max_steps: u64,
    best_value: f64,
    found_at: u64,
    steps: u64,
    not_getting_better: u64,
    is_minimization: bool,
}
impl NotGettingBetter {
    pub fn new(max_steps: u64, not_getting_better: u64, is_minimization: bool) -> Self {
        let best_value = match is_minimization {
            true => f64::MAX,
            false => f64::MIN,
        };
        Self {
            steps: 0,
            found_at: 0,
            best_value,
            max_steps,
            not_getting_better,
            is_minimization,
        }
    }
}
impl Display for NotGettingBetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stop if not better for: {}\nMax steps: {}",
            self.not_getting_better, self.max_steps
        )
    }
}
impl StopCriteria for NotGettingBetter {
    fn should_stop(&mut self, value: f64) -> bool {
        self.steps += 1;
        if self.steps > self.max_steps {
            return true;
        };

        let is_better = match self.is_minimization {
            true => value < self.best_value,
            false => value > self.best_value,
        };

        if is_better {
            self.best_value = value;
            self.found_at = self.steps;
        }

        if self.steps - self.found_at >= self.not_getting_better {
            return true;
        };

        return false;
    }

    fn reset(&mut self) {
        self.steps = 0;
        self.found_at = 0;
        let best_value = match self.is_minimization {
            true => f64::MAX,
            false => f64::MIN,
        };
        self.best_value = best_value;
    }
}

#[cfg(test)]
mod tests {
    use crate::annealing::stop::StopCriteria;

    use super::NotGettingBetter;

    #[test]
    fn not_getting_better_should_stop_stops_after_max() {
        let max = 100;
        let not_getting_better = 10;
        let mut should_stop = NotGettingBetter::new(max, not_getting_better, false);

        let mut counter = 0;
        let mut value = 0.0;

        while !should_stop.should_stop(value) {
            value += 1.0;
            counter += 1;
        }

        assert_eq!(counter, max);
    }

    #[test]
    fn not_getting_better_should_stop_stops_if_not_better() {
        let max = 100;
        let not_getting_better = 10;
        let mut should_stop = NotGettingBetter::new(max, not_getting_better, false);

        let mut counter = 0;
        let value = 0.0;

        while !should_stop.should_stop(value) {
            counter += 1;
        }

        assert_eq!(counter, not_getting_better);
    }
    #[test]
    fn not_getting_better_reset_resets() {
        let max = 100;
        let not_getting_better = 10;
        let mut should_stop = NotGettingBetter::new(max, not_getting_better, false);

        let mut value = 0.0;

        while !should_stop.should_stop(value) {
            value += 1.0;
        }

        should_stop.reset();

        assert_eq!(should_stop.found_at, 0);
        assert_eq!(should_stop.best_value, f64::MIN);
        assert_eq!(should_stop.max_steps, max);
        assert_eq!(should_stop.not_getting_better, not_getting_better);
        assert_eq!(should_stop.steps, 0);
    }

    #[test]
    fn not_getting_better_initializes_best_value_correctly() {
        let max = 100;
        let not_getting_better = 10;
        let mut should_stop = NotGettingBetter::new(max, not_getting_better, false);
        assert_eq!(should_stop.best_value, f64::MIN);
        should_stop = NotGettingBetter::new(max, not_getting_better, true);
        assert_eq!(should_stop.best_value, f64::MAX);
    }
}
