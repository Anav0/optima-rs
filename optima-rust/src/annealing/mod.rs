use self::{coolers::Cooler, stop::StopCriteria};
use crate::base::{Criterion, OptAlgorithm, Solution};
use rand::{prelude::ThreadRng, Rng};
use std::f64::consts::E;

pub mod coolers;
pub mod stop;

pub struct SimulatedAnnealing<'a, S> {
    pub solution: S,
    stop_criteria: &'a mut dyn StopCriteria,
    cooler: &'a mut dyn Cooler,
    change: &'a dyn Fn(&mut Self),
}

impl<'a, S> SimulatedAnnealing<'a, S>
where
    S: Solution + Clone,
{
    pub fn new(
        solution: S,
        stop_criteria: &'a mut dyn StopCriteria,
        cooler: &'a mut dyn Cooler,
        change_sol: &'a dyn Fn(&mut Self),
    ) -> Self {
        Self {
            stop_criteria,
            cooler,
            solution,
            change: change_sol,
        }
    }
    fn hot_enough_to_swap(
        &self,
        rnd: &mut ThreadRng,
        current_value: f64,
        before_move: f64,
    ) -> bool {
        let diff = current_value - before_move;
        if diff == 0.0 {
            return false;
        };

        if diff > 0.0 {
            return true;
        }

        return rnd.gen::<f64>() < E.powf(diff / self.cooler.get_temp());
    }
}

impl<'a, S> OptAlgorithm<'a, S> for SimulatedAnnealing<'a, S>
where
    S: Solution + Clone,
{
    fn solve(&mut self, criterion: &mut Criterion<S>) -> S {
        let mut rnd = rand::thread_rng();
        //Initial evaluation
        criterion.evaluate(&mut self.solution);
        let mut best = self.solution.clone();

        let change = self.change; //INFO: just for clarity

        //Main loop
        while !self
            .stop_criteria
            .should_stop(self.solution.get_eval().value)
        {
            //Save current state and then change and evaluate it
            let before = self.solution.clone();
            (change)(self);
            criterion.evaluate(&mut self.solution);

            let best_eval = best.get_eval();
            if criterion.is_first_better(self.solution.get_eval(), before.get_eval())
                || self.hot_enough_to_swap(
                    &mut rnd,
                    self.solution.get_eval().value,
                    before.get_eval().value,
                )
            {
                if criterion.is_first_better(self.solution.get_eval(), best_eval) {
                    best = self.solution.clone()
                }
            } else {
                self.solution = before.clone();
            }

            self.cooler.cool();
        }
        best
    }
}
