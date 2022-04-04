use self::{coolers::Cooler, stop::StopCriteria};
use crate::{
    analysis::Saver,
    base::{Criterion, OptAlgorithm, Problem, Solution},
};
use rand::{prelude::ThreadRng, Rng};
use std::f64::consts::E;

pub mod coolers;
pub mod stop;

pub struct SimulatedAnnealing<'a, P, S, C, SC> {
    stop_criteria: SC,
    cooler: C,
    change: &'a dyn Fn(&mut S, &P),
    initial_solution: &'a S,
}

impl<'a, P, S, C, SC> SimulatedAnnealing<'a, P, S, C, SC>
where
    S: Solution,
    C: Cooler,
    SC: StopCriteria,
{
    pub fn new(
        initial_solution: &'a S,
        stop_criteria: SC,
        cooler: C,
        change_sol: &'a dyn Fn(&mut S, &P),
    ) -> Self {
        Self {
            initial_solution,
            stop_criteria,
            cooler,
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

impl<'a, P, S, C, SC> OptAlgorithm<'a, P, S> for SimulatedAnnealing<'a, P, S, C, SC>
where
    S: Solution,
    C: Cooler,
    SC: StopCriteria,
    P: Problem,
{
    fn solve(&mut self, problem: P, criterion: &mut Criterion<S>) -> S {
        self.reset();

        let mut rnd = rand::thread_rng();
        let mut solution = self.initial_solution.clone();

        //Initial evaluation
        criterion.evaluate(&mut solution);
        let mut best = solution.clone();

        let change = self.change;

        //Main loop
        while !self.stop_criteria.should_stop(solution.get_value()) {
            //Save current state and then change and evaluate it
            let before = solution.clone();
            (change)(&mut solution, &problem);
            criterion.evaluate(&mut solution);

            let best_eval = best.get_eval();

            let hot_enough = self.hot_enough_to_swap(
                &mut rnd,
                solution.get_eval().value,
                before.get_eval().value,
            );

            if criterion.is_first_better(solution.get_eval(), before.get_eval()) || hot_enough {
                if criterion.is_first_better(solution.get_eval(), best_eval) {
                    best = solution.clone()
                }
            } else {
                solution = before.clone();
            }

            self.cooler.cool();
        }
        best
    }

    fn add_saver(&mut self, saver: &mut dyn Saver<S>) {
        todo!()
    }

    fn clear_savers(&mut self) {
        todo!()
    }

    fn reset(&mut self) {
        self.cooler.reset();
        self.stop_criteria.reset();
    }
}
