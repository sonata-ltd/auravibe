# Measurements

`examples/cache_benchmark.rs` reports the interval between `window::frames()`
callbacks over the last 120 frames. The grid size, the cached toggle, the
`PixelSnap` mode, the `FilterQuality` tier and supersample-in-motion are
controls in the window;
`BENCH_LOG=1` prints the statistics line to stderr every 120 frames.

Reproduce:

```sh
BENCH_LOG=1 cargo run --release -p iced_texture_cache --example cache_benchmark
```

Move the grid slider to the size you want, flip "cached", and read the lines
on stderr (`cached=true 60x40 cells · frame avg ... · p99 ... · records: ...`).

## Numbers

Taken with the earlier `demo` example of this crate (same scene, same
statistics, grid set through `DEMO_COLS`/`DEMO_ROWS`) before the 0.1.0 API
rewrite; the rendering path is unchanged. Release build, Intel Iris Xe /
NVIDIA RTX 3050 laptop, Wayland, on a 3440×1440 display running at 100 Hz
(hence the 10 ms floor):

| grid (cells) | cached: avg / p99 | uncached: avg / p99 |
|---|---|---|
| 30×20 (600) | 10.0 / 14.0-14.2 ms | 10.0 / 13.7-14.0 ms |
| 60×40 (2,400) | 10.0-10.1 / 13.9-14.4 ms | 10.0-10.3 / 14.0-15.5 ms |
| 120×80 (9,600) | 12.0-12.6 / 19.8-21.8 ms | 12.9-14.8 / 18.9-24.2 ms |

The display limits the frame interval, so both modes stay near 10 ms until
the scene is heavy enough to miss frames.
The benchmark also rebuilds and lays out the whole scene every frame (a
per-frame `Tick` message); `Cached` removes only the *draw* of the subtree,
not the rebuild or layout. The difference above only measures drawing. The
record counters give a clearer result: cached content is rasterized once
(twice on the first frames, when the window's scale factor arrives), then
composited as a single textured quad per frame.
