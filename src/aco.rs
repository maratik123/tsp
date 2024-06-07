use crate::distance::DistancesIdx;
use crate::graph::GraphIdx;
use crate::reusable_weighted_index::CumulativeWeightsWrapper;
use crate::util::{block_kahan_sum, cycling, KahanAdder};
use bitvec::bitvec;
use bitvec::vec::BitVec;
use rand::distributions::Distribution;
use rand::{random, Rng};
use rand_pcg::Pcg64Mcg;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

const INIT_INTENSITY_MULTIPLIER: f64 = 10.0;
const MINIMAL_INTENSITY: f64 = 1e-5;

#[derive(Clone, Debug, PartialEq)]
pub struct Aco<'a> {
    size: u32,
    dist_idx: &'a DistancesIdx<'a>,
    intensity: f64,
    q: f64,
}

impl<'a> Aco<'a> {
    pub fn new(dist_idx: &'a DistancesIdx<'a>, intensity: Option<f64>, q: Option<f64>) -> Self {
        let size = dist_idx.graph.size;

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
        let mut intensities = GraphIdx::transform_const(&self.dist_idx.graph, self.intensity);
        let mut weights = GraphIdx::transform_const(&self.dist_idx.graph, 0.0);

        let mut cycles = Vec::with_capacity(ants as usize + 1);

        for i in 0..iterations {
            self.dist_idx
                .graph
                .merge_parallel(&intensities, &mut weights, |dist, intensity| {
                    intensity.max(MINIMAL_INTENSITY).powf(alpha) / dist.powf(beta)
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
                    |(rng, not_visited, cumulative_weights_wrapper), _| {
                        self.traverse_graph(
                            None,
                            &weights,
                            rng,
                            not_visited,
                            cumulative_weights_wrapper,
                        )
                    },
                )
                .collect_into_vec(&mut cycles);
            if let Some(best_cycle_dist) = &best_cycle_dist {
                cycles.push(best_cycle_dist.clone());
            }
            cycles.par_sort_unstable_by(|(_, dist1), (_, dist2)| dist1.total_cmp(dist2));
            cycles.truncate((cycles.len() + 1) / 2);

            intensities.transform_inplace(|value| *value *= degradation_factor);

            for cycle_dist in cycles.drain(..) {
                let (cycle, distance) = &cycle_dist;
                let delta = self.q / distance;

                for (&node1, &node2) in cycling(cycle) {
                    *intensities.between_mut(node1, node2).unwrap_or_else(|| {
                        unreachable!("No pheromones between {node1} and {node2}")
                    }) += delta;
                }

                match best_cycle_dist {
                    Some((_, best_distance)) if distance < &best_distance => {
                        println!(
                            "New cycle: {:?}, len: {:.06}, iteration: [{i}]",
                            cycle, distance
                        );
                        best_cycle_dist = Some(cycle_dist);
                    }
                    None => {
                        println!("First cycle: {:?}, len: {:.05}", cycle, distance);
                        best_cycle_dist = Some(cycle_dist);
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
        weights: &GraphIdx<f64>,
        rng: &mut impl Rng,
        not_visited: &mut BitVec,
        cumulative_weights_wrapper: &mut CumulativeWeightsWrapper<f64>,
    ) -> (Vec<u32>, f64) {
        match self.size {
            0 => return (vec![], 0.0),
            1 => return (vec![0], 0.0),
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
                    break (
                        cycle,
                        total_dist.push_and_result(
                            self.dist_idx
                                .between(current, source_node)
                                .unwrap_or_else(|| {
                                    unreachable!("No distance between {current} and {source_node}")
                                }),
                        ),
                    );
                }
                1 => not_visited
                    .first_one()
                    .unwrap_or_else(|| unreachable!("not_visited should contain one element")),
                _ => {
                    let wi = cumulative_weights_wrapper
                        .fill(not_visited.iter_ones().map(|i| {
                            let i = i as u32;
                            weights.between(0.0, current, i).unwrap_or_else(|| {
                                unreachable!("No weights between {current} and {i}")
                            })
                        }))
                        .unwrap_or_else(|e| unreachable!("No nodes to choose from: {e}"));
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
            total_dist.push_mut(
                self.dist_idx
                    .between(current, chosen)
                    .unwrap_or_else(|| unreachable!("No distance between {current} and {chosen}")),
            );
            current = chosen;
        }
    }
}
