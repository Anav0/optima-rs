use self::{coolers::Cooler, stop::StopCriteria};
use crate::base::{Criterion, OptAlgorithm, Problem, Solution};
use rand::{prelude::ThreadRng, Rng};
use std::{f64::consts::E, fmt::Display};

pub mod coolers;
pub mod stop;

pub type ChangeFn<S, P> = dyn Fn(&mut S, &P, &mut ThreadRng);
pub type AnnealingInsightFn<S, P, C> = dyn FnMut(&C, u32, &P, &S, &S, bool);

pub struct SimulatedAnnealing<'a, P: Problem, S: Solution, C: Cooler, SC: StopCriteria> {
    stop_criteria: SC,
    cooler: C,
    change: &'a ChangeFn<S, P>,
    initial_solution: &'a S,
    insight: Option<&'a mut AnnealingInsightFn<S, P, C>>,
    rnd: ThreadRng,
}

impl<'a, P, S, C, SC> SimulatedAnnealing<'a, P, S, C, SC>
where
    S: Solution,
    P: Problem,
    C: Cooler,
    SC: StopCriteria,
{
    pub fn new(
        initial_solution: &'a S,
        stop_criteria: SC,
        cooler: C,
        change: &'a ChangeFn<S, P>,
    ) -> Self {
        Self {
            initial_solution,
            stop_criteria,
            cooler,
            change,
            insight: None,
            rnd: rand::thread_rng(),
        }
    }

    pub fn register_insight(&mut self, insight: &'a mut AnnealingInsightFn<S, P, C>) {
        self.insight = Some(insight);
    }

    fn hot_enough_to_swap(&mut self, current_value: f64, before_move: f64) -> bool {
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

impl<'a, P, S, C, SC> OptAlgorithm<'a, P, S> for SimulatedAnnealing<'a, P, S, C, SC>
where
    S: Solution,
    C: Cooler,
    SC: StopCriteria,
    P: Problem,
{
    fn solve(&mut self, problem: P, criterion: &mut Criterion<P, S>) -> Vec<S> {
        self.reset();

        let mut solution = self.initial_solution.clone();

        //Initial evaluation
        criterion.evaluate(&problem, &mut solution);
        let mut best = solution.clone();

        let change = self.change;

        //Main loop
        let mut counter = 0;
        while !self.stop_criteria.should_stop() {
            //Save current state and then change and evaluate it
            let before = solution.clone();
            (change)(&mut solution, &problem, &mut self.rnd);
            criterion.evaluate(&problem, &mut solution);

            let best_eval = best.get_eval();

            let hot_enough =
                self.hot_enough_to_swap(solution.get_eval().value, before.get_eval().value);

            if criterion.is_first_better(solution.get_eval(), before.get_eval()) || hot_enough {
                if criterion.is_first_better(solution.get_eval(), best_eval) {
                    best = solution.clone()
                }
            } else {
                solution = before.clone();
            }
            match &mut self.insight {
                Some(f) => f(&self.cooler, counter, &problem, &best, &solution, false),
                _ => {}
            }
            counter += 1;
            self.cooler.cool();
            self.stop_criteria.update(solution.get_value());
        }

        match &mut self.insight {
            Some(f) => f(&self.cooler, counter, &problem, &best, &solution, true),
            _ => {}
        }

        vec![best]
    }

    fn reset(&mut self) {
        self.cooler.reset();
        self.stop_criteria.reset();
    }
}

impl<'a, P: Problem, S: Solution, C: Cooler, SC: StopCriteria> Display
    for SimulatedAnnealing<'a, P, S, C, SC>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Simulated annealing:\n{}", self.stop_criteria)
    }
}
