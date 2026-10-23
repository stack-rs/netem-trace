//! This module contains some predefined packet delivery opportunity (PDO) trace models.
//!
//! Enabled with feature `pdo-model` or `model`.
//!
//! These traces have no concept of bandwidth. They say when bytes can be delivered,
//! like a mahimahi trace or the traces recorded by CellReplay.
//!
//! ## Predefined models
//!
//! - [`StaticHeavyPdo`]: A heavy PDO trace model with a fixed interval and size.
//! - [`TraceHeavyPdo`]: A heavy PDO trace model replaying a list of `(interval, bytes)`.
//! - [`RepeatedHeavyPdoPattern`]: A heavy PDO trace model with a repeated pattern.
//! - [`TraceLightPdo`]: A light PDO trace model replaying a list of [`LightPdo`] records,
//!   which can be loaded from a CellReplay light PDO trace with [`load_cellreplay_light_pdo_trace`].
//!
//! ## Examples
//!
//! ```
//! # use netem_trace::model::StaticHeavyPdoConfig;
//! # use netem_trace::{Duration, HeavyPdoTrace};
//! let mut static_pdo = StaticHeavyPdoConfig::new()
//!     .interval(Duration::from_millis(1))
//!     .bytes(1500)
//!     .count(2)
//!     .build();
//! assert_eq!(static_pdo.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
//! assert_eq!(static_pdo.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
//! assert_eq!(static_pdo.next_heavy_pdo(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::HeavyPdoTraceConfig;
//! # use netem_trace::{Duration, HeavyPdoTrace};
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedHeavyPdoPatternConfig\":{\"pattern\":[{\"TraceHeavyPdoConfig\":{\"pattern\":[[\"1ms\",1500],[\"0s\",1500]]}}],\"count\":2}}";
//! // The content would be "{\"RepeatedHeavyPdoPatternConfig\":{\"pattern\":[{\"TraceHeavyPdoConfig\":{\"pattern\":[[{\"secs\":0,\"nanos\":1000000},1500],[{\"secs\":0,\"nanos\":0},1500]]}}],\"count\":2}}"
//! // if the `human` feature is not enabled.
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedHeavyPdoPatternConfig\":{\"pattern\":[{\"TraceHeavyPdoConfig\":{\"pattern\":[[{\"secs\":0,\"nanos\":1000000},1500],[{\"secs\":0,\"nanos\":0},1500]]}}],\"count\":2}}";
//! let des: Box<dyn HeavyPdoTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
//! assert_eq!(model.next_heavy_pdo(), Some((Duration::ZERO, 1500)));
//! assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
//! assert_eq!(model.next_heavy_pdo(), Some((Duration::ZERO, 1500)));
//! assert_eq!(model.next_heavy_pdo(), None);
//! ```
use crate::{Delay, Duration, HeavyPdoTrace, LightPdo, LightPdoTrace};
use dyn_clone::DynClone;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// This trait is used to convert a heavy PDO trace configuration into a heavy PDO trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait HeavyPdoTraceConfig: DynClone + Send + std::fmt::Debug {
    fn into_model(self: Box<Self>) -> Box<dyn HeavyPdoTrace>;
}

dyn_clone::clone_trait_object!(HeavyPdoTraceConfig);

/// This trait is used to convert a light PDO trace configuration into a light PDO trace model.
///
/// See [`HeavyPdoTraceConfig`] for why the configuration and the model are separated.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait LightPdoTraceConfig: DynClone + Send + std::fmt::Debug {
    fn into_model(self: Box<Self>) -> Box<dyn LightPdoTrace>;
}

dyn_clone::clone_trait_object!(LightPdoTraceConfig);

/// How many bytes one PDO can deliver in CellReplay (both heavy and light PDO traces).
///
/// It is the default PDO size of [`StaticHeavyPdo`] and [`TraceLightPdo`].
/// For mahimahi traces, use `MAHIMAHI_PDO_BYTES` (1504 bytes) instead.
pub const CELLREPLAY_PDO_SIZE: u64 = 1400;

