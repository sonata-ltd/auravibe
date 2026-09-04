# `iced_page_router`

A type-erased page router for [iced](https://iced.rs) 0.14: pages with their
own message types, back/forward history, a lifecycle per page (drop with a
snapshot, suspend, or stay resident), and a type-keyed `Registry` of shared
values. It has no opinion about widgets or themes and depends on `iced_core`,
`iced_runtime` and `log` only.

## Minimal example

```rust
use iced::widget::{button, column, text};
use iced::Element;
use iced_page_router::{Action, Key, Lifecycle, Page, Registry, Router, Shared};

/// Data shared with each page. This example has none.
struct Context;

/// A registry key: the type names the slot, `Value` is what it holds.
struct Count;

impl Key for Count {
    type Value = u32;
}

#[derive(Debug, Clone)]
enum CounterMessage {
    Increment,
}

struct Counter {
    count: Shared<u32>,
}

impl Page for Counter {
    type Message = CounterMessage;
    type NavigationOptions = ();
    type Context = Context;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    // Keep the instance alive while another page is shown.
    const LIFECYCLE: Lifecycle = Lifecycle::Suspend;

    fn new(_: &Context, registry: &Registry) -> Self {
        Self {
            count: registry.get_or_insert_with::<Count>(|| 0),
        }
    }

    fn update(&mut self, message: CounterMessage) -> Action<CounterMessage> {
        match message {
            CounterMessage::Increment => *self.count.write() += 1,
        }
        Action::none()
    }

    fn view(&self) -> Element<'_, CounterMessage> {
        column![
            text(self.count.get()),
            button("+").on_press(CounterMessage::Increment),
        ]
        .into()
    }
}

let mut router: Router<Context, iced::Theme, iced::Renderer> =
    Router::new(Registry::new(), Context);
router.add::<Counter>("Counter");
router.navigate::<Counter>().unwrap();
assert_eq!(router.current(), Some(0));

// Messages come back through the host application's message type
// (`Message::Route(RouteMessage)` → `router.update(route_message)`); the typed
// constructor builds one for a page by type.
let message = router.message::<Counter>(CounterMessage::Increment).unwrap();
let _task = router.update(message);
assert_eq!(router.page::<Counter>().map(|page| page.count.get()), Some(1));
```

Add to `Cargo.toml`:

```toml
[dependencies]
iced = "0.14"
iced_page_router = "0.1"
```

Wire it into an application by mapping `router.view()`, `router.subscription()`
and `router.update(..)` through one variant of the application's message.

## Concepts

### Pages and actions

A `Page` has its own `Message`, `Context` (handed to `new`), `Theme` and
`Renderer`. `update` returns an `Action`: `Action::none()`, `Action::task(t)`,
`Action::navigate::<Other>()`, `navigate_with::<Other>(options)`, `back()`,
`forward()`, `replace::<Other>()`; a `Task` converts into an action, and
`and_task`/`and_navigate` combine them. `on_enter` runs every time a page
becomes current; `on_navigate(options)` only when a navigation carried
`NavigationOptions`, even if the page was already current.

### Lifecycle

`Page::LIFECYCLE` is `Lifecycle::Drop` by default: leaving the page drops the
instance; `into_snapshot(self)` may hand the router a boxed value that the next
instance receives through `restore`. `Suspend` keeps the instance and calls
`on_suspend`/`on_resume`. `Resident` builds the page when it is added and never
drops it, so its `background_subscription` runs for the whole application.

A `Task` returned while leaving a `Drop` page is delivered only if that page
still exists when the task completes. Use `Suspend`, or start the work from
the target page's `on_navigate`.

### Router

Create a router with `Router::new(registry, context)`, then add pages with
`add::<P>(name)`. A duplicate type is a programming error. Debug builds panic;
release builds log and ignore it. `navigate::<P>()`,
`navigate_with::<P>(options)`, `navigate_index(i)`,
`replace::<P>()` return the new index or a `NavigationError`; `back()` and
`forward()` walk the history (bounded by `history_len`, default 50;
consecutive duplicates collapse; any navigation truncates the forward part).
`pages()` yields `PageInfo { index, name, is_current, lifecycle }` for menus;
`page::<P>()`/`page_mut::<P>()` reach a live instance. `mouse_navigation(true)`
maps the mouse back/forward buttons to history unless a page captured the
event; a host can also send `RouteMessage::Navigate(Navigation::index(i))`.

### Registry

`Registry` is a type-keyed map of `Shared<T>` values (`Key` trait):
`insert`, `get`, `get_or_insert_with`. `Shared<T>` offers `read`/`write`
guards, `get`/`set`, `with`/`update` closures and `ptr_eq`. Clone the registry
into a nested router to share state across levels, and turn the nested
router's `mouse_navigation` off so one press does not move both.

## Feature flags

None.

## Limitations

* One page instance per type: a page type can be added once per router; one
  router shows one page (no route tree, no URL parsing).
* Messages for a page that is not live (dropped, or from an older generation)
  are logged and discarded, never delivered late.
* Mouse back/forward is application-wide: every window's presses reach every
  router that has it enabled.

## Related crates

* [`iced_luminate`](https://crates.io/crates/iced_luminate) uses this router
  and re-exports it as `iced_luminate::router`.
* [`iced_animate`](https://crates.io/crates/iced_animate),
  [`iced_texture_cache`](https://crates.io/crates/iced_texture_cache) are
  independent. Combine them as needed.

Changes are listed in the workspace
[CHANGELOG](https://github.com/sonata-ltd/luminate/blob/master/CHANGELOG.md).
