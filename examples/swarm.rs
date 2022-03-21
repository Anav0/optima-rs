use optima_rust::{
    annealing::stop::MaxSteps,
    base::{Criterion, Evaluation, OptAlgorithm, Solution},
    swarm::{Particle, ParticleSwarm},
};
use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    thread_rng, Rng,
};

fn main() {
    let min = -10.0;
    let max = 10.0;

    let mut rng = thread_rng();
    let uniform = Uniform::new_inclusive(min, max);
    let mut stop_criteria = MaxSteps::new(50000);
    let mut swarm = ParticleSwarm::new(10, min, max, &mut stop_criteria);

    //@Info: For this function best value is 2594 (when x and y is limited to range -10..10).
    let mut criterion = Criterion::new(&|_| 0.0, &|part: &Particle| booth(part.x, part.y), true);

    let best = swarm.solve(&mut criterion);
    println!(
        "Best found: booth({},{} = {})",
        best.x,
        best.y,
        best.get_value()
    );
}

//Function to optimize. Booth function
//https://en.wikipedia.org/wiki/Test_functions_for_optimization
fn booth(x: f64, y: f64) -> f64 {
    f64::powf(x + 2.0 * y - 7.0, 2.0) + f64::powf(2.0 * x + y - 5.0, 2.0)
}

fn bukin(x: f64, y: f64) -> f64 {
    100.0 * (y - 0.01 * x.powf(2.0)).sqrt() + 0.001 * (x + 10.0).abs()
}

fn cormick(x: f64, y: f64) -> f64 {
    (x + y).sin() + (x - y).powf(2.0) - 1.5 * x + 2.5 * y + 1.0
}
