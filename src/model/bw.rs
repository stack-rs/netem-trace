//! This module contains some predefined bandwidth trace models.
//!
//! Enabled with feature `bw-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticBw`]: A trace model with static bandwidth.
//! - [`NormalizedBw`]: A trace model whose bandwidth subjects to a normal distribution (can set upper and lower bounds, and can configure it to be truncated with `truncated-normal` feature enabled).
//! - [`RepeatedBwPattern`]: A trace model with a repeated bandwidth pattern.
//! - [`TraceBw`]: A trace model to replay compact bandwidth changes from file, especially useful for online sampled records.
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
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":\"12Mbps\",\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":\"24Mbps\",\"duration\":\"1s\"}}],\"count\":2}}";
//! // The content would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
//! // if the `human` feature is not enabled.
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
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
use rand::{rngs::StdRng, RngCore, SeedableRng};
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

#[cfg(feature = "truncated-normal")]
use super::solve_truncate::solve;

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
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
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
pub struct NormalizedBw<Rng = StdRng>
where
    Rng: RngCore,
{
    pub mean: Bandwidth,
    pub std_dev: Bandwidth,
    pub upper_bound: Option<Bandwidth>,
    pub lower_bound: Option<Bandwidth>,
    pub duration: Duration,
    pub step: Duration,
    pub seed: u64,
    rng: Rng,
    normal: Normal<f64>,
}

/// The configuration struct for [`NormalizedBw`].
///
/// See [`NormalizedBw`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct NormalizedBwConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub mean: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub std_dev: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub upper_bound: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
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
pub struct SawtoothBw<Rng = StdRng>
where
    Rng: RngCore,
{
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
    rng: Rng,
    noise: Normal<f64>,
}

/// The configuration struct for [`SawtoothBw`].
///
/// See [`SawtoothBw`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct SawtoothBwConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub bottom: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
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
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub std_dev: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub upper_noise_bound: Option<Bandwidth>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "human_bandwidth::serde")
    )]
    pub lower_noise_bound: Option<Bandwidth>,
}

/// The model contains an array of bandwidth trace models.
///
/// Combine multiple bandwidth trace models into one bandwidth pattern,
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
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":\"12Mbps\",\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":\"24Mbps\",\"duration\":\"1s\"}}],\"count\":2}}";
/// // The content would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
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
/// # #[cfg(feature = "human")]
/// # let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":\"12Mbps\",\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":\"24Mbps\",\"duration\":\"1s\"}}],\"count\":2}}";
/// // The json string would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// # #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
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

