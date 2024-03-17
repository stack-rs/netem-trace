//! This module contains some predefined bandwidth trace models.
//!
//! Enabled with feature `bw-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticBw`]: A trace model with static bandwidth.
//! - [`NormalizedBw`]: A trace model whose bandwidth subjects to a normal distribution (can set upper and lower bounds).
//! - [`RepeatedBwPattern`]: A trace model with a repeated bandwidth pattern.
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticBwConfig;
//! # use netem_trace::{Bandwidth, Duration, BwTrace};
//! let mut static_bw = StaticBwConfig::new()
//!     .bw(Bandwidth::from_mbps(24))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_bw.next_bw(), Some((Bandwidth::from_mbps(24), Duration::from_secs(1))));
//! assert_eq!(static_bw.next_bw(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{StaticBwConfig, BwTraceConfig};
//! # use netem_trace::{Bandwidth, Duration, BwTrace};
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
//! // The content would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}"
//! // if the `human` feature is enabled.
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}";
//! let des: Box<dyn BwTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(
//!     model.next_bw(),
//!     Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_bw(),
//!     Some((Bandwidth::from_mbps(24), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_bw(),
//!     Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_bw(),
//!     Some((Bandwidth::from_mbps(24), Duration::from_secs(1)))
//! );
//! assert_eq!(model.next_bw(), None);
//! ```
use crate::{Bandwidth, BwTrace, Duration};
use dyn_clone::DynClone;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};

const DEFAULT_RNG_SEED: u64 = 42;

/// This trait is used to convert a bandwidth trace configuration into a bandwidth trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait BwTraceConfig: DynClone + Send {
    fn into_model(self: Box<Self>) -> Box<dyn BwTrace>;
}

dyn_clone::clone_trait_object!(BwTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The model of a static bandwidth trace.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let mut static_bw = StaticBwConfig::new()
///     .bw(Bandwidth::from_mbps(24))
///     .duration(Duration::from_secs(1))
///     .build();
/// assert_eq!(static_bw.next_bw(), Some((Bandwidth::from_mbps(24), Duration::from_secs(1))));
/// assert_eq!(static_bw.next_bw(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticBw {
    pub bw: Bandwidth,
    pub duration: Option<Duration>,
}

/// The configuration struct for [`StaticBw`].
///
/// See [`StaticBw`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct StaticBwConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bw: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
}

/// The model of a bandwidth trace subjects to a normal distribution.
///
/// The bandwidth will subject to N(mean, std_dev), but bounded within [lower_bound, upper_bound] (optional)
///
/// ## Examples
///
/// A simple example without any bound on bandwidth:
///
/// ```
/// # use netem_trace::model::NormalizedBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let mut normal_bw = NormalizedBwConfig::new()
///     .mean(Bandwidth::from_mbps(12))
///     .std_dev(Bandwidth::from_mbps(1))
///     .duration(Duration::from_secs(1))
///     .step(Duration::from_millis(100))
///     .seed(42)
///     .build();
/// assert_eq!(normal_bw.next_bw(), Some((Bandwidth::from_bps(12069427), Duration::from_millis(100))));
/// assert_eq!(normal_bw.next_bw(), Some((Bandwidth::from_bps(12132938), Duration::from_millis(100))));
/// ```
///
/// A more complex example with bounds on bandwidth:
///
/// ```
/// # use netem_trace::model::NormalizedBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let mut normal_bw = NormalizedBwConfig::new()
///     .mean(Bandwidth::from_mbps(12))
///     .std_dev(Bandwidth::from_mbps(1))
///     .duration(Duration::from_secs(1))
///     .step(Duration::from_millis(100))
///     .seed(42)
///     .upper_bound(Bandwidth::from_kbps(12100))
///     .lower_bound(Bandwidth::from_kbps(11900))
///     .build();
/// assert_eq!(normal_bw.next_bw(), Some((Bandwidth::from_bps(12069427), Duration::from_millis(100))));
/// assert_eq!(normal_bw.next_bw(), Some((Bandwidth::from_bps(12100000), Duration::from_millis(100))));
/// ```
#[derive(Debug, Clone)]
pub struct NormalizedBw {
    pub mean: Bandwidth,
    pub std_dev: Bandwidth,
    pub upper_bound: Option<Bandwidth>,
    pub lower_bound: Option<Bandwidth>,
    pub duration: Duration,
    pub step: Duration,
    pub seed: u64,
    rng: StdRng,
    normal: Normal<f64>,
}

/// The configuration struct for [`NormalizedBw`].
///
/// See [`NormalizedBw`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct NormalizedBwConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub mean: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub std_dev: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub upper_bound: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lower_bound: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub step: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<u64>,
}

