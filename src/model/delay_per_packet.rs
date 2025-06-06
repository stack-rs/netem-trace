//! This module contains some predefined delay per-packet trace models.
//!
//! Enabled with feature `delay-per-packet-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticDelayPerPacket`]: A trace model with static delay.
//! - [`RepeatedDelayPerPacketPattern`]: A trace model with a repeated delay pattern.
//! - [`NormalizedDelayPerPacket`]: A trace model whose delay subjects to a normal distribution (can set upper and lower bounds, and can configure it to be truncated with `truncated-normal` feature enabled).
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticDelayPerPacketConfig;
//! # use netem_trace::{Delay, DelayPerPacketTrace};
//! let mut static_delay = StaticDelayPerPacketConfig::new()
//!     .delay(Delay::from_millis(10))
//!     .count(2)
//!     .build();
//! assert_eq!(static_delay.next_delay(), Some(Delay::from_millis(10)));
//! assert_eq!(static_delay.next_delay(), Some(Delay::from_millis(10)));
//! assert_eq!(static_delay.next_delay(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::DelayPerPacketTraceConfig;
//! # use netem_trace::{Delay, DelayPerPacketTrace};
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":\"10ms\",\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":\"20ms\",\"count\":1}}],\"count\":2}}";
//! // The content would be "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}"
//! // if the `human` feature is not enabled.
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}";
//! let des: Box<dyn DelayPerPacketTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(
//!     model.next_delay(),
//!     Some(Delay::from_millis(10))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some(Delay::from_millis(20))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some(Delay::from_millis(10))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some(Delay::from_millis(20))
//! );
//! assert_eq!(model.next_delay(), None);
//! ```
use crate::{Delay, DelayPerPacketTrace};
use dyn_clone::DynClone;

const DEFAULT_RNG_SEED: u64 = 42;

/// This trait is used to convert a per-packet delay trace configuration into a per-packet delay trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait DelayPerPacketTraceConfig: DynClone + Send {
    fn into_model(self: Box<Self>) -> Box<dyn DelayPerPacketTrace>;
}

dyn_clone::clone_trait_object!(DelayPerPacketTraceConfig);

use rand::{rngs::StdRng, SeedableRng as _};
use rand_distr::{Distribution, Normal};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "truncated-normal")]
use super::solve_truncate::solve;

/// The model of a static per-packet delay trace.
///
/// This model always returns the same delay for the given amount of packets.
///
/// If the `count` is 0, the delay will be repeated forever.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticDelayPerPacketConfig;
/// # use netem_trace::{Delay, DelayPerPacketTrace};
/// let mut static_delay = StaticDelayPerPacketConfig::new()
///     .delay(Delay::from_millis(10))
///     .count(2)
///     .build();
/// assert_eq!(static_delay.next_delay(), Some(Delay::from_millis(10)));
/// assert_eq!(static_delay.next_delay(), Some(Delay::from_millis(10)));
/// assert_eq!(static_delay.next_delay(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticDelayPerPacket {
    pub delay: Delay,
    pub count: usize,
    current_count: usize,
}

/// The configuration struct for [`StaticDelayPerPacket`].
///
/// See [`StaticDelayPerPacket`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct StaticDelayPerPacketConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub delay: Option<Delay>,
    pub count: usize,
}

