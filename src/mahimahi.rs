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

pub trait Mahimahi: BwTrace {
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

pub trait MahimahiExt: Mahimahi {
    fn mahimahi_to_string(&mut self, total_dur: &Duration) -> String {
        let ts = self.mahimahi(total_dur);
        itertools::join(&ts, "\n")
    }

    fn mahimahi_to_file<P: AsRef<std::path::Path>>(&mut self, total_dur: &Duration, path: P) {
        let ts = self.mahimahi(total_dur);
        let content = itertools::join(&ts, "\n");
        std::fs::write(path, content).unwrap();
    }
}

impl<T: Mahimahi + ?Sized> MahimahiExt for T {}
