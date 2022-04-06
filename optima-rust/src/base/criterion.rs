use crate::base::{Evaluation, Problem, Solution};

#[derive(Clone, Copy)]
pub struct Criterion<'a, P, S>
where
    S: Solution,
{
    penalty: &'a dyn Fn(&P, &S) -> f64,
    value: &'a dyn Fn(&P, &S) -> f64,
    pub is_minimization: bool,
}

impl<'a, P, S> Criterion<'a, P, S>
where
    S: Solution,
    P: Problem,
{
    pub fn new(
        penalty: &'a dyn Fn(&P, &S) -> f64,
        value: &'a dyn Fn(&P, &S) -> f64,
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

    pub fn evaluate(&self, problem: &P, solution: &mut S) {
        let mut value = (self.penalty)(problem, solution);
        let is_feasible = value == 0.0;
        if is_feasible {
            value = (self.value)(problem, solution);
        }

        let mut eval = solution.get_eval_mut();
        eval.value = value;
        eval.is_feasible = is_feasible;
    }
}

#[cfg(test)]
mod tests {
    use optima_macros::{solution_attr, DerivedSolution};

    use crate::base::{Evaluation, Problem, Solution};

    use super::Criterion;

    #[solution_attr]
    #[derive(Clone, DerivedSolution)]
    struct TestSolution {}
    impl Default for TestSolution {
        fn default() -> Self {
            Self {
                eval: Default::default(),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct TestProblem;
    impl Problem for TestProblem {
        fn get_id(&self) -> u32 {
            1
        }
    }

    #[test]
    fn evaluate_penalty_evaluated_correctly() {
        fn penalty(_: &TestProblem, _: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestProblem, _: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation::default(),
        };
        let problem = TestProblem {};
        criterion.evaluate(&problem, &mut initial_state);

        let info = initial_state.get_eval();

        assert_eq!(10.0, info.value);
        assert_eq!(false, info.is_feasible);
    }

    #[test]
    fn evaluate_value_evaluated_correctly() {
        fn penalty(_: &TestProblem, _: &TestSolution) -> f64 {
            0.0
        }

        fn value(_: &TestProblem, _: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestProblem, TestSolution>::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation::default(),
        };

        let problem = TestProblem {};
        criterion.evaluate(&problem, &mut initial_state);
        let info = initial_state.get_eval();

        assert_eq!(20.0, info.value);
        assert_eq!(true, info.is_feasible);
    }

    #[test]
    fn evaluate_weird_solution() {
        fn penalty(_: &TestProblem, _: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestProblem, _: &TestSolution) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestProblem, TestSolution>::new(&penalty, &value, false);
        let mut initial_state = TestSolution {
            eval: Evaluation {
                value: 10.0,
                is_feasible: false,
            },
        };

        let problem = TestProblem {};
        criterion.evaluate(&problem, &mut initial_state);
        let mut info = initial_state.get_eval_mut();

        info.value = 10.0;
        info.is_feasible = false;
    }

    #[test]
    fn is_first_better_value_comparison() {
        fn penalty(_: &TestProblem, _: &TestSolution) -> f64 {
            10.0
        }

        fn value(_: &TestProblem, _: &TestSolution) -> f64 {
            20.0
        }
        let mut criterion = Criterion::<TestProblem, TestSolution>::new(&penalty, &value, false);
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
        fn penalty<T>(_: &TestProblem, _: &T) -> f64 {
            10.0
        }

        fn value<T>(_: &TestProblem, _: &T) -> f64 {
            20.0
        }
        let mut criterion = Criterion::<TestProblem, TestSolution>::new(&penalty, &value, false);

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