/// The model contains an array of per-packet delay trace models.
///
/// Combine multiple per-packet delay trace models into one per-packet delay pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedDelayPerPacketPatternConfig`]:
///
/// ```
/// use netem_trace::model::{StaticDelayPerPacketConfig, DelayPerPacketTraceConfig};
/// use netem_trace::{Delay, DelayPerPacketTrace};
/// #[cfg(feature = "human")]
/// let config_file_content = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":\"10ms\",\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":\"20ms\",\"count\":1}}],\"count\":2}}";
/// // The content would be "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}";
/// let des: Box<dyn DelayPerPacketTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// assert_eq!(
///     model.next_delay(),
///     Some(Delay::from_millis(10))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some(Delay::from_millis(20))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some(Delay::from_millis(10))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some(Delay::from_millis(20))
/// );
/// assert_eq!(model.next_delay(), None);
/// ```
///
/// You can also build manually:
///
/// ```
/// # use netem_trace::model::{StaticDelayPerPacketConfig, DelayPerPacketTraceConfig, RepeatedDelayPerPacketPatternConfig};
/// # use netem_trace::{Delay, DelayPerPacketTrace};
/// let pat = vec![
///     Box::new(
///         StaticDelayPerPacketConfig::new()
///             .delay(Delay::from_millis(10))
///             .count(1),
///     ) as Box<dyn DelayPerPacketTraceConfig>,
///     Box::new(
///         StaticDelayPerPacketConfig::new()
///             .delay(Delay::from_millis(20))
///             .count(1),
///     ) as Box<dyn DelayPerPacketTraceConfig>,
/// ];
/// let ser = Box::new(RepeatedDelayPerPacketPatternConfig::new().pattern(pat).count(2)) as Box<dyn DelayPerPacketTraceConfig>;
/// let ser_str = serde_json::to_string(&ser).unwrap();
/// #[cfg(feature = "human")]
/// let json_str = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":\"10ms\",\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":\"20ms\",\"count\":1}}],\"count\":2}}";
/// // The content would be "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}";
/// assert_eq!(ser_str, json_str);
/// ```
pub struct RepeatedDelayPerPacketPattern {
    pub pattern: Vec<Box<dyn DelayPerPacketTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn DelayPerPacketTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedDelayPerPacketPattern`].
///
/// See [`RepeatedDelayPerPacketPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedDelayPerPacketPatternConfig {
    pub pattern: Vec<Box<dyn DelayPerPacketTraceConfig>>,
    pub count: usize,
}

/// The model of a per-packet delay trace subjects to a normal distribution.
///
/// The delay will subject to N(mean, std_dev), but bounded within [lower_bound, upper_bound] (optional)
///
/// If the `count` is 0, the delay will be repeated forever, else it will be repeated `count` times.
///
/// ## Examples
///
/// A simple example without any bound on delay:
///
/// ```
/// # use netem_trace::model::NormalizedDelayPerPacketConfig;
/// # use netem_trace::{Delay, DelayPerPacketTrace};
/// let mut normal_delay = NormalizedDelayPerPacketConfig::new()
///     .mean(Delay::from_millis(12))
///     .std_dev(Delay::from_millis(1))
///     .count(2)
///     .seed(42)
///     .build();
/// assert_eq!(normal_delay.next_delay(), Some(Delay::from_nanos(12069428)));
/// assert_eq!(normal_delay.next_delay(), Some(Delay::from_nanos(12132938)));
/// assert_eq!(normal_delay.next_delay(), None);
/// ```
///
/// A more complex example with bounds on delay:
///
/// ```
/// # use netem_trace::model::NormalizedDelayPerPacketConfig;
/// # use netem_trace::{Delay, DelayPerPacketTrace};
/// let mut normal_delay = NormalizedDelayPerPacketConfig::new()
///     .mean(Delay::from_millis(12))
///     .std_dev(Delay::from_millis(1))
///     .count(2)
///     .seed(42)
///     .upper_bound(Delay::from_micros(12100))
///     .lower_bound(Delay::from_micros(11900))
///     .build();
/// assert_eq!(normal_delay.next_delay(), Some(Delay::from_nanos(12069428)));
/// assert_eq!(normal_delay.next_delay(), Some(Delay::from_nanos(12100000)));
/// assert_eq!(normal_delay.next_delay(), None);
/// ```
#[derive(Debug, Clone)]
pub struct NormalizedDelayPerPacket {
    pub mean: Delay,
    pub std_dev: Delay,
    pub upper_bound: Option<Delay>,
    pub lower_bound: Delay,
    pub seed: u64,
    pub count: usize,
    current_count: usize,
    rng: StdRng,
    normal: Normal<f64>,
}

/// The configuration struct for [`NormalizedDelayPerPacket`].
///
/// See [`NormalizedDelayPerPacket`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct NormalizedDelayPerPacketConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub mean: Option<Delay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub std_dev: Option<Delay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub upper_bound: Option<Delay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub lower_bound: Option<Delay>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub count: usize,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<u64>,
}

impl DelayPerPacketTrace for StaticDelayPerPacket {
    fn next_delay(&mut self) -> Option<Delay> {
        if self.count != 0 && self.count == self.current_count {
            None
        } else {
            self.current_count += 1;
            Some(self.delay)
        }
    }
}

impl DelayPerPacketTrace for RepeatedDelayPerPacketPattern {
    fn next_delay(&mut self) -> Option<Delay> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_delay() {
                Some(delay) => Some(delay),
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
                    self.next_delay()
                }
            }
        }
    }
}

