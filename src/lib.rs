#[cfg(feature = "mahimahi")]
pub mod mahimahi;
#[cfg(feature = "mahimahi")]
pub use mahimahi::Mahimahi;

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
/// If the packet 101 is not lost, then the probability of packet 102 being lost is still 0.1
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
    use crate::model::bw::{BwTraceConfig, FixedBwConfig, RepeatedBwPatternConfig};

    #[test]
    fn test_loss_pattern() {
        let mut b = Box::new(
            FixedBwConfig::new()
                .bw(Bandwidth::from_mbps(24))
                .duration(Duration::from_secs(1)),
        )
        .into_model();
        b.mahimahi_to_file(&Duration::from_secs(1), "fixed.trace");
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
        c.mahimahi_to_file(&Duration::from_secs(4), "repeated.trace");
    }
}
