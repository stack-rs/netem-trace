//! Example demonstrating how to expand traces into series for plotting.
//!
//! This example shows how to:
//! 1. Create trace models
//! 2. Expand them into time series within a specified time range
//! 3. Export the series to JSON and CSV files for plotting in other languages
//!
//! Run with:
//! ```bash
//! cargo run --example plot_traces --features trace-ext
//! # or
//! cargo run --example plot_traces --all-features
//! ```

use netem_trace::model::{
    NormalizedBwConfig, RepeatedBwPatternConfig, SawtoothBwConfig, StaticBwConfig,
    StaticDelayConfig, StaticDelayPerPacketConfig,
};
use netem_trace::series::{
    expand_bw_trace, expand_delay_per_packet_trace, expand_delay_trace, write_bw_series_csv,
    write_bw_series_json, write_delay_per_packet_series_csv, write_delay_per_packet_series_json,
    write_delay_series_csv, write_delay_series_json,
};
use netem_trace::{Bandwidth, Duration};

fn main() {
    println!("=== Trace Series Expansion Examples ===\n");

    // Example 1: Static Bandwidth Trace
    println!("1. Static Bandwidth Trace");
    let mut static_bw = StaticBwConfig::new()
        .bw(Bandwidth::from_mbps(10))
        .duration(Duration::from_secs(5))
        .build();

    let series = expand_bw_trace(
        &mut static_bw,
        Duration::from_secs(0),
        Duration::from_secs(5),
    );

    println!("   Generated {} data points", series.len());
    for point in &series {
        println!(
            "   - Start: {:.2}s, BW: {} Mbps, Duration: {:.2}s",
            point.start_time.as_secs_f64(),
            point.value.as_bps() / 1_000_000,
            point.duration.as_secs_f64()
        );
    }

    // Export to files
    write_bw_series_json(&series, "static_bw.json").expect("Failed to write JSON");
    write_bw_series_csv(&series, "static_bw.csv").expect("Failed to write CSV");
    println!("   Exported to static_bw.json and static_bw.csv\n");

    // Example 2: Sawtooth Bandwidth Trace with Time Cutting
    println!("2. Sawtooth Bandwidth Trace (cut from 1s to 4s)");
    let mut sawtooth_bw = SawtoothBwConfig::new()
        .bottom(Bandwidth::from_mbps(5))
        .top(Bandwidth::from_mbps(15))
        .duration(Duration::from_secs(10))
        .step(Duration::from_millis(500))
        .interval(Duration::from_secs(2))
        .duty_ratio(0.8)
        .build();

    let series = expand_bw_trace(
        &mut sawtooth_bw,
        Duration::from_secs(1),
        Duration::from_secs(4),
    );

    println!("   Generated {} data points", series.len());
    println!("   First 5 points:");
    for point in series.iter().take(5) {
        println!(
            "   - Start: {:.2}s, BW: {:.2} Mbps, Duration: {:.2}s",
            point.start_time.as_secs_f64(),
            point.value.as_bps() as f64 / 1_000_000.0,
            point.duration.as_secs_f64()
        );
    }

    write_bw_series_json(&series, "sawtooth_bw.json").expect("Failed to write JSON");
    write_bw_series_csv(&series, "sawtooth_bw.csv").expect("Failed to write CSV");
    println!("   Exported to sawtooth_bw.json and sawtooth_bw.csv\n");

    // Example 3: Normalized (Random) Bandwidth Trace
    println!("3. Normalized Bandwidth Trace");
    let mut normal_bw = NormalizedBwConfig::new()
        .mean(Bandwidth::from_mbps(12))
        .std_dev(Bandwidth::from_mbps(2))
        .duration(Duration::from_secs(5))
        .step(Duration::from_millis(200))
        .seed(42)
        .build();

    let series = expand_bw_trace(
        &mut normal_bw,
        Duration::from_secs(0),
        Duration::from_secs(5),
    );

    println!("   Generated {} data points", series.len());
    println!("   First 3 points:");
    for point in series.iter().take(3) {
        println!(
            "   - Start: {:.2}s, BW: {:.2} Mbps, Duration: {:.2}s",
            point.start_time.as_secs_f64(),
            point.value.as_bps() as f64 / 1_000_000.0,
            point.duration.as_secs_f64()
        );
    }

    write_bw_series_json(&series, "normal_bw.json").expect("Failed to write JSON");
    write_bw_series_csv(&series, "normal_bw.csv").expect("Failed to write CSV");
    println!("   Exported to normal_bw.json and normal_bw.csv\n");

    // Example 4: Repeated Pattern
    println!("4. Repeated Bandwidth Pattern");
    let pattern = vec![
        Box::new(
            StaticBwConfig::new()
                .bw(Bandwidth::from_mbps(10))
                .duration(Duration::from_secs(1)),
        ) as Box<dyn netem_trace::model::BwTraceConfig>,
        Box::new(
            StaticBwConfig::new()
                .bw(Bandwidth::from_mbps(20))
                .duration(Duration::from_secs(1)),
        ) as Box<dyn netem_trace::model::BwTraceConfig>,
    ];

    let mut repeated_bw = RepeatedBwPatternConfig::new()
        .pattern(pattern)
        .count(3)
        .build();

    let series = expand_bw_trace(
        &mut repeated_bw,
        Duration::from_secs(0),
        Duration::from_secs(6),
    );

    println!("   Generated {} data points", series.len());
    for point in &series {
        println!(
            "   - Start: {:.2}s, BW: {} Mbps, Duration: {:.2}s",
            point.start_time.as_secs_f64(),
            point.value.as_bps() / 1_000_000,
            point.duration.as_secs_f64()
        );
    }

    write_bw_series_json(&series, "repeated_bw.json").expect("Failed to write JSON");
    write_bw_series_csv(&series, "repeated_bw.csv").expect("Failed to write CSV");
    println!("   Exported to repeated_bw.json and repeated_bw.csv\n");

    // Example 5: Delay Trace
    println!("5. Static Delay Trace");
    let mut delay_trace = StaticDelayConfig::new()
        .delay(Duration::from_millis(50))
        .duration(Duration::from_secs(3))
        .build();

    let series = expand_delay_trace(
        &mut delay_trace,
        Duration::from_secs(0),
        Duration::from_secs(3),
    );

    println!("   Generated {} data points", series.len());
    for point in &series {
        println!(
            "   - Start: {:.2}s, Delay: {:.2}ms, Duration: {:.2}s",
            point.start_time.as_secs_f64(),
            point.value.as_secs_f64() * 1000.0,
            point.duration.as_secs_f64()
        );
    }

    write_delay_series_json(&series, "delay.json").expect("Failed to write JSON");
    write_delay_series_csv(&series, "delay.csv").expect("Failed to write CSV");
    println!("   Exported to delay.json and delay.csv\n");

    // Example 6: Per-Packet Delay Trace
    println!("6. Per-Packet Delay Trace");
    let mut per_packet_delay = StaticDelayPerPacketConfig::new()
        .delay(Duration::from_millis(25))
        .count(10)
        .build();

    let series = expand_delay_per_packet_trace(&mut per_packet_delay, Some(10));

    println!("   Generated {} data points", series.len());
    println!("   First 5 packets:");
    for point in series.iter().take(5) {
        println!(
            "   - Packet {}: Delay {:.2}ms",
            point.packet_index,
            point.value.as_secs_f64() * 1000.0
        );
    }

    write_delay_per_packet_series_json(&series, "per_packet_delay.json")
        .expect("Failed to write JSON");
    write_delay_per_packet_series_csv(&series, "per_packet_delay.csv")
        .expect("Failed to write CSV");
    println!("   Exported to per_packet_delay.json and per_packet_delay.csv\n");

    println!("=== All examples completed successfully! ===");
    println!("\nYou can now use these JSON/CSV files for plotting in:");
    println!("  - Python (matplotlib, pandas, plotly)");
    println!("  - R (ggplot2)");
    println!("  - JavaScript (D3.js, Chart.js)");
    println!("  - MATLAB");
    println!("  - or any other data visualization tool!");
}
