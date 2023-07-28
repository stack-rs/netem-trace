//! This module contains pre-defined models for BwTrace, DelayTrace and LossTrace.
//!
//! A model has two parts: a configuration struct and a model struct.
//! The configuration struct is used to configure the model and
//! used for serialization/deserialization if `serde` feature is enabled.
//! The model struct which implements trait `BwTrace`
//! is used to generate the trace and maintain inner states.
//!
//! Enable `bw-model` feature to use the BwTrace models.

#[cfg(feature = "bw-model")]
pub mod bw;

#[cfg(feature = "bw-model")]
pub use bw::{
    BwTraceConfig, NormalizedBwConfig, RepeatedBwPatternConfig, SawtoothBwConfig, StaticBwConfig,
};
#[cfg(feature = "bw-model")]
pub use bw::{NormalizedBw, RepeatedBwPattern, SawtoothBw, StaticBw};
