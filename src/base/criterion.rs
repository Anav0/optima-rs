use crate::base::{Evaluation, Solution};

pub struct Criterion<'a, S>
where
    S: Solution,
{
    penalty: &'a dyn Fn(&S) -> f64,
    value: &'a dyn Fn(&S) -> f64,
    pub is_minimization: bool,
}

impl<'a, S> Criterion<'a, S>
where
    S: Solution,
{
    pub fn new(
        penalty: &'a dyn Fn(&S) -> f64,
        value: &'a dyn Fn(&S) -> f64,
        is_minimization: bool,
    ) -> Self {
        Self {
            penalty,
            value,
            is_minimization,
        }
    }

    pub fn is_first_better(&self, first: &Evaluation, second: &Evaluation) -> bool {
        if first.is_feasible && !second.is_feasible {
            return true;
        };

        if !first.is_feasible && second.is_feasible {
            return false;
        }

        // Lower penalty
        if !second.is_feasible {
            return first.value < second.value;
        }

        // Compare value according to problem type
        return match self.is_minimization {
            true => first.value < second.value,
            false => first.value > second.value,
        };
    }

    pub fn evaluate(&self, solution: &mut S) {
        let mut value = (self.penalty)(solution);
        let is_feasible = value == 0.0;
        if is_feasible {
            value = (self.value)(solution);
        }

        let mut eval = solution.get_eval_mut();
        eval.value = value;
        eval.is_feasible = is_feasible;
    }
}

#[cfg(test)]
mod tests {
    use crate::base::{Evaluation, Solution};

    use super::Criterion;

    #[derive(Clone)]
    struct TestSolution {
        eval: Evaluation,
    }
    impl Solution for TestSolution {
        fn get_eval(&self) -> &Evaluation {
            &self.eval
        }

        fn get_eval_mut(&mut self) -> &mut Evaluation {
            &mut self.eval
        }
    }

    #[test]
    fn evaluate_penalty_evaluated_correctly() {
        fn penalty(_: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation::default(),
        };

        criterion.evaluate(&mut initial_state);

        let info = initial_state.get_eval();

        assert_eq!(10.0, info.value);
        assert_eq!(false, info.is_feasible);
    }

    #[test]
    fn evaluate_value_evaluated_correctly() {
        fn penalty(_: &TestSolution) -> f64 {
            0.0
        }

        fn value(_: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestSolution>::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation::default(),
        };

        criterion.evaluate(&mut initial_state);
        let info = initial_state.get_eval();

        assert_eq!(20.0, info.value);
        assert_eq!(true, info.is_feasible);
    }

    #[test]
    fn evaluate_weird_solution() {
        fn penalty(_: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestSolution>::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation {
                value: 10.0,
                is_feasible: false,
            },
        };

        criterion.evaluate(&mut initial_state);
        let mut info = initial_state.get_eval_mut();

        info.value = 10.0;
        info.is_feasible = false;
    }

    #[test]
    fn is_first_better_value_comparison() {
        fn penalty(_: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestSolution) -> f64 {
            20.0
        }
        let mut criterion = Criterion::<TestSolution>::new(&penalty, &value, false);
        let mut info_a = Evaluation {
            value: 10.0,
            is_feasible: true,
        };
        let mut info_b = Evaluation {
            value: 20.0,
            is_feasible: true,
        };

        assert_eq!(false, criterion.is_first_better(&info_a, &info_b));
        info_a.value = 20.0;
        info_b.value = 10.0;
        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));

        criterion.is_minimization = true;

        assert_eq!(false, criterion.is_first_better(&info_a, &info_b));
        info_a.value = 10.0;
        info_b.value = 20.0;
        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));
    }

    #[test]
    fn is_first_better_take_feasibility_into_account() {
        fn penalty<T>(_: &T) -> f64 {
            10.0
        }

        fn value<T>(_: &T) -> f64 {
            20.0
        }
        let mut criterion = Criterion::<TestSolution>::new(&penalty, &value, false);

        let info_a = Evaluation {
            value: 30.0,
            is_feasible: true,
        };

        let info_b = Evaluation {
            value: 2.0,
            is_feasible: false,
        };

        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));
        criterion.is_minimization = true;
        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));
    }
}
