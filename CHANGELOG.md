# CHANGELOG

## [v0.4.1](https://github.com/stack-rs/netem-trace/releases/tag/v0.4.1) - 2025-06-09 08:26:48+00:00

## What's Changed
* Adding a per-packet delay trace by @maxime-bruno in [#21](https://github.com/stack-rs/netem-trace/pull/21)

## New Contributors
* @maxime-bruno made their first contribution in [#21](https://github.com/stack-rs/netem-trace/pull/21)

**Full Comparison**: https://github.com/stack-rs/netem-trace/compare/v0.4.0...v0.4.1

### Feature

- model:
  - add a per-packet delay trace trait and models (#21) ([a1b4995](https://github.com/stack-rs/netem-trace/commit/a1b499550c9eb38d67732d6510393535a6b8e3f9)) ([#21](https://github.com/stack-rs/netem-trace/pull/21))

### Documentation

- readme:
  - update contributors ([6af26ed](https://github.com/stack-rs/netem-trace/commit/6af26ed026a9cb3c9bc50a9c5479b04f06e0d0e8))

## [v0.4.0](https://github.com/stack-rs/netem-trace/releases/tag/v0.4.0) - 2025-01-24 05:15:54

## [v0.3.6](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.6) - 2024-12-08 16:04:14

## [v0.3.5](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.5) - 2024-12-06 05:17:25

## [v0.3.4](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.4) - 2024-06-30 08:37:28

## [v0.3.3](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.3) - 2024-06-25 12:21:00

Add humanized formats of bandwidth to enable a more straightforward configuration

### Feature

- model:
  - add humanized format for bandwidth ([c81ac1c](https://github.com/stack-rs/netem-trace/commit/c81ac1c4f33e31a32d269feba135ef4f23853343))

## [v0.3.2](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.2) - 2024-03-22 10:42:34

Add `Forever` trait to make transition into endless model easier

### Feature

- model:
  - support transition to endless model ([0aa5a68](https://github.com/stack-rs/netem-trace/commit/0aa5a685ade35e462025ac78cac13a9127514319))

## [v0.3.1](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.1) - 2024-03-17 17:14:06

Add `Send` to trace config and model by @Centaurus99 in [#8](https://github.com/stack-rs/netem-trace/pull/8)

### Feature

- model:
  - add send to trace config and model ([1a23a01](https://github.com/stack-rs/netem-trace/commit/1a23a01aa05adbf4f253bbca087af6d17cdad0b2)) ([#8](https://github.com/stack-rs/netem-trace/pull/8))

## [v0.3.0](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.0) - 2023-11-29 17:25:28

*No description*

### Feature

- trace:
  - add function to load mahimahi trace ([23d443a](https://github.com/stack-rs/netem-trace/commit/23d443a59ab6f97c77e28d56bc47477ca322d06f)) ([#7](https://github.com/stack-rs/netem-trace/pull/7))

- model:
  - support for infinite loops ([8ac7d50](https://github.com/stack-rs/netem-trace/commit/8ac7d50f14373453e0b429b5eaf472372b089a72)) ([#5](https://github.com/stack-rs/netem-trace/pull/5))

### Bug Fixes

- trace:
  - start mahimahi trace from zero ([d75eadd](https://github.com/stack-rs/netem-trace/commit/d75eaddfd78e75e7e7bd0e246e0c460f629e5810)) ([#7](https://github.com/stack-rs/netem-trace/pull/7))

### Documentation

- readme:
  - update contributors ([044932f](https://github.com/stack-rs/netem-trace/commit/044932f6e8ca1e7953a8da7179ff077e1cf5daaa))

## [v0.2.1](https://github.com/stack-rs/netem-trace/releases/tag/v0.2.1) - 2023-11-22 09:52:18

Add delay and loss models.

### Feature

- model:
  - add delay model and loss model ([3e9a6b3](https://github.com/stack-rs/netem-trace/commit/3e9a6b3f0fd08c77d6a7fd01b18d7b61d50a2b2d)) ([#3](https://github.com/stack-rs/netem-trace/pull/3))

## [v0.2.0](https://github.com/stack-rs/netem-trace/releases/tag/v0.2.0) - 2023-07-28 14:24:59

- add a new model `SawtoothBw` whose waveform is sawtooth.
- merge two normalized bandwidth models into one `NormalizedBw` model.
- add feature `human` of human-readable duration serialization
- add feature `full` of enabling all features

BREAKING CHANGE: rename `FixedBw` to `StaticBw`

### Feature

- model:
  - add SawtoothBw model ([27d237d](https://github.com/stack-rs/netem-trace/commit/27d237d40c20838818665907615a97dfdde05018))
  - merge normalized bw models ([5f2a446](https://github.com/stack-rs/netem-trace/commit/5f2a4460243939991ded7c300852effecc52fb4d))
  - rename FixedBw to StaticBw ([ca4d804](https://github.com/stack-rs/netem-trace/commit/ca4d80489f77a2b43a290fc50d26474e2793a6f4))

### Documentation

- readme:
  - update contributors ([28c59da](https://github.com/stack-rs/netem-trace/commit/28c59dac3cef8b928ee0c28e73e55d8a4918b677))

## [v0.1.0](https://github.com/stack-rs/netem-trace/releases/tag/v0.1.0) - 2023-02-12 07:30:18

*No description*

### Feature

- trace:
  - add mahimahi output ([39e56cc](https://github.com/stack-rs/netem-trace/commit/39e56cc9838dd4ccc709d484532ff80d8bc36e12))

- model:
  - add bandwidth models ([7a0650c](https://github.com/stack-rs/netem-trace/commit/7a0650caa5125834b265d4ac257f308f4eeae0e8))

- core:
  - add core traits ([b41856a](https://github.com/stack-rs/netem-trace/commit/b41856a83d8e9d2cb45a0907e55f5ab4987ecde2))

### Documentation

- *:
  - add usage examples ([d9f7d29](https://github.com/stack-rs/netem-trace/commit/d9f7d29ed2f88e2ef5019cbf44e06b08bf6fb905))

\* *This CHANGELOG was automatically generated by [auto-generate-changelog](https://github.com/BobAnkh/auto-generate-changelog)*
