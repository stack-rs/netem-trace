//! This module contains some predefined rwnd trace models.
//!
//! Enabled with feature `rwnd-model` or `model`.
//!
//! ## Predefined models
//!
//! - [`StaticRwnd`]: A trace model with a single rwnd decision.
//! - [`RepeatedRwndPattern`]: A trace model with a repeated rwnd pattern.
//!
//! ## Examples
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::StaticRwndConfig;
//! # use netem_trace::{Duration, RwndTrace, RwndAction};
//! let mut static_rwnd = StaticRwndConfig::new()
//!     .set_rcv_buf(65536)
//!     .app_read(1024)
//!     .duration(Duration::from_secs(1))
//!     .build();
//! let (decision, duration) = static_rwnd.next_rwnd().unwrap();
//! assert_eq!(decision.set_rcv_buf, Some(65536));
//! assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
//! assert_eq!(duration, Duration::from_secs(1));
//! assert_eq!(static_rwnd.next_rwnd(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{StaticRwndConfig, RwndTraceConfig};
//! # use netem_trace::{Duration, RwndTrace, RwndAction};
//! # #[cfg(feature = "human")]
//! # let config_file_content = "{\"RepeatedRwndPatternConfig\":{\"pattern\":[{\"StaticRwndConfig\":{\"duration\":\"1s\",\"set_rcv_buf\":65536,\"app_read_bytes\":1024}},{\"StaticRwndConfig\":{\"duration\":\"1s\",\"rwnd_remaining\":32768}}],\"count\":2}}";
//! // The content would be "{\"RepeatedRwndPatternConfig\":{\"pattern\":[{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"set_rcv_buf\":65536,\"app_read_bytes\":1024}},{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"rwnd_remaining\":32768}}],\"count\":2}}"
//! // if the `human` feature is not enabled.
//! # #[cfg(not(feature = "human"))]
//! let config_file_content = "{\"RepeatedRwndPatternConfig\":{\"pattern\":[{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"set_rcv_buf\":65536,\"app_read_bytes\":1024}},{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"rwnd_remaining\":32768}}],\"count\":2}}";
//! let des: Box<dyn RwndTraceConfig> = serde_json::from_str(config_file_content).unwrap();
//! let mut model = des.into_model();
//! let (decision, _) = model.next_rwnd().unwrap();
//! assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
//! let (decision, _) = model.next_rwnd().unwrap();
//! assert_eq!(decision.action, Some(RwndAction::Remaining { rwnd: 32768 }));
//! let (decision, _) = model.next_rwnd().unwrap();
//! assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
//! let (decision, _) = model.next_rwnd().unwrap();
//! assert_eq!(decision.action, Some(RwndAction::Remaining { rwnd: 32768 }));
//! assert_eq!(model.next_rwnd(), None);
//! ```
//!
//! At most one of `app_read_bytes` or `rwnd_remaining` may be set per step —
//! never both. A step with neither produces [`RwndDecision::action`] as `None`,
//! which is valid for steps that only reconfigure the receive buffer.
use crate::{Duration, RwndAction, RwndDecision, RwndTrace};
use dyn_clone::DynClone;

/// This trait is used to convert a rwnd trace configuration into a rwnd trace model.
///
/// Since trace model is often configured with files and often has inner states which
/// is not suitable to be serialized/deserialized, this trait makes it possible to
/// separate the configuration part into a simple struct for serialization/deserialization, and
/// construct the model from the configuration.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait RwndTraceConfig: DynClone + Send {
    fn into_model(self: Box<Self>) -> Box<dyn RwndTrace>;
}

dyn_clone::clone_trait_object!(RwndTraceConfig);

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The config-layer representation of the action at a rwnd step.
///
/// This enum is the deserialized form of the mutually-exclusive
/// `app_read_bytes` / `rwnd_remaining` pair. [`StaticRwndConfig`]'s custom
/// serde impls flatten the active variant into the top level of the JSON
/// object, so this enum's own externally-tagged shape is rarely seen by users
/// — but it's serialized/deserialized on its own when used outside the
/// custom container impl.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum RwndActionConfig {
    AppRead { app_read_bytes: u64 },
    Remaining { rwnd_remaining: u64 },
}

/// The model of a static rwnd trace: a single decision valid for one duration.
///
/// ## Examples
///
/// ```
/// # use netem_trace::model::StaticRwndConfig;
/// # use netem_trace::{Duration, RwndTrace, RwndAction};
/// let mut static_rwnd = StaticRwndConfig::new()
///     .set_rcv_buf(65536)
///     .app_read(1024)
///     .duration(Duration::from_secs(1))
///     .build();
/// let (decision, duration) = static_rwnd.next_rwnd().unwrap();
/// assert_eq!(decision.set_rcv_buf, Some(65536));
/// assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
/// assert_eq!(duration, Duration::from_secs(1));
/// assert_eq!(static_rwnd.next_rwnd(), None);
/// ```
#[derive(Debug, Clone)]
pub struct StaticRwnd {
    pub decision: RwndDecision,
    pub duration: Option<Duration>,
}

