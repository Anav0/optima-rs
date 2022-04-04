use rand::{distributions::Uniform, prelude::ThreadRng, thread_rng, Rng};

use crate::base::Solution;

pub fn tournament<S>(
    id: u32,
    population: &Vec<S>,
    is_minimization: bool,
    cross: &dyn Fn(u32, &S, &S, &mut ThreadRng) -> [S; 2],
    rng: &mut ThreadRng,
) -> [S; 2]
where
    S: Solution,
{
    let dist = Uniform::new(0, population.len());
    let mut parents: Vec<&S> = Vec::with_capacity(2);

    for _ in 0..2 {
        let knights = thread_rng().sample_iter(&dist).take(2).collect::<Vec<_>>();

        let knight_a_eval = population[knights[0]].get_eval();
        let knight_b_eval = population[knights[1]].get_eval();

        if is_minimization {
            if knight_a_eval.value < knight_b_eval.value {
                parents.push(&population[knights[0]]);
            } else {
                parents.push(&population[knights[1]]);
            }
        } else {
            if knight_a_eval.value > knight_b_eval.value {
                parents.push(&population[knights[0]]);
            } else {
                parents.push(&population[knights[1]]);
            }
        }
    }

    cross(id, parents[0], parents[1], rng)
}