/// The model of a static heavy PDO trace.
///
/// This model always returns the same `(interval, bytes)` for `count` times.
///
/// If the `count` is 0, it will be repeated forever.
///
/// If `bytes` is not set, it defaults to [`CELLREPLAY_PDO_SIZE`].
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticHeavyPdoConfig;
/// # use netem_trace::{Duration, HeavyPdoTrace};
/// let mut static_pdo = StaticHeavyPdoConfig::new()
///     .interval(Duration::from_millis(1))
///     .bytes(1500)
///     .count(1)
///     .build();
/// assert_eq!(static_pdo.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
/// assert_eq!(static_pdo.next_heavy_pdo(), None);
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StaticHeavyPdo {
    pub interval: Duration,
    pub bytes: u64,
    pub count: usize,
    current_count: usize,
}

/// The configuration struct for [`StaticHeavyPdo`].
///
/// See [`StaticHeavyPdo`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct StaticHeavyPdoConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub interval: Option<Duration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bytes: Option<u64>,
    pub count: usize,
}

/// The model of a heavy PDO trace replaying a list of `(interval, bytes)`.
///
/// Each item is returned once, in order. Use [`RepeatedHeavyPdoPattern`] to repeat it.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::TraceHeavyPdoConfig;
/// # use netem_trace::{Duration, HeavyPdoTrace};
/// let mut trace_pdo = TraceHeavyPdoConfig::new()
///     .pattern(vec![
///         (Duration::from_millis(1), 3000),
///         (Duration::from_millis(2), 1500),
///     ])
///     .build();
/// assert_eq!(trace_pdo.next_heavy_pdo(), Some((Duration::from_millis(1), 3000)));
/// assert_eq!(trace_pdo.next_heavy_pdo(), Some((Duration::from_millis(2), 1500)));
/// assert_eq!(trace_pdo.next_heavy_pdo(), None);
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TraceHeavyPdo {
    pub pattern: Vec<(Duration, u64)>,
    pub index: usize,
}

/// The configuration struct for [`TraceHeavyPdo`].
///
/// See [`TraceHeavyPdo`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TraceHeavyPdoConfig {
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "heavy_pattern_serde")
    )]
    pub pattern: Vec<(Duration, u64)>,
}

/// The model contains an array of heavy PDO trace models.
///
/// Combine multiple heavy PDO trace models into one pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::{HeavyPdoTraceConfig, RepeatedHeavyPdoPatternConfig, StaticHeavyPdoConfig};
/// # use netem_trace::{Duration, HeavyPdoTrace};
/// let pat = vec![
///     Box::new(
///         StaticHeavyPdoConfig::new()
///             .interval(Duration::from_millis(1))
///             .bytes(1500)
///             .count(1),
///     ) as Box<dyn HeavyPdoTraceConfig>,
///     Box::new(
///         StaticHeavyPdoConfig::new()
///             .interval(Duration::from_millis(2))
///             .bytes(3000)
///             .count(1),
///     ) as Box<dyn HeavyPdoTraceConfig>,
/// ];
/// let mut model = RepeatedHeavyPdoPatternConfig::new().pattern(pat).count(2).build();
/// assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
/// assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(2), 3000)));
/// assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(1), 1500)));
/// assert_eq!(model.next_heavy_pdo(), Some((Duration::from_millis(2), 3000)));
/// assert_eq!(model.next_heavy_pdo(), None);
/// ```
#[derive(Default)]
pub struct RepeatedHeavyPdoPattern {
    pub pattern: Vec<Box<dyn HeavyPdoTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn HeavyPdoTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedHeavyPdoPattern`].
///
/// See [`RepeatedHeavyPdoPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Debug, Clone)]
pub struct RepeatedHeavyPdoPatternConfig {
    pub pattern: Vec<Box<dyn HeavyPdoTraceConfig>>,
    pub count: usize,
}

