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

pub trait Solution {
    fn reset(&mut self);
    fn get_info(&self, state: State) -> &SolutionInfo;
    fn set_info(&mut self, value: f64, is_feasible: bool, check_penalty: bool);
    fn update_best(&mut self);
    fn update_current(&mut self);
    fn update_before(&mut self);
}

pub trait StateChanger {
    type SolutionType;
    fn change_state(&mut self, solution: &mut Self::SolutionType);
}
pub enum State {
    Best,
    Current,
    BeforeChange,
}
