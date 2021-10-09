use optima_rust::{
    algorithms::OptAlghorithm,
    annealing::SimmulatedAnnealing,
    base::{Solution, SolutionInfo, State, StateChanger},
    constraints::{Constraint, WeightedConstraints},
    coolers::QuadriaticCooler,
    criterion::Criterion,
    objectives::{Objective, WeightedObjectives},
    stop::NotGettingBetter,
};
use rand::{thread_rng, Rng};

#[derive(Debug, Clone)]
struct KnapsackState {
    pub info: SolutionInfo,
    pub picked_items: Vec<bool>,
}

impl KnapsackState {
    pub fn new(info: SolutionInfo, picked_items: Vec<bool>) -> Self {
        Self { info, picked_items }
    }
}

#[derive(Debug)]
struct KnapsackSolution {
    current_state: KnapsackState,
    before_state: KnapsackState,
    best_state: KnapsackState,
}

impl KnapsackSolution {
    pub fn new(picked_items: Vec<bool>) -> Self {
        Self {
            current_state: KnapsackState::new(SolutionInfo::default(), picked_items.clone()),
            before_state: KnapsackState::new(SolutionInfo::default(), picked_items.clone()),
            best_state: KnapsackState::new(SolutionInfo::default(), picked_items),
        }
    }
    pub fn get_state(&self, state: State) -> &KnapsackState {
        match state {
            State::Current => &self.current_state,
            State::BeforeChange => &self.before_state,
            State::Best => &self.best_state,
        }
    }
    pub fn get_mut_state(&mut self) -> &mut KnapsackState {
        &mut self.current_state
    }
}

impl Solution for KnapsackSolution {
    fn reset(&mut self) {
        self.set_info(f64::NAN, false, true);
        self.update_before();
        self.update_best();
    }

    fn get_info(&self, state: State) -> &SolutionInfo {
        match state {
            State::Current => &self.current_state.info,
            State::Best => &self.best_state.info,
            State::BeforeChange => &self.best_state.info,
        }
    }

    fn set_info(&mut self, value: f64, is_feasible: bool, check_penalties: bool) {
        self.current_state.info.value = value;
        self.current_state.info.is_feasible = is_feasible;
        self.current_state.info.check_penalty = check_penalties;
    }

    //TODO: do not copy
    fn update_best(&mut self) {
        self.best_state = self.current_state.clone();
    }

    fn update_current(&mut self) {
        self.current_state = self.before_state.clone();
    }

    fn update_before(&mut self) {
        self.before_state = self.current_state.clone();
    }
}

struct KnapsackInstance<'a, const LENGTH: usize> {
    pub weights: &'a [f64; LENGTH],
    pub values: &'a [f64; LENGTH],
    pub capacity: usize,
}

impl<'a, const LENGTH: usize> KnapsackInstance<'a, LENGTH> {
    pub fn new(weights: &'a [f64; LENGTH], values: &'a [f64; LENGTH], capacity: usize) -> Self {
        Self {
            weights,
            values,
            capacity,
        }
    }
    pub fn as_solution(&self) -> KnapsackSolution {
        KnapsackSolution::new(vec![true; LENGTH])
    }
}

struct KnapsackConstraint<'a, const LENGTH: usize> {
    instance: &'a KnapsackInstance<'a, LENGTH>,
}

impl<'a, const LENGTH: usize> KnapsackConstraint<'a, LENGTH> {
    pub fn new(instance: &'a KnapsackInstance<'a, LENGTH>) -> Self {
        Self { instance }
    }
}

impl<'a, const LENGTH: usize> Constraint<KnapsackSolution> for KnapsackConstraint<'a, LENGTH> {
    fn penalty(&mut self, solution: &KnapsackSolution) -> f64 {
        let state = solution.get_state(State::Current);
        let mut total_weight = 0.0;
        for i in 0..self.instance.weights.len() {
            let bool_as_number: i8 = state.picked_items[i].into();
            total_weight += bool_as_number as f64 * self.instance.weights[i];
        }
        if total_weight > self.instance.capacity as f64 {
            total_weight - self.instance.capacity as f64
        } else {
            0.0
        }
    }
}

struct KnapsackObjective<'a, const LENGTH: usize> {
    instance: &'a KnapsackInstance<'a, LENGTH>,
}

impl<'a, const LENGTH: usize> KnapsackObjective<'a, LENGTH> {
    pub fn new(instance: &'a KnapsackInstance<'a, LENGTH>) -> Self {
        Self { instance }
    }
}

impl<'a, const LENGTH: usize> Objective for KnapsackObjective<'a, LENGTH> {
    type SolutionType = KnapsackSolution;

    fn value(&mut self, solution: &mut Self::SolutionType) -> f64 {
        let state = solution.get_state(State::Current);
        let mut total_value = 0.0;
        for i in 0..self.instance.weights.len() {
            let bool_as_number: i8 = state.picked_items[i].into();
            total_value += bool_as_number as f64 * self.instance.values[i];
        }
        total_value
    }
}

struct KnapsackMove;
impl Default for KnapsackMove {
    fn default() -> Self {
        Self {}
    }
}

impl<'a> StateChanger for KnapsackMove {
    type SolutionType = KnapsackSolution;

    fn change_state(&mut self, solution: &mut Self::SolutionType) {
        let state = solution.get_mut_state();
        let mut rng = thread_rng();

        let random_index = rng.gen_range(0..state.picked_items.len());
        state.picked_items[random_index] = !state.picked_items[random_index];
    }
}

fn main() {
    let instance = KnapsackInstance::new(&[5.0, 5.0, 5.0, 5.0], &[1.0, 1.0, 1.0, 1.0], 12);
    let mut solution = instance.as_solution();
    let mut mover = KnapsackMove::default();

    let objective = KnapsackObjective::new(&instance);
    let mut objectives_aggr = WeightedObjectives::new(vec![Box::new(objective)], &[1.0]);

    let constraint = KnapsackConstraint::new(&instance);
    let mut constraints_aggr = WeightedConstraints::new(vec![Box::new(constraint)], &[1.0]);

    let mut stop_criteria = NotGettingBetter::new(500, 500, false);
    let mut cooler = QuadriaticCooler::new(800.0, 0.998);
    let mut sa = SimmulatedAnnealing::new(&mut stop_criteria, &mut cooler);
    let mut criterion = Criterion::new(&mut constraints_aggr, &mut objectives_aggr, false);

    sa.solve(&mut solution, &mut criterion, &mut mover);

    let best = solution.get_state(State::Best);

    println!("{:?}", best);

    if !best.info.is_feasible {
        panic!("No feasible solution found")
    }
}
