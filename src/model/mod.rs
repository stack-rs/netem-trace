//! This module contains pre-defined models for BwTrace, DelayTrace, LossTrace and DuplicateTrace.
//!
//! A model has two parts: a configuration struct and a model struct.
//! The configuration struct is used to configure the model and
//! used for serialization/deserialization if `serde` feature is enabled.
//! The model struct which implements trait `BwTrace`, `DelayTrace`, `LossTrace` or `DuplicateTrace`
//! is used to generate the trace and maintain inner states.
//!
//! Enable `bw-model` feature to use the BwTrace models.
//! Enable `delay-model` feature to use the DelayTrace models.
//! Enable `loss-model` feature to use the LossTrace models.
//! Enable `duplicate-model` feature to use the DuplicateTrace models.

#[cfg(feature = "bw-model")]
pub mod bw;

#[cfg(feature = "bw-model")]
pub use bw::{
    BwTraceConfig, Forever, NormalizedBwConfig, RepeatedBwPatternConfig, SawtoothBwConfig,
    StaticBwConfig, TraceBwConfig,
};
#[cfg(feature = "bw-model")]
pub use bw::{NormalizedBw, RepeatedBwPattern, SawtoothBw, StaticBw, TraceBw};

#[cfg(feature = "delay-model")]
pub mod delay;

#[cfg(feature = "delay-model")]
pub use delay::{DelayTraceConfig, RepeatedDelayPatternConfig, StaticDelayConfig};
#[cfg(feature = "delay-model")]
pub use delay::{RepeatedDelayPattern, StaticDelay};

#[cfg(feature = "delay-per-packet-model")]
pub mod delay_per_packet;

#[cfg(feature = "delay-per-packet-model")]
pub use delay_per_packet::{
    DelayPerPacketTraceConfig, NormalizedDelayPerPacketConfig, RepeatedDelayPerPacketPatternConfig,
    StaticDelayPerPacketConfig,
};
#[cfg(feature = "delay-per-packet-model")]
pub use delay_per_packet::{
    NormalizedDelayPerPacket, RepeatedDelayPerPacketPattern, StaticDelayPerPacket,
};

#[cfg(feature = "loss-model")]
pub mod loss;

#[cfg(feature = "loss-model")]
pub use loss::{LossTraceConfig, RepeatedLossPatternConfig, StaticLossConfig};
#[cfg(feature = "loss-model")]
pub use loss::{RepeatedLossPattern, StaticLoss};

#[cfg(feature = "duplicate-model")]
pub mod duplicate;

#[cfg(feature = "duplicate-model")]
pub use duplicate::{DuplicateTraceConfig, RepeatedDuplicatePatternConfig, StaticDuplicateConfig};
#[cfg(feature = "duplicate-model")]
pub use duplicate::{RepeatedDuplicatePattern, StaticDuplicate};

#[cfg(feature = "truncated-normal")]
pub mod solve_truncate;
