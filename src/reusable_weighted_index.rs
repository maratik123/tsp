// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// based on rand-0.8.5/src/distributions/weighted_index.rs

//! Reusable weighted index sampling

use core::cmp::Ordering;
use core::cmp::PartialOrd;
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::distributions::{Distribution, WeightedError};
use rand::Rng;

/// A distribution using weighted sampling of discrete items
///
/// Sampling a `ReusableWeightedIndex` distribution returns the index of a randomly
/// selected element from the iterator used when the `ReusableWeightedIndex` was
/// created. The chance of a given element being picked is proportional to the
/// value of the element. The weights can use any type `X` for which an
/// implementation of [`Uniform<X>`] exists.
///
/// # Performance
///
/// Time complexity of sampling from `ReusableWeightedIndex` is `O(log N)` where
/// `N` is the number of weights. As an alternative,
/// [`rand_distr::weighted_alias`](https://docs.rs/rand_distr/*/rand_distr/weighted_alias/index.html)
/// supports `O(1)` sampling, but with much higher initialisation cost.
///
/// A `ReusableWeightedIndex<X>` contains a `Vec<X>` and a [`Uniform<X>`] and so its
/// size is the sum of the size of those objects, possibly plus some alignment.
///
/// Creating a `ReusableWeightedIndex<X>` will allocate enough space to hold `N - 1`
/// weights of type `X`, where `N` is the number of weights. However, since
/// `Vec` doesn't guarantee a particular growth strategy, additional memory
/// might be allocated but not used. Since the `ReusableWeightedIndex` object also
/// contains, this might cause additional allocations, though for primitive
/// types, [`Uniform<X>`] doesn't allocate any memory.
///
/// Reusing `ReusableWeightedIndex<X>` does not cause additional memory allocations as
/// long as the `ReusableWeightedIndex` is not dropped and number of weights is not
/// increased.
///
/// # Panics
///
/// Panics if the sum of the weights is zero.
///
/// # Implementation
///
/// The `ReusableWeightedIndex` object contains a `Vec<X>` and a [`Uniform<X>`].
/// When sampling from the distribution, the `Uniform` object is used to sample
/// from the `Vec`. The `Vec` is reused, so it is not necessary to drop the
/// `ReusableWeightedIndex before creating a new one.
///
/// Sampling from `ReusableWeightedIndex` will result in a single call to
/// `Uniform<X>::sample` (method of the [`Distribution`] trait), which typically
/// will request a single value from the underlying [`RngCore`], though the
/// exact number depends on the implementation of `Uniform<X>::sample`.
///
/// # Example
///
/// ```
/// use rand::prelude::*;
/// use tsp::reusable_weighted_index::CumulativeWeightsWrapper;
///
/// let choices = ['a', 'b', 'c'];
/// let weights = [2,   1,   1];
/// let mut dist = CumulativeWeightsWrapper::new();
/// let dist = dist.fill(&weights).unwrap();
/// let mut rng = thread_rng();
/// for _ in 0..100 {
///     // 50% chance to print 'a', 25% chance to print 'b', 25% chance to print 'c'
///     println!("{}", choices[dist.sample(&mut rng)]);
/// }
///
/// let items = [('a', 0), ('b', 3), ('c', 7)];
/// let mut dist2 = CumulativeWeightsWrapper::new();
/// let dist2 = dist2.fill(items.iter().map(|item| item.1)).unwrap();
/// for _ in 0..100 {
///     // 0% chance to print 'a', 30% chance to print 'b', 70% chance to print 'c'
///     println!("{}", items[dist2.sample(&mut rng)].0);
/// }
/// ```
///
/// [`Uniform<X>`]: crate::distributions::Uniform
/// [`RngCore`]: crate::RngCore
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReusableWeightedIndex<'a, X: SampleUniform + PartialOrd> {
    wrapper: &'a CumulativeWeightsWrapper<X>,
    total_weight: X,
    weight_distribution: X::Sampler,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CumulativeWeightsWrapper<X> {
    cumulative_weights: Vec<X>,
}