/// The configuration struct for [`StaticRwnd`].
///
/// The serialized JSON form is **flat** — the active variant of [`RwndActionConfig`]
/// is hoisted to the top level, so a step looks like
/// `{"duration":"1s","set_rcv_buf":65536,"app_read_bytes":1024}` (or
/// `{"duration":"1s","rwnd_remaining":32768}`), never with an `action` wrapper.
///
/// At most one of `app_read_bytes` / `rwnd_remaining` may be set; the deserializer
/// rejects inputs where both are present. A step with neither is valid and produces
/// [`RwndDecision::action`] as `None` (useful for steps that only reconfigure the
/// receive buffer).
#[derive(Debug, Clone, Default)]
pub struct StaticRwndConfig {
    pub duration: Option<Duration>,
    pub set_rcv_buf: Option<u64>,
    pub action: Option<RwndActionConfig>,
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for StaticRwndConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Helper {
            #[cfg_attr(feature = "human", serde(with = "humantime_serde"))]
            #[serde(default)]
            duration: Option<Duration>,
            #[serde(default)]
            set_rcv_buf: Option<u64>,
            #[serde(default)]
            app_read_bytes: Option<u64>,
            #[serde(default)]
            rwnd_remaining: Option<u64>,
        }

        let h = Helper::deserialize(deserializer)?;
        let action = match (h.app_read_bytes, h.rwnd_remaining) {
            (Some(bytes), None) => Some(RwndActionConfig::AppRead {
                app_read_bytes: bytes,
            }),
            (None, Some(rwnd)) => Some(RwndActionConfig::Remaining {
                rwnd_remaining: rwnd,
            }),
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(
                    "rwnd step cannot set both `app_read_bytes` and `rwnd_remaining`",
                ));
            }
            (None, None) => None,
        };
        Ok(Self {
            duration: h.duration,
            set_rcv_buf: h.set_rcv_buf,
            action,
        })
    }
}

#[cfg(feature = "serde")]
impl Serialize for StaticRwndConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Out {
            #[serde(skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "human", serde(with = "humantime_serde"))]
            duration: Option<Duration>,
            #[serde(skip_serializing_if = "Option::is_none")]
            set_rcv_buf: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            app_read_bytes: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            rwnd_remaining: Option<u64>,
        }

        let (app_read_bytes, rwnd_remaining) = match &self.action {
            Some(RwndActionConfig::AppRead { app_read_bytes }) => (Some(*app_read_bytes), None),
            Some(RwndActionConfig::Remaining { rwnd_remaining }) => (None, Some(*rwnd_remaining)),
            None => {
                return Err(serde::ser::Error::custom(
                    "rwnd step must set exactly one of `app_read_bytes` or `rwnd_remaining`",
                ));
            }
        };
        Out {
            duration: self.duration,
            set_rcv_buf: self.set_rcv_buf,
            app_read_bytes,
            rwnd_remaining,
        }
        .serialize(serializer)
    }
}

/// The model contains an array of rwnd trace models.
///
/// Combine multiple rwnd trace models into one rwnd pattern,
/// and repeat the pattern for `count` times.
///
/// If `count` is 0, the pattern will be repeated forever.
///
/// ## Examples
///
/// The most common use case is to read from a configuration file and
/// deserialize it into a [`RepeatedRwndPatternConfig`]:
///
/// ```
/// # use netem_trace::model::{StaticRwndConfig, RwndTraceConfig};
/// # use netem_trace::{Duration, RwndTrace, RwndAction};
/// # #[cfg(feature = "human")]
/// # let config_file_content = "{\"RepeatedRwndPatternConfig\":{\"pattern\":[{\"StaticRwndConfig\":{\"duration\":\"1s\",\"set_rcv_buf\":65536,\"app_read_bytes\":1024}},{\"StaticRwndConfig\":{\"duration\":\"1s\",\"rwnd_remaining\":32768}}],\"count\":2}}";
/// # #[cfg(not(feature = "human"))]
/// let config_file_content = "{\"RepeatedRwndPatternConfig\":{\"pattern\":[{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"set_rcv_buf\":65536,\"app_read_bytes\":1024}},{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"rwnd_remaining\":32768}}],\"count\":2}}";
/// let des: Box<dyn RwndTraceConfig> = serde_json::from_str(config_file_content).unwrap();
/// let mut model = des.into_model();
/// let (decision, _) = model.next_rwnd().unwrap();
/// assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
/// ```
pub struct RepeatedRwndPattern {
    pub pattern: Vec<Box<dyn RwndTraceConfig>>,
    pub count: usize,
    current_model: Option<Box<dyn RwndTrace>>,
    current_cycle: usize,
    current_pattern: usize,
}

