use std::{
    fmt::Display,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use optima_macros::{solution_attr, DerivedSolution};
use optima_rust::{
    annealing::{coolers::QuadraticCooler, stop::MaxSteps, SimulatedAnnealing},
    base::{Criterion, Evaluation, OptAlgorithm, Problem, Solution},
};
use rand::{prelude::ThreadRng, thread_rng, Rng};

#[derive(Clone)]
struct TspProblem {
    pub best_known: Option<f64>,
    pub distances: Vec<Vec<f64>>,
}
impl TspProblem {
    pub fn random(n: usize) -> Self {
        let mut distances = vec![vec![0.0; n]; n];
        let mut rng = thread_rng();

        for i in 0..n {
            for j in i + 1..n {
                distances[i][j] = 10.0 * rng.gen::<f64>();
                distances[j][i] = distances[i][j];
            }
            distances[i][i] = 0.0;
        }

        Self {
            distances,
            best_known: None,
        }
    }

    pub fn file<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path).expect("Failed to read TSP file");
        let lines: Vec<&str> = content.lines().collect();

        let best_known: f64 = lines[0]
            .parse()
            .expect("Failed to parse best known value as f32");

        let mut distances: Vec<Vec<f64>> = vec![];
        let mut index = 0;
        for i in 1..lines.len() {
            let line = lines[i];
            if line == "" {
                continue;
            }

            let values: Vec<&str> = line.split(" ").collect();

            distances.push(vec![]);
            for v in values {
                distances[index].push(v.parse().unwrap());
            }
            index += 1;
        }

        Self {
            distances,
            best_known: Some(best_known),
        }
    }
}

impl Display for TspProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from("");
        for row in &self.distances {
            for cell in row {
                output += &format!("{:.3} ", cell);
            }
            output += "\n";
        }
        write!(f, "{}", output)
    }
}

impl Problem for TspProblem {}

#[solution_attr]
#[derive(DerivedSolution, Clone)]
struct TspSolution {
    pub rout: Vec<usize>,
}

impl TspSolution {
    pub fn linear_rout(problem: &TspProblem) -> Self {
        let mut rout = vec![];

        for i in 0..problem.distances[0].len() {
            rout.push(i);
        }

        Self {
            rout,
            eval: Evaluation::default(),
        }
    }
}

fn change(sol: &mut TspSolution, _problem: &TspProblem, rng: &mut ThreadRng) {
    let first_random_index = rng.gen_range(0..sol.rout.len());
    let second_random_index = rng.gen_range(0..sol.rout.len());
    let first_element = sol.rout[first_random_index];

    if second_random_index < first_random_index {
        let mut i = first_random_index;
        while i > second_random_index {
            sol.rout[i] = sol.rout[i - 1];
            i -= 1;
        }
        sol.rout[second_random_index] = first_element;
    } else if second_random_index > first_random_index {
        for i in first_random_index..second_random_index {
            sol.rout[i] = sol.rout[i + 1];
        }
        sol.rout[second_random_index] = first_element;
    }
}

fn penalty(_problem: &TspProblem, _solution: &TspSolution) -> f64 {
    0.0
}

fn value(problem: &TspProblem, solution: &TspSolution) -> f64 {
    let mut total_distance = 0.0;
    for i in 0..solution.rout.len() - 1 {
        let from_index = solution.rout[i];
        let to_index = solution.rout[i + 1];
        total_distance += problem.distances[from_index][to_index];
    }
    total_distance
}

fn main() {
    let problem = TspProblem::random(100);

    println!("{}", problem);

    let max_steps = MaxSteps::new(20000);
    let cooler = QuadraticCooler::new(1000.0, 0.997);

    let initial_solution = TspSolution::linear_rout(&problem);

    let mut criterion = Criterion::new(&penalty, &value, true);
    let mut annealing = SimulatedAnnealing::new(&initial_solution, max_steps, cooler, &change);

    let solutions = annealing.solve(problem.clone(), &mut criterion);

    println!(
        "{} {:.3}",
        solutions[0].get_eval().is_feasible,
        solutions[0].get_value()
    );

    if problem.best_known.is_some() {
        println!(
            "Diff between known best and actual solution: {:.3}",
            solutions[0].get_eval().value - problem.best_known.unwrap(),
        );
    }
}
