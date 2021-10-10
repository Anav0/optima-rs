pub use crate::evaluator::DefaultEvaluator;
pub use crate::evaluator::Evaluator;

pub trait Speciment {
    fn score(&self) -> f64;
}

pub trait Crosser<S: Speciment> {
    fn cross(&mut self, population: &mut Vec<S>, best: &Vec<S>);
}

pub trait Mutator<S: Speciment> {
    fn mutate(&mut self, population: &mut Vec<S>);
}

pub struct GeneticAlgorithm<'a, S: Speciment> {
    pub print_rate: u32,
    is_minimalization: bool,
    crosser: &'a mut dyn Crosser<S>,
    mutator: &'a mut dyn Mutator<S>,
    evaluator: &'a mut dyn Evaluator<S>,
}

impl<'b, 'a, S: Speciment + Clone> GeneticAlgorithm<'a, S> {
    pub fn is_minimalization(mut self) -> Self {
        self.is_minimalization = true;
        self
    }

    pub fn new(
        print_rate: u32,
        crosser: &'a mut dyn Crosser<S>,
        mutator: &'a mut dyn Mutator<S>,
        evaluator: &'a mut dyn Evaluator<S>,
    ) -> Self {
        Self {
            is_minimalization: false,
            print_rate,
            crosser,
            mutator,
            evaluator,
        }
    }

    pub fn evolve(&mut self, population: &mut Vec<S>, cycles: u32, best_number: u8) -> Vec<S> {
        if best_number == 0 {
            panic!("Best number must be greater than 0");
        }

        let mut best: Vec<S> = vec![population[0].clone(); best_number.into()];

        for i in 0..cycles {
            self.mutator.mutate(population);
            self.crosser.cross(population, &best);
            self.evaluator.evaluate(population);
            self.evaluator
                .extract_best(&mut best, population, self.is_minimalization);

            if i % self.print_rate == 0 {
                println!("Cycle {}", i);
            }
        }

        best
    }
}
