use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::rngs::ThreadRng;
use rand::Rng;
use roaring::RoaringBitmap;

use crate::distance::DistancesIdx;
use crate::graph::GraphIdx;
use crate::util::cycling;

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

        let mut rng = rand::thread_rng();
        let mut best_cycle_dist: Option<(Vec<_>, f64)> = None;
        let mut intensities = GraphIdx::transform_const(&self.dist_idx.graph, self.intensity);

        for i in 0..iterations {
            let mut cycles: Vec<_> = (0..ants)
                .map(|_| self.traverse_graph(None, &mut rng, &intensities, alpha, beta))
                .chain(best_cycle_dist.iter().cloned())
                .collect();
            cycles.sort_unstable_by(|(_, dist1), (_, dist2)| dist1.total_cmp(dist2));
            cycles.truncate((cycles.len() + 1) / 2);

            for cycle_dist in cycles {
                let (cycle, distance) = &cycle_dist;
                let delta = self.q / distance;
                for (&node1, &node2) in cycling(cycle) {
                    *intensities.between_mut(node1, node2).unwrap_or_else(|| {
                        unreachable!("No pheromones between {node1} and {node2}")
                    }) += delta;
                }
                intensities.transform_inplace(|value| *value *= degradation_factor);

                match best_cycle_dist {
                    Some((_, best_distance)) if distance < &best_distance => {
                        println!(
                            "New cycle: {:?}, len: {:.05}, iteration: [{i}]",
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
        rng: &mut ThreadRng,
        intensities: &GraphIdx<'_, f64>,
        alpha: f64,
        beta: f64,
    ) -> (Vec<u32>, f64) {
        match self.size {
            0 => return (vec![], 0.0),
            1 => return (vec![0], 0.0),
            _ => {}
        }

        let source_node = source_node.unwrap_or_else(|| rng.gen_range(0..self.size));

        let mut not_visited = RoaringBitmap::new();
        not_visited.insert_range(0..self.size);
        not_visited.remove(source_node);

        let mut cycle = Vec::with_capacity(self.size as usize);
        cycle.push(source_node);

        let mut total_length = 0.0;

        let mut current = source_node;

        loop {
            let chosen = match not_visited.len() {
                0 => {
                    break (
                        cycle,
                        total_length
                            + self
                                .dist_idx
                                .between(current, source_node)
                                .unwrap_or_else(|| {
                                    unreachable!("No distance between {current} and {source_node}")
                                }),
                    )
                }
                1 => not_visited
                    .min()
                    .unwrap_or_else(|| unreachable!("not_visited should contain one element")),
                _ => {
                    let chosen = WeightedIndex::new(not_visited.iter().map(|i| {
                        intensities
                            .between(0.0, current, i)
                            .unwrap_or_else(|| {
                                unreachable!("No pheromones between {current} and {i}")
                            })
                            .max(MINIMAL_INTENSITY)
                            .powf(alpha)
                            / self
                                .dist_idx
                                .between(current, i)
                                .unwrap_or_else(|| {
                                    unreachable!("No distance between {current} and {i}")
                                })
                                .powf(beta)
                    }))
                    .unwrap_or_else(|e| unreachable!("No nodes to choose from: {e}"))
                    .sample(rng) as u32;
                    not_visited
                        .iter()
                        .nth(chosen as usize)
                        .unwrap_or_else(|| unreachable!("No node in {chosen} position"))
                }
            };
            not_visited.remove(chosen);
            cycle.push(chosen);
            total_length += self
                .dist_idx
                .between(current, chosen)
                .unwrap_or_else(|| unreachable!("No distance between {current} and {chosen}"));
            current = chosen;
        }
    }
}
