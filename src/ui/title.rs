//! Pantalla de título: "Continuar" (si hay partida guardada) o "Nueva partida".

use bevy::prelude::*;

use crate::AppState;
use crate::render::camera::MainCamera;
use crate::pokemon::{CHARMANDER, Companion, Creature, PIKACHU, Party, SQUIRTLE, BULBASAUR};
use crate::ui::dialog::{Dialog, speak};
use crate::game::items::Bag;
use crate::world::kanto;
use crate::render::models::Kit;
use crate::world::overworld::{CurrentMap, Facing, PlayerPos, Trail, enter_map};
use crate::game::progress::Progress;
use crate::game::save::{self, LoadError, SaveData};
use crate::ui::{self, ACCENT, bold, bold_in, muted};

const STAGE: Vec3 = Vec3::new(-800.0, 0.0, 0.0);

#[derive(Resource)]
struct Title {
    save: Result<SaveData, LoadError>,
    cursor: usize,
}

impl Title {
    fn options(&self) -> Vec<&'static str> {
        if self.save.is_ok() { vec!["Continuar", "Nueva partida"] } else { vec!["Nueva partida"] }
    }
}

#[derive(Component)]
struct TitleUi;

#[derive(Component)]
struct Spin(f32);

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), enter)
            .add_systems(Update, (title_input, draw, spin, camera).chain().run_if(in_state(AppState::Title)));
    }
}

fn enter(mut commands: Commands, mut kit: Kit) {
    commands.insert_resource(Title { save: save::load(), cursor: 0 });
    let scoped = || DespawnOnExit(AppState::Title);
    let ground = kit.piece(&crate::render::models::part(crate::render::models::Shape::Cylinder, [104, 184, 84], STAGE - Vec3::Y * 0.1, Vec3::new(14.0, 0.2, 14.0)));
    commands.spawn((ground, scoped()));
    for (i, species) in [BULBASAUR, CHARMANDER, SQUIRTLE, PIKACHU].into_iter().enumerate() {
        let x = (i as f32 - 1.5) * 1.35;
        let pos = STAGE + Vec3::new(x, 0.0, -(x.abs() * 0.35));
        kit.spawn_creature(&mut commands, (Transform::from_translation(pos), Spin(i as f32), scoped()), species);
    }
}

fn spin(time: Res<Time>, mut q: Query<(&Spin, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (s, mut transform) in &mut q {
        transform.rotation = Quat::from_rotation_y((t * 0.7 + s.0).sin() * 0.5);
        transform.translation.y = (t * 2.0 + s.0 * 1.7).sin().abs() * 0.06;
    }
}

fn camera(time: Res<Time>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    let t = time.elapsed_secs() * 0.12;
    let eye = STAGE + Vec3::new(t.sin() * 1.2, 1.5, 5.4);
    **camera = Transform::from_translation(eye).looking_at(STAGE + Vec3::new(0.0, 0.9, 0.0), Vec3::Y);
}

#[allow(clippy::too_many_arguments)]
fn title_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut title: ResMut<Title>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let n = title.options().len();
    if input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        title.cursor = (title.cursor + n - 1) % n;
    }
    if input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        title.cursor = (title.cursor + 1) % n;
    }
    if !input.any_just_pressed([KeyCode::Enter, KeyCode::KeyZ, KeyCode::Space]) {
        return;
    }
    match (title.options()[title.cursor], &title.save) {
        ("Continuar", Ok(data)) => apply(&mut commands, data.clone()),
        _ => new_game(&mut commands),
    }
    next_state.set(AppState::Overworld);
}

/// Pone en marcha una partida guardada.
pub fn apply(commands: &mut Commands, data: SaveData) {
    let (map, actors) = enter_map(data.place.map, &data.progress);
    commands.insert_resource(CurrentMap(map));
    commands.insert_resource(actors);
    commands.insert_resource(PlayerPos(data.place.pos));
    commands.insert_resource(Facing(data.facing));
    commands.insert_resource(Trail::at(data.place.pos - data.facing));
    commands.insert_resource(Companion(data.companion));
    commands.insert_resource(Party(data.party));
    commands.insert_resource(data.bag);
    commands.insert_resource(data.progress);
}