/// The model of a light PDO trace replaying a list of [`LightPdo`] records.
///
/// Each record is returned once, in order. Every light PDO can deliver `pdo_size` bytes,
/// which defaults to [`CELLREPLAY_PDO_SIZE`].
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::TraceLightPdoConfig;
/// # use netem_trace::{Delay, Duration, LightPdo, LightPdoTrace};
/// let record = LightPdo {
///     timestamp: Duration::from_millis(0),
///     base_delay: Delay::from_millis(30),
///     opportunities: vec![Duration::from_millis(0), Duration::from_millis(2)],
/// };
/// let mut trace_pdo = TraceLightPdoConfig::new()
///     .pattern(vec![record.clone()])
///     .pdo_size(1200)
///     .build();
/// assert_eq!(trace_pdo.pdo_size(), 1200);
/// assert_eq!(trace_pdo.next_light_pdo(), Some(record));
/// assert_eq!(trace_pdo.next_light_pdo(), None);
/// ```
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TraceLightPdo {
    pub pattern: Vec<LightPdo>,
    pub pdo_size: u64,
    pub index: usize,
}

/// The configuration struct for [`TraceLightPdo`].
///
/// See [`TraceLightPdo`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TraceLightPdoConfig {
    pub pattern: Vec<LightPdo>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub pdo_size: Option<u64>,
}

impl HeavyPdoTrace for StaticHeavyPdo {
    fn next_heavy_pdo(&mut self) -> Option<(Duration, u64)> {
        if self.count != 0 && self.count == self.current_count {
            None
        } else {
            self.current_count += 1;
            Some((self.interval, self.bytes))
        }
    }
}

impl HeavyPdoTrace for TraceHeavyPdo {
    fn next_heavy_pdo(&mut self) -> Option<(Duration, u64)> {
        let pdo = self.pattern.get(self.index).copied();
        if pdo.is_some() {
            self.index += 1;
        }
        pdo
    }
}

impl HeavyPdoTrace for RepeatedHeavyPdoPattern {
    fn next_heavy_pdo(&mut self) -> Option<(Duration, u64)> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_heavy_pdo() {
                Some(pdo) => Some(pdo),
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
                    self.next_heavy_pdo()
                }
            }
        }
    }
}

impl LightPdoTrace for TraceLightPdo {
    fn next_light_pdo(&mut self) -> Option<LightPdo> {
        let record = self.pattern.get(self.index).cloned();
        if record.is_some() {
            self.index += 1;
        }
        record
    }

    fn pdo_size(&self) -> u64 {
        self.pdo_size
    }
}

