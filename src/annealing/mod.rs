use std::f64::consts::E;

use rand::{prelude::ThreadRng, Rng};

use crate::{
    algorithms::OptAlgorithm,
    base::{InfoHolder, Solution, State},
    criterion::Criterion,
};

use self::{coolers::Cooler, stop::StopCriteria};

pub mod coolers;
pub mod stop;

pub struct SimmulatedAnnealing<'a, T> {
    stop_criteria: &'a mut dyn StopCriteria<T>,
    cooler: &'a mut dyn Cooler,
    rnd: ThreadRng,
}

impl<'a, T> SimmulatedAnnealing<'a, T>
where
    T: Clone,
    T: InfoHolder,
{
    pub fn new(stop_criteria: &'a mut dyn StopCriteria<T>, cooler: &'a mut dyn Cooler) -> Self {
        Self {
            stop_criteria,
            cooler,
            rnd: rand::thread_rng(),
        }
    }
    fn hot_enought_to_swap(&mut self, current_value: f64, before_move: f64) -> bool {
        let diff = current_value - before_move;
        if diff == 0.0 {
            return false;
        };

        if diff > 0.0 {
            return true;
        }

        return self.rnd.gen::<f64>() < E.powf(diff / self.cooler.get_temp());
    }
}

impl<'a, T> OptAlgorithm<'a, T> for SimmulatedAnnealing<'a, T>
where
    T: Clone,
    T: InfoHolder,
{
    fn solve(
        &mut self,
        solution: &'a mut Solution<T>,
        criterion: &mut Criterion<T>,
        change: &dyn Fn(&mut T),
    ) {
        //Initial evaluation
        criterion.evaluate(solution);
        solution.swap_info(State::BeforeChange, State::Current);
        solution.swap_info(State::Best, State::Current);

        //Main loop
        while !self
            .stop_criteria
            .should_stop(solution.get_state_info_ref(State::Current).value)
        {
            //Save current state and then change and evaluate it
            solution.swap_info(State::BeforeChange, State::Current);
            solution.set_state_info(State::Current, f64::NAN, false, true);
            change(solution.get_state_mut(State::Current));
            criterion.evaluate(solution);

            let current = solution.get_state_info_ref(State::Current);
            let before = solution.get_state_info_ref(State::BeforeChange);
            let best = solution.get_state_info_ref(State::Best);
            if criterion.is_first_better(current, before)
                || self.hot_enought_to_swap(current.value, before.value)
            {
                if criterion.is_first_better(current, best) {
                    solution.swap_info(State::Best, State::Current)
                }
            } else {
                solution.swap_info(State::Current, State::BeforeChange)
            }

            self.cooler.cool();
        }
    }
}