/// This model is used to enable a more compact trace.
/// It replays the bandwidth changes according to a trace file,
/// and is necessary for replaying sampled traces from Internet or production.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::TraceBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// use crate::netem_trace::model::Forever;
/// use netem_trace::model::BwTraceConfig;
///
/// let mut tracebw = TraceBwConfig::new().pattern(
///     vec![
///         (Duration::from_millis(1), vec![Bandwidth::from_mbps(2),Bandwidth::from_mbps(4),]),
///         (Duration::from_millis(2), vec![Bandwidth::from_mbps(1)]),
///        ]
/// ).build();
///
/// assert_eq!(tracebw.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(tracebw.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(tracebw.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
/// assert_eq!(tracebw.next_bw(), None);
/// assert_eq!(tracebw.next_bw(), None);
/// assert_eq!(tracebw.next_bw(), None);
///
/// let repeated_tracebw_config = TraceBwConfig::new().pattern(
///     vec![
///         (Duration::from_millis(1), vec![Bandwidth::from_mbps(2),Bandwidth::from_mbps(4),]),
///         (Duration::from_millis(2), vec![Bandwidth::from_mbps(1)]),
///        ]
/// ).forever();
///
/// let mut repeated_tracebw = repeated_tracebw_config.clone().build();
///
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
///
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
///
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(repeated_tracebw.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
///
/// let ser : Box<dyn BwTraceConfig> = Box::new(repeated_tracebw_config);
/// let ser_str = serde_json::to_string(&ser).unwrap();
///
/// # #[cfg(feature = "human")]
/// let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"TraceBwConfig\":{\"pattern\":[[\"1ms\",[\"2Mbps\",\"4Mbps\"]],[\"2ms\",[\"1Mbps\"]]]}}],\"count\":0}}";
/// // The json string would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"TraceBwConfig\":{\"pattern\":[[{\"secs\":0,\"nanos\":1000000},[{\"gbps\":0,\"bps\":2000000},{\"gbps\":0,\"bps\":4000000}]],[{\"secs\":0,\"nanos\":2000000},[{\"gbps\":0,\"bps\":1000000}]]]}}],\"count\":0}}"
/// // if the `human` feature is not enabled.
/// #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"TraceBwConfig\":{\"pattern\":[[{\"secs\":0,\"nanos\":1000000},[{\"gbps\":0,\"bps\":2000000},{\"gbps\":0,\"bps\":4000000}]],[{\"secs\":0,\"nanos\":2000000},[{\"gbps\":0,\"bps\":1000000}]]]}}],\"count\":0}}";
/// assert_eq!(ser_str, json_str);
///
/// let des: Box<dyn BwTraceConfig> = serde_json::from_str(json_str).unwrap();
/// let mut model = des.into_model();
///
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
///
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(2), Duration::from_millis(1))));
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(4), Duration::from_millis(1))));
/// assert_eq!(model.next_bw(), Some((Bandwidth::from_mbps(1), Duration::from_millis(2))));
/// ```
pub struct TraceBw {
    pub pattern: Vec<(Duration, Vec<Bandwidth>)>, // inner vector is never empty
    pub outer_index: usize,
    pub inner_index: usize,
}

/// The configuration struct for [`TraceBw`].
///
/// See [`TraceBw`] for more details.
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct TraceBwConfig {
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "tracebw_serde")
    )]
    pub pattern: Vec<(Duration, Vec<Bandwidth>)>,
}

impl TraceBwConfig {
    pub fn new() -> Self {
        Self { pattern: vec![] }
    }

    pub fn pattern(mut self, pattern: Vec<(Duration, Vec<Bandwidth>)>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn build(self) -> TraceBw {
        TraceBw {
            pattern: self
                .pattern
                .into_iter()
                .filter(|(_, bandwidths)| !bandwidths.is_empty())
                .collect(),
            outer_index: 0,
            inner_index: 0,
        }
    }
}

#[cfg(all(feature = "serde", feature = "human"))]
mod tracebw_serde {
    use super::*;
    use serde::ser::SerializeSeq;
    use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;
    use std::ops::{Deref, DerefMut};
    /// Deserializes a `Bandwidth` in human-readable format.
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `deserialize_with` annotations.
    pub fn deserialize<'a, T, D>(d: D) -> Result<T, D::Error>
    where
        Serde<T>: Deserialize<'a>,
        D: Deserializer<'a>,
    {
        Serde::deserialize(d).map(Serde::into_inner)
    }

    /// Serializes a `Bandwidth` in human-readable format.
    ///
    /// This function can be used with `serde_derive`'s `with` and
    /// `serialize_with` annotations.
    pub fn serialize<T, S>(d: &T, s: S) -> Result<S::Ok, S::Error>
    where
        for<'a> Serde<&'a T>: Serialize,
        S: Serializer,
    {
        Serde::from(d).serialize(s)
    }

    /// A wrapper type which implements `Serialize` and `Deserialize` for
    /// types involving `Bandwidth`.
    #[derive(Copy, Clone, Eq, Hash, PartialEq)]
    pub struct Serde<T>(T);

    impl<T> fmt::Debug for Serde<T>
    where
        T: fmt::Debug,
    {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            self.0.fmt(formatter)
        }
    }