/// The model of a bandwidth trace whose waveform is sawtooth.
///
/// The lowest value of the sawtooth is set by `bottom` while the highest value is set by `top`.
/// The `interval` describes how long a sawtooth lasts. The `duty_ratio` describes how much the rising time of a sawtooth
/// occupies the `interval`.
///
/// The `step` describes how long between two consecutive bandwidth samples.
///
/// The noise of the sawtooth bandwidth will subject to N(0, std_dev), but bounded within [-lower_noise_bound, upper_noise_bound] (optional)
///
/// ## Examples
///
/// A simple example without any bound on bandwidth:
///
/// ```
/// # use netem_trace::model::SawtoothBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let mut sawtooth_bw = SawtoothBwConfig::new()
///     .bottom(Bandwidth::from_mbps(12))
///     .top(Bandwidth::from_mbps(16))
///     .duration(Duration::from_secs(1))
///     .step(Duration::from_millis(100))
///     .interval(Duration::from_millis(500))
///     .duty_ratio(0.8)
///     .build();
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(12), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(13), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(14), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(15), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(16), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(12), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(13), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(14), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(15), Duration::from_millis(100)))
/// );
/// ```
///
/// A more complex example with bounds on noise:
///
/// ```
/// # use netem_trace::model::SawtoothBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let mut sawtooth_bw = SawtoothBwConfig::new()
///     .bottom(Bandwidth::from_mbps(12))
///     .top(Bandwidth::from_mbps(16))
///     .duration(Duration::from_secs(1))
///     .step(Duration::from_millis(100))
///     .interval(Duration::from_millis(500))
///     .duty_ratio(0.8)
///     .std_dev(Bandwidth::from_mbps(5))
///     .upper_noise_bound(Bandwidth::from_mbps(1))
///     .lower_noise_bound(Bandwidth::from_kbps(500))
///     .build();
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_bps(12347139), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_bps(13664690), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_mbps(15), Duration::from_millis(100)))
/// );
/// assert_eq!(
///     sawtooth_bw.next_bw(),
///     Some((Bandwidth::from_bps(14500000), Duration::from_millis(100)))
/// );
/// ```
#[derive(Debug, Clone)]
pub struct SawtoothBw {
    pub bottom: Bandwidth,
    pub top: Bandwidth,
    pub interval: Duration,
    pub duty_ratio: f64,
    pub duration: Duration,
    pub step: Duration,
    pub seed: u64,
    pub std_dev: Bandwidth,
    pub upper_noise_bound: Option<Bandwidth>,
    pub lower_noise_bound: Option<Bandwidth>,
    current: Duration,
    rng: StdRng,
    noise: Normal<f64>,
}

/// The configuration struct for [`SawtoothBw`].
///
/// See [`SawtoothBw`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct SawtoothBwConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bottom: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub top: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub interval: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub duty_ratio: Option<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub step: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub std_dev: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub upper_noise_bound: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lower_noise_bound: Option<Bandwidth>,
}

/// The model contains an array of bandwidth trace models.
///
/// Combines multiple bandwidth trace models into one bandwidth pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedBwPatternConfig`]:
///
/// ```
/// # use netem_trace::model::{StaticBwConfig, BwTraceConfig};
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The content would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}";
/// let des: Box<dyn BwTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// assert_eq!(
///     model.next_bw(),
///     Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_bw(),
///     Some((Bandwidth::from_mbps(24), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_bw(),
///     Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_bw(),
///     Some((Bandwidth::from_mbps(24), Duration::from_secs(1)))
/// );
/// assert_eq!(model.next_bw(), None);
/// ```
///
/// You can also build manually:
///
/// ```
/// # use netem_trace::model::{StaticBwConfig, BwTraceConfig, RepeatedBwPatternConfig};
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// let pat = vec![
///     Box::new(
///         StaticBwConfig::new()
///             .bw(Bandwidth::from_mbps(12))
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn BwTraceConfig>,
///     Box::new(
///         StaticBwConfig::new()
///             .bw(Bandwidth::from_mbps(24))
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn BwTraceConfig>,
/// ];
/// let ser = Box::new(RepeatedBwPatternConfig::new().pattern(pat).count(2)) as Box<dyn BwTraceConfig>;
/// let ser_str = serde_json::to_string(&ser).unwrap();
/// # #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The json string would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":\"1s\"}}],\"count\":2}}";
/// assert_eq!(ser_str, json_str);
/// ```
pub struct RepeatedBwPattern {
    pub pattern: Vec<Box<dyn BwTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn BwTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedBwPattern`].
///
/// See [`RepeatedBwPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedBwPatternConfig {
    pub pattern: Vec<Box<dyn BwTraceConfig>>,
    pub count: usize,
}

impl BwTrace for StaticBw {
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
            let mut bw = Bandwidth::from_bps(bw);
            if let Some(lower_bound) = self.lower_bound {
                bw = bw.max(lower_bound);
            }
            if let Some(upper_bound) = self.upper_bound {
                bw = bw.min(upper_bound);
            }
            let duration = self.step.min(self.duration);
            self.duration -= duration;
            Some((bw, duration))
        }
    }
}

