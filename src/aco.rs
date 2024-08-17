use crate::distance::DistancesIdx;
use crate::graph::GraphIdx;
use crate::kahan::KahanAdder;
use crate::reusable_weighted_index::CumulativeWeightsWrapper;
use crate::util::cycling;
use bitvec::bitvec;
use bitvec::vec::BitVec;
use lambert_w::lambert_w0;
use rand::distributions::Distribution;
use rand::{random, Rng};
use rand_pcg::Pcg64Mcg;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::borrow::Cow;
use std::f64;

const INIT_INTENSITY_MULTIPLIER: f64 = 10.0;
const MINIMAL_INTENSITY: f64 = 1e-5;

#[derive(Clone, Debug, PartialEq)]
pub struct Aco<'a> {
    size: u32,
    dist_idx: Cow<'a, DistancesIdx<'a>>,
    intensity: f64,
    q: f64,
    opt_dist: Option<f64>,
}

impl<'a> Aco<'a> {
    pub fn new(
        dist_idx: &'a DistancesIdx<'a>,
        intensity: Option<f64>,
        q: Option<f64>,
        opt_dist: Option<f64>,
    ) -> Self {
        let size = dist_idx.graph.size;

        let dist_idx = match opt_dist {
            Some(opt_dist) => {
                let a = eval_a(opt_dist);
                let recip_plank_law_ext = recip_plank_law_ext(opt_dist, a);
                Cow::Owned(dist_idx.transform(|v| plank_law(v, a, recip_plank_law_ext).recip()))
            }
            None => Cow::Borrowed(dist_idx),
        };

        let mean_dist = dist_idx.graph.triangle_sum() / (size * (size - 1) / 2) as f64;

        let q = match q {
            Some(q) => q,
            None if size > 1 => mean_dist,
            None => 1.0,
        };

        let intensity = match intensity {
            Some(intensity) => intensity,
            None if size > 1 => INIT_INTENSITY_MULTIPLIER * mean_dist,
            None => 0.0,
        };

        Self {
            size,
            dist_idx,
            intensity,
            q,
            opt_dist,
        }
    }

    pub fn aco(
        &self,
        iterations: u32,
        ants: u32,
        degradation_factor: f64,
        alpha: f64,
        beta: f64,
    ) -> (Vec<u32>, f64) {
        match self.size {
            0 => {
                return (vec![], 0.0);
            }
            1 => return (vec![0], 0.0),
            _ => {}
        };

        let mut best_cycle_dist: Option<(Vec<_>, f64)> = None;
        let mut intensities =
            GraphIdx::transform(&self.dist_idx.graph, |d| d.map(|_| self.intensity));
        let mut weights = GraphIdx::transform_const(&self.dist_idx.graph, None);

        let mut cycles = Vec::with_capacity(ants as usize + 1);

        for i in 0..iterations {
            self.dist_idx
                .graph
                .merge_parallel_into(&intensities, &mut weights, |dist, intensity| {
                    intensity.zip(dist).map(|(intensity, dist)| {
                        intensity.max(MINIMAL_INTENSITY).powf(alpha) / dist.powf(beta)
                    })
                })
                .unwrap_or_else(|| {
                    unreachable!(
                        "Mismatched graph sizes: {} vs {}",
                        self.dist_idx.graph.size, intensities.size
                    )
                });
            (0..ants)
                .into_par_iter()
                .map_init(
                    || {
                        (
                            Pcg64Mcg::new(random()),
                            bitvec![1; self.size as usize],
                            CumulativeWeightsWrapper::with_capacity(self.size as usize),
                        )
                    },
                    |(rng, not_visited, cumulative_weights_wrapper), _| loop {
                        if let Some((cycle, dist)) = self.traverse_graph(
                            None,
                            &weights,
                            rng,
                            not_visited,
                            cumulative_weights_wrapper,
                        ) {
                            if cycle.len() == self.size as usize {
                                break (cycle, dist);
                            }
                        }
                    },
                )
                .collect_into_vec(&mut cycles);
            if let Some(best_cycle_dist) = &best_cycle_dist {
                cycles.push(best_cycle_dist.clone());
            }
            cycles.par_sort_unstable_by(|(_, dist1), (_, dist2)| dist1.total_cmp(dist2));
            cycles.truncate((cycles.len() + 1) / 2);

            intensities.transform_inplace(|value| {
                if let Some(value) = value {
                    *value *= degradation_factor;
                }
            });

            for (cycle, distance) in cycles.drain(..) {
                let delta = self.q / distance;

                for (&node1, &node2) in cycling(&cycle) {
                    if let Some(intencity) =
                        intensities.between_mut(node1, node2).unwrap_or_else(|| {
                            unreachable!("No pheromones between {node1} and {node2}")
                        })
                    {
                        *intencity += delta;
                    }
                }

                match best_cycle_dist {
                    Some((_, best_distance)) if distance < best_distance => {
                        println!("New cycle: {cycle:?}, len: {distance:.06}, iteration: [{i}]");
                        best_cycle_dist = Some((cycle, distance));
                    }
                    None => {
                        println!("First cycle: {cycle:?}, len: {distance:.05}");
                        best_cycle_dist = Some((cycle, distance));
                    }
                    _ => {}
                }
            }
        }

        println!("Best cycle: {best_cycle_dist:?}");

        best_cycle_dist.unwrap_or_else(|| {
            #[allow(unreachable_code)]
            !unreachable!("best_cycle is None")
        })
    }

