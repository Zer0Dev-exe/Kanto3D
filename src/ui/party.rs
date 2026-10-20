//! HUD del equipo (Pokéballs + Pokémon) y menú de equipo (Tab): elegir quién te acompaña
//! por el mapa y quién sale primero en los combates.

use bevy::prelude::*;

use crate::AppState;
use crate::pokemon::{Companion, Creature, Party};
use crate::game::items::Bag;
use crate::game::progress::Progress;
use crate::ui::dialog::Dialog;
use crate::render::portraits::Portraits;
use crate::ui::{self, ACCENT, FRAME, TRACK, XP, bar, bold, hp_color, muted, pill, pokeball, text};

/// Menú de equipo abierto; `cursor` es el Pokémon seleccionado.
#[derive(Resource)]
pub struct PartyMenu {
    cursor: usize,
}

impl PartyMenu {
    pub fn at(cursor: usize) -> Self {
        Self { cursor }
    }
}

#[derive(Component)]
struct HudBody;

#[derive(Component)]
struct MenuRoot;

/// Para que el resto del overworld se ejecute después y no reciba las mismas teclas.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartySet;

pub struct PartyPlugin;

impl Plugin for PartyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Companion(Some(0)))
            .add_systems(OnEnter(AppState::Overworld), spawn_hud)
            .add_systems(OnExit(AppState::Overworld), |mut commands: Commands| commands.remove_resource::<PartyMenu>())
            .add_systems(
                Update,
                (menu_input.run_if(resource_exists::<PartyMenu>), open_menu, refresh_hud, refresh_menu)
                    .chain()
                    .in_set(PartySet)
                    .run_if(in_state(AppState::Overworld)),
            );
    }
}

// ---------------------------------------------------------------- HUD

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(AppState::Overworld),
        ui::panel(Node {
            position_type: PositionType::Absolute,
            left: px(16),
            top: px(16),
            width: px(280),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        }),
        children![(HudBody, Node { flex_direction: FlexDirection::Column, row_gap: px(10), ..default() })],
    ));
}

fn refresh_hud(
    mut commands: Commands,
    party: Res<Party>,
    bag: Res<Bag>,
    progress: Res<Progress>,
    companion: Res<Companion>,
    portraits: Option<Res<Portraits>>,
    body: Single<(Entity, Option<&Children>), With<HudBody>>,
) {
    let (body, children) = *body;
    let empty = children.is_none_or(|c| c.is_empty());
    if !(empty || party.is_changed() || bag.is_changed() || companion.is_changed() || progress.is_changed()) {
        return;
    }
    commands.entity(body).despawn_related::<Children>();

    commands.spawn((
        ChildOf(body),
        Node { align_items: AlignItems::Center, column_gap: px(10), ..default() },
        children![
            pokeball(26.0),
            (bold(format!("× {}", bag.balls()), 19.0), Node { flex_grow: 1.0, ..default() }),
            bold(format!("{} ₽", progress.money), 19.0),
        ],
    ));
    commands.spawn((ChildOf(body), Node { height: px(2), ..default() }, BackgroundColor(TRACK)));

    for (i, c) in party.0.iter().enumerate() {
        let line = commands
            .spawn((ChildOf(body), Node { align_items: AlignItems::Center, column_gap: px(10), ..default() }))
            .id();
        spawn_icon(&mut commands, line, portraits.as_deref(), c, 40.0);
        let row = commands
            .spawn((ChildOf(line), Node { flex_direction: FlexDirection::Column, row_gap: px(4), flex_grow: 1.0, ..default() }))
            .id();
        let top = commands
            .spawn((ChildOf(row), Node { align_items: AlignItems::Center, column_gap: px(6), ..default() }))
            .id();
        commands.spawn((ChildOf(top), bold(c.name(), 17.0)));
        commands.spawn((ChildOf(top), muted(format!("Nv.{}", c.level), 15.0), Node { flex_grow: 1.0, ..default() }));
        if companion.0 == Some(i) {
            commands.spawn((ChildOf(top), pill("Te sigue", ACCENT)));
        }
        let hp = c.hp.max(0) as f32 / c.max_hp() as f32;
        commands.spawn((ChildOf(row), bar((), hp, 7.0, hp_color(hp))));
    }

    commands.spawn((ChildOf(body), muted("Enter: menú   ·   Tab: equipo", 14.0)));
}

/// Retrato en vivo del Pokémon sobre un fondo del color de su tipo.
fn spawn_icon(commands: &mut Commands, parent: Entity, portraits: Option<&Portraits>, c: &Creature, size: f32) {
    let frame = commands
        .spawn((
            ChildOf(parent),
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(size * 0.28)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(c.spec().ty.color().mix(&Color::WHITE, 0.72)),
            BorderColor::all(c.spec().ty.color()),
        ))
        .id();
    if let Some(portraits) = portraits {
        commands.spawn((
            ChildOf(frame),
            ImageNode::new(portraits.0[c.species].clone()),
            Node { width: percent(100), height: percent(100), ..default() },
        ));
    }
}

// ---------------------------------------------------------------- menú de equipo

