use optima_rust::{
    annealing::stop::MaxSteps,
    base::{Criterion, OptAlgorithm, Solution},
    swarm::{FnProblem, Particle, ParticleSwarm},
};

pub type MathFunction = dyn Fn(f64, f64) -> f64;

struct FnBench<'a> {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub global_minimum: (f64, f64, f64),
    pub func: &'a MathFunction,
}

fn main() {
    let booth_bench = FnBench {
        global_minimum: (1.0, 3.0, 0.0),
        max: 10.0,
        min: -10.0,
        name: String::from("booth"),
        func: &booth,
    };

    let cromick_bench = FnBench {
        global_minimum: (-0.54719, -1.54719, -1.9133),
        max: 4.0,
        min: -3.0,
        name: String::from("cormick"),
        func: &cormick,
    };

    let benches = vec![booth_bench, cromick_bench];

    let stop_criteria = MaxSteps::new(6000);
    let mut swarm = ParticleSwarm::new(3500, stop_criteria);

    let longest_fn = benches
        .iter()
        .max_by(|a, b| a.name.len().cmp(&b.name.len()))
        .unwrap();

    for bench in benches {
        let problem = FnProblem::new(0, bench.max, bench.min);

        let value_fn = |_problem: &FnProblem, part: &Particle| (bench.func)(part.x, part.y);

        let mut criterion = Criterion::new(&|_, _| 0.0, &value_fn, true);

        let particles = swarm.solve(problem, &mut criterion);
        let best = &particles[0];

        // let padding = longest_fn.name.len() - bench.name.len();
        println!(
            "{:<8}({:+.3}, {:+.3}) = {:+.3} | Smallest known value: {:>8}({:+.3}, {:+.3}) = {:+.3} | Delta: {:+.3}",
            bench.name,
            best.x,
            best.y,
            best.get_value(),
            bench.name,
            bench.global_minimum.0,
            bench.global_minimum.1,
            bench.global_minimum.2,
            best.get_value() - bench.global_minimum.2 
        );
    }
}

//Some function to optimize.
//https://en.wikipedia.org/wiki/Test_functions_for_optimization
fn booth(x: f64, y: f64) -> f64 {
    f64::powf(x + 2.0 * y - 7.0, 2.0) + f64::powf(2.0 * x + y - 5.0, 2.0)
}

fn cormick(x: f64, y: f64) -> f64 {
    (x + y).sin() + (x - y).powf(2.0) - 1.5 * x + 2.5 * y + 1.0
}
