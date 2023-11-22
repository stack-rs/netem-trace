//! This module can generate traces in mahimahi format for struct implementing [`BwTrace`].
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
//! assert_eq!(static_bw.mahimahi(&Duration::from_millis(5)), [0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
//! let mut static_bw = StaticBwConfig::new()
//!     .bw(Bandwidth::from_mbps(12))
//!     .duration(Duration::from_secs(1))
//!     .build();
//! assert_eq!(static_bw.mahimahi_to_string(&Duration::from_millis(5)), "0\n1\n2\n3\n4");
//! ```

use crate::{Bandwidth, BwTrace, Duration};

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
    /// \[0, 1, 2, 3, 4\]
    fn mahimahi(&mut self, total_dur: &Duration) -> Vec<u64> {
        let mut timestamp = Duration::from_secs(0);
        let mut v = Vec::new();
        let mut transfer = Bandwidth::from_bps(0);
        let mut bin_rem = MAHIMAHI_TS_BIN;
        while let Some((bw, mut dur)) = self.next_bw() {
            if timestamp >= *total_dur {
                break;
            }
            while (timestamp < *total_dur) && !dur.is_zero() {
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
