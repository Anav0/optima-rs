use std::{collections::HashMap, thread::Thread};

use rand::{distributions::weighted, prelude::ThreadRng, thread_rng, Rng};

use crate::KnapsackProblem;

#[derive(Debug)]
pub enum Generator {
    Uncorrelated,
    WeaklyCorrelated,
    StronglyCorrelated,
    InverseStrong,
    SimilarWeight,
    SubsetSum,
    AlmostStrong,
}

fn uncorrelated_instance<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(1.0..=R);
        values[i] = rng.gen_range(1.0..=R);
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

fn strongly_correlated<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(1.0..=R);
        values[i] = weights[i] + R / 10.0;
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

fn subsetsum<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(1.0..=R);
        values[i] = weights[i];
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

fn uncorrelated_similar_weights<const LENGTH: usize>(
    name: String,
    _R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(100000.0..=100100.0);
        values[i] = rng.gen_range(1.0..=1000.0);
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

fn weakly_correlated<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        let x = rng.gen_range(1.0..=R);
        weights[i] = x;
        let min = x - R / 10.0;
        let max = x + R / 10.0;
        values[i] = rng.gen_range(min..=max);
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

fn inverse_strong<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(1.0..=R);
        values[i] = weights[i] + R / 10.0;
    }

    KnapsackProblem::new(name, weights, values, capacity)
}
fn almost_strong<const LENGTH: usize>(
    name: String,
    R: f64,
    capacity: f64,
    rng: &mut ThreadRng,
) -> KnapsackProblem<LENGTH> {
    let mut weights = [0.0; LENGTH];
    let mut values = [0.0; LENGTH];

    for i in 0..LENGTH {
        weights[i] = rng.gen_range(1.0..=R);
        values[i] =
            rng.gen_range(weights[i] + R * 10.0 - R / 500.0..=weights[i] + R / 10.0 + R / 500.0);
    }

    KnapsackProblem::new(name, weights, values, capacity)
}

pub type GeneratorFn<const LENGTH: usize> =
    dyn Fn(String, f64, f64, &mut ThreadRng) -> KnapsackProblem<LENGTH>;

pub struct KnapsackInstanceFactory<const LENGTH: usize> {
    n: usize,
    b: f64,
    how_many_problems: u32,
    problems: Vec<KnapsackProblem<LENGTH>>,
    rng: ThreadRng,
}

impl<const LENGTH: usize> KnapsackInstanceFactory<LENGTH> {
    pub fn new(n: usize, b: f64, how_many_problems: u32) -> Self {
        Self {
            n,
            b,
            how_many_problems,
            rng: thread_rng(),
            problems: Vec::with_capacity(how_many_problems as usize),
        }
    }
    fn get_generator<'a>(generator: Generator) -> &'a GeneratorFn<LENGTH> {
        let f: &GeneratorFn<LENGTH> = match generator {
            Generator::Uncorrelated => &uncorrelated_instance,
            Generator::WeaklyCorrelated => &weakly_correlated,
            Generator::StronglyCorrelated => &strongly_correlated,
            Generator::InverseStrong => &inverse_strong,
            Generator::SimilarWeight => &uncorrelated_similar_weights,
            Generator::SubsetSum => &subsetsum,
            Generator::AlmostStrong => &almost_strong,
        };
        f
    }
    pub fn generate_distribution_problem(
        &mut self,
        generator: Generator,
        coefficient: f64,
    ) -> &mut Self {
        let R = f64::powf(10.0, coefficient);

        let name = format!("{:?}", generator);
        let f = KnapsackInstanceFactory::get_generator(generator);
        for _ in 0..self.how_many_problems {
            let problem = f(name.clone(), R, self.b, &mut self.rng);
            self.problems.push(problem);
        }
        self
    }
    pub fn generate_spanner_problems(
        &mut self,
        generator: Generator,
        coefficient: f64,
        v: usize,
        m: f64,
    ) -> &mut Self {
        let R = f64::powf(10.0, coefficient);

        let name = format!("{:?}", generator);
        let f = KnapsackInstanceFactory::get_generator(generator);

        for _ in 0..self.how_many_problems {
            let mut problem = f(name.clone(), R, self.b, &mut self.rng);
            let mut spannerV = vec![0.0; v];
            let mut spannerW = vec![0.0; v];

            for k in 0..v {
                spannerW[k] = (self.rng.gen_range(1.0..=R) * 2.0) / m;
                spannerV[k] = (problem.values[k] * 2.0) / m;
            }
            for i in 0..self.n {
                let s = self.rng.gen_range(0..=v);
                let a = self.rng.gen_range(1.0..=m);
                problem.weights[i] = spannerW[s] * a;
                problem.values[i] = spannerV[s] * a;
            }
            self.problems.push(problem);
        }
        self
    }
    pub fn generate_mstr_problems(
        &mut self,
        generator: Generator,
        coefficient: f64,
        k1: f64,
        k2: f64,
        d: f64,
    ) -> &mut Self {
        let R = f64::powf(10.0, coefficient);

        let name = format!("{:?}", generator);
        let f = KnapsackInstanceFactory::get_generator(generator);

        assert_ne!(k1, k2);

        for _ in 0..self.how_many_problems {
            let mut problem = f(name.clone(), R, self.b, &mut self.rng);

            for i in 0..self.n {
                problem.weights[i] = self.rng.gen_range(1.0..=R);
                if problem.weights[i] % d == 0.0 {
                    problem.values[i] = problem.weights[i] + k1;
                } else {
                    problem.values[i] = problem.weights[i] + k2;
                }
            }

            self.problems.push(problem);
        }
        self
    }
    pub fn generate_pceil_problems(
        &mut self,
        generator: Generator,
        coefficient: f64,
        d: f64,
    ) -> &mut Self {
        let R = f64::powf(10.0, coefficient);

        let name = format!("{:?}", generator);
        let f = KnapsackInstanceFactory::get_generator(generator);

        for _ in 0..self.how_many_problems {
            let mut problem = f(name.clone(), R, self.b, &mut self.rng);

            for i in 0..self.n {
                problem.weights[i] = self.rng.gen_range(1.0..=R);
                problem.values[i] = d * (problem.weights[i] / d).abs();
            }

            self.problems.push(problem);
        }
        self
    }
    pub fn generate_circle_problems(
        &mut self,
        generator: Generator,
        coefficient: f64,
        d: f64,
    ) -> &mut Self {
        let R = f64::powf(10.0, coefficient);

        let name = format!("{:?}", generator);
        let f = KnapsackInstanceFactory::get_generator(generator);

        for _ in 0..self.how_many_problems {
            let mut problem = f(name.clone(), R, self.b, &mut self.rng);

            for i in 0..self.n {
                problem.weights[i] = self.rng.gen_range(1.0..=R);
                problem.values[i] =
                    d * f64::sqrt(4.0 * (R.powf(2.0) - (problem.weights[i] - 2.0 * R).powf(2.0)));
            }

            self.problems.push(problem);
        }
        self
    }
    pub fn collect(&mut self) -> Vec<KnapsackProblem<LENGTH>> {
        let tmp = self.problems.clone();

        self.problems.clear();

        tmp
    }
    pub fn change_parameters(&mut self, n: usize, b: f64, how_many_problems: u32) -> &mut Self {
        self.n = n;
        self.b = b;
        self.how_many_problems = how_many_problems;
        self
    }
}
