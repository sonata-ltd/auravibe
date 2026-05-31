use iced::{
    Background, Border, Color, Rectangle,
    advanced::{layout, renderer::Quad},
};

use super::BorderLayer;

// Draws all `BorderSide::Outer` layers, ordered largest -> smallest
pub fn outer_borders<Renderer>(
    renderer: &mut Renderer,
    layers: &[BorderLayer],
    content_bounds: Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let mut rects: Vec<(BorderLayer, Rectangle)> = Vec::new();
    let mut cumulative = 0.0;

    for layer in layers.iter().filter(|l| l.side == super::BorderSide::Outer) {
        cumulative += layer.offset;
        let expand = cumulative + layer.width;

        rects.push((
            *layer,
            Rectangle {
                x: content_bounds.x - expand,
                y: content_bounds.y - expand,
                width: content_bounds.width + 2.0 * expand,
                height: content_bounds.height + 2.0 * expand,
            },
        ));

        cumulative += layer.width;
    }

    // Largest rect first so inner ones are visible on top
    rects.sort_by(|a, b| {
        let area = |r: &Rectangle| r.width * r.height;
        area(&b.1).partial_cmp(&area(&a.1)).unwrap()
    });

    for (layer, rect) in &rects {
        fill_border(renderer, *rect, layer.radius, layer.width, layer.color);
    }
}

pub fn background_full<Renderer>(
    renderer: &mut Renderer,
    appearance: &super::Appearance,
    content_bounds: Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let Some(bg) = appearance.background else {
        return;
    };

    let outer_layers: Vec<_> = appearance
        .layers
        .iter()
        .filter(|l| l.side == super::BorderSide::Outer)
        .collect();

    let (bg_bounds, radius) = if let Some(innermost) = outer_layers.last() {
        let cumulative: f32 = outer_layers
            .iter()
            .take(outer_layers.len().saturating_sub(1))
            .map(|l| l.width + l.offset)
            .sum::<f32>()
            + innermost.offset;

        let expand = cumulative + innermost.width;
        let rect = Rectangle {
            x: content_bounds.x - expand,
            y: content_bounds.y - expand,
            width: content_bounds.width + 2.0 * expand,
            height: content_bounds.height + 2.0 * expand,
        };
        let inset_rect = Rectangle {
            x: rect.x + innermost.width,
            y: rect.y + innermost.width,
            width: rect.width - 2.0 * innermost.width,
            height: rect.height - 2.0 * innermost.width,
        };
        let radius = (innermost.radius - innermost.width).max(0.0);
        (inset_rect, radius)
    } else {
        (content_bounds, 0.0)
    };

    renderer.fill_quad(
        Quad {
            bounds: bg_bounds,
            border: Border {
                radius: radius.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Default::default(),
            snap: false,
        },
        bg,
    );
}

pub fn content<'a, Message, Theme, Renderer>(
    widget: &super::MultiBorder<'a, Message, Theme, Renderer>,
    tree: &iced::advanced::widget::Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    style: &iced::advanced::renderer::Style,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    viewport: &Rectangle,
) where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    if let Some(child_layout) = layout.children().next() {
        widget.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            child_layout,
            cursor,
            viewport,
        );
    }
}

pub fn inner_borders<Renderer>(
    renderer: &mut Renderer,
    layers: &[BorderLayer],
    content_bounds: Rectangle,
) where
    Renderer: iced::advanced::Renderer,
{
    let mut cumulative = 0.0;

    for layer in layers.iter().filter(|l| l.side == super::BorderSide::Inner) {
        cumulative += layer.offset;

        let inner_bounds = Rectangle {
            x: content_bounds.x + cumulative,
            y: content_bounds.y + cumulative,
            width: (content_bounds.width - 2.0 * cumulative).max(0.0),
            height: (content_bounds.height - 2.0 * cumulative).max(0.0),
        };

        fill_border(
            renderer,
            inner_bounds,
            layer.radius,
            layer.width,
            layer.color,
        );

        cumulative += layer.width;
    }
}

fn fill_border<Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: f32,
    width: f32,
    color: Color,
) where
    Renderer: iced::advanced::Renderer,
{
    renderer.fill_quad(
        Quad {
            bounds,
            border: Border {
                radius: radius.into(),
                width,
                color,
            },
            shadow: Default::default(),
            snap: false,
        },
        Background::Color(Color::TRANSPARENT),
    );
}
