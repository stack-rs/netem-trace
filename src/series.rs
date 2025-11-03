//! This module provides functionality to expand trace models into time series data
//! that can be used for plotting and analysis in other programming languages.
//!
//! **Note:** This module is only available when the `trace-ext` feature is enabled.
//!
//! ## Overview
//!
//! Trace models generate values by calling `next_xxx()` methods. This module provides
//! functions to expand these traces into a complete series within a specified time range,
//! making them suitable for visualization and export.
//!
//! ## Examples
//!
//! ```
//! # use netem_trace::model::StaticBwConfig;
//! # use netem_trace::{Bandwidth, Duration, BwTrace};
//! # use netem_trace::series::expand_bw_trace;
//! let mut static_bw = StaticBwConfig::new()
//!     .bw(Bandwidth::from_mbps(24))
//!     .duration(Duration::from_secs(2))
//!     .build();
//!
//! let series = expand_bw_trace(
//!     &mut static_bw,
//!     Duration::from_secs(0),
//!     Duration::from_secs(2)
//! );
//!
//! assert_eq!(series.len(), 1);
//! assert_eq!(series[0].start_time, Duration::from_secs(0));
//! assert_eq!(series[0].value, Bandwidth::from_mbps(24));
//! assert_eq!(series[0].duration, Duration::from_secs(2));
//! ```

use crate::{
    Bandwidth, BwTrace, Delay, DelayPerPacketTrace, DelayTrace, DuplicatePattern, DuplicateTrace,
    Duration, LossPattern, LossTrace,
};
use std::io::Write;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single point in a bandwidth trace series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BwSeriesPoint {
    /// The time when this bandwidth value starts (relative to trace start)
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub start_time: Duration,
    /// The bandwidth value
    pub value: Bandwidth,
    /// How long this bandwidth value lasts
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub duration: Duration,
}

/// A single point in a delay trace series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DelaySeriesPoint {
    /// The time when this delay value starts (relative to trace start)
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub start_time: Duration,
    /// The delay value
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub value: Delay,
    /// How long this delay value lasts
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub duration: Duration,
}

/// A single point in a per-packet delay trace series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DelayPerPacketSeriesPoint {
    /// The packet index (0-based)
    pub packet_index: usize,
    /// The delay value for this packet
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub value: Delay,
}

/// A single point in a loss trace series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LossSeriesPoint {
    /// The time when this loss pattern starts (relative to trace start)
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub start_time: Duration,
    /// The loss pattern
    pub value: LossPattern,
    /// How long this loss pattern lasts
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub duration: Duration,
}

/// A single point in a duplicate trace series.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DuplicateSeriesPoint {
    /// The time when this duplicate pattern starts (relative to trace start)
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub start_time: Duration,
    /// The duplicate pattern
    pub value: DuplicatePattern,
    /// How long this duplicate pattern lasts
    #[cfg_attr(feature = "serde", serde(with = "duration_serde"))]
    pub duration: Duration,
}

#[cfg(feature = "serde")]
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs = duration.as_secs_f64();
        serializer.serialize_f64(secs)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

/// Expands a bandwidth trace into a series within the specified time range.
///
/// This function repeatedly calls `next_bw()` on the trace to build a complete
/// series, cutting the trace to fit within `start_time` to `end_time`.
///
/// # Arguments
///
/// * `trace` - The bandwidth trace to expand
/// * `start_time` - The start time of the desired range
/// * `end_time` - The end time of the desired range
///
/// # Returns
///
/// A vector of `BwSeriesPoint` representing the trace within the time range.
///
/// # Examples
///
/// ```
/// # use netem_trace::model::StaticBwConfig;
/// # use netem_trace::{Bandwidth, Duration, BwTrace};
/// # use netem_trace::series::expand_bw_trace;
/// let mut trace = StaticBwConfig::new()
///     .bw(Bandwidth::from_mbps(10))
///     .duration(Duration::from_secs(5))
///     .build();
///
/// let series = expand_bw_trace(
///     &mut trace,
///     Duration::from_secs(1),
///     Duration::from_secs(4)
/// );
///
/// // The series will be cut to fit [1s, 4s], normalized to start at 0
/// assert_eq!(series[0].start_time, Duration::from_secs(0));
/// assert_eq!(series[0].duration, Duration::from_secs(3));
/// ```
pub fn expand_bw_trace(
    trace: &mut dyn BwTrace,
    start_time: Duration,
    end_time: Duration,
) -> Vec<BwSeriesPoint> {
    let mut series = Vec::new();
    let mut current_time = Duration::ZERO;

    while let Some((value, duration)) = trace.next_bw() {
        let segment_end = current_time + duration;

        // Skip segments that end before start_time
        if segment_end <= start_time {
            current_time = segment_end;
            continue;
        }

        // Stop if we've passed end_time
        if current_time >= end_time {
            break;
        }

        // Calculate the actual start and duration for this segment
        let actual_start = current_time.max(start_time);
        let actual_end = segment_end.min(end_time);
        let actual_duration = actual_end.saturating_sub(actual_start);

        if !actual_duration.is_zero() {
            series.push(BwSeriesPoint {
                start_time: actual_start - start_time, // Normalize to start at 0
                value,
                duration: actual_duration,
            });
        }

        current_time = segment_end;

        // Stop if we've reached end_time
        if current_time >= end_time {
            break;
        }
    }

    series
}