    fn traverse_graph(
        &self,
        source_node: Option<u32>,
        weights: &GraphIdx<Option<f64>>,
        rng: &mut impl Rng,
        not_visited: &mut BitVec,
        cumulative_weights_wrapper: &mut CumulativeWeightsWrapper<f64>,
    ) -> Option<(Vec<u32>, f64)> {
        match self.size {
            0 => return Some((vec![], 0.0)),
            1 => return Some((vec![0], 0.0)),
            _ => {}
        }

        let source_node = source_node.unwrap_or_else(|| rng.gen_range(0..self.size));

        not_visited.set(source_node as usize, false);

        let mut cycle = Vec::with_capacity(self.size as usize);
        cycle.push(source_node);

        let mut current = source_node;
        let mut total_dist = KahanAdder::default();

        loop {
            let chosen = match not_visited.count_ones() {
                0 => {
                    not_visited.fill(true);
                    break self
                        .dist_idx
                        .between(current, source_node)
                        .map(|dist| (cycle, total_dist.push_and_result(dist)));
                }
                1 => not_visited
                    .first_one()
                    .unwrap_or_else(|| unreachable!("not_visited should contain one element")),
                _ => {
                    let wi = cumulative_weights_wrapper
                        .fill(not_visited.iter_ones().map(|i| {
                            let i = i as u32;
                            // todo: do not account in weight map unacceptable distances
                            // todo: as it leads to useless idle cycles
                            weights
                                .between(None, current, i)
                                .unwrap_or_else(|| {
                                    unreachable!("No weights between {current} and {i}")
                                })
                                .unwrap_or(0.0)
                        }))
                        .ok()?;
                    let chosen = wi.sample(rng);
                    not_visited
                        .iter_ones()
                        .nth(chosen)
                        .unwrap_or_else(|| unreachable!("No node in {chosen} position"))
                }
            };
            not_visited.set(chosen, false);
            let chosen = chosen as u32;
            cycle.push(chosen);
            total_dist.push_mut(self.dist_idx.between(current, chosen)?);
            current = chosen;
        }
    }
}

fn eval_a(opt_dist: f64) -> f64 {
    (3.0 + lambert_w0(-3.0 / f64::consts::E.powi(3))) / opt_dist
}

fn recip_plank_law_ext(opt_dist: f64, a: f64) -> f64 {
    plank_law(opt_dist, a, 1.0).recip()
}

fn plank_law(x: f64, a: f64, recip_law_ext: f64) -> f64 {
    if x.is_finite() && x != 0.0 {
        recip_law_ext * x.powi(3) / (x * a).exp_m1()
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plank_law() {
        let a = eval_a(500.0);
        let recip_law_ext = recip_plank_law_ext(500.0, a);
        let v_499 = plank_law(499.0, a, recip_law_ext);
        let v_500 = plank_law(500.0, a, recip_law_ext);
        let v_501 = plank_law(501.0, a, recip_law_ext);

        assert!((v_500 - 1.0).abs() < 1e-9);
        assert!(v_499 < v_500);
        assert!(v_501 < v_500);
    }
}
