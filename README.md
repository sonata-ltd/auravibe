# Luminate

[![CI](https://github.com/sonata-ltd/luminate/actions/workflows/ci.yml/badge.svg)](https://github.com/sonata-ltd/luminate/actions/workflows/ci.yml)
[![License: LGPL 2.1 or later](https://img.shields.io/badge/license-LGPL--2.1%2B-blue.svg)](https://github.com/sonata-ltd/luminate/blob/master/LICENSE)

Animation, texture caching, page routing, and a design kit for
[iced](https://iced.rs) 0.14. Each crate is a normal dependency and does not
require an iced fork.

| Crate | | | Purpose |
|---|---|---|---|
| `iced_animate` | [![crates.io](https://img.shields.io/crates/v/iced_animate.svg)](https://crates.io/crates/iced_animate) | [![docs.rs](https://docs.rs/iced_animate/badge.svg)](https://docs.rs/iced_animate) | Tree-external animation engine: keyed springs and eases, resolved inside widgets. |
| `iced_texture_cache` | [![crates.io](https://img.shields.io/crates/v/iced_texture_cache.svg)](https://crates.io/crates/iced_texture_cache) | [![docs.rs](https://docs.rs/iced_texture_cache/badge.svg)](https://docs.rs/iced_texture_cache) | Render-to-texture caching (`Cached`, `Pager`) and the renderer/compositor that make it possible. |
| `iced_page_router` | [![crates.io](https://img.shields.io/crates/v/iced_page_router.svg)](https://crates.io/crates/iced_page_router) | [![docs.rs](https://docs.rs/iced_page_router/badge.svg)](https://docs.rs/iced_page_router) | Type-erased page router with history, lifecycle, snapshots and a shared-state registry. |
| `iced_luminate` | [![crates.io](https://img.shields.io/crates/v/iced_luminate.svg)](https://crates.io/crates/iced_luminate) | [![docs.rs](https://docs.rs/iced_luminate/badge.svg)](https://docs.rs/iced_luminate) | The Luminate design kit: theme, descriptors and widgets built on the three crates above. |

## Getting started

Choose the crate that matches your needs:

### Animate stock widgets

Use springs and eases without sending a message for every frame:

```toml
[dependencies]
iced = "0.14"
iced_animate = "0.1"
```

### Cache and animate widget subtrees

This crate provides the iced renderer, so views return
`iced_texture_cache::Element`. It re-exports `iced_animate` as
`iced_texture_cache::iced_animate`:

```toml
[dependencies]
iced = "0.14"
iced_texture_cache = "0.1"
```

### Add pages with history and lifecycle

This crate does not depend on the animation or texture crates:

```toml
[dependencies]
iced = "0.14"
iced_page_router = "0.1"
```

### Use the whole kit

One dependency re-exports `iced` and the other three crates:

```toml
[dependencies]
iced_luminate = "0.1"
```

All four crates are released together with the same version.

## Examples

Run with `cargo run -p <crate> --example <name>`.

| Crate | Example | Shows | Environment |
|---|---|---|---|
| `iced_animate` | `tiers` | paint and layout tiers, keys, sets, springs vs eases, enter/exit | `ANIM_AUTOPLAY=1` flips every 1.5 s |
| `iced_texture_cache` | `compositor` | translate/scale/opacity of cached textures, auto-invalidate, nesting | `ANIM_AUTOPLAY=1` |
| `iced_texture_cache` | `pager` | `Pager` sliding between pages of different heights | `ANIM_AUTOPLAY=1` |
| `iced_texture_cache` | `cache_benchmark` | a heavy scene cached vs direct, `PixelSnap`, supersampling, the z-order rule (all knobs in-UI) | `BENCH_LOG=1` prints statistics to stderr |
| `iced_luminate` | `overview` | all five descriptors (button hierarchies, an input with its error bubble, a collapsible sidebar, a card, a standalone pager), the theme, the motion tiers, a router with `Suspend` and `Drop` pages, a nested router | none |

`ICED_BACKEND=wgpu|tiny-skia` forces a backend for any of them;
`RUST_LOG=info` logs the adapter and surface format (the examples install
`env_logger`; an application has to install a `log` backend itself).

## Development

```sh
nix develop      # dev shell with the wgpu/winit system libraries
./ci.sh          # fmt, clippy -D warnings, tests, docs, per-crate feature checks, cargo-deny, publish dry-run
```

`./ci.sh` mirrors the `stable`, Linux `per-crate`, `deny`, and
`package` jobs of the GitHub workflow, including the multi-package
`cargo publish --dry-run`. The workflow also runs `per-crate` on
Linux, macOS and Windows, and `cargo-semver-checks`
(`semver`) jobs, and `gpu`, which repeats the tests on wgpu over Mesa's
software Vulkan driver.

See [CONTRIBUTING.md](https://github.com/sonata-ltd/luminate/blob/master/CONTRIBUTING.md)
and [RELEASING.md](https://github.com/sonata-ltd/luminate/blob/master/RELEASING.md).
Design notes live in
[docs/design](https://github.com/sonata-ltd/luminate/blob/master/docs/design).
Measurements are in
[texture_cache/BENCHMARKS.md](https://github.com/sonata-ltd/luminate/blob/master/texture_cache/BENCHMARKS.md);
changes in [CHANGELOG.md](https://github.com/sonata-ltd/luminate/blob/master/CHANGELOG.md).

## License

LGPL 2.1 or later. See [LICENSE](https://github.com/sonata-ltd/luminate/blob/master/LICENSE). `iced_luminate` bundles the Inter font under the
SIL Open Font License 1.1 (`OFL.txt` next to the font files).