impl DelayPerPacketTrace for NormalizedDelayPerPacket {
    fn next_delay(&mut self) -> Option<Delay> {
        if self.count != 0 && self.count == self.current_count {
            None
        } else {
            self.current_count += 1;
            let delay = self.normal.sample(&mut self.rng).max(0.0);
            let mut delay = Delay::from_secs_f64(delay);
            delay = delay.max(self.lower_bound);
            if let Some(upper_bound) = self.upper_bound {
                delay = delay.min(upper_bound);
            }
            Some(delay)
        }
    }
}

impl StaticDelayPerPacketConfig {
    pub fn new() -> Self {
        Self {
            delay: None,
            count: 0,
        }
    }

    pub fn delay(mut self, delay: Delay) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(&self) -> StaticDelayPerPacket {
        StaticDelayPerPacket {
            delay: self.delay.unwrap_or_else(|| Delay::from_millis(10)),
            count: self.count,
            current_count: 0,
        }
    }
}

impl RepeatedDelayPerPacketPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn DelayPerPacketTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedDelayPerPacketPattern {
        RepeatedDelayPerPacketPattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

impl NormalizedDelayPerPacketConfig {
    pub fn new() -> Self {
        Self {
            mean: None,
            std_dev: None,
            upper_bound: None,
            lower_bound: None,
            count: 0,
            seed: None,
        }
    }

    pub fn mean(mut self, mean: Delay) -> Self {
        self.mean = Some(mean);
        self
    }

    pub fn std_dev(mut self, std_dev: Delay) -> Self {
        self.std_dev = Some(std_dev);
        self
    }

