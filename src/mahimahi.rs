//! This module can generate traces in mahimahi format for struct implementing [`BwTrace`] and
//! load traces in mahimahi format to [`RepeatedBwPatternConfig`].
//!
//! Enable `mahimahi` feature to use this module.
//!
//! ## Examples
//!
//! ```
//! # use netem_trace::{Mahimahi, MahimahiExt};
//! # use netem_trace::model::StaticBwConfig;
//! # use netem_trace::{Bandwidth, Duration};
//! let mut static_bw = StaticBwConfig::new()
//!     .bw(Bandwidth::from_mbps(24))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_bw.mahimahi(&Duration::from_millis(5)), [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]);
//! let mut static_bw = StaticBwConfig::new()
//!     .bw(Bandwidth::from_mbps(12))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_bw.mahimahi_to_string(&Duration::from_millis(5)), "1\n2\n3\n4\n5");
//! ```

use crate::{
    model::{BwTraceConfig, RepeatedBwPatternConfig, StaticBwConfig},
    Bandwidth, BwTrace, Duration,
};

const MTU_IN_BYTES: u64 = 1500;
const MTU_IN_BITS: u64 = MTU_IN_BYTES * 8;
const MTU_PER_MILLIS: Bandwidth = Bandwidth::from_kbps(MTU_IN_BITS);
const MAHIMAHI_TS_BIN: Duration = Duration::from_millis(1);

macro_rules! saturating_duration_as_millis_u64 {
    ($duration:expr) => {
        $duration
            .as_secs()
            .saturating_mul(1_000)
            .saturating_add($duration.subsec_millis() as u64)
    };
}

/// The `Mahimahi` trait provides a method to generate a trace in mahimahi format.
///
/// The trace is a sequence of timestamps, each timestamp represents an opportunity
/// of sending a packet at that timestamp.
///
/// This trait is automatically implemented for all types that implement `BwTrace`.
///
/// This trait is often used with [`MahimahiExt`] trait. [`MahimahiExt`] provides
/// methods that generates trace and writes it to a string or a file.
pub trait Mahimahi: BwTrace {
    /// Generate a timestamp sequence in mahimahi format.
    ///
    /// Each timestamp represents an opportunity of sending a packet at that timestamp (in milliseconds).
    ///
    /// For example, if the bandwidth is 12Mbps (one packet per millisecond), then the sequence can be:
    /// \[1, 2, 3, 4, 5\]
    fn mahimahi(&mut self, total_dur: &Duration) -> Vec<u64> {
        let mut timestamp = MAHIMAHI_TS_BIN;
        let mut v = Vec::new();
        let mut transfer = Bandwidth::from_bps(0);
        let mut bin_rem = MAHIMAHI_TS_BIN;
        while let Some((bw, mut dur)) = self.next_bw() {
            if timestamp > *total_dur {
                break;
            }
            while (timestamp <= *total_dur) && !dur.is_zero() {
                let bin = bin_rem.min(dur);
                bin_rem -= bin;
                dur -= bin;
                let bin_factor = bin.as_secs_f64() / MAHIMAHI_TS_BIN.as_secs_f64();
                transfer += bw.mul_f64(bin_factor);
                while transfer >= MTU_PER_MILLIS {
                    v.push(saturating_duration_as_millis_u64!(timestamp));
                    transfer -= MTU_PER_MILLIS;
                }
                if bin_rem.is_zero() {
                    bin_rem = MAHIMAHI_TS_BIN;
                    timestamp += MAHIMAHI_TS_BIN;
                }
            }
        }
        v
    }
}

impl<T: BwTrace + ?Sized> Mahimahi for T {}

/// The `MahimahiExt` trait provides some convenient methods to generate a trace in mahimahi format.
pub trait MahimahiExt: Mahimahi {
    /// Join the mahimahi timestamp sequence to a string.
    fn mahimahi_to_string(&mut self, total_dur: &Duration) -> String {
        let ts = self.mahimahi(total_dur);
        itertools::join(ts, "\n")
    }