impl StaticHeavyPdoConfig {
    pub fn new() -> Self {
        Self {
            interval: None,
            bytes: None,
            count: 0,
        }
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn bytes(mut self, bytes: u64) -> Self {
        self.bytes = Some(bytes);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> StaticHeavyPdo {
        StaticHeavyPdo {
            interval: self.interval.unwrap_or_else(|| Duration::from_millis(1)),
            bytes: self.bytes.unwrap_or(CELLREPLAY_PDO_SIZE),
            count: self.count,
            current_count: 0,
        }
    }
}

impl TraceHeavyPdoConfig {
    pub fn new() -> Self {
        Self { pattern: vec![] }
    }

    pub fn pattern(mut self, pattern: Vec<(Duration, u64)>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn build(self) -> TraceHeavyPdo {
        TraceHeavyPdo {
            pattern: self.pattern,
            index: 0,
        }
    }
}

impl RepeatedHeavyPdoPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn HeavyPdoTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedHeavyPdoPattern {
        RepeatedHeavyPdoPattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

impl TraceLightPdoConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            pdo_size: None,
        }
    }

    pub fn pattern(mut self, pattern: Vec<LightPdo>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn pdo_size(mut self, pdo_size: u64) -> Self {
        self.pdo_size = Some(pdo_size);
        self
    }

    pub fn build(self) -> TraceLightPdo {
        TraceLightPdo {
            pattern: self.pattern,
            pdo_size: self.pdo_size.unwrap_or(CELLREPLAY_PDO_SIZE),
            index: 0,
        }
    }
}

/// Load a CellReplay light PDO (packet train) trace to a [`TraceLightPdoConfig`].
///
/// Each line of the trace is `timestamp base_delay pdo_0 pdo_1 ...`, separated by spaces,
/// all in milliseconds. The PDO times are measured from the start of the active period.
///
/// Returns `Err` string if the trace is invalid.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::load_cellreplay_light_pdo_trace;
/// # use netem_trace::{Delay, Duration, LightPdo, LightPdoTrace};
/// let mut trace = load_cellreplay_light_pdo_trace("0 30 0 2 2\n100 32 0 1")
///     .unwrap()
///     .build();
/// assert_eq!(
///     trace.next_light_pdo(),
///     Some(LightPdo {
///         timestamp: Duration::from_millis(0),
///         base_delay: Delay::from_millis(30),
///         opportunities: vec![
///             Duration::from_millis(0),
///             Duration::from_millis(2),
///             Duration::from_millis(2),
///         ],
///     })
/// );
/// ```
pub fn load_cellreplay_light_pdo_trace(trace: &str) -> Result<TraceLightPdoConfig, &'static str> {
    let mut pattern: Vec<LightPdo> = vec![];
    for line in trace.lines() {
        let mut words = line.split_whitespace();
        let mut next_ms = || -> Result<Option<Duration>, &'static str> {
            match words.next() {
                Some(word) => word
                    .parse::<u64>()
                    .map(|ms| Some(Duration::from_millis(ms)))
                    .map_err(|_| "values must be nonnegative integers in milliseconds"),
                None => Ok(None),
            }
        };
        let timestamp = next_ms()?.ok_or("invalid empty line")?;
        let base_delay: Delay = next_ms()?.ok_or("each line must have a base delay")?;
        let mut opportunities = vec![];
        while let Some(opportunity) = next_ms()? {
            if opportunities.last().is_some_and(|last| opportunity < *last) {
                return Err("PDO times must be monotonically nondecreasing");
            }
            opportunities.push(opportunity);
        }
        if pattern
            .last()
            .is_some_and(|last| timestamp < last.timestamp)
        {
            return Err("timestamps must be monotonically nondecreasing");
        }
        pattern.push(LightPdo {
            timestamp,
            base_delay,
            opportunities,
        });
    }
    if pattern.is_empty() {
        return Err("trace must have at least one line");
    }
    Ok(TraceLightPdoConfig::new().pattern(pattern))
}

macro_rules! impl_heavy_pdo_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl HeavyPdoTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn HeavyPdoTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_heavy_pdo_trace_config!(StaticHeavyPdoConfig);
impl_heavy_pdo_trace_config!(TraceHeavyPdoConfig);
impl_heavy_pdo_trace_config!(RepeatedHeavyPdoPatternConfig);

#[cfg_attr(feature = "serde", typetag::serde)]
impl LightPdoTraceConfig for TraceLightPdoConfig {
    fn into_model(self: Box<TraceLightPdoConfig>) -> Box<dyn LightPdoTrace> {
        Box::new(self.build())
    }
}

impl<Model: HeavyPdoTrace + 'static> From<Model> for Box<dyn HeavyPdoTrace> {
    fn from(model: Model) -> Self {
        Box::new(model)
    }
}

impl<Model: LightPdoTrace + 'static> From<Model> for Box<dyn LightPdoTrace> {
    fn from(model: Model) -> Self {
        Box::new(model)
    }
}

/// Turn a [`HeavyPdoTraceConfig`] into a forever repeated [`RepeatedHeavyPdoPatternConfig`].
pub trait Forever: HeavyPdoTraceConfig {
    fn forever(self) -> RepeatedHeavyPdoPatternConfig;
}

