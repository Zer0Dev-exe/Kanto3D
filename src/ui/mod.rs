//! Piezas de interfaz compartidas: paneles, textos, barras de vida e icono de Pokéball.
//! Todo se dibuja con nodos de UI (sin imágenes ni emojis) para que siempre se vea igual.

pub mod dialog;
pub mod menu;
pub mod party;
pub mod title;

use bevy::prelude::*;

pub const INK: Color = Color::srgb(0.12, 0.12, 0.16);
pub const MUTED: Color = Color::srgb(0.45, 0.46, 0.52);
pub const PANEL: Color = Color::srgb(0.98, 0.98, 0.96);
pub const FRAME: Color = Color::srgb(0.17, 0.18, 0.24);
pub const ACCENT: Color = Color::srgb(0.2, 0.45, 0.9);
pub const TRACK: Color = Color::srgb(0.82, 0.83, 0.86);
pub const XP: Color = Color::srgb(0.3, 0.62, 0.98);

pub fn text(value: impl Into<String>, size: f32) -> impl Bundle {
    (Text::new(value), TextFont { font_size: FontSize::Px(size), ..default() }, TextColor(INK))
}

/// Texto secundario, en gris.
pub fn muted(value: impl Into<String>, size: f32) -> impl Bundle {
    (Text::new(value), TextFont { font_size: FontSize::Px(size), ..default() }, TextColor(MUTED))
}

pub fn bold(value: impl Into<String>, size: f32) -> impl Bundle {
    bold_in(value, size, INK)
}

pub fn bold_in(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont { font_size: FontSize::Px(size), weight: FontWeight::BOLD, ..default() },
        TextColor(color),
    )
}

/// Panel blanco con borde oscuro, esquinas redondeadas y sombra.
pub fn panel(node: Node) -> impl Bundle {
    (
        Node { border: UiRect::all(px(3)), border_radius: BorderRadius::all(px(14)), ..node },
        BackgroundColor(PANEL),
        BorderColor::all(FRAME),
        BoxShadow::new(Color::srgba(0.0, 0.0, 0.0, 0.3), px(0), px(4), px(0), px(10)),
    )
}

/// Etiqueta de color (tipo, "Te sigue"...).
pub fn pill(value: impl Into<String>, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont { font_size: FontSize::Px(14.0), weight: FontWeight::BOLD, ..default() },
        TextColor(Color::WHITE),
        Node { padding: UiRect::axes(px(8), px(2)), border_radius: BorderRadius::all(px(8)), ..default() },
        BackgroundColor(color),
    )
}

pub fn hp_color(fraction: f32) -> Color {
    if fraction > 0.5 {
        Color::srgb(0.3, 0.8, 0.35)
    } else if fraction > 0.2 {
        Color::srgb(0.96, 0.75, 0.15)
    } else {
        Color::srgb(0.92, 0.27, 0.22)
    }
}

/// Barra con fondo gris; el relleno lleva `fill` para poder actualizarlo.
pub fn bar(fill: impl Bundle, fraction: f32, height: f32, color: Color) -> impl Bundle {
    (
        Node { width: percent(100), height: px(height), border_radius: BorderRadius::all(px(height / 2.0)), overflow: Overflow::clip(), ..default() },
        BackgroundColor(TRACK),
        children![(
            fill,
            Node { width: percent(fraction.clamp(0.0, 1.0) * 100.0), height: percent(100), ..default() },
            BackgroundColor(color),
        )],
    )
}

/// Icono de Pokéball de `size` píxeles.
pub fn pokeball(size: f32) -> impl Bundle {
    let band = (size * 0.12).max(2.0);
    let button = size * 0.38;
    (
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::WHITE),
        BorderColor::all(FRAME),
        children![
            // El recorte del círculo no respeta las esquinas redondeadas: la mitad roja lleva las suyas
            (
                Node { width: percent(100), height: percent(50), border_radius: BorderRadius::top(px(size / 2.0)), ..default() },
                BackgroundColor(Color::srgb(0.9, 0.18, 0.2)),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px((size - 4.0) / 2.0 - band / 2.0),
                    height: px(band),
                    ..default()
                },
                BackgroundColor(FRAME),
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px((size - 4.0 - button) / 2.0),
                    top: px((size - 4.0 - button) / 2.0),
                    width: px(button),
                    height: px(button),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderColor::all(FRAME),
            ),
        ],
    )
}

/// Icono de un objeto: Poké Balls con su color, medicinas como frasco, el resto como caja.
pub fn item_icon(item: &crate::game::items::Item, size: f32) -> impl Bundle {
    use crate::game::items::Pocket;
    let [r, g, b] = item.color;
    let color = Color::srgb_u8(r, g, b);
    let (radius, top, band) = match item.pocket() {
        Pocket::Balls => (percent(50), percent(50), Color::WHITE),
        Pocket::Medicine => (px(size * 0.25), percent(30), Color::srgb(0.95, 0.95, 0.97)),
        Pocket::Field => (px(size * 0.2), percent(35), color.mix(&Color::WHITE, 0.5)),
    };
    (
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(radius),
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(band),
        BorderColor::all(FRAME),
        children![(
            Node { width: percent(100), height: top, border_radius: BorderRadius::top(px(size / 2.0)), ..default() },
            BackgroundColor(color),
        )],
    )
}

/// Medalla de gimnasio: un círculo de su color (gris y hueco si aún no la tienes).
pub fn badge(color: [u8; 3], earned: bool, size: f32) -> impl Bundle {
    let [r, g, b] = color;
    let c = Color::srgb_u8(r, g, b);
    (
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.0,
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::all(percent(50)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(if earned { c } else { Color::srgb(0.9, 0.9, 0.92) }),
        BorderColor::all(if earned { c.mix(&Color::BLACK, 0.35) } else { TRACK }),
        children![(
            Node { width: px(size * 0.32), height: px(size * 0.32), border_radius: BorderRadius::all(percent(50)), ..default() },
            BackgroundColor(if earned { Color::srgba(1.0, 1.0, 1.0, 0.7) } else { Color::NONE }),
        )],
    )
}
