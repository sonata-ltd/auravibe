pub mod builder;
mod vars;
pub mod widget;

#[macro_export]
macro_rules! sidebar {
    () => (
        $Sidebar::new()
    );
    ($($x:expr),+ $(,)?) => (
        $Sidebar::with_children([$($Element::from($x)),+])
    );
}
