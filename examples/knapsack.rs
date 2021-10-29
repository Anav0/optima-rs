use optima_rust::{
    annealing::{coolers::QuadriaticCooler, stop::NotGettingBetter, SimmulatedAnnealing},
    base::{InfoHolder, OptAlgorithm, Solution, SolutionInfo, State},
    criterion::Criterion,
};
use rand::{thread_rng, Rng};

#[derive(Debug, Clone)]
struct KnapsackState {
    pub info: SolutionInfo,
    pub picked_items: Vec<bool>,
}

impl KnapsackState {
    pub fn new(picked_items: Vec<bool>) -> Self {
        Self {
            info: SolutionInfo::default(),
            picked_items,
        }
    }
}

impl InfoHolder for KnapsackState {
    fn get_info(&self) -> &SolutionInfo {
        &self.info
    }
    fn get_info_mut(&mut self) -> &mut SolutionInfo {
        &mut self.info
    }
}

fn value(values: &Vec<f64>, current: &KnapsackState) -> f64 {
    let mut total_value = 0.0;
    for i in 0..values.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_value += bool_as_number as f64 * values[i];
    }
    total_value
}

fn penalty(capacity: f64, weights: &Vec<f64>, current: &KnapsackState) -> f64 {
    let mut total_weight = 0.0;
    for i in 0..weights.len() {
        let bool_as_number: i8 = current.picked_items[i].into();
        total_weight += bool_as_number as f64 * weights[i];
    }
    if total_weight > capacity {
        total_weight - capacity
    } else {
        0.0
    }
}

fn main() {
    //BASIC PARAMETERS
    let weights = vec![1.0, 2.0, 3.0];
    let values = vec![4.0, 5.0, 1.0];
    let capacity = 4.0; //Expect: 3

    let mut stop_criteria = NotGettingBetter::new(500, 500, false);
    let mut cooler = QuadriaticCooler::new(800.0, 0.998);
    let mut sa = SimmulatedAnnealing::new(&mut stop_criteria, &mut cooler);

    let value_closure: &dyn Fn(&KnapsackState) -> f64 = &|current| value(&values, current);
    let penalty_closure: &dyn Fn(&KnapsackState) -> f64 =
        &|current| penalty(capacity, &weights, current);

    let mut criterion: Criterion<KnapsackState> =
        Criterion::new(value_closure, penalty_closure, false);
    let initial_state = KnapsackState::new(vec![true; values.len()]);
    let mut solution = Solution::new(initial_state);

    sa.solve(&mut solution, &mut criterion, &|current| {
        let mut rng = thread_rng();

        let random_index = rng.gen_range(0..current.picked_items.len());
        current.picked_items[random_index] = !current.picked_items[random_index];
    });

    println!("{:?}", solution.get_state_ref(State::Best));
}

#[cfg(test)]
mod tests {
    use crate::{penalty, value, KnapsackState};

    #[test]
    fn value_works() {
        let values = vec![1.0, 2.0, 3.0];
        let taken = vec![true, true, false];

        let mut expected_value = 0.0;

        for i in 0..values.len() {
            let taken_as_i8: i8 = taken[i].into();
            expected_value += taken_as_i8 as f64 * values[i];
        }

        let current = KnapsackState::new(taken);
        assert_eq!(expected_value, value(&values, &current));
    }

    #[test]
    fn penalty_works() {
        let weights = vec![1.0, 2.0, 10.0];
        let taken = vec![true, true, true];
        let capacity = 3.0;

        let mut expected_penalty = 0.0;

        for i in 0..weights.len() {
            let taken_as_i8: i8 = taken[i].into();
            expected_penalty += taken_as_i8 as f64 * weights[i];
        }

        expected_penalty -= capacity;

        let current = KnapsackState::new(taken);
        assert_eq!(expected_penalty, penalty(capacity, &weights, &current));
    }
}
