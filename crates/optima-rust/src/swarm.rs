use std::{
    fmt::Display,
    ops::{Bound, RangeBounds, RangeInclusive},
    thread::{self, current},
    time::Duration,
};

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

pub type SwarmInsightFn = dyn FnMut(&FnProblem<RangeInclusive<f64>>, &Vec<Particle>, usize, bool) -> bool;

pub fn min_value_of_range<R, T>(range: &R) -> Option<T>
where
    R: RangeBounds<T>,
    T: PartialOrd + Clone,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Included(start), _) => Some(start.clone()), // Inclusive start bound
        (Bound::Excluded(start), _) => Some(start.clone()), // Exclusive start bound
        _ => None,
    }
}

pub fn max_value_of_range<R, T>(range: &R) -> Option<T>
where
    R: RangeBounds<T>,
    T: PartialOrd + Clone,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Included(_), Bound::Included(end)) => Some(end.clone()), // Inclusive end bound
        (Bound::Included(start), Bound::Excluded(end)) => {
            if *start < *end {
                Some(end.clone()) // Exclusive end bound
            } else {
                Some(start.clone())
            }
        }
        (Bound::Excluded(_), Bound::Included(end)) => Some(end.clone()), // Inclusive end bound
        (Bound::Excluded(start), Bound::Excluded(end)) => {
            if *start < *end {
                Some(end.clone())
            } else {
                Some(start.clone())
            }
        }
        (Bound::Unbounded, Bound::Included(end)) => Some(end.clone()), // Inclusive end bound
        (Bound::Unbounded, Bound::Excluded(end)) => Some(end.clone()), // Exclusive end bound
        _ => None,
    }
}

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
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            best_local_index: 0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            eval: Evaluation {
                value: 0.0,
                is_feasible: true,
            },
        }
    }
    pub fn update_position(&mut self, problem: &FnProblem<RangeInclusive<f64>>) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;

        let x_min = min_value_of_range(&problem.x_range).unwrap();
        let x_max = max_value_of_range(&problem.x_range).unwrap();

        let y_min = min_value_of_range(&problem.y_range).unwrap();
        let y_max = max_value_of_range(&problem.y_range).unwrap();

        self.x = f64::clamp(self.x, x_min, x_max);
        self.y = f64::clamp(self.y, y_min, y_max);
    }
}
#[derive(Copy, Clone)]
pub struct FnProblem<R: RangeBounds<f64>> {
    pub id: u32,
    pub x_range: R,
    pub y_range: R,

    points_distribution_x: Uniform<f64>,
    points_distribution_y: Uniform<f64>,
}

impl FnProblem<RangeInclusive<f64>> {
    pub fn new(id: u32, x_range: RangeInclusive<f64>, y_range: RangeInclusive<f64>) -> Self {
        Self {
            points_distribution_x: Uniform::from(x_range.clone()),
            points_distribution_y: Uniform::from(y_range.clone()),
            x_range: x_range,
            y_range: y_range,
            id,
        }
    }
}

impl<R: RangeBounds<f64>> Problem for FnProblem<R> {}

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

    pub fn with_attraction(
        size: usize,
        stop_criteria: SC,
        global_attraction: f64,
        local_attraction: f64,
        inertia: f64,
    ) -> Self {
        let rng = thread_rng();
        Self {
            particles: Vec::with_capacity(size),
            best_global_index: 0,
            stop_criteria,
            global_attraction,
            local_attraction,
            inertia,
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

    fn is_better(&self, this: usize, known_best_index: usize, is_minimization: bool) -> bool {
        let best_value = self.particles[known_best_index].get_value();
        let current_value = self.particles[this].get_value();

        return if is_minimization {
            current_value < best_value
        } else {
            current_value > best_value
        };
    }

    fn initialize(
        &mut self,
        problem: &FnProblem<RangeInclusive<f64>>,
        criterion: &mut Criterion<FnProblem<RangeInclusive<f64>>, Particle>,
    ) {
        self.particles.clear();

        for i in 0..self.particles.capacity() {
            let mut particle = Particle {
                best_local_index: i,
                velocity_x: self.rng.gen(),
                velocity_y: self.rng.gen(),
                x: problem.points_distribution_x.sample(&mut self.rng),
                y: problem.points_distribution_y.sample(&mut self.rng),
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

impl<'a, SC> OptAlgorithm<'a, FnProblem<RangeInclusive<f64>>, Particle> for ParticleSwarm<'a, SC>
where
    SC: StopCriteria,
{
    fn solve(
        &mut self,
        problem: FnProblem<RangeInclusive<f64>>,
        criterion: &mut Criterion<FnProblem<RangeInclusive<f64>>, Particle>,
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
                let particle = &mut self.particles[i];
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
                particle.update_position(&problem);

                criterion.evaluate(&problem, particle);

                let particle = &self.particles[i];

                let best_local_index = particle.best_local_index;
                let is_local_better =
                    self.is_better(i, best_local_index, criterion.is_minimization);

                let is_this_better =
                    self.is_better(i, self.best_global_index, criterion.is_minimization);

                let particle = &mut self.particles[i];
                if is_local_better {
                    particle.best_local_index = i;
                }
                if is_this_better {
                    self.best_global_index = i;
                }
            }

            let should_stop = match &mut self.insight {
                Some(f) => f(&problem, &self.particles, self.best_global_index, false),
                _ => false,
            };

            if should_stop {
                break;
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
