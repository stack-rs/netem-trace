//! This crate provides a set of tools to generate traces for network emulation.
//!
//! ## Examples
//!
//! If you want to use the pre-defined models, please enable the `model` or `bw-model` feature.
//!
//! And if you want read configuration from file, `serde` feature should also be enabled.
//!
//! An example to build model from configuration:
//!
//! ```
//! # use netem_trace::model::FixedBwConfig;
//! # use netem_trace::{Bandwidth, Duration, BwTrace};
//! let mut fixed_bw = FixedBwConfig::new()
//!     .bw(Bandwidth::from_mbps(24))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(fixed_bw.next_bw(), Some((Bandwidth::from_mbps(24), Duration::from_secs(1))));
//! assert_eq!(fixed_bw.next_bw(), None);
//! ```
//!
//! A more common use case is to build model from a configuration file (e.g. json file):
//!
//! ```
//! # use netem_trace::model::{FixedBwConfig, BwTraceConfig};
//! # use netem_trace::{Bandwidth, Duration, BwTrace};
//! let config_file_content = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"FixedBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"FixedBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
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
//!
//! ## Make your own model
//!
//! Here is an simple example of how to do this. For more complicated examples, please refer to our pre-defined models.
//!
//! ```
//! use netem_trace::BwTrace;
//! use netem_trace::{Bandwidth, Duration};
//!
//! struct MyFixedBw {
//!    bw: Bandwidth,
//!    duration: Option<Duration>,
//! }
//!
//! impl BwTrace for MyFixedBw {
//!     fn next_bw(&mut self) -> Option<(Bandwidth, Duration)> {
//!         if let Some(duration) = self.duration.take() {
//!             if duration.is_zero() {
//!                 None
//!             } else {
//!                 Some((self.bw, duration))
//!             }
//!         } else {
//!             None
//!         }
//!     }
//! }
//! ```
//!
//! This is almost the same as how this library implements the [`FixedBw`] model.
//!
//! ## Features
//!
//! ### Model Features
//!
//! - `model`: Enable this feature if you want to use all pre-defined models.
//!     - `bw-model`: Enable this feature if you want to use the pre-defined [`BwTrace`] models.
//!
//! ### Trace Format Features
//!
//! - `mahimahi`: Enable this feature if you want to output traces in [mahimahi](https://github.com/ravinet/mahimahi) format.
//!
//! ### Other Features
//!
//! - `serde`: Enable this features if you want some structs to be serializable/deserializable. Often used with model features.

#[cfg(feature = "mahimahi")]
pub mod mahimahi;
#[cfg(feature = "mahimahi")]
pub use mahimahi::{Mahimahi, MahimahiExt};

#[cfg(any(feature = "bw-model", feature = "model"))]
pub mod model;

pub use bandwidth::Bandwidth;
pub use std::time::Duration;

/// The delay describes how long a packet is delayed when going through.
pub type Delay = std::time::Duration;

/// The loss_pattern describes how the packets are dropped when going through.
///
/// The loss_pattern is a sequence of conditional probabilities describing how packets are dropped.
/// The probability is a f64 between 0 and 1.
///
/// The meaning of the loss_pattern sequence is as follows:
///
/// - The probability on index 0 describes how likely a packet will be dropped **if the previous packet was not lost**.
/// - The probability on index 1 describes how likely a packet will be dropped **if the previous packet was lost**.
/// - The probability on index 2 describes how likely a packet will be dropped **if the previous 2 packet was lost**.
/// - ...
///
/// For example, if the loss_pattern is [0.1, 0.2], and packet 100 is not lost,
/// then the probability of packet 101 being lost is 0.1.
///
/// If the packet 101 is lost, then the probability of packet 102 being lost is 0.2.
/// If the packet 101 is not lost, then the probability of packet 102 being lost is still 0.1.
pub type LossPattern = Vec<f64>;

/// This is a trait that represents a trace of bandwidths.
///
/// The trace is a sequence of `(bandwidth, duration)` pairs.
/// The bandwidth describes how many bits can be sent per second.
/// The duration is the time that the bandwidth lasts.
///
/// For example, if the sequence is [(1Mbps, 1s), (2Mbps, 2s), (3Mbps, 3s)],
/// then the bandwidth will be 1Mbps for 1s, then 2Mbps for 2s, then 3Mbps for 3s.
///
/// The next_bw function either returns **the next bandwidth and its duration**
/// in the sequence, or **None** if the trace goes to end.
pub trait BwTrace {
    fn next_bw(&mut self) -> Option<(Bandwidth, Duration)>;
}

