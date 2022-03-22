use optima_rust::annealing::{
    coolers::QuadraticCooler, stop::NotGettingBetter, SimulatedAnnealing,
};
use optima_rust::base::{Criterion, Evaluation, OptAlgorithm, Solution};

use optima_rust::genetic::crossover::tournament;
use optima_rust::genetic::GeneticAlgorithm;
use rand::prelude::ThreadRng;
use rand::{random, thread_rng, Rng};

// =====================================================
// === Solution for annealing and genetic algorithms ===
// =====================================================
#[derive(Clone, Debug)]
pub struct KnapsackSolution {
    pub info: Evaluation,
    pub picked_items: Vec<bool>,
}

impl KnapsackSolution {
    pub fn new(picked_items: Vec<bool>) -> Self {
        Self {
            info: Evaluation::default(),
            picked_items,
        }
    }
    pub fn random_init(size: usize) -> Self {
        let mut picked_items = Vec::with_capacity(size);
        for _ in 0..picked_items.capacity() {
            picked_items.push(random::<bool>());
        }
        Self {
            info: Evaluation::default(),
            picked_items,
        }
    }
}

impl Solution for KnapsackSolution {
    fn get_eval(&self) -> &Evaluation {
        &self.info
    }
    fn get_eval_mut(&mut self) -> &mut Evaluation {
        &mut self.info
    }
}

pub fn value(values: &Vec<f64>, current: &KnapsackSolution) -> f64 {
    let mut total_value = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_value += bool_as_number as f64 * values[i];
    }
    total_value
}

pub fn penalty(capacity: f64, weights: &Vec<f64>, current: &KnapsackSolution) -> f64 {
    let mut total_weight = 0.0;
    for i in 0..current.picked_items.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_weight += bool_as_number as f64 * weights[i];
    }
    if total_weight > capacity {
        capacity - total_weight
    } else {
        0.0
    }
}

pub fn mutate(specimen: &mut KnapsackSolution, mutate_rate: f64) {
    for i in 0..specimen.picked_items.len() {
        if random::<f64>() < mutate_rate {
            specimen.picked_items[i] = !specimen.picked_items[i];
        }
    }
}

fn point_crossover(
    father: &KnapsackSolution,
    mather: &KnapsackSolution,
    rng: &mut ThreadRng,
) -> [KnapsackSolution; 2] {
    let cross_point = rng.gen_range(0..father.picked_items.len()); //TODO: Guard from picking point at 0 or at max length

    let from_father = father.picked_items[..cross_point].to_vec();
    let from_mather = mather.picked_items[cross_point..].to_vec();

    let child_a_picked_items = vec![from_father, from_mather].concat();

    let from_mather = mather.picked_items[..cross_point].to_vec();
    let from_father = father.picked_items[cross_point..].to_vec();

    let child_b_picked_items = vec![from_mather, from_father].concat();

    let child_a = KnapsackSolution::new(child_a_picked_items);
    let child_b = KnapsackSolution::new(child_b_picked_items);

    [child_a, child_b]
}

fn tournament_wrapper(
    population: &Vec<KnapsackSolution>,
    rng: &mut ThreadRng,
) -> [KnapsackSolution; 2] {
    tournament(population, false, &point_crossover, rng)
}

fn mutate_wrapper(solution: &mut KnapsackSolution) {
    mutate(solution, 0.25);
}

fn main() {
    //BASIC PARAMETERS
    let weights = vec![1.0, 2.0, 3.0, 8.0, 12.0, 20.0, 30.0];
    let values = vec![4.0, 5.0, 1.0, 2.0, 8.0, 5.0, 6.0];
    let capacity = 6.0;

    // Solution and population of solutions
    let solution = KnapsackSolution::new(vec![true; values.len()]);
    let population = vec![KnapsackSolution::random_init(values.len()); values.len() * 2];

    // Algorithms used to solve knapsack problem
    let mut stop_criteria = NotGettingBetter::new(500, 500, false);
    let mut cooler = QuadraticCooler::new(800.0, 0.998);
    let mut annealing =
        SimulatedAnnealing::new(solution, &mut stop_criteria, &mut cooler, &|annealing| {
            let mut rng = thread_rng();
            let random_index = rng.gen_range(0..annealing.solution.picked_items.len());
            annealing.solution.picked_items[random_index] =
                !annealing.solution.picked_items[random_index];
        });

    let mut genetic =
        GeneticAlgorithm::new(population, &mutate_wrapper, &tournament_wrapper, 500, None);

    let value_closure: &dyn Fn(&KnapsackSolution) -> f64 = &|current| value(&values, current);
    let penalty_closure: &dyn Fn(&KnapsackSolution) -> f64 =
        &|current| penalty(capacity, &weights, current);

    let mut criterion = Criterion::new(penalty_closure, value_closure, false);

    println!("{:?}", annealing.solve(&mut criterion));
    println!("{:?}", genetic.solve(&mut criterion));
}

#[cfg(test)]
mod tests {
    use crate::{penalty, value, KnapsackSolution};

    #[test]
    fn value_works() {
        let values = vec![1.0, 2.0, 3.0];
        let taken = vec![true, true, false];

        let mut expected_value = 0.0;

        for i in 0..values.len() {
            let taken_as_i8: i8 = taken[i].into();
            expected_value += taken_as_i8 as f64 * values[i];
        }

        let current = KnapsackSolution::new(taken);
        assert_eq!(expected_value, value(&values, &current));
    }

    #[test]
    fn penalty_works() {
        let weights = vec![1.0, 2.0, 10.0];
        let taken = vec![true, true, true];
        let capacity = 3.0;

        let current = KnapsackSolution::new(taken);
        assert_eq!(-8.0, penalty(capacity, &weights, &current));
    }
}
