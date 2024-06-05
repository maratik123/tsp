use crate::model::{Airport, AirportIdx};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::iter::Sum;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct GraphIdx<'a, T: Copy> {
    pub(crate) size: u32,
    pub(crate) edges: Vec<T>,
    pub(crate) _pd: PhantomData<AirportIdx<'a>>,
}

impl<'a, T: Copy> GraphIdx<'a, T> {
    pub fn between(&self, default: T, apt1: u32, apt2: u32) -> Option<T> {
        if apt1 >= self.size || apt2 >= self.size {
            return None;
        }
        if apt1 == apt2 {
            return Some(default);
        }
        Some(self.edges[Self::pos(apt1, apt2)])
    }

    pub fn between_mut(&mut self, apt1: u32, apt2: u32) -> Option<&mut T> {
        if apt1 >= self.size || apt2 >= self.size || apt1 == apt2 {
            return None;
        }
        Some(&mut self.edges[Self::pos(apt1, apt2)])
    }

    fn pos(apt1: u32, apt2: u32) -> usize {
        let (apt1, apt2) = if apt1 > apt2 {
            (apt1, apt2)
        } else {
            (apt2, apt1)
        };
        let (apt1, apt2) = (apt1 as usize, apt2 as usize);
        apt1 * (apt1 - 1) / 2 + apt2
    }

    pub fn set(&mut self, apt1: u32, apt2: u32, val: T) -> Option<()> {
        if apt1 >= self.size || apt2 >= self.size || apt1 == apt2 {
            return None;
        }
        self.edges[Self::pos(apt1, apt2)] = val;
        Some(())
    }

    pub fn new(
        AirportIdx { aps, .. }: &'a AirportIdx,
        f: impl Fn(&Airport, &Airport) -> T,
    ) -> Self {
        let size = aps.len() as u32;
        let edges = aps
            .iter()
            .enumerate()
            .flat_map(|(apt1_i, apt1)| aps[..apt1_i].iter().map(|apt2| f(apt1, apt2)))
            .collect();
        Self {
            size,
            edges,
            _pd: PhantomData,
        }
    }

    pub fn merge<B: Copy, C: Copy>(
        &self,
        other: &GraphIdx<'a, B>,
        f: impl Fn(T, B) -> C,
    ) -> Option<GraphIdx<'a, C>> {
        if self.size != other.size {
            return None;
        }
        Some(GraphIdx {
            size: self.size,
            edges: self
                .edges
                .iter()
                .zip(other.edges.iter())
                .map(|(&a, &b)| f(a, b))
                .collect(),
            _pd: PhantomData,
        })
    }

    pub fn merge_parallel<B, C>(
        &self,
        other: &GraphIdx<'a, B>,
        target: &mut GraphIdx<'a, C>,
        f: impl (Fn(T, B) -> C) + Sync,
    ) -> Option<()>
    where
        T: Send + Sync,
        B: Send + Sync + Copy,
        C: Send + Sync + Copy,
    {
        if self.size != other.size {
            return None;
        }
        target.size = self.size;
        self.edges
            .par_iter()
            .zip(&other.edges)
            .map(|(&a, &b)| f(a, b))
            .collect_into_vec(&mut target.edges);
        Some(())
    }

    pub fn transform_inplace(&mut self, f: impl Fn(&mut T)) {
        for edge in &mut self.edges {
            f(edge);
        }
    }

    pub fn transform<B: Copy>(&self, f: impl Fn(T) -> B) -> GraphIdx<'a, B> {
        GraphIdx {
            size: self.size,
            edges: self.edges.iter().map(|&a| f(a)).collect(),
            _pd: PhantomData,
        }
    }

    pub fn transform_const<B: Copy>(&self, c: B) -> GraphIdx<'a, B> {
        GraphIdx {
            size: self.size,
            edges: vec![c; self.edges.len()],
            _pd: PhantomData,
        }
    }
}
impl<'a, T: Copy + Sum<T>> GraphIdx<'a, T> {
    pub fn triangle_sum(&self) -> T {
        self.edges.iter().copied().sum()
    }
}
