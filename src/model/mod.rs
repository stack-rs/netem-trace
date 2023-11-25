//! This module contains pre-defined models for BwTrace, DelayTrace and LossTrace.
//!
//! A model has two parts: a configuration struct and a model struct.
//! The configuration struct is used to configure the model and
//! used for serialization/deserialization if `serde` feature is enabled.
//! The model struct which implements trait `BwTrace`, `DelayTrace` or `LossTrace`
//! is used to generate the trace and maintain inner states.
//!
//! Enable `bw-model` feature to use the BwTrace models.
//!
//! Enable `delay-model` feature to use the DelayTrace models.
//!
//! Enable `loss-model` feature to use the LossTrace models.

#[cfg(feature = "bw-model")]
pub mod bw;

#[cfg(feature = "bw-model")]
pub use bw::{
    BwTraceConfig, NormalizedBwConfig, RepeatableBwTraceConfig, RepeatedBwPatternConfig,
    SawtoothBwConfig, StaticBwConfig,
};
#[cfg(feature = "bw-model")]
pub use bw::{NormalizedBw, RepeatedBwPattern, SawtoothBw, StaticBw};

#[cfg(feature = "delay-model")]
pub mod delay;

#[cfg(feature = "delay-model")]
pub use delay::{DelayTraceConfig, RepeatedDelayPatternConfig, StaticDelayConfig};
#[cfg(feature = "delay-model")]
pub use delay::{RepeatedDelayPattern, StaticDelay};

#[cfg(feature = "loss-model")]
pub mod loss;

#[cfg(feature = "loss-model")]
pub use loss::{LossTraceConfig, RepeatedLossPatternConfig, StaticLossConfig};
#[cfg(feature = "loss-model")]
pub use loss::{RepeatedLossPattern, StaticLoss};
