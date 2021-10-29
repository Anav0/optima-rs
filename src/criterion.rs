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

    pub fn is_first_better(&self, first_info: &SolutionInfo, second_info: &SolutionInfo) -> bool {
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

#[cfg(test)]
mod tests {
    use crate::base::{InfoHolder, Solution, SolutionInfo, State};

    use super::Criterion;

    #[derive(Clone)]
    struct TestState {
        info: SolutionInfo,
    }
    impl InfoHolder for TestState {
        fn get_info(&self) -> &SolutionInfo {
            &self.info
        }

        fn get_info_mut(&mut self) -> &mut SolutionInfo {
            &mut self.info
        }
    }

    #[test]
    fn evaluate_penalty_evaluated_correctly() {
        fn penalty<T>(_: &T) -> f64 {
            10.0
        }

        fn value<T>(_: &T) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestState>::new(&penalty, &value, false);
        let initial_state = TestState {
            info: SolutionInfo::default(),
        };
        let mut solution = Solution::new(initial_state);

        criterion.evaluate(&mut solution);

        let info = solution.get_state_info_ref(State::Current);

        assert_eq!(10.0, info.value);
        assert_eq!(false, info.check_penalty);
        assert_eq!(false, info.is_feasible);
    }

    #[test]
    fn evaluate_value_evaluated_correctly() {
        fn penalty<T>(_: &T) -> f64 {
            0.0
        }

        fn value<T>(_: &T) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestState>::new(&penalty, &value, false);
        let initial_state = TestState {
            info: SolutionInfo::default(),
        };
        let mut solution = Solution::new(initial_state);

        criterion.evaluate(&mut solution);
        let info = solution.get_state_info_ref(State::Current);

        assert_eq!(20.0, info.value);
        assert_eq!(false, info.check_penalty);
        assert_eq!(true, info.is_feasible);
    }

    #[test]
    fn evaluate_weird_solution() {
        fn penalty<T>(_: &T) -> f64 {
            10.0
        }

        fn value<T>(_: &T) -> f64 {
            20.0
        }
        let criterion = Criterion::<TestState>::new(&penalty, &value, false);
        let initial_state = TestState {
            info: SolutionInfo::default(),
        };
        let mut solution = Solution::new(initial_state);
        solution.set_state_info(State::Current, 10.0, false, true);

        criterion.evaluate(&mut solution);
        let info = solution.get_state_info_ref(State::Current);

        assert_eq!(10.0, info.value);
        assert_eq!(false, info.check_penalty);
        assert_eq!(false, info.is_feasible);
    }

    #[test]
    fn is_first_better_value_comparison() {
        fn penalty<T>(_: &T) -> f64 {
            10.0
        }

        fn value<T>(_: &T) -> f64 {
            20.0
        }
        let mut criterion = Criterion::<TestState>::new(&penalty, &value, false);
        let mut info_a = SolutionInfo {
            value: 10.0,
            check_penalty: false,
            is_feasible: true,
        };
        let mut info_b = SolutionInfo {
            value: 20.0,
            check_penalty: false,
            is_feasible: true,
        };

        assert_eq!(false, criterion.is_first_better(&info_a, &info_b));
        info_a.value = 20.0;
        info_b.value = 10.0;
        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));

        criterion.is_minimalization_problem = true;

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
        let mut criterion = Criterion::<TestState>::new(&penalty, &value, false);

        let info_a = SolutionInfo {
            value: 30.0,
            check_penalty: false,
            is_feasible: true,
        };

        let info_b = SolutionInfo {
            value: 2.0,
            check_penalty: false,
            is_feasible: false,
        };

        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));
        criterion.is_minimalization_problem = true;
        assert_eq!(true, criterion.is_first_better(&info_a, &info_b));
    }
}