fn open_menu(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    dialog: Option<Res<Dialog>>,
    menu: Option<Res<PartyMenu>>,
    game_menu: Option<Res<crate::ui::menu::GameMenu>>,
    shop: Option<Res<crate::ui::menu::Shop>>,
) {
    if dialog.is_none() && menu.is_none() && game_menu.is_none() && shop.is_none() && input.clear_just_pressed(KeyCode::Tab) {
        commands.insert_resource(PartyMenu { cursor: 0 });
    }
}

fn menu_input(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut menu: ResMut<PartyMenu>,
    mut party: ResMut<Party>,
    mut companion: ResMut<Companion>,
) {
    let n = party.0.len();
    if input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        menu.cursor = (menu.cursor + 1) % n;
    }
    if input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        menu.cursor = (menu.cursor + n - 1) % n;
    }
    if input.any_just_pressed([KeyCode::KeyZ, KeyCode::Space, KeyCode::Enter]) {
        companion.0 = if companion.0 == Some(menu.cursor) { None } else { Some(menu.cursor) };
    }
    if input.just_pressed(KeyCode::KeyX) && menu.cursor != 0 {
        let i = menu.cursor;
        party.0.swap(0, i);
        // El acompañante sigue siendo el mismo Pokémon aunque cambie de puesto
        companion.0 = companion.0.map(|c| if c == i { 0 } else if c == 0 { i } else { c });
        menu.cursor = 0;
    }
    if input.any_just_pressed([KeyCode::Tab, KeyCode::Escape]) {
        commands.remove_resource::<PartyMenu>();
    }

    // Las teclas del menú no deben mover al jugador ni abrir diálogos
    for key in [
        KeyCode::ArrowDown, KeyCode::ArrowUp, KeyCode::KeyS, KeyCode::KeyW,
        KeyCode::KeyZ, KeyCode::Space, KeyCode::Enter, KeyCode::KeyX, KeyCode::Tab, KeyCode::Escape,
    ] {
        input.clear_just_pressed(key);
    }
}

fn refresh_menu(
    mut commands: Commands,
    menu: Option<Res<PartyMenu>>,
    party: Res<Party>,
    companion: Res<Companion>,
    portraits: Option<Res<Portraits>>,
    roots: Query<Entity, With<MenuRoot>>,
) {
    let Some(menu) = menu else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    };
    if !roots.is_empty() && !(menu.is_changed() || party.is_changed() || companion.is_changed()) {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let root = commands
        .spawn((
            MenuRoot,
            DespawnOnExit(AppState::Overworld),
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            GlobalZIndex(10),
        ))
        .id();
    let window = commands
        .spawn((
            ChildOf(root),
            ui::panel(Node { width: px(660), padding: UiRect::all(px(22)), flex_direction: FlexDirection::Column, row_gap: px(12), ..default() }),
        ))
        .id();
    commands.spawn((
        ChildOf(window),
        Node { align_items: AlignItems::Center, column_gap: px(12), margin: UiRect::bottom(px(4)), ..default() },
        children![pokeball(30.0), bold("Tu equipo", 28.0)],
    ));

    for (i, c) in party.0.iter().enumerate() {
        let selected = i == menu.cursor;
        let card_row = commands
            .spawn((
                ChildOf(window),
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(16),
                    padding: UiRect::axes(px(16), px(10)),
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(if selected { Color::srgb(0.9, 0.94, 1.0) } else { Color::WHITE }),
                BorderColor::all(if selected { ACCENT } else { TRACK }),
            ))
            .id();
        spawn_icon(&mut commands, card_row, portraits.as_deref(), c, 84.0);
        let card = commands
            .spawn((ChildOf(card_row), Node { flex_direction: FlexDirection::Column, row_gap: px(6), flex_grow: 1.0, ..default() }))
            .id();
        let top = commands
            .spawn((ChildOf(card), Node { align_items: AlignItems::Center, column_gap: px(8), ..default() }))
            .id();
        commands.spawn((ChildOf(top), bold(c.name(), 22.0)));
        commands.spawn((ChildOf(top), muted(format!("Nv.{}", c.level), 18.0)));
        commands.spawn((ChildOf(top), pill(c.spec().ty.name().to_uppercase(), c.spec().ty.color())));
        commands.spawn((ChildOf(top), Node { flex_grow: 1.0, ..default() }));
        if i == 0 {
            commands.spawn((ChildOf(top), pill("Sale primero", FRAME)));
        }
        if companion.0 == Some(i) {
            commands.spawn((ChildOf(top), pill("Te sigue", ACCENT)));
        }

        let hp = c.hp.max(0) as f32 / c.max_hp() as f32;
        let stats = commands
            .spawn((ChildOf(card), Node { align_items: AlignItems::Center, column_gap: px(12), ..default() }))
            .id();
        commands.spawn((ChildOf(stats), Node { flex_grow: 1.0, ..default() }, children![bar((), hp, 10.0, hp_color(hp))]));
        commands.spawn((ChildOf(stats), text(format!("PS {}/{}", c.hp.max(0), c.max_hp()), 16.0), Node { width: px(96), ..default() }));
        let xp = c.xp as f32 / c.xp_to_next() as f32;
        commands.spawn((ChildOf(card), bar((), xp, 5.0, XP)));
    }

    commands.spawn((
        ChildOf(window),
        muted("Flechas: elegir   ·   Z: que te siga   ·   X: que salga primero   ·   Tab: cerrar", 15.0),
        Node { margin: UiRect::top(px(4)), ..default() },
    ));
}