/// The configuration struct for [`RepeatedRwndPattern`].
///
/// See [`RepeatedRwndPattern`] for more details.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(default))]
#[derive(Default, Clone)]
pub struct RepeatedRwndPatternConfig {
    pub pattern: Vec<Box<dyn RwndTraceConfig>>,
    pub count: usize,
}

impl RwndTrace for StaticRwnd {
    fn next_rwnd(&mut self) -> Option<(RwndDecision, Duration)> {
        if let Some(duration) = self.duration.take() {
            if duration.is_zero() {
                None
            } else {
                Some((self.decision.clone(), duration))
            }
        } else {
            None
        }
    }
}

impl RwndTrace for RepeatedRwndPattern {
    fn next_rwnd(&mut self) -> Option<(RwndDecision, Duration)> {
        if self.pattern.is_empty() || (self.count != 0 && self.current_cycle >= self.count) {
            None
        } else {
            if self.current_model.is_none() {
                self.current_model = Some(self.pattern[self.current_pattern].clone().into_model());
            }
            match self.current_model.as_mut().unwrap().next_rwnd() {
                Some(rwnd) => Some(rwnd),
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
                    self.next_rwnd()
                }
            }
        }
    }
}

impl StaticRwndConfig {
    pub fn new() -> Self {
        Self {
            duration: None,
            set_rcv_buf: None,
            action: None,
        }
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn set_rcv_buf(mut self, set_rcv_buf: u64) -> Self {
        self.set_rcv_buf = Some(set_rcv_buf);
        self
    }

    pub fn app_read(mut self, bytes: u64) -> Self {
        self.action = Some(RwndActionConfig::AppRead {
            app_read_bytes: bytes,
        });
        self
    }

    pub fn remaining(mut self, rwnd: u64) -> Self {
        self.action = Some(RwndActionConfig::Remaining {
            rwnd_remaining: rwnd,
        });
        self
    }

    pub fn build(self) -> StaticRwnd {
        let action = self.action.map(|cfg| match cfg {
            RwndActionConfig::AppRead { app_read_bytes } => RwndAction::AppRead {
                bytes: app_read_bytes,
            },
            RwndActionConfig::Remaining { rwnd_remaining } => RwndAction::Remaining {
                rwnd: rwnd_remaining,
            },
        });
        StaticRwnd {
            decision: RwndDecision {
                set_rcv_buf: self.set_rcv_buf,
                action,
            },
            duration: Some(self.duration.unwrap_or_else(|| Duration::from_secs(1))),
        }
    }
}

impl RepeatedRwndPatternConfig {
    pub fn new() -> Self {
        Self {
            pattern: vec![],
            count: 0,
        }
    }

    pub fn pattern(mut self, pattern: Vec<Box<dyn RwndTraceConfig>>) -> Self {
        self.pattern = pattern;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> RepeatedRwndPattern {
        RepeatedRwndPattern {
            pattern: self.pattern,
            count: self.count,
            current_model: None,
            current_cycle: 0,
            current_pattern: 0,
        }
    }
}

macro_rules! impl_rwnd_trace_config {
    ($name:ident) => {
        #[cfg_attr(feature = "serde", typetag::serde)]
        impl RwndTraceConfig for $name {
            fn into_model(self: Box<$name>) -> Box<dyn RwndTrace> {
                Box::new(self.build())
            }
        }
    };
}

impl_rwnd_trace_config!(StaticRwndConfig);
impl_rwnd_trace_config!(RepeatedRwndPatternConfig);

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticRwndConfig;
    use crate::RwndTrace;

    #[test]
    fn test_static_rwnd_model_app_read() {
        let mut static_rwnd = StaticRwndConfig::new()
            .set_rcv_buf(65536)
            .app_read(1024)
            .duration(Duration::from_secs(1))
            .build();
        let (decision, duration) = static_rwnd.next_rwnd().unwrap();
        assert_eq!(decision.set_rcv_buf, Some(65536));
        assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
        assert_eq!(duration, Duration::from_secs(1));
        assert_eq!(static_rwnd.next_rwnd(), None);
    }

    #[test]
    fn test_static_rwnd_model_remaining() {
        let mut static_rwnd = StaticRwndConfig::new()
            .remaining(32768)
            .duration(Duration::from_secs(2))
            .build();
        let (decision, duration) = static_rwnd.next_rwnd().unwrap();
        assert_eq!(decision.set_rcv_buf, None);
        assert_eq!(decision.action, Some(RwndAction::Remaining { rwnd: 32768 }));
        assert_eq!(duration, Duration::from_secs(2));
        assert_eq!(static_rwnd.next_rwnd(), None);
    }