    impl<T> Deref for Serde<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &self.0
        }
    }

    impl<T> DerefMut for Serde<T> {
        fn deref_mut(&mut self) -> &mut T {
            &mut self.0
        }
    }

    impl<T> Serde<T> {
        /// Consumes the `De`, returning the inner value.
        pub fn into_inner(self) -> T {
            self.0
        }
    }

    impl<T> From<T> for Serde<T> {
        fn from(val: T) -> Serde<T> {
            Serde(val)
        }
    }

    impl<'de> Deserialize<'de> for Serde<Vec<(Duration, Vec<Bandwidth>)>> {
        fn deserialize<D>(d: D) -> Result<Serde<Vec<(Duration, Vec<Bandwidth>)>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct V;

            impl<'de> de::Visitor<'de> for V {
                type Value = Vec<(Duration, Vec<Bandwidth>)>;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    fmt.write_str("a sequence of [str, [str, str, ...]]")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let mut pattern = Vec::with_capacity(seq.size_hint().unwrap_or(0));

                    while let Some((duration_str, bandwidths_str)) =
                        seq.next_element::<(String, Vec<String>)>()?
                    {
                        let duration =
                            humantime_serde::re::humantime::parse_duration(duration_str.as_str())
                                .map_err(|e| {
                                serde::de::Error::custom(format!(
                                    "Failed to parse duration '{}': {}",
                                    duration_str, e
                                ))
                            })?;

                        let bandwidths = bandwidths_str
                            .into_iter()
                            .map(|b| {
                                human_bandwidth::parse_bandwidth(&b).map_err(|e| {
                                    serde::de::Error::custom(format!(
                                        "Failed to parse bandwidth '{}': {}",
                                        b, e
                                    ))
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        pattern.push((duration, bandwidths));
                    }
                    Ok(pattern)
                }
            }

            d.deserialize_seq(V).map(Serde)
        }
    }

    impl ser::Serialize for Serde<Vec<(Duration, Vec<Bandwidth>)>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for (duration, bandwidths) in &self.0 {
                let time = humantime_serde::re::humantime::format_duration(*duration).to_string();
                let bandwidths = bandwidths
                    .iter()
                    .map(|item| human_bandwidth::format_bandwidth(*item).to_string())
                    .collect::<Vec<_>>();
                seq.serialize_element(&(time, bandwidths))?;
            }
            seq.end()
        }
    }

    impl ser::Serialize for Serde<&Vec<(Duration, Vec<Bandwidth>)>> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
            for (duration, bandwidths) in self.0 {
                let time = humantime_serde::re::humantime::format_duration(*duration).to_string();
                let bandwidths = bandwidths
                    .iter()
                    .map(|item| human_bandwidth::format_bandwidth(*item).to_string())
                    .collect::<Vec<_>>();
                seq.serialize_element(&(time, bandwidths))?;
            }
            seq.end()
        }
    }
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

impl<Rng: RngCore + Send> BwTrace for NormalizedBw<Rng> {
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

impl<Rng: RngCore + Send> BwTrace for SawtoothBw<Rng> {
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

impl BwTrace for TraceBw {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
        let result = self
            .pattern
            .get(self.outer_index)
            .and_then(|(duration, bandwidth)| {
                bandwidth
                    .get(self.inner_index)
                    .map(|bandwidth| (*bandwidth, *duration))
            });
        if result.is_some() {
            if self.pattern[self.outer_index].1.len() > self.inner_index + 1 {
                self.inner_index += 1;
            } else {
                self.outer_index += 1;
                self.inner_index = 0;
            }
        }
        result
    }
}

impl<Rng: RngCore> NormalizedBw<Rng> {
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
    /// Creates an uninitialized config
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

    /// Sets the mean
    ///
    /// If the mean is not set, 12Mbps will be used.
    pub fn mean(mut self, mean: Bandwidth) -> Self {
        self.mean = Some(mean);
        self
    }

    /// Sets the standard deviation
    ///
    /// If the standard deviation is not set, 0Mbps will be used.
    pub fn std_dev(mut self, std_dev: Bandwidth) -> Self {
        self.std_dev = Some(std_dev);
        self
    }

