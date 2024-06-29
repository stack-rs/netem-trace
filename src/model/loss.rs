//! This module contains some predefined loss trace models.
//!
//! Enabled with feature `loss-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticLoss`]: A trace model with static loss.
//! - [`RepeatedLossPattern`]: A trace model with a repeated loss pattern.
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticLossConfig;
//! # use netem_trace::{LossPattern, Duration, LossTrace};
//! let mut static_loss = StaticLossConfig::new()
//!     .loss(vec![0.1, 0.2])
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_loss.next_loss(), Some((vec![0.1, 0.2], Duration::from_secs(1))));
//! assert_eq!(static_loss.next_loss(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{StaticLossConfig, LossTraceConfig};
//! # use netem_trace::{LossPattern, Duration, LossTrace};
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
//! // The content would be "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}"
//! // if the `human` feature is enabled.
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
//! let des: Box<dyn LossTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! assert_eq!(
//!     model.next_loss(),
//!     Some((vec![0.1, 0.2], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_loss(),
//!     Some((vec![0.2, 0.4], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_loss(),
//!     Some((vec![0.1, 0.2], Duration::from_secs(1)))
//! );
//! assert_eq!(
//!     model.next_loss(),
//!     Some((vec![0.2, 0.4], Duration::from_secs(1)))
//! );
//! assert_eq!(model.next_loss(), None);
//! ```
use crate::{Duration, LossPattern, LossTrace};
use dyn_clone::DynClone;

/// This trait is used to convert a loss trace configuration into a loss trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait LossTraceConfig: DynClone + Send {
    fn into_model(self: Box<Self>) -> Box<dyn LossTrace>;
}

dyn_clone::clone_trait_object!(LossTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The model of a static loss trace.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticLossConfig;
/// # use netem_trace::{LossPattern, Duration, LossTrace};
/// let mut static_loss = StaticLossConfig::new()
///     .loss(vec![0.1, 0.2])
///     .duration(Duration::from_secs(1))
///     .build();
/// assert_eq!(static_loss.next_loss(), Some((vec![0.1, 0.2], Duration::from_secs(1))));
/// assert_eq!(static_loss.next_loss(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticLoss {
    pub loss: LossPattern,
    pub duration: Option<Duration>,
}

/// The configuration struct for [`StaticLoss`].
///
/// See [`StaticLoss`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Debug, Clone, Default)]
pub struct StaticLossConfig {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub loss: Option<LossPattern>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(
        all(feature = "serde", feature = "human"),
        serde(with = "humantime_serde")
    )]
    pub duration: Option<Duration>,
}

/// The model contains an array of loss trace models.
///
/// Combine multiple loss trace models into one loss pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedLossPatternConfig`]:
///
/// ```
/// # use netem_trace::model::{StaticLossConfig, LossTraceConfig};
/// # use netem_trace::{LossPattern, Duration, LossTrace};
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The content would be "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
/// let des: Box<dyn LossTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// assert_eq!(
///     model.next_loss(),
///     Some((vec![0.1, 0.2], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_loss(),
///     Some((vec![0.2, 0.4], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_loss(),
///     Some((vec![0.1, 0.2], Duration::from_secs(1)))
/// );
/// assert_eq!(
///     model.next_loss(),
///     Some((vec![0.2, 0.4], Duration::from_secs(1)))
/// );
/// assert_eq!(model.next_loss(), None);
/// ```
///
/// You can also build manually:
///
/// ```
/// # use netem_trace::model::{StaticLossConfig, LossTraceConfig, RepeatedLossPatternConfig};
/// # use netem_trace::{LossPattern, Duration, LossTrace};
/// let pat = vec![
///     Box::new(
///         StaticLossConfig::new()
///             .loss(vec![0.1, 0.2])
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn LossTraceConfig>,
///     Box::new(
///         StaticLossConfig::new()
///             .loss(vec![0.2, 0.4])
///             .duration(Duration::from_secs(1)),
///     ) as Box<dyn LossTraceConfig>,
/// ];
/// let ser = Box::new(RepeatedLossPatternConfig::new().pattern(pat).count(2)) as Box<dyn LossTraceConfig>;
/// let ser_str = serde_json::to_string(&ser).unwrap();
/// # #[cfg(not(feature = "human"))]
/// let json_str = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
/// // The json string would be "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}"
/// // if the `human` feature is enabled.
/// # #[cfg(feature = "human")]
/// # let json_str = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
/// assert_eq!(ser_str, json_str);
/// ```
pub struct RepeatedLossPattern {
    pub pattern: Vec<Box<dyn LossTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn LossTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedLossPattern`].
///
/// See [`RepeatedLossPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedLossPatternConfig {
    pub pattern: Vec<Box<dyn LossTraceConfig>>,
    pub count: usize,
}

impl LossTrace for StaticLoss {
    fn next_loss(&mut self) -> Option<(LossPattern, Duration)> {
        if let Some(duration) = self.duration.take() {
            if duration.is_zero() {
                None
            } else {
                Some((self.loss.clone(), duration))
            }
        } else {
            None
        }
    }
}

impl LossTrace for RepeatedLossPattern {
    fn next_loss(&mut self) -> Option<(LossPattern, Duration)> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_loss() {
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
                    self.next_loss()
                }
            }
        }
    }
}

impl StaticLossConfig {
    pub fn new() -> Self {
        Self {
            loss: None,
            duration: None,
        }
    }

    pub fn loss(mut self, loss: LossPattern) -> Self {
        self.loss = Some(loss);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn build(self) -> StaticLoss {
        StaticLoss {
            loss: self.loss.unwrap_or_else(|| vec![0.1, 0.2]),
            duration: Some(self.duration.unwrap_or_else(|| Duration::from_secs(1))),
        }
    }
}

impl RepeatedLossPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn LossTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedLossPattern {
        RepeatedLossPattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

macro_rules! impl_loss_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl LossTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn LossTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_loss_trace_config!(StaticLossConfig);
impl_loss_trace_config!(RepeatedLossPatternConfig);

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticLossConfig;
    use crate::LossTrace;

    #[test]
    fn test_static_loss_model() {
        let mut static_loss = StaticLossConfig::new()
            .loss(vec![0.1, 0.2])
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            static_loss.next_loss(),
            Some((vec![0.1, 0.2], Duration::from_secs(1)))
        );
        assert_eq!(static_loss.next_loss(), None);
    }

    #[test]
    fn test_serde() {
        let a = vec![
            Box::new(
                StaticLossConfig::new()
                    .loss(vec![0.1, 0.2])
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn LossTraceConfig>,
            Box::new(
                StaticLossConfig::new()
                    .loss(vec![0.2, 0.4])
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn LossTraceConfig>,
        ];
        let ser = Box::new(RepeatedLossPatternConfig::new().pattern(a).count(2))
            as Box<dyn LossTraceConfig>;
        let ser_str = serde_json::to_string(&ser).unwrap();
        #[cfg(not(feature = "human"))]
        let des_str = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
        #[cfg(feature = "human")]
        let des_str = "{\"RepeatedLossPatternConfig\":{\"pattern\":[{\"StaticLossConfig\":{\"loss\":[0.1,0.2],\"duration\":\"1s\"}},{\"StaticLossConfig\":{\"loss\":[0.2,0.4],\"duration\":\"1s\"}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn LossTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_loss(),
            Some((vec![0.1, 0.2], Duration::from_secs(1)))
        );
    }
}