impl<X: SampleUniform + PartialOrd> CumulativeWeightsWrapper<X> {
    pub fn new() -> Self {
        Self {
            cumulative_weights: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cumulative_weights: Vec::with_capacity(capacity),
        }
    }
}

impl<X: SampleUniform + PartialOrd + Default> CumulativeWeightsWrapper<X> {
    /// Fills a `ReusableWeightedIndex` [`Distribution`] using the values
    /// in `weights`. The weights can use any type `X` for which an
    /// implementation of [`Uniform<X>`] exists.
    ///
    /// Deinitializes `CumulativeWeightsWrapper` and returns error if the iterator is empty,
    /// if any weight is `< 0`, or if its total value is 0
    ///
    /// [`Uniform<X>`]: crate::distributions::uniform::Uniform
    pub fn fill<'a, I>(
        &'a mut self,
        weights: I,
    ) -> Result<ReusableWeightedIndex<'a, X>, WeightedError>
    where
        I: IntoIterator,
        I::Item: SampleBorrow<X>,
        X: for<'b> core::ops::AddAssign<&'b X> + Clone + Default,
    {
        self.cumulative_weights.clear();
        let mut iter = weights.into_iter();
        let mut total_weight: X = iter.next().ok_or(WeightedError::NoItem)?.borrow().clone();
        let zero = <X as Default>::default();

        if matches!(total_weight.partial_cmp(&zero), None | Some(Ordering::Less)) {
            return Err(WeightedError::InvalidWeight);
        }
        self.cumulative_weights.reserve(iter.size_hint().0);
        for w in iter {
            // Note that `!(w >= x)` is not equivalent to `w < x` for partially
            // ordered types due to NaNs which are equal to nothing.
            if matches!(w.borrow().partial_cmp(&zero), None | Some(Ordering::Less)) {
                self.cumulative_weights.clear();
                return Err(WeightedError::InvalidWeight);
            }
            self.cumulative_weights.push(total_weight.clone());
            total_weight += w.borrow();
        }

        if total_weight == zero {
            self.cumulative_weights.clear();
            return Err(WeightedError::AllWeightsZero);
        }

        let weight_distribution = X::Sampler::new(zero, total_weight.clone());

        Ok(ReusableWeightedIndex {
            wrapper: self,
            weight_distribution,
            total_weight,
        })
    }
}

impl<'a, X: SampleUniform + PartialOrd> ReusableWeightedIndex<'a, X> {}

