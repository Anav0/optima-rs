use std::str::ParseBoolError;

use rand::{thread_rng, Rng};

use crate::{
    annealing::stop::StopCriteria,
    base::{Criterion, OptAlgorithm, Solution},
};

pub trait Point {
    fn subtract(&self, other: &Self) -> f64;
    fn update(&mut self, velocity: f64);
    fn new_random(&self) -> Self;
}

pub struct Particle<S> {
    best_local_index: usize,
    velocity: f64,
    solution: S,
}

impl<S> Particle<S>
where
    S: Solution + Point,
{
    pub fn get_value(&self) -> f64 {
        self.solution.get_eval().value
    }
}

pub struct ParticleSwarm<'a, S: Solution + Clone + Point> {
    pub particles: Vec<Particle<S>>,
    best_global_index: usize,
    stop_criteria: &'a mut dyn StopCriteria,
    initial_solution: S,
    local_attraction: f64,
    global_attraction: f64,
    inertia: f64,
}

impl<'a, S> ParticleSwarm<'a, S>
where
    S: Solution + Clone + Point,
{
    fn is_better(&self, that: usize, this: usize, is_minimization: bool) -> bool {
        let best_value = self.particles[this].get_value();
        let current_value = self.particles[that].get_value();

        if is_minimization {
            if current_value < best_value {
                return true;
            }
        } else {
            if current_value > best_value {
                return true;
            }
        }
        false
    }

    pub fn new(size: usize, initial_solution: S, stop_criteria: &'a mut dyn StopCriteria) -> Self {
        Self {
            particles: Vec::with_capacity(size),
            best_global_index: 0,
            initial_solution,
            stop_criteria,
            global_attraction: 0.5,
            local_attraction: 0.5,
            inertia: 0.05,
        }
    }
    fn initialize(&mut self, criterion: &mut Criterion<S>) {
        self.particles.clear();

        let mut rng = thread_rng();
        for i in 0..self.particles.capacity() {
            // Pick random position in search space.
            let mut particle = Particle {
                best_local_index: i,
                solution: self.initial_solution.new_random(),
                velocity: rng.gen(),
            };

            criterion.evaluate(&mut particle.solution);
            self.particles.push(particle);

            //If current particle is better then best save that info
            if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                self.best_global_index = i;
            }
        }
    }
}

impl<'a, S> OptAlgorithm<'a, S> for ParticleSwarm<'a, S>
where
    S: Solution + Clone + Point,
{
    fn solve(&mut self, criterion: &mut Criterion<S>) -> S {
        self.initialize(criterion);

        let best_value = self.particles[self.best_global_index].get_value();
        let mut rng = thread_rng();
        while !self.stop_criteria.should_stop(best_value) {
            for i in 0..self.particles.len() {
                let particle = &self.particles[i];
                //Pick random parameters r_i and r_g
                let r_local: f64 = rng.gen();
                let r_global: f64 = rng.gen();

                //Update velocity
                let best_local = &self.particles[particle.best_local_index];
                let best_global = &self.particles[self.best_global_index];
                let local_pos_sub = best_local.solution.subtract(&particle.solution);
                let global_pos_sub = best_global.solution.subtract(&particle.solution);

                let local = self.local_attraction * r_local * local_pos_sub;
                let global = self.global_attraction * r_global * global_pos_sub;

                let particle = &mut self.particles[i];
                particle.velocity = self.inertia * particle.velocity + local + global;

                //Update position in search space according to velocity
                particle.solution.update(particle.velocity);

                //Update best and local trackers
                if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                    self.best_global_index = i;
                }
            }
        }
        //@ Improvement: Do not clone
        self.particles[self.best_global_index].solution.clone()
    }
}