    #[test]
    fn test_repeated_rwnd_pattern() {
        let pat = vec![
            Box::new(
                StaticRwndConfig::new()
                    .app_read(1024)
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn RwndTraceConfig>,
            Box::new(
                StaticRwndConfig::new()
                    .remaining(32768)
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn RwndTraceConfig>,
        ];
        let mut model = RepeatedRwndPatternConfig::new()
            .pattern(pat)
            .count(2)
            .build();
        let next = model.next_rwnd().unwrap();
        assert_eq!(next.0.action, Some(RwndAction::AppRead { bytes: 1024 }));
        assert_eq!(next.1, Duration::from_secs(1));
        let next = model.next_rwnd().unwrap();
        assert_eq!(next.0.action, Some(RwndAction::Remaining { rwnd: 32768 }));
        let next = model.next_rwnd().unwrap();
        assert_eq!(next.0.action, Some(RwndAction::AppRead { bytes: 1024 }));
        let next = model.next_rwnd().unwrap();
        assert_eq!(next.0.action, Some(RwndAction::Remaining { rwnd: 32768 }));
        assert_eq!(model.next_rwnd(), None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip_app_read() {
        let cfg = Box::new(
            StaticRwndConfig::new()
                .set_rcv_buf(65536)
                .app_read(1024)
                .duration(Duration::from_secs(1)),
        ) as Box<dyn RwndTraceConfig>;
        let ser_str = serde_json::to_string(&cfg).unwrap();
        #[cfg(feature = "human")]
        let expected = "{\"StaticRwndConfig\":{\"duration\":\"1s\",\"set_rcv_buf\":65536,\"app_read_bytes\":1024}}";
        #[cfg(not(feature = "human"))]
        let expected = "{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"set_rcv_buf\":65536,\"app_read_bytes\":1024}}";
        assert_eq!(ser_str, expected);

        let des: Box<dyn RwndTraceConfig> = serde_json::from_str(&ser_str).unwrap();
        let mut model = des.into_model();
        let (decision, duration) = model.next_rwnd().unwrap();
        assert_eq!(decision.set_rcv_buf, Some(65536));
        assert_eq!(decision.action, Some(RwndAction::AppRead { bytes: 1024 }));
        assert_eq!(duration, Duration::from_secs(1));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip_remaining() {
        let cfg = Box::new(
            StaticRwndConfig::new()
                .remaining(32768)
                .duration(Duration::from_secs(1)),
        ) as Box<dyn RwndTraceConfig>;
        let ser_str = serde_json::to_string(&cfg).unwrap();
        #[cfg(feature = "human")]
        let expected = "{\"StaticRwndConfig\":{\"duration\":\"1s\",\"rwnd_remaining\":32768}}";
        #[cfg(not(feature = "human"))]
        let expected = "{\"StaticRwndConfig\":{\"duration\":{\"secs\":1,\"nanos\":0},\"rwnd_remaining\":32768}}";
        assert_eq!(ser_str, expected);

        let des: Box<dyn RwndTraceConfig> = serde_json::from_str(&ser_str).unwrap();
        let mut model = des.into_model();
        let (decision, _) = model.next_rwnd().unwrap();
        assert_eq!(decision.action, Some(RwndAction::Remaining { rwnd: 32768 }));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_rejects_both() {
        // Omit duration to avoid the human/non-human format ambiguity; we're testing
        // the action constraint, not duration parsing.
        let json = "{\"StaticRwndConfig\":{\"app_read_bytes\":1024,\"rwnd_remaining\":32768}}";
        let result: Result<Box<dyn RwndTraceConfig>, _> = serde_json::from_str(json);
        let err = result
            .err()
            .expect("deserialization should have failed")
            .to_string();
        assert!(
            err.contains("cannot set both"),
            "expected 'cannot set both' in error, got: {err}"
        );
    }

    #[test]
    fn test_static_rwnd_set_rcv_buf_only() {
        let mut model = StaticRwndConfig::new()
            .set_rcv_buf(131072)
            .duration(Duration::from_secs(1))
            .build();
        let (decision, duration) = model.next_rwnd().unwrap();
        assert_eq!(decision.set_rcv_buf, Some(131072));
        assert_eq!(decision.action, None);
        assert_eq!(duration, Duration::from_secs(1));
        assert_eq!(model.next_rwnd(), None);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_action_none_when_neither_set() {
        // A step with only set_rcv_buf and no action fields should deserialize to action: None.
        let json = "{\"StaticRwndConfig\":{\"set_rcv_buf\":65536}}";
        let des: Box<dyn RwndTraceConfig> = serde_json::from_str(json).unwrap();
        let mut model = des.into_model();
        let (decision, _) = model.next_rwnd().unwrap();
        assert_eq!(decision.set_rcv_buf, Some(65536));
        assert_eq!(decision.action, None);
    }
}
