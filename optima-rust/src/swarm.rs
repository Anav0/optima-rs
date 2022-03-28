use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    thread_rng, Rng,
};

use crate::{
    analysis::{AsCsvRow, Saver},
    annealing::stop::StopCriteria,
    base::{solution_attr, Criterion, DerivedSolution, Evaluation, OptAlgorithm, Solution},
};

#[solution_attr]
#[derive(Clone, DerivedSolution)]
pub struct Particle {
    best_local_index: usize,
    velocity_x: f64,
    velocity_y: f64,
    pub x: f64,
    pub y: f64,
}

impl AsCsvRow for Particle {
    fn as_row(&self, iter: usize) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            iter,
            self.x,
            self.y,
            self.velocity_x,
            self.velocity_y,
            self.best_local_index,
            self.get_value()
        )
    }
}

impl Particle {
    pub fn update_position(&mut self, min: f64, max: f64) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;

        if self.x > max {
            self.x = max;
        }
        if self.x < min {
            self.x = min;
        }

        if self.y > max {
            self.y = max;
        }
        if self.y < min {
            self.y = min;
        }
    }
}

pub struct ParticleSwarm<'a> {
    pub particles: Vec<Particle>,
    best_global_index: usize,
    stop_criteria: &'a mut dyn StopCriteria,
    local_attraction: f64,
    global_attraction: f64,
    min: f64,
    max: f64,
    inertia: f64,
    rng: ThreadRng,
    points_distribution: Uniform<f64>,
    savers: Vec<&'a mut dyn Saver<Particle>>,
}

impl<'a> ParticleSwarm<'a> {
    pub fn new(size: usize, min: f64, max: f64, stop_criteria: &'a mut dyn StopCriteria) -> Self {
        let rng = thread_rng();
        let distribution = Uniform::new_inclusive(min, max);
        Self {
            particles: Vec::with_capacity(size),
            best_global_index: 0,
            stop_criteria,
            global_attraction: 0.5,
            local_attraction: 0.5,
            inertia: 0.05,
            rng,
            min,
            max,
            points_distribution: distribution,
            savers: vec![],
        }
    }

    pub fn reset(&mut self, min: f64, max: f64) {
        self.stop_criteria.reset();
        self.min = min;
        self.max = max;

        let rng = thread_rng();
        self.rng = rng;

        let distribution = Uniform::new_inclusive(min, max);
        self.points_distribution = distribution;
        self.best_global_index = 0;

        for saver in &mut self.savers {
            saver.reset();
        }
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

    fn initialize(&mut self, criterion: &mut Criterion<Particle>) {
        self.particles.clear();

        for i in 0..self.particles.capacity() {
            // Pick random position in search space.
            let mut particle = Particle {
                best_local_index: i,
                velocity_x: self.rng.gen(),
                velocity_y: self.rng.gen(),
                x: self.points_distribution.sample(&mut self.rng),
                y: self.points_distribution.sample(&mut self.rng),
                eval: Evaluation::default(),
            };

            criterion.evaluate(&mut particle);
            self.particles.push(particle);

            //If current particle is better then best save that info
            if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                self.best_global_index = i;
            }
        }
    }
}

impl<'b> OptAlgorithm<'b, Particle> for ParticleSwarm<'b> {
    fn solve(&mut self, criterion: &mut Criterion<Particle>) -> Particle {
        self.initialize(criterion);

        let best_value = self.particles[self.best_global_index].get_value();
        while !self.stop_criteria.should_stop(best_value) {
            for i in 0..self.particles.len() {
                for saver in &mut self.savers {
                    saver.save_element(&self.particles[i]);
                }
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
                particle.update_position(self.min, self.max);

                //Update best and local trackers
                if self.is_better(i, self.best_global_index, criterion.is_minimization) {
                    self.best_global_index = i;
                }
            }
        }
        for saver in &mut self.savers {
            saver.flush();
        }
        //@ Improvement: Do not clone
        self.particles[self.best_global_index].clone()
    }

    fn add_saver(&mut self, saver: &'b mut dyn Saver<Particle>) {
        self.savers.push(saver);
    }

    fn clear_savers(&mut self) {
        self.savers.clear();
    }
}
