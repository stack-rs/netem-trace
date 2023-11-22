//! This module contains some predefined delay trace models.
//!
//! Enabled with feature `delay-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticDelay`]: A trace model with static delay.
//! - [`RepeatedDelayPattern`]: A trace model with a repeated delay pattern.
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticDelayConfig;
//! # use netem_trace::{Delay, Duration, DelayTrace};
//! let mut static_delay = StaticDelayConfig::new()
//!     .delay(Delay::from_millis(10))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_delay.next_delay(), Some((Delay::from_millis(10), Duration::from_secs(1))));
//! assert_eq!(static_delay.next_delay(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{StaticDelayConfig, DelayTraceConfig};
//! # use netem_trace::{Delay, Duration, DelayTrace};
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
//! // The content would be "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}"
//! // if the `human` feature is enabled.
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}";
//! let des: Box<dyn DelayTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(
//!     model.next_delay(),
//!     Some((Delay::from_millis(10), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some((Delay::from_millis(20), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some((Delay::from_millis(10), Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_delay(),
//!     Some((Delay::from_millis(20), Duration::from_secs(1)))
//! );
//! assert_eq!(model.next_delay(), None);
//! ```
use crate::{Delay, DelayTrace, Duration};
use dyn_clone::DynClone;
use std::collections::VecDeque;

/// This trait is used to convert a delay trace configuration into a delay trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait DelayTraceConfig: DynClone {
    fn into_model(self: Box<Self>) -> Box<dyn DelayTrace>;
}

dyn_clone::clone_trait_object!(DelayTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The model of a static delay trace.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticDelayConfig;
/// # use netem_trace::{Delay, Duration, DelayTrace};
/// let mut static_delay = StaticDelayConfig::new()
///     .delay(Delay::from_millis(10))
///     .duration(Duration::from_secs(1))
///     .build();
/// assert_eq!(static_delay.next_delay(), Some((Delay::from_millis(10), Duration::from_secs(1))));
/// assert_eq!(static_delay.next_delay(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticDelay {
    pub delay: Delay,
    pub duration: Option<Duration>,
}

/// The configuration struct for [`StaticDelay`].
///
/// See [`StaticDelay`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct StaticDelayConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub delay: Option<Delay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
}

/// The model contains an array of delay trace models.
///
/// Combines multiple delay trace models into one delay pattern,
/// and repeat the pattern for `count` times.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedDelayPatternConfig`]:
///
/// ```
/// # use netem_trace::model::{StaticDelayConfig, DelayTraceConfig};
/// # use netem_trace::{Delay, Duration, DelayTrace};
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The content would be "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}";
/// let des: Box<dyn DelayTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// assert_eq!(
///     model.next_delay(),
///     Some((Delay::from_millis(10), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some((Delay::from_millis(20), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some((Delay::from_millis(10), Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_delay(),
///     Some((Delay::from_millis(20), Duration::from_secs(1)))
/// );
/// assert_eq!(model.next_delay(), None);
/// ```
///
/// You can also build manually:
///
/// ```
/// # use netem_trace::model::{StaticDelayConfig, DelayTraceConfig, RepeatedDelayPatternConfig};
/// # use netem_trace::{Delay, Duration, DelayTrace};
/// let pat = vec![
///     Box::new(
///         StaticDelayConfig::new()
///             .delay(Delay::from_millis(10))
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn DelayTraceConfig>,
///     Box::new(
///         StaticDelayConfig::new()
///             .delay(Delay::from_millis(20))
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn DelayTraceConfig>,
/// ];
/// let ser = Box::new(RepeatedDelayPatternConfig::new().pattern(pat).count(2)) as Box<dyn DelayTraceConfig>;
/// let ser_str = serde_json::to_string(&ser).unwrap();
/// # #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The json string would be "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let json_str = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}";
/// assert_eq!(ser_str, json_str);
/// ```
pub struct RepeatedDelayPattern {
    pub pattern: VecDeque<Box<dyn DelayTrace>>,
}

/// The configuration struct for [`RepeatedDelayPattern`].
///
/// See [`RepeatedDelayPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedDelayPatternConfig {
    pub pattern: Vec<Box<dyn DelayTraceConfig>>,
    pub count: usize,
}

impl DelayTrace for StaticDelay {
    fn next_delay(&mut self) -> Option<(Delay, Duration)> {
        if let Some(duration) = self.duration.take() {
            if duration.is_zero() {
                None
            } else {
                Some((self.delay, duration))
            }
        } else {
            None
        }
    }
}

impl DelayTrace for RepeatedDelayPattern {
    fn next_delay(&mut self) -> Option<(Delay, Duration)> {
        if self.pattern.is_empty() {
            None
        } else {
            match self.pattern[0].next_delay() {
                Some((delay, duration)) => Some((delay, duration)),
                None => {
                    self.pattern.pop_front();
                    self.next_delay()
                }
            }
        }
    }
}

impl StaticDelayConfig {
    pub fn new() -> Self {
        Self {
            delay: None,
            duration: None,
        }
    }

    pub fn delay(mut self, delay: Delay) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn build(self) -> StaticDelay {
        StaticDelay {
            delay: self.delay.unwrap_or_else(|| Delay::from_millis(10)),
            duration: Some(self.duration.unwrap_or_else(|| Duration::from_secs(1))),
        }
    }
}

impl RepeatedDelayPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 1,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn DelayTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedDelayPattern {
        let pattern = vec![self.pattern; self.count]
            .drain(..)
            .flatten()
            .map(|config| config.into_model())
            .collect();
        RepeatedDelayPattern { pattern }
    }
}

macro_rules! impl_delay_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl DelayTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn DelayTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_delay_trace_config!(StaticDelayConfig);
impl_delay_trace_config!(RepeatedDelayPatternConfig);

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticDelayConfig;
    use crate::DelayTrace;

    #[test]
    fn test_static_delay_model() {
        let mut static_delay = StaticDelayConfig::new()
            .delay(Delay::from_millis(10))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            static_delay.next_delay(),
            Some((Delay::from_millis(10), Duration::from_secs(1)))
        );
        assert_eq!(static_delay.next_delay(), None);
    }

    #[test]
    fn test_serde() {
        let a = vec![
            Box::new(
                StaticDelayConfig::new()
                    .delay(Delay::from_millis(10))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn DelayTraceConfig>,
            Box::new(
                StaticDelayConfig::new()
                    .delay(Delay::from_millis(20))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn DelayTraceConfig>,
        ];
        let ser = Box::new(RepeatedDelayPatternConfig::new().pattern(a).count(2))
            as Box<dyn DelayTraceConfig>;
        let ser_str = serde_json::to_string(&ser).unwrap();
        #[cfg(not(feature = "human"))]
        let des_str = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":10000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDelayConfig\":{\"delay\":{\"secs\":0,\"nanos\":20000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
        #[cfg(feature = "human")]
        let des_str = "{\"RepeatedDelayPatternConfig\":{\"pattern\":[{\"StaticDelayConfig\":{\"delay\":\"10ms\",\"duration\":\"1s\"}},{\"StaticDelayConfig\":{\"delay\":\"20ms\",\"duration\":\"1s\"}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn DelayTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_delay(),
            Some((Delay::from_millis(10), Duration::from_secs(1)))
        );
    }
}