    pub fn upper_bound(mut self, upper_bound: Delay) -> Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    pub fn lower_bound(mut self, lower_bound: Delay) -> Self {
        self.lower_bound = Some(lower_bound);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn random_seed(mut self) -> Self {
        self.seed = Some(rand::random());
        self
    }

    pub fn build(&self) -> NormalizedDelayPerPacket {
        let mean = self.mean.unwrap_or_else(|| Delay::from_millis(10));
        let std_dev = self.std_dev.unwrap_or(Delay::ZERO);
        let upper_bound = self.upper_bound;
        let lower_bound = self.lower_bound.unwrap_or(Delay::ZERO);
        let count = self.count;
        let seed = self.seed.unwrap_or(DEFAULT_RNG_SEED);
        let rng = StdRng::seed_from_u64(seed);
        let delay_mean = mean.as_secs_f64();
        let delay_std_dev = std_dev.as_secs_f64();
        let normal: Normal<f64> = Normal::new(delay_mean, delay_std_dev).unwrap();
        NormalizedDelayPerPacket {
            mean,
            std_dev,
            upper_bound,
            lower_bound,
            count,
            current_count: 0,
            seed,
            rng,
            normal,
        }
    }
}

#[cfg(feature = "truncated-normal")]
impl NormalizedDelayPerPacketConfig {
    /// This is another implementation for converting NormalizedPerPacketConfig into NormalizedDelayPerPacket, where the impact
    /// of truncation (`lower_bound` and `upper_bound` field) on the mathematical expectation of the distribution
    /// is taking account by modifying the center of the distribution.
    ///
    /// ## Examples
    ///
    /// ```
    ///
    /// # use netem_trace::model::NormalizedDelayPerPacketConfig;
    /// # use netem_trace::{Delay, DelayPerPacketTrace};
    /// # use crate::netem_trace::model::Forever;
    /// let normal_delay = NormalizedDelayPerPacketConfig::new()
    ///     .mean(Delay::from_millis(12))
    ///     .std_dev(Delay::from_millis(12))
    ///     .count(1_000_000)
    ///     .seed(42);
    ///
    /// let mut default_build = normal_delay.clone().build();
    /// let mut truncate_build = normal_delay.clone().build_truncated();
    ///
    /// fn avg_delay(mut model: impl DelayPerPacketTrace) -> Delay {
    ///     let mut count = 0;
    ///     std::iter::from_fn( move ||{
    ///         model.next_delay()
    ///     }).inspect(|_| count += 1).sum::<Delay>() / count
    /// }
    ///
    /// assert_eq!(avg_delay(default_build), Delay::from_nanos(12998335)); // significantly higher than the expected mean
    /// assert_eq!(avg_delay(truncate_build), Delay::from_nanos(11998818));
    ///
    /// let normal_delay = NormalizedDelayPerPacketConfig::new()
    ///     .mean(Delay::from_millis(12))
    ///     .std_dev(Delay::from_millis(12))
    ///     .lower_bound(Delay::from_millis(8))
    ///     .upper_bound(Delay::from_millis(20))
    ///     .count(1_000_000)
    ///     .seed(42);
    ///
    /// let mut default_build = normal_delay.clone().build();
    /// let mut truncate_build = normal_delay.clone().build_truncated();
    ///
    /// assert_eq!(avg_delay(default_build),  Delay::from_nanos(13234261)); // significantly higher than the expected mean
    /// assert_eq!(avg_delay(truncate_build), Delay::from_nanos(11999151));
    ///
    /// ```
    pub fn build_truncated(mut self) -> NormalizedDelayPerPacket {
        let mean = self
            .mean
            .unwrap_or_else(|| Delay::from_millis(12))
            .as_secs_f64();
        let sigma = self.std_dev.unwrap_or_else(|| Delay::ZERO).as_secs_f64() / mean;
        let lower = self
            .lower_bound
            .unwrap_or_else(|| Delay::ZERO)
            .as_secs_f64()
            / mean;
        let upper = self.upper_bound.map(|upper| upper.as_secs_f64() / mean);
        let new_mean = mean * solve(1f64, sigma, Some(lower), upper).unwrap_or(1f64);
        self.mean = Some(Delay::from_secs_f64(new_mean));
        self.build()
    }
}
macro_rules! impl_delay_per_packet_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl DelayPerPacketTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn DelayPerPacketTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_delay_per_packet_trace_config!(StaticDelayPerPacketConfig);
impl_delay_per_packet_trace_config!(NormalizedDelayPerPacketConfig);
impl_delay_per_packet_trace_config!(RepeatedDelayPerPacketPatternConfig);

/// Turn a [`DelayPerPacketTraceConfig`] into a forever repeated [`RepeatedDelayPerPacketPatternConfig`].
pub trait Forever: DelayPerPacketTraceConfig {
    fn forever(self) -> RepeatedDelayPerPacketPatternConfig;
}

/// Implement the [`Forever`] trait for the per-packet delay trace model config (any struct implements [`DelayPerPacketTraceConfig`]).
#[macro_export]
macro_rules! impl_forever_delay_per_packet {
    ($name:ident) => {
        impl Forever for $name {
            fn forever(self) -> RepeatedDelayPerPacketPatternConfig {
                RepeatedDelayPerPacketPatternConfig::new()
                    .pattern(vec![Box::new(self)])
                    .count(0)
            }
        }
    };
}

impl_forever_delay_per_packet!(StaticDelayPerPacketConfig);
impl_forever_delay_per_packet!(NormalizedDelayPerPacketConfig);

impl Forever for RepeatedDelayPerPacketPatternConfig {
    fn forever(self) -> RepeatedDelayPerPacketPatternConfig {
        self.count(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticDelayPerPacketConfig;
    use crate::DelayPerPacketTrace;

    #[test]
    fn test_static_delay_model() {
        let mut static_delay = StaticDelayPerPacketConfig::new()
            .delay(Delay::from_millis(10))
            .count(1)
            .build();
        assert_eq!(static_delay.next_delay(), Some(Delay::from_millis(10)));
        assert_eq!(static_delay.next_delay(), None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        let a = vec![
            Box::new(
                StaticDelayPerPacketConfig::new()
                    .delay(Delay::from_millis(10))
                    .count(1),
            ) as Box<dyn DelayPerPacketTraceConfig>,
            Box::new(
                StaticDelayPerPacketConfig::new()
                    .delay(Delay::from_millis(20))
                    .count(1),
            ) as Box<dyn DelayPerPacketTraceConfig>,
        ];
        let ser = Box::new(
            RepeatedDelayPerPacketPatternConfig::new()
                .pattern(a)
                .count(2),
        ) as Box<dyn DelayPerPacketTraceConfig>;
        let ser_str = serde_json::to_string(&ser).unwrap();
        #[cfg(feature = "human")]
        let des_str = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":\"10ms\",\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":\"20ms\",\"count\":1}}],\"count\":2}}";
        #[cfg(not(feature = "human"))]
        let des_str = "{\"RepeatedDelayPerPacketPatternConfig\":{\"pattern\":[{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"count\":1}},{\"StaticDelayPerPacketConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"count\":1}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn DelayPerPacketTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(model.next_delay(), Some(Delay::from_millis(10)));
    }
}