impl<'a, X> Distribution<usize> for ReusableWeightedIndex<'a, X>
where
    X: SampleUniform + PartialOrd + Default,
{
    /// Returns 0
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let chosen_weight = self.weight_distribution.sample(rng);
        // Find the first item which has a weight *higher* than the chosen weight.
        self.wrapper
            .cumulative_weights
            .binary_search_by(|w| {
                if *w <= chosen_weight {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::RngCore;

    /// Construct a deterministic RNG with the given seed
    fn rng(seed: u64) -> impl RngCore {
        // For tests, we want a statistically good, fast, reproducible RNG.
        // PCG32 will do fine, and will be easy to embed if we ever need to.
        const INC: u64 = 11634580027462260723;
        rand_pcg::Pcg32::new(seed, INC)
    }

    #[test]
    fn test_accepting_nan() {
        let mut dist = CumulativeWeightsWrapper::new();
        assert_eq!(
            dist.fill([f32::NAN, 0.5]).unwrap_err(),
            WeightedError::InvalidWeight,
        );
        assert_eq!(
            dist.fill([f32::NAN]).unwrap_err(),
            WeightedError::InvalidWeight,
        );
        assert_eq!(
            dist.fill([0.5, f32::NAN]).unwrap_err(),
            WeightedError::InvalidWeight,
        );
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Miri is too slow
    fn test_weighted_index() {
        let mut r = rng(700);
        const N_REPS: u32 = 5000;
        let weights = [1u32, 2, 3, 0, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7];
        let total_weight = weights.iter().sum::<u32>() as f32;

        let verify = |result: [i32; 14]| {
            for (i, count) in result.iter().enumerate() {
                let exp = (weights[i] * N_REPS) as f32 / total_weight;
                let mut err = (*count as f32 - exp).abs();
                if err != 0.0 {
                    err /= exp;
                }
                assert!(err <= 0.25);
            }
        };

        // WeightedIndex from vec
        let mut chosen = [0i32; 14];
        let mut distr_w = CumulativeWeightsWrapper::new();
        let distr = distr_w.fill(weights.to_vec()).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        // WeightedIndex from slice
        chosen = [0i32; 14];
        let distr = distr_w.fill(&weights[..]).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        // WeightedIndex from iterator
        chosen = [0i32; 14];
        let distr = distr_w.fill(weights.iter()).unwrap();
        for _ in 0..N_REPS {
            chosen[distr.sample(&mut r)] += 1;
        }
        verify(chosen);

        for _ in 0..5 {
            let distr = distr_w.fill([0, 1]).unwrap();
            assert_eq!(distr.sample(&mut r), 1);
            let distr = distr_w.fill([1, 0]).unwrap();
            assert_eq!(distr.sample(&mut r), 0);
            let distr = distr_w.fill([0, 0, 0, 0, 10, 0]).unwrap();
            assert_eq!(distr.sample(&mut r), 4);
        }

        assert_eq!(
            distr_w.fill(&[10][0..0]).unwrap_err(),
            WeightedError::NoItem
        );
        assert_eq!(
            distr_w.fill([0]).unwrap_err(),
            WeightedError::AllWeightsZero
        );
        let mut distr_w = CumulativeWeightsWrapper::new();
        assert_eq!(
            distr_w.fill([10, 20, -1, 30]).unwrap_err(),
            WeightedError::InvalidWeight
        );
        assert_eq!(
            distr_w.fill([-10, 20, 1, 30]).unwrap_err(),
            WeightedError::InvalidWeight
        );
        assert_eq!(
            distr_w.fill([-10]).unwrap_err(),
            WeightedError::InvalidWeight
        );
    }

    #[test]
    fn value_stability() {
        fn test_samples<X, I>(weights: I, buf: &mut [usize], expected: &[usize])
        where
            I: IntoIterator,
            I::Item: SampleBorrow<X>,
            X: for<'a> core::ops::AddAssign<&'a X> + Clone + Default + SampleUniform + PartialOrd,
        {
            assert_eq!(buf.len(), expected.len());
            let mut distr = CumulativeWeightsWrapper::new();
            let distr = distr.fill(weights).unwrap();
            let mut rng = rng(701);
            for r in buf.iter_mut() {
                *r = rng.sample(&distr);
            }
            assert_eq!(buf, expected);
        }

        let mut buf = [0; 10];
        test_samples(
            [1i32, 1, 1, 1, 1, 1, 1, 1, 1],
            &mut buf,
            &[0, 6, 2, 6, 3, 4, 7, 8, 2, 5],
        );
        test_samples(
            [0.7f32, 0.1, 0.1, 0.1],
            &mut buf,
            &[0, 0, 0, 1, 0, 0, 2, 3, 0, 0],
        );
        test_samples(
            [1.0f64, 0.999, 0.998, 0.997],
            &mut buf,
            &[2, 2, 1, 3, 2, 1, 3, 3, 2, 1],
        );
    }

    #[test]
    fn weighted_index_distributions_can_be_compared() {
        let mut distr1 = CumulativeWeightsWrapper::new();
        let mut distr2 = CumulativeWeightsWrapper::new();
        assert_eq!(distr1.fill([1, 2]), distr2.fill([1, 2]));
    }
}
