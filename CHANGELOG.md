# CHANGELOG

## [v0.4.0](https://github.com/stack-rs/netem-trace/releases/tag/v0.4.0) - 2025-01-24 05:15:54

## What's Changed
* feat(model): simplify de/ser of TraceBw model


**Full Changelog**: https://github.com/stack-rs/netem-trace/compare/v0.3.6...v0.4.0

### Feature

- model:
  - simplify de/ser of `TraceBw` model ([7446928](https://github.com/stack-rs/netem-trace/commit/7446928c5e570cd510b93ad4606656ecd16e77c4))

### Documentation

- model:
  - update documentation on models ([862f485](https://github.com/stack-rs/netem-trace/commit/862f4852a9a7c703a09304e3ec67b5729fa79368))

- readme:
  - update contributors ([71197e9](https://github.com/stack-rs/netem-trace/commit/71197e94a70696a3105328013933a5cc25d93b0f))

## [v0.3.6](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.6) - 2024-12-08 16:04:14

## What's Changed

* fix(model): resolve deserialization issue with figment by @Lethe10137 in https://github.com/stack-rs/netem-trace/pull/15


**Full Changelog**: https://github.com/stack-rs/netem-trace/compare/v0.3.5...v0.3.6

### Bug Fixes

- model:
  - resolve deserialization issue with figment (#15) ([752f52b](https://github.com/stack-rs/netem-trace/commit/752f52b39169d8e5d4c3e42ec3c5ed3c67a9bcbd)) ([#15](https://github.com/stack-rs/netem-trace/pull/15))

### Documentation

- readme:
  - update contributors ([dd37e9c](https://github.com/stack-rs/netem-trace/commit/dd37e9cdaa842472788c32c06cee5394fbe44b89))

## [v0.3.5](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.5) - 2024-12-06 05:17:25

## What's Changed
* Support TraceBwConfig model to replay any bandwidth trace by @Lethe10137 in https://github.com/stack-rs/netem-trace/pull/13

## New Contributors
* @Lethe10137 made their first contribution in https://github.com/stack-rs/netem-trace/pull/13

**Full Changelog**: https://github.com/stack-rs/netem-trace/compare/v0.3.4...v0.3.5

### Feature

- model:
  - add bandwidth trace replay model (#13) ([1cd2019](https://github.com/stack-rs/netem-trace/commit/1cd20192da9660b1195a5bb0de108de5ba316079)) ([#13](https://github.com/stack-rs/netem-trace/pull/13))

### Documentation

- readme:
  - update contributors ([9b221cf](https://github.com/stack-rs/netem-trace/commit/9b221cf713c3142dbc30af00d86f66b26f70500c))

## [v0.3.4](https://github.com/stack-rs/netem-trace/releases/tag/v0.3.4) - 2024-06-30 08:37:28

## What's Changed

* add packet duplicate models by @un-lock-able in https://github.com/stack-rs/netem-trace/pull/11
* fix typos in comments by @un-lock-able in https://github.com/stack-rs/netem-trace/pull/12

## New Contributors

* @un-lock-able made their first contribution in https://github.com/stack-rs/netem-trace/pull/11

### Feature

- model:
  - add packet duplicate models ([dedcb15](https://github.com/stack-rs/netem-trace/commit/dedcb15b180db3043fbf901d7b31358e0c347032)) ([#11](https://github.com/stack-rs/netem-trace/pull/11))

### Documentation

- readme:
  - correct the examples ([fa5fd48](https://github.com/stack-rs/netem-trace/commit/fa5fd48752c686e05cec3998c8d3385b25c0a4a6))

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