impl BwTrace for SawtoothBw {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if self.duration.is_zero() {
            None
        } else {
            let current = self.current.as_secs_f64();
            let change_point = self.interval.as_secs_f64() * self.duty_ratio;
            let base_bw = if current < change_point {
                let ratio = current / change_point;
                self.bottom + (self.top - self.bottom).mul_f64(ratio)
            } else {
                let ratio = (current - change_point) / (self.interval.as_secs_f64() - change_point);
                self.top - (self.top - self.bottom).mul_f64(ratio)
            };
            let mut offset = self.noise.sample(&mut self.rng);
            if let Some(upper_noise_bound) = self.upper_noise_bound {
                offset = offset.min(upper_noise_bound.as_bps() as f64);
            }
            if let Some(lower_noise_bound) = self.lower_noise_bound {
                offset = offset.max(-(lower_noise_bound.as_bps() as f64));
            }
            let bw = Bandwidth::from_bps((base_bw.as_bps() as f64 + offset) as u64);
            let duration = self.step.min(self.duration);
            self.duration -= duration;
            self.current += duration;
            if self.current >= self.interval {
                self.current -= self.interval;
            }
            Some((bw, duration))
        }
    }
}

impl BwTrace for RepeatedBwPattern {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_bw() {
                Some(bw) => Some(bw),
                None => {
                    self.current_model = None;
                    self.current_pattern += 1;
                    if self.current_pattern >= self.pattern.len() {
                        self.current_pattern = 0;
                        self.current_cycle += 1;
                        if self.count != 0 && self.current_cycle >= self.count {
                            return None;
                        }
                    }
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

impl StaticBwConfig {
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

    pub fn build(self) -> StaticBw {
        StaticBw {
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

    pub fn build(self) -> NormalizedBw {
        let mean = self.mean.unwrap_or_else(|| Bandwidth::from_mbps(12));
        let std_dev = self.std_dev.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let upper_bound = self.upper_bound;
        let lower_bound = self.lower_bound;
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

impl SawtoothBwConfig {
    pub fn new() -> Self {
        Self {
            bottom: None,
            top: None,
            interval: None,
            duty_ratio: None,
            duration: None,
            step: None,
            seed: None,
            std_dev: None,
            upper_noise_bound: None,
            lower_noise_bound: None,
        }
    }

    pub fn bottom(mut self, bottom: Bandwidth) -> Self {
        self.bottom = Some(bottom);
        self
    }

    pub fn top(mut self, top: Bandwidth) -> Self {
        self.top = Some(top);
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn duty_ratio(mut self, duty_ratio: f64) -> Self {
        self.duty_ratio = Some(duty_ratio);
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

    pub fn std_dev(mut self, std_dev: Bandwidth) -> Self {
        self.std_dev = Some(std_dev);
        self
    }

    pub fn upper_noise_bound(mut self, upper_noise_bound: Bandwidth) -> Self {
        self.upper_noise_bound = Some(upper_noise_bound);
        self
    }

    pub fn lower_noise_bound(mut self, lower_noise_bound: Bandwidth) -> Self {
        self.lower_noise_bound = Some(lower_noise_bound);
        self
    }

    pub fn build(self) -> SawtoothBw {
        let bottom = self.bottom.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let top = self.top.unwrap_or_else(|| Bandwidth::from_mbps(12));
        if bottom > top {
            panic!("SawtoothBw: bottom bw must be less than top bw");
        }
        let interval = self.interval.unwrap_or_else(|| Duration::from_secs(1));
        let duty_ratio = self.duty_ratio.unwrap_or(0.5);
        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(1));
        let step = self.step.unwrap_or_else(|| Duration::from_millis(1));
        let seed = self.seed.unwrap_or(DEFAULT_RNG_SEED);
        let rng = StdRng::seed_from_u64(seed);
        let std_dev = self.std_dev.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let upper_noise_bound = self.upper_noise_bound;
        let lower_noise_bound = self.lower_noise_bound;
        let current = Duration::ZERO;
        let bw_std_dev = saturating_bandwidth_as_bps_u64!(std_dev) as f64;
        let noise: Normal<f64> = Normal::new(0.0, bw_std_dev).unwrap();
        SawtoothBw {
            bottom,
            top,
            interval,
            duty_ratio,
            duration,
            step,
            seed,
            std_dev,
            upper_noise_bound,
            lower_noise_bound,
            current,
            rng,
            noise,
        }
    }
}

impl RepeatedBwPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
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
        RepeatedBwPattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

macro_rules! impl_bw_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl BwTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn BwTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_bw_trace_config!(StaticBwConfig);
impl_bw_trace_config!(NormalizedBwConfig);
impl_bw_trace_config!(SawtoothBwConfig);
impl_bw_trace_config!(RepeatedBwPatternConfig);
