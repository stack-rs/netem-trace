//! This module contains some predefined bandwidth trace models.
//!
//! Enabled with feature `bw-model` or `model`.
//!
//! Predefined models:
//!
//! - [`FixedBw`]: A trace model with fixed bandwidth.
//! - [`NormalizedBw`]: A trace model whose bandwidth subjects to a normal distribution.
//! - [`BoundedNormalizedBw`]: A trace model whose bandwidth subjects to a normal distribution with upper and lower bounds.
use crate::{Bandwidth, BwTrace, Duration};
use dyn_clone::DynClone;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use std::collections::VecDeque;

const DEFAULT_RNG_SEED: u64 = 42;

/// This trait is used to convert a bandwidth trace configuration into a bandwidth trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[typetag::serde]
pub trait BwTraceConfig: DynClone {
    fn into_model(self: Box<Self>) -> Box<dyn BwTrace>;
}

dyn_clone::clone_trait_object!(BwTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct FixedBw {
    pub bw: Bandwidth,
    pub duration: Option<Duration>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(default)]
#[derive(Debug, Clone, Default)]
pub struct FixedBwConfig {
    pub bw: Option<Bandwidth>,
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct NormalizedBw {
    pub mean: Bandwidth,
    pub std_dev: Bandwidth,
    pub duration: Duration,
    pub step: Duration,
    pub seed: u64,
    rng: StdRng,
    normal: Normal<f64>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(default)]
#[derive(Debug, Clone, Default)]
pub struct NormalizedBwConfig {
    pub mean: Option<Bandwidth>,
    pub std_dev: Option<Bandwidth>,
    pub duration: Option<Duration>,
    pub step: Option<Duration>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct BoundedNormalizedBw {
    pub mean: Bandwidth,
    pub std_dev: Bandwidth,
    pub upper_bound: Bandwidth,
    pub lower_bound: Bandwidth,
    pub duration: Duration,
    pub step: Duration,
    pub seed: u64,
    rng: StdRng,
    normal: Normal<f64>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(default)]
#[derive(Debug, Clone, Default)]
pub struct BoundedNormalizedBwConfig {
    pub mean: Option<Bandwidth>,
    pub std_dev: Option<Bandwidth>,
    pub upper_bound: Option<Bandwidth>,
    pub lower_bound: Option<Bandwidth>,
    pub duration: Option<Duration>,
    pub step: Option<Duration>,
    pub seed: Option<u64>,
}

pub struct RepeatedBwPattern {
    pub pattern: VecDeque<Box<dyn BwTrace>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(default)]
#[derive(Default, Clone)]
pub struct RepeatedBwPatternConfig {
    pub pattern: Vec<Box<dyn BwTraceConfig>>,
    pub count: usize,
}

impl BwTrace for FixedBw {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if let Some(duration) = self.duration.take() {
            if duration.is_zero() {
                None
            } else {
                Some((self.bw, duration))
            }
        } else {
            None
        }
    }
}

impl BwTrace for NormalizedBw {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if self.duration.is_zero() {
            None
        } else {
            let bw = self.sample() as u64;
            let bw = Bandwidth::from_bps(bw);
            let duration = self.step.min(self.duration);
            self.duration -= duration;
            Some((bw, duration))
        }
    }
}

impl BwTrace for BoundedNormalizedBw {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if self.duration.is_zero() {
            None
        } else {
            let bw = self.sample() as u64;
            let bw = Bandwidth::from_bps(bw);
            let bw = bw.max(self.lower_bound).min(self.upper_bound);
            let duration = self.step.min(self.duration);
            self.duration -= duration;
            Some((bw, duration))
        }
    }
}

impl BwTrace for RepeatedBwPattern {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if self.pattern.is_empty() {
            None
        } else {
            match self.pattern[0].next_bw() {
                Some((bw, duration)) => Some((bw, duration)),
                None => {
                    self.pattern.pop_front();
                    self.next_bw()
                }
            }
        }
    }
}

impl NormalizedBw {
    pub fn sample(&mut self) -> f64 {
        self.normal.sample(&mut self.rng)
    }
}

impl BoundedNormalizedBw {
    pub fn sample(&mut self) -> f64 {
        self.normal.sample(&mut self.rng)
    }
}

impl FixedBwConfig {
    pub fn new() -> Self {
        Self {
            bw: None,
            duration: None,
        }
    }