fn new_game(commands: &mut Commands) {
    let progress = Progress::default();
    let start = kanto::house_start();
    let (map, actors) = enter_map(kanto::PLAYER_HOUSE, &progress);
    commands.insert_resource(CurrentMap(map));
    commands.insert_resource(actors);
    commands.insert_resource(PlayerPos(start));
    commands.insert_resource(Facing(IVec2::NEG_Y));
    commands.insert_resource(Trail::at(start + IVec2::X));
    commands.insert_resource(Companion(Some(0)));
    commands.insert_resource(Party(vec![Creature::new(CHARMANDER, 5)]));
    commands.insert_resource(Bag::default());
    commands.insert_resource(Dialog::message(vec![
        speak("Profe Roble", "¡Hola! ¡Bienvenido al mundo de los Pokémon!"),
        speak("Profe Roble", "Este mundo está lleno de criaturas llamadas Pokémon. Unos viven con la gente; otros, en la hierba alta, las cuevas y el mar."),
        speak("Profe Roble", "Tu Charmander y tú vais a recorrer Kanto: 8 gimnasios, la Liga Pokémon... ¡y una Pokédex que llenar!"),
        speak("Profe Roble", "Pásate por mi laboratorio, al sur del pueblo. ¡Te estaré esperando!"),
        format!("Controles: flechas para andar (Shift para correr), Z para hablar, Enter para el menú y Tab para tu equipo."),
    ]));
    commands.insert_resource(progress);
}

fn draw(mut commands: Commands, title: Res<Title>, roots: Query<Entity, With<TitleUi>>) {
    if !roots.is_empty() && !title.is_changed() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let root = commands
        .spawn((
            TitleUi,
            DespawnOnExit(AppState::Title),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::axes(px(24), px(36)),
                ..default()
            },
        ))
        .id();
    commands.spawn((
        ChildOf(root),
        ui::panel(Node { padding: UiRect::axes(px(40), px(16)), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() }),
        children![
            (Node { align_items: AlignItems::Center, column_gap: px(16), ..default() }, children![ui::pokeball(48.0), bold_in("Kanto3D", 64.0, Color::srgb(0.86, 0.22, 0.24)), ui::pokeball(48.0)]),
            muted("Edición Kanto", 22.0),
        ],
    ));
    let menu = commands
        .spawn((ChildOf(root), ui::panel(Node { width: px(460), padding: UiRect::all(px(18)), flex_direction: FlexDirection::Column, row_gap: px(8), ..default() })))
        .id();
    for (i, option) in title.options().into_iter().enumerate() {
        let selected = i == title.cursor;
        let row = commands
            .spawn((
                ChildOf(menu),
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::axes(px(14), px(8)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    row_gap: px(4),
                    ..default()
                },
                BackgroundColor(if selected { Color::srgb(0.9, 0.94, 1.0) } else { Color::NONE }),
                BorderColor::all(if selected { ACCENT } else { Color::NONE }),
            ))
            .id();
        commands.spawn((ChildOf(row), bold(option, 26.0)));
        if let ("Continuar", Ok(data)) = (option, &title.save) {
            let p = &data.progress;
            let place = kanto::MAPS[data.place.map].name;
            commands.spawn((
                ChildOf(row),
                muted(format!("{} · {} · {} medallas · {} en la Pokédex · {}", p.name, place, p.badge_count(), p.caught.count(), p.time_text()), 15.0),
            ));
        }
    }
    if let Err(LoadError::Corrupt(why)) = &title.save {
        commands.spawn((ChildOf(menu), muted(format!("La partida guardada no se puede leer: {why}."), 14.0)));
    }
    commands.spawn((ChildOf(menu), muted("Flechas: elegir   ·   Enter: empezar", 14.0)));
}