    /// Sets the upper bound
    ///
    /// If the upper bound is not set, the upper bound will be the one of [`Bandwidth`].
    pub fn upper_bound(mut self, upper_bound: Bandwidth) -> Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    /// Sets the lower bound
    ///
    /// If the lower bound is not set, the lower bound will be the one of [`Bandwidth`].
    pub fn lower_bound(mut self, lower_bound: Bandwidth) -> Self {
        self.lower_bound = Some(lower_bound);
        self
    }

    /// Sets the total duration of the trace
    ///
    /// If the total duration is not set, 1 second will be used.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets the duration of each value
    ///
    /// If the step is not set, 1ms will be used.
    pub fn step(mut self, step: Duration) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the seed for a random generator
    ///
    /// If the seed is not set, `42` will be used.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Allows to use a randomly generated seed
    ///
    /// This is equivalent to: `self.seed(rand::random())`
    pub fn random_seed(mut self) -> Self {
        self.seed = Some(rand::random());
        self
    }

    /// Creates a new [`NormalizedBw`] corresponding to this config.
    ///
    /// The created model will use [`StdRng`] as source of randomness (the call is equivalent to `self.build_with_rng::<StdRng>()`).
    /// It should be sufficient for most cases, but [`StdRng`] is not a portable random number generator,
    /// so one may want to use a portable random number generator like [`ChaCha`](https://crates.io/crates/rand_chacha),
    /// to this end one can use [`build_with_rng`](Self::build_with_rng).
    pub fn build(self) -> NormalizedBw {
        self.build_with_rng()
    }