    pub fn bw(mut self, bw: Bandwidth) -> Self {
        self.bw = Some(bw);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn build(self) -> FixedBw {
        FixedBw {
            bw: self.bw.unwrap_or_else(|| Bandwidth::from_mbps(12)),
            duration: Some(self.duration.unwrap_or_else(|| Duration::from_secs(1))),
        }
    }
}

/// Convert a bandwidth to bps as u64 with saturating operation.
macro_rules! saturating_bandwidth_as_bps_u64 {
    ($bw:expr) => {
        $bw.as_gbps()
            .saturating_mul(1_000_000_000)
            .saturating_add($bw.subgbps_bps() as u64)
    };
}

impl NormalizedBwConfig {
    pub fn new() -> Self {
        Self {
            mean: None,
            std_dev: None,
            duration: None,
            step: None,
            seed: None,
        }
    }

    pub fn mean(mut self, mean: Bandwidth) -> Self {
        self.mean = Some(mean);
        self
    }

    pub fn std_dev(mut self, std_dev: Bandwidth) -> Self {
        self.std_dev = Some(std_dev);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn step(mut self, step: Duration) -> Self {
        self.step = Some(step);
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(self) -> NormalizedBw {
        let mean = self.mean.unwrap_or_else(|| Bandwidth::from_mbps(12));
        let std_dev = self.std_dev.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(1));
        let step = self.step.unwrap_or_else(|| Duration::from_millis(1));
        let seed = self.seed.unwrap_or(DEFAULT_RNG_SEED);
        let rng = StdRng::seed_from_u64(seed);
        let bw_mean = saturating_bandwidth_as_bps_u64!(mean) as f64;
        let bw_std_dev = saturating_bandwidth_as_bps_u64!(std_dev) as f64;
        let normal: Normal<f64> = Normal::new(bw_mean, bw_std_dev).unwrap();
        NormalizedBw {
            mean,
            std_dev,
            duration,
            step,
            seed,
            rng,
            normal,
        }
    }
}

impl BoundedNormalizedBwConfig {
    pub fn new() -> Self {
        Self {
            mean: None,
            std_dev: None,
            upper_bound: None,
            lower_bound: None,
            duration: None,
            step: None,
            seed: None,
        }
    }

    pub fn mean(mut self, mean: Bandwidth) -> Self {
        self.mean = Some(mean);
        self
    }

    pub fn std_dev(mut self, std_dev: Bandwidth) -> Self {
        self.std_dev = Some(std_dev);
        self
    }

    pub fn upper_bound(mut self, upper_bound: Bandwidth) -> Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    pub fn lower_bound(mut self, lower_bound: Bandwidth) -> Self {
        self.lower_bound = Some(lower_bound);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn step(mut self, step: Duration) -> Self {
        self.step = Some(step);
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn build(self) -> BoundedNormalizedBw {
        let mean = self.mean.unwrap_or_else(|| Bandwidth::from_mbps(12));
        let std_dev = self.std_dev.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let upper_bound = self.upper_bound.unwrap_or_else(|| Bandwidth::from_mbps(24));
        let lower_bound = self.lower_bound.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(1));
        let step = self.step.unwrap_or_else(|| Duration::from_millis(1));
        let seed = self.seed.unwrap_or(DEFAULT_RNG_SEED);
        let rng = StdRng::seed_from_u64(seed);
        let bw_mean = saturating_bandwidth_as_bps_u64!(mean) as f64;
        let bw_std_dev = saturating_bandwidth_as_bps_u64!(std_dev) as f64;
        let normal: Normal<f64> = Normal::new(bw_mean, bw_std_dev).unwrap();
        BoundedNormalizedBw {
            mean,
            std_dev,
            upper_bound,
            lower_bound,
            duration,
            step,
            seed,
            rng,
            normal,
        }
    }
}

impl RepeatedBwPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 1,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn BwTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedBwPattern {
        let pattern = vec![self.pattern; self.count]
            .drain(..)
            .flatten()
            .map(|config| config.into_model())
            .collect();
        RepeatedBwPattern { pattern }
    }
}

macro_rules! impl_bw_trace_config {
    ($name:ident) => {
        #[typetag::serde]
        impl BwTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn BwTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_bw_trace_config!(FixedBwConfig);
impl_bw_trace_config!(NormalizedBwConfig);
impl_bw_trace_config!(BoundedNormalizedBwConfig);
impl_bw_trace_config!(RepeatedBwPatternConfig);
