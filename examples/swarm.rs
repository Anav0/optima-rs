use optima_rust::{
    annealing::stop::MaxSteps,
    base::{Criterion, Evaluation, OptAlgorithm, Solution},
    swarm::{ParticleSwarm, Point},
};
use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

fn main() {
    let min = -10.0;
    let max = 10.0;

    let initial_sol = FunctionSolution::new(min, max);
    let mut stop_criteria = MaxSteps::new(1000);
    let mut swarm = ParticleSwarm::new(10, initial_sol, &mut stop_criteria);

    let mut criterion = Criterion::new(
        &|_| 0.0,
        &|sol: &FunctionSolution| booth(sol.x, sol.y),
        false,
    );

    let best = swarm.solve(&mut criterion);
    println!(
        "Best found: booth({},{} = {})",
        best.x,
        best.y,
        best.get_eval().value
    );
}

#[derive(Debug, Copy, Clone)]
struct FunctionSolution {
    eval: Evaluation,
    min: f64,
    max: f64,
    x: f64,
    y: f64,
}
impl FunctionSolution {
    pub fn new(min: f64, max: f64) -> Self {
        let mut rng = thread_rng();
        let uniform = Uniform::new_inclusive(min, max);
        Self {
            eval: Evaluation::default(),
            max,
            min,
            x: uniform.sample(&mut rng),
            y: uniform.sample(&mut rng),
        }
    }
}
impl Solution for FunctionSolution {
    fn get_eval(&self) -> &optima_rust::base::Evaluation {
        &self.eval
    }

    fn get_eval_mut(&mut self) -> &mut optima_rust::base::Evaluation {
        &mut self.eval
    }
}

impl Point for FunctionSolution {
    fn subtract(&self, other: &Self) -> f64 {
        (self.x - other.x) + (self.y - self.y)
    }

    fn update(&mut self, velocity: f64) {
        self.x += velocity;
        self.y += velocity;

        if self.x > self.max {
            self.x = self.max;
        }
        if self.x < self.min {
            self.x = self.min;
        }

        if self.y > self.max {
            self.y = self.max;
        }
        if self.y < self.min {
            self.y = self.min;
        }
    }

    fn new_random(&self) -> Self {
        let mut rng = thread_rng();
        let uniform = Uniform::new_inclusive(self.min, self.max);

        let mut eval = self.eval.clone();
        eval.value = 0.0;
        eval.is_feasible = false;

        Self {
            eval,
            min: self.min,
            max: self.max,
            x: uniform.sample(&mut rng),
            y: uniform.sample(&mut rng),
        }
    }
}

//Function to optimize. Booth function
//https://en.wikipedia.org/wiki/Test_functions_for_optimization
fn booth(x: f64, y: f64) -> f64 {
    f64::powf(x + 2.0 * y - 7.2, 2.0) + f64::powf(2.0 * x + y - 5.2, 2.0)
}
