use std::cmp::Ordering;

pub use crate::genetic::Speciment;

pub trait Evaluator<S: Speciment + Clone> {
    fn get_scores_mut(&mut self) -> &mut Vec<(usize, f64)>;
    fn evaluate(&mut self, population: &mut Vec<S>) {
        let scores = self.get_scores_mut();
        for i in 0..population.len() {
            scores[i].0 = i;
            scores[i].1 = population[i].score();
        }
    }
    fn extract_best(&mut self, best: &mut Vec<S>, population: &Vec<S>, is_minimalization: bool) {
        let scores = self.get_scores_mut();
        if is_minimalization {
            scores.sort_by(|a, b| {
                if a.1 < b.1 {
                    return Ordering::Less;
                }

                if a.1 > b.1 {
                    return Ordering::Greater;
                }

                Ordering::Equal
            });
        } else {
            scores.sort_by(|a, b| {
                if a.1 > b.1 {
                    return Ordering::Less;
                }

                if a.1 < b.1 {
                    return Ordering::Greater;
                }

                Ordering::Equal
            });
        }

        for i in 0..best.len() {
            best[i] = population[scores[i].0].clone();
        }
    }
}

pub struct DefaultEvaluator {
    scores: Vec<(usize, f64)>,
}

impl DefaultEvaluator {
    pub fn new(population_size: usize) -> Self {
        Self {
            scores: vec![(0, 0.0); population_size],
        }
    }
}

impl<S: Speciment + Clone> Evaluator<S> for DefaultEvaluator {
    fn get_scores_mut(&mut self) -> &mut Vec<(usize, f64)> {
        &mut self.scores
    }
}