/// Expands a delay trace into a series within the specified time range.
///
/// This function repeatedly calls `next_delay()` on the trace to build a complete
/// series, cutting the trace to fit within `start_time` to `end_time`.
pub fn expand_delay_trace(
    trace: &mut dyn DelayTrace,
    start_time: Duration,
    end_time: Duration,
) -> Vec<DelaySeriesPoint> {
    let mut series = Vec::new();
    let mut current_time = Duration::ZERO;

    while let Some((value, duration)) = trace.next_delay() {
        let segment_end = current_time + duration;

        if segment_end <= start_time {
            current_time = segment_end;
            continue;
        }

        if current_time >= end_time {
            break;
        }

        let actual_start = current_time.max(start_time);
        let actual_end = segment_end.min(end_time);
        let actual_duration = actual_end.saturating_sub(actual_start);

        if !actual_duration.is_zero() {
            series.push(DelaySeriesPoint {
                start_time: actual_start - start_time,
                value,
                duration: actual_duration,
            });
        }

        current_time = segment_end;

        if current_time >= end_time {
            break;
        }
    }

    series
}

/// Expands a per-packet delay trace into a series.
///
/// Since per-packet delays don't have time durations, this function collects
/// delays for a specified number of packets or until the trace ends.
///
/// # Arguments
///
/// * `trace` - The per-packet delay trace to expand
/// * `max_packets` - Maximum number of packets to collect (None for unlimited)
///
/// # Returns
///
/// A vector of `DelayPerPacketSeriesPoint` representing the delays.
pub fn expand_delay_per_packet_trace(
    trace: &mut dyn DelayPerPacketTrace,
    max_packets: Option<usize>,
) -> Vec<DelayPerPacketSeriesPoint> {
    let mut series = Vec::new();
    let mut packet_index = 0;

    while let Some(value) = trace.next_delay() {
        series.push(DelayPerPacketSeriesPoint {
            packet_index,
            value,
        });

        packet_index += 1;

        if let Some(max) = max_packets {
            if packet_index >= max {
                break;
            }
        }
    }

    series
}

/// Expands a loss trace into a series within the specified time range.
pub fn expand_loss_trace(
    trace: &mut dyn LossTrace,
    start_time: Duration,
    end_time: Duration,
) -> Vec<LossSeriesPoint> {
    let mut series = Vec::new();
    let mut current_time = Duration::ZERO;

    while let Some((value, duration)) = trace.next_loss() {
        let segment_end = current_time + duration;

        if segment_end <= start_time {
            current_time = segment_end;
            continue;
        }

        if current_time >= end_time {
            break;
        }

        let actual_start = current_time.max(start_time);
        let actual_end = segment_end.min(end_time);
        let actual_duration = actual_end.saturating_sub(actual_start);

        if !actual_duration.is_zero() {
            series.push(LossSeriesPoint {
                start_time: actual_start - start_time,
                value,
                duration: actual_duration,
            });
        }

        current_time = segment_end;

        if current_time >= end_time {
            break;
        }
    }

    series
}

/// Expands a duplicate trace into a series within the specified time range.
pub fn expand_duplicate_trace(
    trace: &mut dyn DuplicateTrace,
    start_time: Duration,
    end_time: Duration,
) -> Vec<DuplicateSeriesPoint> {
    let mut series = Vec::new();
    let mut current_time = Duration::ZERO;

    while let Some((value, duration)) = trace.next_duplicate() {
        let segment_end = current_time + duration;

        if segment_end <= start_time {
            current_time = segment_end;
            continue;
        }

        if current_time >= end_time {
            break;
        }

        let actual_start = current_time.max(start_time);
        let actual_end = segment_end.min(end_time);
        let actual_duration = actual_end.saturating_sub(actual_start);

        if !actual_duration.is_zero() {
            series.push(DuplicateSeriesPoint {
                start_time: actual_start - start_time,
                value,
                duration: actual_duration,
            });
        }

        current_time = segment_end;

        if current_time >= end_time {
            break;
        }
    }

    series
}

