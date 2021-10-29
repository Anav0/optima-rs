use std::collections::HashMap;

use crate::criterion::Criterion;

#[derive(Clone, Copy, Debug)]
pub struct SolutionInfo {
    pub value: f64,
    pub is_feasible: bool,
    pub check_penalty: bool,
}

impl Default for SolutionInfo {
    fn default() -> Self {
        Self {
            value: f64::NAN,
            is_feasible: false,
            check_penalty: true,
        }
    }
}

pub trait InfoHolder {
    fn get_info(&self) -> &SolutionInfo;
    fn get_info_mut(&mut self) -> &mut SolutionInfo;
}

pub struct Solution<T>
where
    T: Clone,
    T: InfoHolder,
{
    states: HashMap<State, T>,
}
impl<T> Solution<T>
where
    T: Clone,
    T: InfoHolder,
{
    pub fn new(initial_state: T) -> Self {
        let mut states = HashMap::with_capacity(3);
        states.insert(State::BeforeChange, initial_state.clone());
        states.insert(State::Current, initial_state.clone());
        states.insert(State::Best, initial_state.clone());
        Self { states }
    }

    pub fn reset(&mut self) {
        self.set_state_info(State::Current, f64::NAN, false, true);
        self.swap_info(State::BeforeChange, State::Current);
        self.swap_info(State::Best, State::Current);
    }

    pub fn get_state_info_ref(&self, state: State) -> &SolutionInfo {
        self.states.get(&state).unwrap().get_info()
    }

    pub fn get_state_info_mut(&mut self, state: State) -> &mut SolutionInfo {
        self.states.get_mut(&state).unwrap().get_info_mut()
    }

    pub fn get_state_ref(&self, state: State) -> &T {
        self.states.get(&state).unwrap()
    }

    pub fn get_state_mut(&mut self, state: State) -> &mut T {
        let tmp = self.states.get_mut(&state).unwrap();
        tmp
    }

    pub fn set_state_info(
        &mut self,
        state: State,
        value: f64,
        is_feasible: bool,
        check_penalty: bool,
    ) {
        let info = self.get_state_info_mut(state);
        info.value = value;
        info.is_feasible = is_feasible;
        info.check_penalty = check_penalty;
    }

    pub fn swap_info(&mut self, this: State, with_this: State) {
        let with_this_info = self.get_state_info_ref(with_this);

        self.set_state_info(
            this,
            with_this_info.value,
            with_this_info.is_feasible,
            with_this_info.check_penalty,
        );
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum State {
    Best,
    Current,
    BeforeChange,
}

pub trait OptAlgorithm<'a, T>
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