/// This is a trait that represents a trace of delays.
///
/// The trace is a sequence of `(delay, duration)` pairs.
/// The delay describes how long a packet is delayed when going through.
/// The duration is the time that the delay lasts.
///
/// For example, if the sequence is [(10ms, 1s), (20ms, 2s), (30ms, 3s)],
/// then the delay will be 10ms for 1s, then 20ms for 2s, then 30ms for 3s.
///
/// The next_delay function either returns **the next delay and its duration**
/// in the sequence, or **None** if the trace goes to end.
pub trait DelayTrace {
    fn next_delay(&mut self) -> Option<(Delay, Duration)>;
}

/// This is a trait that represents a trace of loss patterns.
///
/// The trace is a sequence of `(loss_pattern, duration)` pairs.
/// The loss_pattern describes how packets are dropped when going through.
/// The duration is the time that the loss_pattern lasts.
///
/// The next_loss function either returns **the next loss_pattern and its duration**
/// in the sequence, or **None** if the trace goes to end.
pub trait LossTrace {
    fn next_loss(&mut self) -> Option<(LossPattern, Duration)>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mahimahi::MahimahiExt;
    use crate::model::{
        BoundedNormalizedBwConfig, BwTraceConfig, FixedBwConfig, NormalizedBwConfig,
        RepeatedBwPatternConfig,
    };

    #[test]
    fn test_bw_trace() {
        let mut fixed_bw = FixedBwConfig::new()
            .bw(Bandwidth::from_mbps(24))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            fixed_bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_secs(1)))
        );
        let mut normal_bw = NormalizedBwConfig::new()
            .mean(Bandwidth::from_mbps(12))
            .std_dev(Bandwidth::from_mbps(1))
            .duration(Duration::from_secs(1))
            .step(Duration::from_millis(100))
            .seed(42)
            .build();
        assert_eq!(
            normal_bw.next_bw(),
            Some((Bandwidth::from_bps(12069427), Duration::from_millis(100)))
        );
        assert_eq!(
            normal_bw.next_bw(),
            Some((Bandwidth::from_bps(12132938), Duration::from_millis(100)))
        );
        let mut normal_bw = BoundedNormalizedBwConfig::new()
            .mean(Bandwidth::from_mbps(12))
            .std_dev(Bandwidth::from_mbps(1))
            .duration(Duration::from_secs(1))
            .step(Duration::from_millis(100))
            .seed(42)
            .upper_bound(Bandwidth::from_kbps(12100))
            .lower_bound(Bandwidth::from_kbps(11900))
            .build();
        assert_eq!(
            normal_bw.next_bw(),
            Some((Bandwidth::from_bps(12069427), Duration::from_millis(100)))
        );
        assert_eq!(
            normal_bw.next_bw(),
            Some((Bandwidth::from_bps(12100000), Duration::from_millis(100)))
        );
    }

    #[test]
    fn test_mahimahi() {
        let mut fixed_bw = FixedBwConfig::new()
            .bw(Bandwidth::from_mbps(24))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            fixed_bw.mahimahi(&Duration::from_millis(5)),
            [0, 0, 1, 1, 2, 2, 3, 3, 4, 4]
        );
        let mut fixed_bw = FixedBwConfig::new()
            .bw(Bandwidth::from_mbps(12))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            fixed_bw.mahimahi_to_string(&Duration::from_millis(5)),
            "0\n1\n2\n3\n4"
        );
        let a = vec![
            Box::new(
                FixedBwConfig::new()
                    .bw(Bandwidth::from_mbps(12))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn BwTraceConfig>,
            Box::new(
                FixedBwConfig::new()
                    .bw(Bandwidth::from_mbps(24))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn BwTraceConfig>,
        ];
        let mut c = Box::new(RepeatedBwPatternConfig::new().pattern(a).count(2)).into_model();
        assert_eq!(c.mahimahi(&Duration::from_millis(5)), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_model_serde() {
        let a = vec![
            Box::new(
                FixedBwConfig::new()
                    .bw(Bandwidth::from_mbps(12))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn BwTraceConfig>,
            Box::new(
                FixedBwConfig::new()
                    .bw(Bandwidth::from_mbps(24))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn BwTraceConfig>,
        ];
        let ser =
            Box::new(RepeatedBwPatternConfig::new().pattern(a).count(2)) as Box<dyn BwTraceConfig>;
        let ser_str = serde_json::to_string(&ser).unwrap();
        let des_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"FixedBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"FixedBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
        assert_eq!(ser_str, des_str);
        let des: Box<dyn BwTraceConfig> = serde_json::from_str(des_str).unwrap();
        let mut model = des.into_model();
        assert_eq!(
            model.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
        );
    }
}