/// Implement the [`Forever`] trait for the heavy PDO trace model config (any struct implements [`HeavyPdoTraceConfig`]).
#[macro_export]
macro_rules! impl_forever_heavy_pdo {
    ($name:ident) => {
        impl Forever for $name {
            fn forever(self) -> RepeatedHeavyPdoPatternConfig {
                RepeatedHeavyPdoPatternConfig::new()
                    .pattern(vec![Box::new(self)])
                    .count(0)
            }
        }
    };
}

impl_forever_heavy_pdo!(StaticHeavyPdoConfig);
impl_forever_heavy_pdo!(TraceHeavyPdoConfig);

impl Forever for RepeatedHeavyPdoPatternConfig {
    fn forever(self) -> RepeatedHeavyPdoPatternConfig {
        self.count(0)
    }
}

#[cfg(all(feature = "serde", feature = "human"))]
mod heavy_pattern_serde {
    use super::Duration;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        v: &[(Duration, u64)],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        v.iter()
            .map(|(interval, bytes)| (humantime_serde::Serde::from(interval), bytes))
            .collect::<Vec<_>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<(Duration, u64)>, D::Error> {
        let v: Vec<(humantime_serde::Serde<Duration>, u64)> = Vec::deserialize(deserializer)?;
        Ok(v.into_iter()
            .map(|(interval, bytes)| (interval.into_inner(), bytes))
            .collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_static_heavy_pdo() {
        let mut model = StaticHeavyPdoConfig::new()
            .interval(Duration::from_millis(2))
            .bytes(1400)
            .count(2)
            .build();
        assert_eq!(
            model.next_heavy_pdo(),
            Some((Duration::from_millis(2), 1400))
        );
        assert_eq!(
            model.next_heavy_pdo(),
            Some((Duration::from_millis(2), 1400))
        );
        assert_eq!(model.next_heavy_pdo(), None);
    }

    #[test]
    fn test_default_pdo_size() {
        let mut heavy = StaticHeavyPdoConfig::new().count(1).build();
        assert_eq!(
            heavy.next_heavy_pdo(),
            Some((Duration::from_millis(1), CELLREPLAY_PDO_SIZE))
        );

        let light = TraceLightPdoConfig::new().build();
        assert_eq!(light.pdo_size(), CELLREPLAY_PDO_SIZE);
        let light = load_cellreplay_light_pdo_trace("0 30 0").unwrap().build();
        assert_eq!(light.pdo_size(), CELLREPLAY_PDO_SIZE);
        let light = TraceLightPdoConfig::new().pdo_size(1500).build();
        assert_eq!(light.pdo_size(), 1500);
    }

    #[test]
    fn test_forever() {
        let mut model = TraceHeavyPdoConfig::new()
            .pattern(vec![(Duration::from_millis(1), 1500)])
            .forever()
            .build();
        for _ in 0..5 {
            assert_eq!(
                model.next_heavy_pdo(),
                Some((Duration::from_millis(1), 1500))
            );
        }
    }

    #[test]
    fn test_empty_repeated() {
        let mut model = RepeatedHeavyPdoPatternConfig::new().build();
        assert_eq!(model.next_heavy_pdo(), None);
    }

    #[test]
    fn test_load_cellreplay_light_pdo() {
        assert_eq!(
            load_cellreplay_light_pdo_trace(""),
            Err("trace must have at least one line")
        );
        assert_eq!(
            load_cellreplay_light_pdo_trace("0 30\n\n10 30"),
            Err("invalid empty line")
        );
        assert_eq!(
            load_cellreplay_light_pdo_trace("0"),
            Err("each line must have a base delay")
        );
        assert_eq!(
            load_cellreplay_light_pdo_trace("0 30 0 x"),
            Err("values must be nonnegative integers in milliseconds")
        );
        assert_eq!(
            load_cellreplay_light_pdo_trace("0 30 0 3 2"),
            Err("PDO times must be monotonically nondecreasing")
        );
        assert_eq!(
            load_cellreplay_light_pdo_trace("10 30 0\n5 30 0"),
            Err("timestamps must be monotonically nondecreasing")
        );

        // a line with no PDO is fine, it means no light PDO for that period
        let mut trace = load_cellreplay_light_pdo_trace("0 30 0 1 1 4\n100 25\n")
            .unwrap()
            .build();
        assert_eq!(
            trace.next_light_pdo(),
            Some(LightPdo {
                timestamp: Duration::ZERO,
                base_delay: Delay::from_millis(30),
                opportunities: [0, 1, 1, 4].map(Duration::from_millis).to_vec(),
            })
        );
        assert_eq!(
            trace.next_light_pdo(),
            Some(LightPdo {
                timestamp: Duration::from_millis(100),
                base_delay: Delay::from_millis(25),
                opportunities: vec![],
            })
        );
        assert_eq!(trace.next_light_pdo(), None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        let heavy = Box::new(
            RepeatedHeavyPdoPatternConfig::new()
                .pattern(vec![Box::new(TraceHeavyPdoConfig::new().pattern(vec![
                    (Duration::from_millis(1), 1500),
                    (Duration::ZERO, 1500),
                ])) as Box<dyn HeavyPdoTraceConfig>])
                .count(2),
        ) as Box<dyn HeavyPdoTraceConfig>;
        let ser_str = serde_json::to_string(&heavy).unwrap();
        #[cfg(feature = "human")]
        let des_str = "{\"RepeatedHeavyPdoPatternConfig\":{\"pattern\":[{\"TraceHeavyPdoConfig\":{\"pattern\":[[\"1ms\",1500],[\"0s\",1500]]}}],\"count\":2}}";
        #[cfg(not(feature = "human"))]
        let des_str = "{\"RepeatedHeavyPdoPatternConfig\":{\"pattern\":[{\"TraceHeavyPdoConfig\":{\"pattern\":[[{\"secs\":0,\"nanos\":1000000},1500],[{\"secs\":0,\"nanos\":0},1500]]}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn HeavyPdoTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_heavy_pdo(),
            Some((Duration::from_millis(1), 1500))
        );

        let light = Box::new(load_cellreplay_light_pdo_trace("5 30 0 2").unwrap())
            as Box<dyn LightPdoTraceConfig>;
        let ser_str = serde_json::to_string(&light).unwrap();
        #[cfg(feature = "human")]
        let des_str = "{\"TraceLightPdoConfig\":{\"pattern\":[{\"timestamp\":\"5ms\",\"base_delay\":\"30ms\",\"opportunities\":[\"0s\",\"2ms\"]}]}}";
        #[cfg(not(feature = "human"))]
        let des_str = "{\"TraceLightPdoConfig\":{\"pattern\":[{\"timestamp\":{\"secs\":0,\"nanos\":5000000},\"base_delay\":{\"secs\":0,\"nanos\":30000000},\"opportunities\":[{\"secs\":0,\"nanos\":0},{\"secs\":0,\"nanos\":2000000}]}]}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn LightPdoTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_light_pdo().unwrap().opportunities,
            vec![Duration::ZERO, Duration::from_millis(2)]
        );
        assert_eq!(model.pdo_size(), CELLREPLAY_PDO_SIZE);

        // pdo_size is kept in the config when it is set
        let light = Box::new(
            TraceLightPdoConfig::new()
                .pattern(vec![LightPdo::default()])
                .pdo_size(1500),
        ) as Box<dyn LightPdoTraceConfig>;
        let ser_str = serde_json::to_string(&light).unwrap();
        assert!(ser_str.ends_with(",\"pdo_size\":1500}}"));
        let des: Box<dyn LightPdoTraceConfig> = serde_json::from_str(&ser_str).unwrap();
        assert_eq!(des.into_model().pdo_size(), 1500);
    }
}
