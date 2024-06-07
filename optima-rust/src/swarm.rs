use std::fmt::Display;

use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    thread_rng, Rng,
};

use crate::{
    annealing::stop::StopCriteria,
    base::{
        solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Problem, Solution,
    },
};

pub type SwarmInsightFn = dyn Fn(&FnProblem, &Vec<Particle>);

#[solution_attr]
#[derive(Clone, DerivedSolution)]
pub struct Particle {
    best_local_index: usize,
    velocity_x: f64,
    velocity_y: f64,
    pub x: f64,
    pub y: f64,
}

impl Particle {
    pub fn update_position(&mut self, min: f64, max: f64) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;

        self.x = f64::clamp(self.x, min, max);
        self.x = f64::clamp(self.y, min, max);
    }
}
#[derive(Copy, Clone)]
pub struct FnProblem {
    pub id: u32,
    pub max: f64,
    pub min: f64,
    pub points_distribution: Uniform<f64>,
}

impl FnProblem {
    pub fn new(id: u32, max: f64, min: f64) -> Self {
        Self {
            min,
            max,
            id,
            points_distribution: Uniform::new_inclusive(min, max),
        }
    }
}

impl Problem for FnProblem {}

pub struct ParticleSwarm<'a, SC: StopCriteria> {
    pub particles: Vec<Particle>,
    best_global_index: usize,
    stop_criteria: SC,
    local_attraction: f64,
    global_attraction: f64,
    inertia: f64,
    rng: ThreadRng,
    insight: Option<&'a mut SwarmInsightFn>,
}

impl<'a, SC> ParticleSwarm<'a, SC>
where
    SC: StopCriteria,
{
    pub fn new(size: usize, stop_criteria: SC) -> Self {
        let rng = thread_rng();
        Self {
            particles: Vec::with_capacity(size),
            best_global_index: 0,
            stop_criteria,
            global_attraction: 0.5,
            local_attraction: 0.5,
            inertia: 0.05,
            insight: None,
            rng,
        }
    }

    pub fn register_insight(&mut self, f: &'a mut SwarmInsightFn) {
        self.insight = Some(f);
    }

    fn reset(&mut self) {
        self.stop_criteria.reset();
        self.best_global_index = 0;
    }

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

    fn initialize(&mut self, problem: &FnProblem, criterion: &mut Criterion<FnProblem, Particle>) {
        self.particles.clear();

        for i in 0..self.particles.capacity() {
            let mut particle = Particle {
                best_local_index: i,
                velocity_x: self.rng.gen(),
                velocity_y: self.rng.gen(),
                x: problem.points_distribution.sample(&mut self.rng),
                y: problem.points_distribution.sample(&mut self.rng),
                eval: Evaluation::default(),
            };

            criterion.evaluate(&problem, &mut particle);
            self.particles.push(particle);

            if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                self.best_global_index = i;
            }
        }
    }
}

impl<'a, SC> OptAlgorithm<'a, FnProblem, Particle> for ParticleSwarm<'a, SC>
where
    SC: StopCriteria,
{
    fn solve(
        &mut self,
        problem: FnProblem,
        criterion: &mut Criterion<FnProblem, Particle>,
    ) -> Vec<Particle> {
        self.reset();
        self.initialize(&problem, criterion);

        let best_value = self.particles[self.best_global_index].get_value();
        while !self.stop_criteria.should_stop(best_value) {
            for i in 0..self.particles.len() {
                //Pick random parameters r_i and r_g
                let r_local: f64 = self.rng.gen();
                let r_global: f64 = self.rng.gen();

                //Update x velocity
                let particle = &self.particles[i];
                let best_local = &self.particles[particle.best_local_index];
                let best_global = &self.particles[self.best_global_index];
                let local = self.local_attraction * r_local * (best_local.x - particle.x);
                let global = self.global_attraction * r_global * (best_global.x - particle.x);
                let mut particle = &mut self.particles[i];
                particle.velocity_x = self.inertia * particle.velocity_x + local + global;

                //Update y velocity
                let particle = &self.particles[i];
                let best_local = &self.particles[particle.best_local_index];
                let best_global = &self.particles[self.best_global_index];
                let local = self.local_attraction * r_local * (best_local.y - particle.y);
                let global = self.global_attraction * r_global * (best_global.y - particle.y);
                let particle = &mut self.particles[i];
                particle.velocity_y = self.inertia * particle.velocity_y + local + global;

                //Update position in search space according to velocity
                particle.update_position(problem.min, problem.max);

                //Update best and local trackers
                if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                    self.best_global_index = i;
                }
            }
            match &mut self.insight {
                Some(f) => f(&problem, &self.particles),
                _ => {}
            }
        }
        vec![self.particles[self.best_global_index].clone()]
    }

    fn reset(&mut self) {
        self.stop_criteria.reset();
    }
}

impl<'a, SC: StopCriteria> Display for ParticleSwarm<'a, SC> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Partical swarm: particles: {}, inertia: {}, global attr: {}, local attr: {}\n\t{}",
            self.particles.len(),
            self.inertia,
            self.global_attraction,
            self.local_attraction,
            self.stop_criteria
        )
    }
}
