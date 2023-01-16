# Changelog

All notable changes to this project will be documented in this file.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Additionally we have an `Internal` section for changes that are of interest to developers.

Dates in this file are formattes as `YYYY-MM-DD`.

## [`0.2.0`] - 2023-01-16

### Added

- The `README` now contains an example of the generated assembly for the `call` impls.
- Added a link to the `#[union_fn]` docs for an interpreter-like usage.

### Changed

- Improve `call` performance of the non-call optimized generated `<enum>` type. (https://github.com/Robbepop/union-fn/pull/2)
- Improve performance of the generated call-optimized type for calls. (https://github.com/Robbepop/union-fn/pull/3)
    - Internally we now propagate the `union` arguments via reference which allows
      the Rust/LLVM compiler to optimize the `call` implementation into a tail call
      boosting performance.
- Moved associated type `Opt` and `Impls` and `Delegator` into the `IntoOpt` trait.
- Improved docs and examples.

### Internal

- Added benchmarks to compare the `call` performance between the generated `<enum>`
  and `<opt>` types for interpreter use cases. (https://github.com/Robbepop/union-fn/pull/4)

## [`0.1.0`] - 2023-01-11

Initial release of the crate and its `#[union_fn]` proc. macro.
