# Contributing

## Before every commit

Run `./ci.sh`. It enters `nix develop` and runs formatting, Clippy with
warnings denied, tests, README doctests, rustdoc, feature checks, and
`cargo deny`. CI also checks other operating systems, and
`cargo-semver-checks` on tags. Never force-push shared branches.

## Commits

Keep each commit to one logical change. Use a conventional commit subject
with the crate as its scope, such as `feat(animate): ...` or
`fix(texture_cache): ...`. Explain what changed and why in the body.

## Versioning

All four crates are released together with the same version
(`version.workspace = true`). `iced_texture_cache` re-exports `iced_animate`,
and `iced_luminate` re-exports all three, so their versions must match.
internal dependencies are declared as `version = "0.1.0", path = "…"` in
`[workspace.dependencies]` and the lockstep release keeps them in sync.
`luminate_examples_support` is never published. Before 1.0, breaking changes are
acceptable and are listed in `CHANGELOG.md` under **Changed**/**Removed**.
See `RELEASING.md` for the publish order.

## Code

- New behaviour comes with a test; a bug fix comes with the test that would
  have caught it. Unit tests live inline in the module; black-box tests in
  `tests/`.
- Every public item and every module is documented (`missing_docs` is
  denied). Crate roots include their README; every README snippet is a
  doctest, marked `no_run` only when it opens a window.
- Library code uses `iced_core`/`iced_runtime`/`iced_graphics`; only
  `iced_luminate` and the examples use the `iced` facade. Everything builds on
  Linux, macOS and Windows; `x11`/`wayland` are pass-throughs only.
- `with_*` names a constructor; builder setters are unprefixed; no `is_*`
  setters. Constructors, builders and pure getters are `#[must_use]`.
- Prose is British ("colour"), identifiers follow iced ("Color").

## Errors and logging

- Programming errors (adding a page twice, a message of the wrong type, a
  non-finite animation target): `debug_assert!` plus `log::error!`, then a
  documented degradation in release builds.
- Environmental problems (no GPU adapter, an oversize texture): `log::warn!`
  and degrade (fall back, draw inline).
- Broken invariants: `unreachable!`/`expect` with a message.
- Log records carry no hand-written crate prefix; `log` records the module
  path.

## Documentation layout

- `docs/design/` holds short, current, path-accurate design notes. Update the
  matching note when you move or rename a file.
- Working documents for in-progress rewrites are not checked in; only the
  design notes above and this file describe the project.