    /// Creates a new [`NormalizedBw`] corresponding to this config.
    ///
    /// Unlike [`build`](Self::build), this method let you choose the random generator.
    ///
    /// # Example
    /// ```rust
    /// # use netem_trace::model::NormalizedBwConfig;
    /// # use netem_trace::{Bandwidth, BwTrace};
    /// # use std::time::Duration;
    /// # use rand::rngs::StdRng;
    /// # use rand_chacha::ChaCha20Rng;
    ///
    /// let normal_bw = NormalizedBwConfig::new()
    ///     .mean(Bandwidth::from_mbps(12))
    ///     .std_dev(Bandwidth::from_mbps(1))
    ///     .duration(Duration::from_millis(3))
    ///     .seed(42);
    ///
    /// let mut default_build = normal_bw.clone().build();
    /// let mut std_build = normal_bw.clone().build_with_rng::<StdRng>();
    /// // ChaCha is deterministic and portable, unlike StdRng
    /// let mut chacha_build = normal_bw.clone().build_with_rng::<ChaCha20Rng>();
    ///
    /// for cha in [12044676, 11754367, 11253775] {
    ///     let default = default_build.next_bw();
    ///     let std = std_build.next_bw();
    ///     let chacha = chacha_build.next_bw();
    ///
    ///     assert!(default.is_some());
    ///     assert_eq!(default, std);
    ///     assert_ne!(default, chacha);
    ///     assert_eq!(chacha, Some((Bandwidth::from_bps(cha), Duration::from_millis(1))));
    /// }
    ///
    /// assert_eq!(default_build.next_bw(), None);
    /// assert_eq!(std_build.next_bw(), None);
    /// assert_eq!(chacha_build.next_bw(), None);
    /// ```
    pub fn build_with_rng<Rng: SeedableRng + RngCore>(self) -> NormalizedBw<Rng> {
        let mean = self.mean.unwrap_or_else(|| Bandwidth::from_mbps(12));
        let std_dev = self.std_dev.unwrap_or_else(|| Bandwidth::from_mbps(0));
        let upper_bound = self.upper_bound;
        let lower_bound = self.lower_bound;
        let duration = self.duration.unwrap_or_else(|| Duration::from_secs(1));
        let step = self.step.unwrap_or_else(|| Duration::from_millis(1));
        let seed = self.seed.unwrap_or(DEFAULT_RNG_SEED);
        let rng = Rng::seed_from_u64(seed);
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

#[cfg(feature = "truncated-normal")]
impl NormalizedBwConfig {
    /// This is another implementation for converting NormalizedBwConfig into NormalizedBw, where the impact
    /// of truncation (`lower_bound` and `upper_bound` field) on the mathematical expectation of the distribution
    /// is taking account by modifying the center of the distribution.
    ///
    /// ## Examples
    ///
    /// ```
    ///
    /// # use netem_trace::model::NormalizedBwConfig;
    /// # use netem_trace::{Bandwidth, Duration, BwTrace};
    /// # use crate::netem_trace::model::Forever;
    /// let normal_bw = NormalizedBwConfig::new()
    ///     .mean(Bandwidth::from_mbps(12))
    ///     .std_dev(Bandwidth::from_mbps(12))
    ///     .duration(Duration::from_secs(100))
    ///     .step(Duration::from_millis(1))
    ///     .seed(42);
    ///
    /// let mut default_build = normal_bw.clone().build();
    /// let mut truncate_build = normal_bw.clone().build_truncated();
    ///
    /// fn avg_mbps(mut model: impl BwTrace) -> f64{
    ///     let mut count = 0;
    ///     std::iter::from_fn( move ||{
    ///         model.next_bw().map(|b| b.0.as_gbps_f64() * 1000.0)
    ///     }).inspect(|_| count += 1).sum::<f64>() / count as f64
    /// }
    ///
    /// assert_eq!(avg_mbps(default_build), 12.974758080079994); // significantly higher than the expected mean
    /// assert_eq!(avg_mbps(truncate_build), 11.97642456625989);
    ///
    /// let normal_bw = NormalizedBwConfig::new()
    ///     .mean(Bandwidth::from_mbps(12))
    ///     .std_dev(Bandwidth::from_mbps(12))
    ///     .duration(Duration::from_secs(100))
    ///     .lower_bound(Bandwidth::from_mbps(8))
    ///     .upper_bound(Bandwidth::from_mbps(20))
    ///     .step(Duration::from_millis(1))
    ///     .seed(42);
    ///
    /// let mut default_build = normal_bw.clone().build();
    /// let mut truncate_build = normal_bw.clone().build_truncated();
    ///
    /// assert_eq!(avg_mbps(default_build),  13.221356471729928); // significantly higher than the expected mean
    /// assert_eq!(avg_mbps(truncate_build), 11.978819427569897);
    ///
    /// ```
    pub fn build_truncated(self) -> NormalizedBw {
        self.build_truncated_with_rng()
    }

    /// Similar to [`build_truncated`](Self::build_truncated) but let you choose the random generator.
    ///
    /// See [`build`](Self::build) for details about the reason for using another random number generator than [`StdRng`].
    pub fn build_truncated_with_rng<Rng: SeedableRng + RngCore>(mut self) -> NormalizedBw<Rng> {
        let mean = self
            .mean
            .unwrap_or_else(|| Bandwidth::from_mbps(12))
            .as_gbps_f64();
        let sigma = self
            .std_dev
            .unwrap_or_else(|| Bandwidth::from_mbps(0))
            .as_gbps_f64()
            / mean;
        let lower = self
            .lower_bound
            .unwrap_or_else(|| Bandwidth::from_mbps(0))
            .as_gbps_f64()
            / mean;
        let upper = self.upper_bound.map(|upper| upper.as_gbps_f64() / mean);
        let new_mean = mean * solve(1f64, sigma, Some(lower), upper).unwrap_or(1f64);
        self.mean = Some(Bandwidth::from_gbps_f64(new_mean));
        self.build_with_rng()
    }
}

impl SawtoothBwConfig {
    /// Creates an uninitialized config
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

    /// Sets the total duration of the trace
    ///
    /// If the total duration is not set, 1 second will be used.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets the duration of each value
    ///
    /// If the step is not set, 1ms will be used.
    pub fn step(mut self, step: Duration) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the seed for a random generator
    ///
    /// If the seed is not set, `42` will be used.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Allows to use a randomly generated seed
    ///
    /// This is equivalent to: `self.seed(rand::random())`
    pub fn random_seed(mut self) -> Self {
        self.seed = Some(rand::random());
        self
    }

    /// Sets the standard deviation
    ///
    /// If the standard deviation is not set, 0Mbps will be used.
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

    /// Creates a new [`SawtoothBw`] corresponding to this config.
    ///
    /// The created model will use [`StdRng`] as source of randomness (the call is equivalent to `self.build_with_rng::<StdRng>()`).
    /// It should be sufficient for most cases, but [`StdRng`] is not a portable random number generator,
    /// so one may want to use a portable random number generator like [`ChaCha`](https://crates.io/crates/rand_chacha),
    /// to this end one can use [`build_with_rng`](Self::build_with_rng).
    pub fn build(self) -> SawtoothBw {
        self.build_with_rng()
    }

    /// Creates a new [`SawtoothBw`] corresponding to this config.
    ///
    /// Unlike [`build`](Self::build), this method let you choose the random generator.
    ///
    /// # Example
    /// ```rust
    /// # use netem_trace::model::SawtoothBwConfig;
    /// # use netem_trace::{Bandwidth, BwTrace};
    /// # use std::time::Duration;
    /// # use rand::rngs::StdRng;
    /// # use rand_chacha::ChaCha20Rng;
    ///
    /// let sawtooth_bw = SawtoothBwConfig::new()
    ///     .bottom(Bandwidth::from_mbps(12))
    ///     .top(Bandwidth::from_mbps(16))
    ///     .std_dev(Bandwidth::from_mbps(1))
    ///     .duration(Duration::from_millis(3))
    ///     .interval(Duration::from_millis(5))
    ///     .duty_ratio(0.8)
    ///     .seed(42);
    ///
    /// let mut default_build = sawtooth_bw.clone().build();
    /// let mut std_build = sawtooth_bw.clone().build_with_rng::<StdRng>();
    /// // ChaCha is deterministic and portable, unlike StdRng
    /// let mut chacha_build = sawtooth_bw.clone().build_with_rng::<ChaCha20Rng>();
    ///
    /// for cha in [12044676, 12754367, 13253775] {
    ///     let default = default_build.next_bw();
    ///     let std = std_build.next_bw();
    ///     let chacha = chacha_build.next_bw();
    ///
    ///     assert!(default.is_some());
    ///     assert_eq!(default, std);
    ///     assert_ne!(default, chacha);
    ///     assert_eq!(chacha, Some((Bandwidth::from_bps(cha), Duration::from_millis(1))));
    /// }
    ///
    /// assert_eq!(default_build.next_bw(), None);
    /// assert_eq!(std_build.next_bw(), None);
    /// assert_eq!(chacha_build.next_bw(), None);
    /// ```
    pub fn build_with_rng<Rng: RngCore + SeedableRng>(self) -> SawtoothBw<Rng> {
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
        let rng = Rng::seed_from_u64(seed);
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
impl_bw_trace_config!(TraceBwConfig);

/// Turn a [`BwTraceConfig`] into a forever repeated [`RepeatedBwPatternConfig`].
pub trait Forever: BwTraceConfig {
    fn forever(self) -> RepeatedBwPatternConfig;
}

/// Implement the [`Forever`] trait for the bandwidth trace model config (any struct implements [`BwTraceConfig`]).
#[macro_export]
macro_rules! impl_forever {
    ($name:ident) => {
        impl Forever for $name {
            fn forever(self) -> RepeatedBwPatternConfig {
                RepeatedBwPatternConfig::new()
                    .pattern(vec![Box::new(self)])
                    .count(0)
            }
        }
    };
}

impl_forever!(StaticBwConfig);
impl_forever!(NormalizedBwConfig);
impl_forever!(SawtoothBwConfig);
impl_forever!(TraceBwConfig);

impl Forever for RepeatedBwPatternConfig {
    fn forever(self) -> RepeatedBwPatternConfig {
        self.count(0)
    }
}
