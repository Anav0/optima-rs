use std::fmt::Display;

use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::{Distribution, ThreadRng},
};

use crate::base::Solution;

use super::GeneticAlgorithm;

pub fn roulette<S: Solution>(
    population: &Vec<S>,
    is_minimization: bool,
    rng: &mut ThreadRng,
) -> Vec<S> {
    let mut new_population = Vec::with_capacity(population.len());
    let mut weights = Vec::with_capacity(population.len());

    let mut sum = 0.0;
    for specimen in population {
        let eval = specimen.get_eval();
        if !eval.is_feasible {
            continue;
        }
        sum += eval.value;
    }

    for specimen in population {
        let eval = specimen.get_eval();
        if eval.is_feasible {
            let mut el = eval.value / sum;
            if is_minimization {
                el = 1.0 - el;
            }
            let e = population.len() as f64 * el;
            let integer = e.round() as usize;
            for _ in 0..integer {
                new_population.push(specimen.clone());
            }
            let fraction = e.fract();
            weights.push(fraction);
        } else {
            //@ Info: We ignore unfeasible solutions
            weights.push(0.0);
        }
    }
    let uniform = WeightedIndex::new(vec![1.0; population.len()]).unwrap();
    let dist = WeightedIndex::new(&weights).unwrap_or(uniform);
    while new_population.len() < population.len() {
        let index = dist.sample(rng);
        let specimen = population[index].clone();
        new_population.push(specimen);
    }

    new_population
}

pub fn tournament<S: Solution>(
    tournament_size: u16,
    population: &Vec<S>,
    is_minimization: bool,
    rng: &mut ThreadRng,
    keep_elite: u8,
) -> Vec<S> {
    let mut new_population = Vec::with_capacity(population.len());
    let dist = Uniform::new(0, population.len());

    while new_population.len() < population.len() {
        let mut best_knight_index = 0;
        let mut best_knight_value = population[0].get_value();
        //Tournament begins
        for _ in 0..tournament_size {
            let opponent_index = dist.sample(rng);
            let opponent_value = population[opponent_index].get_value();

            if is_minimization {
                if opponent_value < best_knight_value {
                    best_knight_value = opponent_value;
                    best_knight_index = opponent_index;
                }
            } else {
                if opponent_value > best_knight_value {
                    best_knight_value = opponent_value;
                    best_knight_index = opponent_index;
                }
            }
        }
        new_population.push(population[best_knight_index].clone());
    }

    new_population
}