    /// Write the mahimahi timestamp sequence to a file.
    fn mahimahi_to_file<P: AsRef<std::path::Path>>(&mut self, total_dur: &Duration, path: P) {
        let content = self.mahimahi_to_string(total_dur);
        std::fs::write(path, content).unwrap();
    }
}

impl<T: Mahimahi + ?Sized> MahimahiExt for T {}

/// Load a mahimahi trace to a [`RepeatedBwPatternConfig`].
///
/// The `trace` is a sequence of timestamps, each timestamp represents an opportunity
/// of sending a packet at that timestamp.
///
/// The `count` is the number of times the trace repeats. If `count` is `None` or `Some(0)`,
/// then the trace will repeat forever.
///
/// Each timestamp will be converted into a 12Mbps bandwidth lasting for 1 millisecond,
/// and then accumulated. For example, if the trace is \[1, 1, 5, 6\] and count is `Some(1)`,
/// then the bandwidth pattern will be \[24Mbps for 1ms, 0Mbps for 3ms, 12Mbps for 2ms\].
///
/// **Warning:** In some cases, this trace may slightly deviate from the behavior of mahimahi.
///
/// Returns `Err` string if the mahimahi trace is invalid.
pub fn load_mahimahi_trace(
    trace: Vec<u64>,
    count: Option<usize>,
) -> Result<RepeatedBwPatternConfig, &'static str> {
    let mut pattern: Vec<StaticBwConfig> = vec![];
    // The closure inserts a bandwidth config into the pattern.
    let mut insert_into_pattern = |config| {
        if pattern.is_empty() {
            pattern.push(config);
        } else {
            let last_config = pattern.last_mut().unwrap();
            if last_config.bw.unwrap() == config.bw.unwrap() {
                last_config.duration =
                    Some(last_config.duration.unwrap() + config.duration.unwrap());
            } else {
                pattern.push(config);
            }
        }
    };

    let mut zeor_ts_cnt = 0; // count of zero timestamps
    let mut last_ts = 0; // last non-zero timestamp
    let mut last_cnt = 0; // count of last non-zero timestamp
    for ts in trace {
        // count zero timestamps
        if ts == 0 {
            zeor_ts_cnt += 1;
            continue;
        }
        // non-zero timestamps
        match ts.cmp(&last_ts) {
            std::cmp::Ordering::Less => {
                return Err("timestamps must be monotonically nondecreasing");
            }
            std::cmp::Ordering::Equal => {
                last_cnt += 1;
            }
            std::cmp::Ordering::Greater => {
                if last_ts > 0 {
                    // insert new bandwidth config
                    insert_into_pattern(
                        StaticBwConfig::new()
                            .bw(MTU_PER_MILLIS * last_cnt)
                            .duration(MAHIMAHI_TS_BIN),
                    );
                }
                if ts - last_ts > 1 {
                    // insert zero bandwidth
                    insert_into_pattern(
                        StaticBwConfig::new()
                            .bw(Bandwidth::ZERO)
                            .duration(MAHIMAHI_TS_BIN * ((ts - last_ts - 1) as u32)),
                    );
                }
                last_cnt = 1;
                last_ts = ts;
            }
        }
    }
    if last_cnt == 0 {
        // no non-zero timestamps
        return Err("trace must last for a nonzero amount of time");
    } else {
        // merge final timestamps and zero timestamps
        insert_into_pattern(
            StaticBwConfig::new()
                .bw(MTU_PER_MILLIS * (last_cnt + zeor_ts_cnt))
                .duration(MAHIMAHI_TS_BIN),
        );
    }
    Ok(RepeatedBwPatternConfig::new()
        .count(count.unwrap_or(0))
        .pattern(
            pattern
                .drain(..)
                .map(|config| Box::new(config) as Box<dyn BwTraceConfig>)
                .collect(),
        ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::StaticBwConfig;
    use crate::Bandwidth;

    #[test]
    fn test_trait() {
        let mut static_bw = StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(24))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            static_bw.mahimahi(&Duration::from_millis(5)),
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]
        );
        let mut static_bw = StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(12))
            .duration(Duration::from_secs(1))
            .build();
        assert_eq!(
            static_bw.mahimahi_to_string(&Duration::from_millis(5)),
            "1\n2\n3\n4\n5"
        );
        let a = vec![
            Box::new(
                StaticBwConfig::new()
                    .bw(Bandwidth::from_mbps(12))
                    .duration(Duration::from_millis(2)),
            ) as Box<dyn BwTraceConfig>,
            Box::new(
                StaticBwConfig::new()
                    .bw(Bandwidth::from_mbps(24))
                    .duration(Duration::from_millis(2)),
            ) as Box<dyn BwTraceConfig>,
        ];
        let mut c = Box::new(RepeatedBwPatternConfig::new().pattern(a).count(2)).into_model();
        assert_eq!(
            c.mahimahi(&Duration::MAX),
            [1, 2, 3, 3, 4, 4, 5, 6, 7, 7, 8, 8]
        );
    }

    #[test]
    fn test_load() {
        assert!(matches!(
            load_mahimahi_trace(vec![0, 2, 4, 3], None),
            Err("timestamps must be monotonically nondecreasing")
        ));
        assert!(matches!(
            load_mahimahi_trace(vec![0, 0, 0], None),
            Err("trace must last for a nonzero amount of time")
        ));

        let trace = vec![1, 1, 5, 6];
        let mut bw = load_mahimahi_trace(trace, None).unwrap().build();
        // first cycle
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(1)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(0), Duration::from_millis(3)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_millis(2)))
        );
        // second cycle
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(1)))
        );

        let trace = vec![0, 0, 2, 2, 3, 3, 6, 6];
        let mut bw = load_mahimahi_trace(trace, Some(0)).unwrap().build();
        // first cycle
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(0), Duration::from_millis(1)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(2)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(0), Duration::from_millis(2)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(48), Duration::from_millis(1)))
        );
        // second cycle
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(0), Duration::from_millis(1)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(2)))
        );

        let mut bw = RepeatedBwPatternConfig::new()
            .count(2)
            .pattern(vec![
                Box::new(load_mahimahi_trace(vec![1, 1, 2, 2, 3, 3], Some(1)).unwrap())
                    as Box<dyn BwTraceConfig>,
                Box::new(load_mahimahi_trace(vec![1, 2], Some(2)).unwrap())
                    as Box<dyn BwTraceConfig>,
            ])
            .build();
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(3)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_millis(2)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_millis(2)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(24), Duration::from_millis(3)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_millis(2)))
        );
        assert_eq!(
            bw.next_bw(),
            Some((Bandwidth::from_mbps(12), Duration::from_millis(2)))
        );
        assert_eq!(bw.next_bw(), None);
    }

    #[test]
    fn test_interoperability() {
        // this check only works on non-zero timestamps trace, which has full interoperability
        let check = |trace: Vec<u64>| {
            let mut bw = load_mahimahi_trace(trace.clone(), None).unwrap().build();
            assert_eq!(
                bw.mahimahi(&Duration::from_millis(*trace.last().unwrap())),
                trace
            );
        };
        check(vec![1, 1, 5, 6]);
        check(vec![2, 2, 3, 3, 4, 4, 5, 5, 8, 9]);

        let mut bw = load_mahimahi_trace(vec![0, 0, 2, 2, 3, 3, 6, 6], None)
            .unwrap()
            .build();
        assert_eq!(
            bw.mahimahi(&Duration::from_millis(12)),
            vec![2, 2, 3, 3, 6, 6, 6, 6, 8, 8, 9, 9, 12, 12, 12, 12]
        );

        let mut bw = RepeatedBwPatternConfig::new()
            .count(2)
            .pattern(vec![
                Box::new(load_mahimahi_trace(vec![1, 1, 2, 2, 3, 3], Some(1)).unwrap())
                    as Box<dyn BwTraceConfig>,
                Box::new(load_mahimahi_trace(vec![1, 2], Some(2)).unwrap())
                    as Box<dyn BwTraceConfig>,
            ])
            .build();
        assert_eq!(
            bw.mahimahi(&Duration::MAX),
            vec![1, 1, 2, 2, 3, 3, 4, 5, 6, 7, 8, 8, 9, 9, 10, 10, 11, 12, 13, 14]
        );
    }
}