/// Writes a bandwidth series to a file in JSON format.
///
/// # Arguments
///
/// * `series` - The bandwidth series to write
/// * `path` - The file path to write to
///
/// # Errors
///
/// Returns an error if the file cannot be created or written to, or if
/// serialization fails.
#[cfg(feature = "serde")]
pub fn write_bw_series_json<P: AsRef<std::path::Path>>(
    series: &[BwSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(series).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Writes a bandwidth series to a file in CSV format.
///
/// The CSV will have columns: start_time_secs, bandwidth_bps, duration_secs
///
/// # Arguments
///
/// * `series` - The bandwidth series to write
/// * `path` - The file path to write to
pub fn write_bw_series_csv<P: AsRef<std::path::Path>>(
    series: &[BwSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "start_time_secs,bandwidth_bps,duration_secs")?;

    for point in series {
        writeln!(
            file,
            "{},{},{}",
            point.start_time.as_secs_f64(),
            point.value.as_bps(),
            point.duration.as_secs_f64()
        )?;
    }

    Ok(())
}

/// Writes a delay series to a file in JSON format.
#[cfg(feature = "serde")]
pub fn write_delay_series_json<P: AsRef<std::path::Path>>(
    series: &[DelaySeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(series).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Writes a delay series to a file in CSV format.
///
/// The CSV will have columns: start_time_secs, delay_secs, duration_secs
pub fn write_delay_series_csv<P: AsRef<std::path::Path>>(
    series: &[DelaySeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "start_time_secs,delay_secs,duration_secs")?;

    for point in series {
        writeln!(
            file,
            "{},{},{}",
            point.start_time.as_secs_f64(),
            point.value.as_secs_f64(),
            point.duration.as_secs_f64()
        )?;
    }

    Ok(())
}

/// Writes a per-packet delay series to a file in JSON format.
#[cfg(feature = "serde")]
pub fn write_delay_per_packet_series_json<P: AsRef<std::path::Path>>(
    series: &[DelayPerPacketSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(series).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Writes a per-packet delay series to a file in CSV format.
///
/// The CSV will have columns: packet_index, delay_secs
pub fn write_delay_per_packet_series_csv<P: AsRef<std::path::Path>>(
    series: &[DelayPerPacketSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "packet_index,delay_secs")?;

    for point in series {
        writeln!(file, "{},{}", point.packet_index, point.value.as_secs_f64())?;
    }

    Ok(())
}

/// Writes a loss series to a file in JSON format.
#[cfg(feature = "serde")]
pub fn write_loss_series_json<P: AsRef<std::path::Path>>(
    series: &[LossSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(series).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Writes a loss series to a file in CSV format.
///
/// The CSV will have columns: start_time_secs, loss_pattern, duration_secs
/// where loss_pattern is a semicolon-separated list of probabilities.
pub fn write_loss_series_csv<P: AsRef<std::path::Path>>(
    series: &[LossSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "start_time_secs,loss_pattern,duration_secs")?;

    for point in series {
        let pattern_str = point
            .value
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(";");

        writeln!(
            file,
            "{},{},{}",
            point.start_time.as_secs_f64(),
            pattern_str,
            point.duration.as_secs_f64()
        )?;
    }

    Ok(())
}

/// Writes a duplicate series to a file in JSON format.
#[cfg(feature = "serde")]
pub fn write_duplicate_series_json<P: AsRef<std::path::Path>>(
    series: &[DuplicateSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(series).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Writes a duplicate series to a file in CSV format.
///
/// The CSV will have columns: start_time_secs, duplicate_pattern, duration_secs
/// where duplicate_pattern is a semicolon-separated list of probabilities.
pub fn write_duplicate_series_csv<P: AsRef<std::path::Path>>(
    series: &[DuplicateSeriesPoint],
    path: P,
) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "start_time_secs,duplicate_pattern,duration_secs")?;

    for point in series {
        let pattern_str = point
            .value
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(";");

        writeln!(
            file,
            "{},{},{}",
            point.start_time.as_secs_f64(),
            pattern_str,
            point.duration.as_secs_f64()
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StaticBwConfig;

    #[test]
    fn test_expand_bw_trace_basic() {
        let mut trace = StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(10))
            .duration(Duration::from_secs(5))
            .build();

        let series = expand_bw_trace(&mut trace, Duration::from_secs(0), Duration::from_secs(5));

        assert_eq!(series.len(), 1);
        assert_eq!(series[0].start_time, Duration::from_secs(0));
        assert_eq!(series[0].value, Bandwidth::from_mbps(10));
        assert_eq!(series[0].duration, Duration::from_secs(5));
    }

    #[test]
    fn test_expand_bw_trace_with_cutting() {
        let mut trace = StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(10))
            .duration(Duration::from_secs(10))
            .build();

        let series = expand_bw_trace(&mut trace, Duration::from_secs(2), Duration::from_secs(7));

        assert_eq!(series.len(), 1);
        assert_eq!(series[0].start_time, Duration::from_secs(0)); // Normalized to start at 0
        assert_eq!(series[0].value, Bandwidth::from_mbps(10));
        assert_eq!(series[0].duration, Duration::from_secs(5)); // 7 - 2 = 5
    }
}
