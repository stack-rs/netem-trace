//! This module provides a compat serialization and deserialization for `TraceBwConfig`.
//!
//!
//! Enable `serde` feature to use this module.
//!
//! Use example can be found in the doc of `TraceBwConfig`.
//!  

use super::TraceBwConfig;
use bandwidth::Bandwidth;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::time::Duration;

impl Serialize for TraceBwConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.pattern.len()))?;
        for (duration, bandwidths) in &self.pattern {
            let duration_ms_f64 = duration.as_secs_f64() * 1000f64;
            let bandwidth_mbps_f64: Vec<f64> = bandwidths
                .iter()
                .map(|b| b.as_gbps_f64() * 1000f64)
                .collect();
            seq.serialize_element(&(duration_ms_f64, bandwidth_mbps_f64))?;
        }
        Ok(seq.end().unwrap())
    }
}

impl<'de> Deserialize<'de> for TraceBwConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TraceBwConfigVisitor;
        impl<'de> Visitor<'de> for TraceBwConfigVisitor {
            type Value = TraceBwConfig;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of [f64, [f64, f64, ...]]")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut pattern = Vec::new();
                while let Some((duration_ms_f64, bandwidths_mbps_f64)) =
                    seq.next_element::<(f64, Vec<f64>)>()?
                {
                    let duration = Duration::from_secs_f64(duration_ms_f64 / 1000f64);
                    let bandwidths = bandwidths_mbps_f64
                        .into_iter()
                        .map(|b| Bandwidth::from_gbps_f64(b * 0.001))
                        .collect();
                    pattern.push((duration, bandwidths));
                }
                Ok(TraceBwConfig { pattern })
            }
        }
        deserializer.deserialize_seq(TraceBwConfigVisitor)
    }
}
