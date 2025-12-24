# Rust UIKit for Iced

A flexible, runtime-switchable UI kit for [`iced`](https://github.com/iced-rs/iced) that allows you to define buttons, inputs, and other widgets in a reusable way.  

Unlike typical iced widgets, this library allows you to:

- Directly emit application messages (`AppMessage`) from UIKit widgets.
- Switch themes (e.g., Adwaita, Breeze, Custom) at runtime without incurring significant overhead.
- Cache elements for zero-overhead rendering in hot paths.
- Keep a clean, React/SolidJS-like API for building UIs declaratively.

---

## Features

- **Generic `Kit`**: UIKit can be generic over your application `Message`.
- **Runtime theme switching**: Choose different themes at runtime using a simple strategy.
- **Message passthrough**: Send application-specific messages directly from UIKit buttons and widgets.
- **Zero-cost caching**: Use iced `Component`s to cache widget state and reduce layout/text shaping overhead.
- **Extensible**: Easily implement new themes without changing your application logic.

---

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
iced_auravibe = { git = "https://github.com/sonata-ltd/auravibe", branch = "master" }
