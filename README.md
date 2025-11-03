# netem-trace

[![github-repo](https://img.shields.io/badge/github-stack--rs/netem--trace-f5dc23?logo=github)](https://github.com/stack-rs/netem-trace)
[![crates.io](https://img.shields.io/crates/v/netem-trace.svg?logo=rust)](https://crates.io/crates/netem-trace)
[![docs.rs](https://img.shields.io/badge/docs.rs-netem--trace-blue?logo=docsdotrs)](https://docs.rs/netem-trace)
[![LICENSE Apache-2.0](https://img.shields.io/github/license/stack-rs/netem-trace?logo=Apache)](https://github.com/stack-rs/netem-trace/blob/main/LICENSE)

A library for generating network emulation trace. Now only supported [mahimahi](https://github.com/ravinet/mahimahi).

**Attention**: This library is still under development. The API is not stable.

MSRV: 1.60

## Examples

Use bandwidth model directly (`model` or `bw-model` feature should be enabled):

```rust
use netem_trace::model::{StaticBwConfig, NormalizedBwConfig};
use netem_trace::{Bandwidth, Duration, BwTrace};
let mut static_bw = StaticBwConfig::new()
    .bw(Bandwidth::from_mbps(24))
    .duration(Duration::from_secs(1))
    .build();
assert_eq!(
    static_bw.next_bw(),
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
let mut normal_bw = NormalizedBwConfig::new()
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
```

Produce traces in [mahimahi](https://github.com/ravinet/mahimahi) format (`mahimahi` feature should also be enabled):

```rust
use netem_trace::model::{StaticBwConfig};
use netem_trace::{Bandwidth, Duration};
use netem_trace::{Mahimahi, MahimahiExt};
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
            .duration(Duration::from_secs(1)),
    ) as Box<dyn BwTraceConfig>,
    Box::new(
        StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(24))
            .duration(Duration::from_secs(1)),
    ) as Box<dyn BwTraceConfig>,
];
let mut c = Box::new(RepeatedBwPatternConfig::new().pattern(a).count(2)).into_model();
assert_eq!(c.mahimahi(&Duration::from_millis(5)), [1, 2, 3, 4, 5]);
```

Load traces in [mahimahi](https://github.com/ravinet/mahimahi) format (`mahimahi` feature should also be enabled):

```rust
use netem_trace::load_mahimahi_trace;
use netem_trace::{Bandwidth, BwTrace, Duration};
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
```

Work with configuration files (`serde` feature should also be enabled):

```rust
use netem_trace::model::{StaticBwConfig, BwTraceConfig, RepeatedBwPatternConfig};
use netem_trace::{Bandwidth, Duration, BwTrace};

let a = vec![
    Box::new(
        StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(12))
            .duration(Duration::from_secs(1)),
    ) as Box<dyn BwTraceConfig>,
    Box::new(
        StaticBwConfig::new()
            .bw(Bandwidth::from_mbps(24))
            .duration(Duration::from_secs(1)),
    ) as Box<dyn BwTraceConfig>,
];
let ser =
    Box::new(RepeatedBwPatternConfig::new().pattern(a).count(2)) as Box<dyn BwTraceConfig>;
let ser_str = serde_json::to_string(&ser).unwrap();
let des_str = "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":12000000},\"duration\":{\"secs\":1,\"nanos\":0}}},{\"StaticBwConfig\":{\"bw\":{\"gbps\":0,\"bps\":24000000},\"duration\":{\"secs\":1,\"nanos\":0}}}],\"count\":2}}";
// The content would be "{\"RepeatedBwPatternConfig\":{\"pattern\":[{\"StaticBwConfig\":{\"bw\":\"12Mbps\",\"duration\":\"1s\"}},{\"StaticBwConfig\":{\"bw\":\"24Mbps\",\"duration\":\"1s\"}}],\"count\":2}}"
// if the `human` feature is also enabled.
assert_eq!(ser_str, des_str);
let des: Box<dyn BwTraceConfig> = serde_json::from_str(des_str).unwrap();
let mut model = des.into_model();
assert_eq!(
    model.next_bw(),
    Some((Bandwidth::from_mbps(12), Duration::from_secs(1)))
);
```

## Plotting and Visualization

The `series` module (requires `trace-ext` feature) allows you to expand trace models into time series data for plotting and analysis.

### Quick Example

```rust
use netem_trace::model::StaticBwConfig;
use netem_trace::series::{expand_bw_trace, write_bw_series_json, write_bw_series_csv};
use netem_trace::{Bandwidth, Duration};

// Create and expand a trace
let mut trace = StaticBwConfig::new()
    .bw(Bandwidth::from_mbps(10))
    .duration(Duration::from_secs(5))
    .build();

let series = expand_bw_trace(&mut trace, Duration::from_secs(0), Duration::from_secs(5));

// Export to JSON or CSV
write_bw_series_json(&series, "trace.json").unwrap();
write_bw_series_csv(&series, "trace.csv").unwrap();
```

### Supported Trace Types

- **Bandwidth traces**: `expand_bw_trace()` → `BwSeriesPoint` (start_time, bandwidth, duration)
- **Delay traces**: `expand_delay_trace()` → `DelaySeriesPoint` (start_time, delay, duration)
- **Per-packet delay**: `expand_delay_per_packet_trace()` → `DelayPerPacketSeriesPoint` (packet_index, delay)
- **Loss/Duplicate traces**: `expand_loss_trace()`, `expand_duplicate_trace()`

### Export Formats

**JSON**: Full structure with nested fields

```json
[{"start_time": 0.0, "value": {"gbps": 0, "bps": 10000000}, "duration": 1.0}, ...]

```

**CSV**: Flat format for easy plotting

```csv
start_time_secs,bandwidth_bps,duration_secs
0.0,10000000,1.0
```

### Plotting in Python

```python
import json
import matplotlib.pyplot as plt

with open('trace.json') as f:
    data = json.load(f)

times, bandwidths = [], []
for point in data:
    start = point['start_time']
    duration = point['duration']
    bw = point['value']['bps'] / 1_000_000
    times.extend([start, start + duration])
    bandwidths.extend([bw, bw])

plt.plot(times, bandwidths)
plt.xlabel('Time (s)')
plt.ylabel('Bandwidth (Mbps)')
plt.show()
```

See `examples/plot_traces.rs` and `examples/plot_example.py` for complete examples. Run with:

```bash
cargo run --example plot_traces --features trace-ext
```

## Maintainer

[@BobAnkh](https://github.com/BobAnkh)

## How to contribute

You should follow our [Code of Conduct](/CODE_OF_CONDUCT.md).

See [CONTRIBUTING GUIDELINES](/CONTRIBUTING.md) for contributing conventions.

Make sure to pass all the tests before submitting your code.

### Contributors

<table>
<tr>
    <td align="center" style="word-wrap: break-word; width: 150.0; height: 150.0">
        <a href=https://github.com/BobAnkh>
            <img src=https://avatars.githubusercontent.com/u/44333669?v=4 width="100;"  style="border-radius:50%;align-items:center;justify-content:center;overflow:hidden;padding-top:10px" alt=Yixin Shen/>
            <br />
            <sub style="font-size:14px"><b>Yixin Shen</b></sub>
        </a>
    </td>
    <td align="center" style="word-wrap: break-word; width: 150.0; height: 150.0">
        <a href=https://github.com/Centaurus99>
            <img src=https://avatars.githubusercontent.com/u/45268165?v=4 width="100;"  style="border-radius:50%;align-items:center;justify-content:center;overflow:hidden;padding-top:10px" alt=Haixuan Tong/>
            <br />
            <sub style="font-size:14px"><b>Haixuan Tong</b></sub>
        </a>
    </td>
    <td align="center" style="word-wrap: break-word; width: 150.0; height: 150.0">
        <a href=https://github.com/Lethe10137>
            <img src=https://avatars.githubusercontent.com/u/87625844?v=4 width="100;"  style="border-radius:50%;align-items:center;justify-content:center;overflow:hidden;padding-top:10px" alt=Lethe Lee/>
            <br />
            <sub style="font-size:14px"><b>Lethe Lee</b></sub>
        </a>
    </td>
    <td align="center" style="word-wrap: break-word; width: 150.0; height: 150.0">
        <a href=https://github.com/un-lock-able>
            <img src=https://avatars.githubusercontent.com/u/57709292?v=4 width="100;"  style="border-radius:50%;align-items:center;justify-content:center;overflow:hidden;padding-top:10px" alt=Yixuan Gao/>
            <br />
            <sub style="font-size:14px"><b>Yixuan Gao</b></sub>
        </a>
    </td>
    <td align="center" style="word-wrap: break-word; width: 150.0; height: 150.0">
        <a href=https://github.com/maxime-bruno>
            <img src=https://avatars.githubusercontent.com/u/201727293?v=4 width="100;"  style="border-radius:50%;align-items:center;justify-content:center;overflow:hidden;padding-top:10px" alt=maxime-bruno/>
            <br />
            <sub style="font-size:14px"><b>maxime-bruno</b></sub>
        </a>
    </td>
</tr>
</table>

## LICENSE

[Apache-2.0](LICENSE)
