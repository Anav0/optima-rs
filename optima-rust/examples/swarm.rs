use optima_rust::{
    analysis::{CsvSaver, Saver},
    annealing::stop::{MaxSteps, StopCriteria},
    base::{Criterion, OptAlgorithm, Solution},
    swarm::{FnProblem, Particle, ParticleSwarm},
};

type MathFunction = dyn Fn(f64, f64) -> f64;
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
    let bukin_bench = FnBench {
        global_minimum: (-10.0, 1.0, 0.0),
        max: 10.0,
        min: -10.0,
        name: String::from("bukin"),
        func: &bukin,
    };
    let cromick_bench = FnBench {
        global_minimum: (-0.54719, -1.54719, -1.91),
        max: 10.0,
        min: -10.0,
        name: String::from("cromic"),
        func: &cormick,
    };

    let benches = vec![booth_bench, bukin_bench, cromick_bench];

    let stop_criteria = MaxSteps::new(10000);
    let mut swarm = ParticleSwarm::new(40, stop_criteria);

    let mut csv = CsvSaver::new(
        String::from("./test.csv"),
        String::from("iter,x,y,velocity_x,velocity_y,best_local,value\n"),
    );

    swarm.add_saver(&mut csv);

    for bench in benches {
        let problem = FnProblem::new(0, bench.max, bench.min);

        let value_fn = |part: &Particle| (bench.func)(part.x, part.y);

        let mut criterion = Criterion::new(&|_| 0.0, &value_fn, true);

        let best = swarm.solve(problem, &mut criterion);
        println!(
            "{}({}, {} = {}), Known smallest value: {}({}, {}) = {}",
            bench.name,
            best.x,
            best.y,
            best.get_value(),
            bench.name,
            bench.global_minimum.0,
            bench.global_minimum.1,
            bench.global_minimum.2,
        );
    }
}

//Some function to optimize.
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
