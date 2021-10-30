use std::f64::consts::E;

use rand::{prelude::ThreadRng, Rng};

use crate::{
    base::{Evaluation, OptAlgorithm, Solution, State},
    criterion::Criterion,
};

use self::{coolers::Cooler, stop::StopCriteria};

pub mod coolers;
pub mod stop;

pub struct SimmulatedAnnealing<'a> {
    stop_criteria: &'a mut dyn StopCriteria,
    cooler: &'a mut dyn Cooler,
    rnd: ThreadRng,
}

impl<'a> SimmulatedAnnealing<'a> {
    pub fn new(stop_criteria: &'a mut dyn StopCriteria, cooler: &'a mut dyn Cooler) -> Self {
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

impl<'a, S> OptAlgorithm<'a, S> for SimmulatedAnnealing<'a>
where
    S: Solution + Clone,
{
    fn solve(
        &mut self,
        mut solution: S,
        criterion: &mut Criterion<S>,
        change: &dyn Fn(&mut S),
    ) -> S {
        //Initial evaluation
        criterion.evaluate(&mut solution);
        let mut best = solution.clone();
        let mut before = solution.clone();

        //Main loop
        while !self.stop_criteria.should_stop(solution.get_eval().value) {
            //Save current state and then change and evaluate it
            before = solution.clone();
            change(&mut solution);
            criterion.evaluate(&mut solution);

            let current_eval = solution.get_eval();
            let before_eval = before.get_eval();
            let best_eval = best.get_eval();
            if criterion.is_first_better(current_eval, before_eval)
                || self.hot_enought_to_swap(current_eval.value, before_eval.value)
            {
                if criterion.is_first_better(current_eval, best_eval) {
                    best = solution.clone()
                }
            } else {
                solution = before.clone();
            }

            self.cooler.cool();
        }
        best
    }
}
