//! This module contains some predefined duplicate trace models.
//!
//! Enabled with feature `duplicate-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticDuplicate`]: A trace model with static duplicate rate.
//! - [`RepeatedDuplicatePattern`]: A trace model with a repeated duplicate pattern.
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticDuplicateConfig;
//! # use netem_trace::{DuplicatePattern, Duration, DuplicateTrace};
//! let mut static_duplicate = StaticDuplicateConfig::new()
//!     .duplicate(vec![0.1, 0.2])
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_duplicate.next_duplicate(), Some((vec![0.1, 0.2], Duration::from_secs(1))));
//! assert_eq!(static_duplicate.next_duplicate(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{StaticDuplicateConfig, DuplicateTraceConfig};
//! # use netem_trace::{DuplicatePattern, Duration, DuplicateTrace};
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
//! // The content would be "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
//! // if the `human` feature is not enabled.
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
//! let des: Box<dyn DuplicateTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(
//!     model.next_duplicate(),
//!     Some((vec![0.1, 0.2], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_duplicate(),
//!     Some((vec![0.2, 0.4], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_duplicate(),
//!     Some((vec![0.1, 0.2], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_duplicate(),
//!     Some((vec![0.2, 0.4], Duration::from_secs(1)))
//! );
//! assert_eq!(model.next_duplicate(), None);
//! ```
use crate::{DuplicatePattern, DuplicateTrace, Duration};
use dyn_clone::DynClone;

/// This trait is used to convert a duplicate trace configuration into a duplicate trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be seialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait DuplicateTraceConfig: DynClone + Send {
    fn into_model(self: Box<Self>) -> Box<dyn DuplicateTrace>;
}

dyn_clone::clone_trait_object!(DuplicateTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The model of a static duplicate trace.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticDuplicateConfig;
/// # use netem_trace::{DuplicatePattern, Duration, DuplicateTrace};
/// let mut static_duplicate = StaticDuplicateConfig::new()
///     .duplicate(vec![0.1, 0.2])
///     .duration(Duration::from_secs(1))
///     .build();
/// assert_eq!(static_duplicate.next_duplicate(), Some((vec![0.1, 0.2], Duration::from_secs(1))));
/// assert_eq!(static_duplicate.next_duplicate(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticDuplicate {
    pub duplicate: DuplicatePattern,
    pub duration: Option<Duration>,
}

/// The configuration struct for [`StaticDuplicate`]
///
/// See [`StaticDuplicate`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct StaticDuplicateConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub duplicate: Option<DuplicatePattern>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
}

/// The model contains an array of duplicate trace models.
///
/// Combine multiple duplicate trace models into one duplicate pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedDuplicatePatternConfig`]:
///
/// ```
/// # use netem_trace::model::{StaticDuplicateConfig, DuplicateTraceConfig};
/// # use netem_trace::{DuplicatePattern, Duration, DuplicateTrace};
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
/// // The content would be "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// let des: Box<dyn DuplicateTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// assert_eq!(
///     model.next_duplicate(),
///     Some((vec![0.1, 0.2], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_duplicate(),
///     Some((vec![0.2, 0.4], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_duplicate(),
///     Some((vec![0.1, 0.2], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_duplicate(),
///     Some((vec![0.2, 0.4], Duration::from_secs(1)))
/// );
/// assert_eq!(model.next_duplicate(), None);
/// ```
///
/// You can also build manually:
///
/// ```
/// # use netem_trace::model::{StaticDuplicateConfig, DuplicateTraceConfig, RepeatedDuplicatePatternConfig};
/// # use netem_trace::{DuplicatePattern, Duration, DuplicateTrace};
/// let pat = vec![
///     Box::new(
///         StaticDuplicateConfig::new()
///             .duplicate(vec![0.1, 0.2])
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn DuplicateTraceConfig>,
///     Box::new(
///         StaticDuplicateConfig::new()
///             .duplicate(vec![0.2, 0.4])
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn DuplicateTraceConfig>,
/// ];
/// let ser = Box::new(RepeatedDuplicatePatternConfig::new().pattern(pat).count(2)) as Box<dyn DuplicateTraceConfig>;
/// let ser_str = serde_json::to_string(&ser).unwrap();
/// # #[cfg(feature = "human")]
/// # let json_str = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
/// // The json string would be "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}"
/// // if the `human` feature is not enabled.
/// # #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// assert_eq!(ser_str, json_str);
/// ```
pub struct RepeatedDuplicatePattern {
    pub pattern: Vec<Box<dyn DuplicateTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn DuplicateTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedDuplicatePattern`].
///
/// See [`RepeatedDuplicatePattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedDuplicatePatternConfig {
    pub pattern: Vec<Box<dyn DuplicateTraceConfig>>,
    pub count: usize,
}

impl DuplicateTrace for StaticDuplicate {
    fn next_duplicate(&mut self) -> Option<(DuplicatePattern, Duration)> {
        if let Some(duration) = self.duration.take() {
            if duration.is_zero() {
                None
            } else {
                Some((self.duplicate.clone(), duration))
            }
        } else {
            None
        }
    }
}

impl DuplicateTrace for RepeatedDuplicatePattern {
    fn next_duplicate(&mut self) -> Option<(DuplicatePattern, Duration)> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_duplicate() {
                Some(duplicate) => Some(duplicate),
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
                    self.next_duplicate()
                }
            }
        }
    }
}

impl StaticDuplicateConfig {
    pub fn new() -> Self {
        Self {
            duplicate: None,
            duration: None,
        }
    }

    pub fn duplicate(mut self, duplicate: DuplicatePattern) -> Self {
        self.duplicate = Some(duplicate);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn build(self) -> StaticDuplicate {
        StaticDuplicate {
            duplicate: self.duplicate.unwrap_or_else(|| vec![0.1, 0.2]),
            duration: Some(self.duration.unwrap_or_else(|| Duration::from_secs(1))),
        }
    }
}

impl RepeatedDuplicatePatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn DuplicateTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedDuplicatePattern {
        RepeatedDuplicatePattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

macro_rules! impl_duplicate_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl DuplicateTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn DuplicateTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_duplicate_trace_config!(StaticDuplicateConfig);
impl_duplicate_trace_config!(RepeatedDuplicatePatternConfig);

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticDuplicateConfig;
    use crate::DuplicateTrace;

    #[test]
    fn test_static_loss_model() {
        let mut static_loss = StaticDuplicateConfig::new()
            .duplicate(vec![0.1, 0.2])
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            static_loss.next_duplicate(),
            Some((vec![0.1, 0.2], Duration::from_secs(1)))
        );
        assert_eq!(static_loss.next_duplicate(), None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        let a = vec![
            Box::new(
                StaticDuplicateConfig::new()
                    .duplicate(vec![0.1, 0.2])
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn DuplicateTraceConfig>,
            Box::new(
                StaticDuplicateConfig::new()
                    .duplicate(vec![0.2, 0.4])
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn DuplicateTraceConfig>,
        ];
        let ser = Box::new(RepeatedDuplicatePatternConfig::new().pattern(a).count(2))
            as Box<dyn DuplicateTraceConfig>;
        let ser_str = serde_json::to_string(&ser).unwrap();
        #[cfg(feature = "human")]
        let des_str = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
        #[cfg(not(feature = "human"))]
        let des_str = "{\"RepeatedDuplicatePatternConfig\":{\"pattern\":[{\"StaticDuplicateConfig\":{\"duplicate\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticDuplicateConfig\":{\"duplicate\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn DuplicateTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_duplicate(),
            Some((vec![0.1, 0.2], Duration::from_secs(1)))
        );
    }
}
