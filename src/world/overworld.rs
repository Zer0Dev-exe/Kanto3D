//! El mundo: construir el mapa en 3D, andar (con las piernas), hablar con la gente, entrar
//! en edificios, cambiar de mapa, los entrenadores que te ven y los Pokémon de la hierba alta.

use bevy::prelude::*;
use rand::Rng;

use crate::AppState;
use crate::battle::PendingBattle;
use crate::render::camera::{CameraRig, MainCamera};
use crate::pokemon::{Companion, Party, random_wild};
use crate::ui::dialog::{After, Dialog, DialogSet, speak};
use crate::game::items::{Bag, ITEMS};
use crate::world::kanto;
use crate::ui::menu::{MenuSet, overlay_open};
use crate::render::models::{self, Kit, NPC_OUTFITS, PLAYER_OUTFIT, Shape, Walker, part};
use crate::world::npcs::{Role, npc_id};
use crate::ui::party::PartySet;
use crate::game::progress::{Place, Progress};
use crate::ui;
use crate::world::{BuildingKind, Door, Link, LoadedMap, Look, S, Theme, Tile};

const WALK_SPEED: f32 = 4.4; // casillas por segundo
const RUN_SPEED: f32 = 7.6;
const JUMP_SPEED: f32 = 3.4;
const ENCOUNTER_CHANCE: f64 = 0.1;
/// Un toque corto de una flecha solo gira al personaje; si la mantienes, anda.
const TURN_GRACE: f32 = 0.1;
/// Tamaño del Pokémon acompañante respecto a su modelo de combate.
const FOLLOWER_SCALE: f32 = 0.62;
const FADE_SPEED: f32 = 4.0;

/// El mapa en el que estás.
#[derive(Resource)]
pub struct CurrentMap(pub LoadedMap);

/// Posición del jugador en casillas; sobrevive a los combates.
#[derive(Resource)]
pub struct PlayerPos(pub IVec2);

#[derive(Resource)]
pub struct Facing(pub IVec2);

/// La gente del mapa actual: dónde está cada uno ahora.
#[derive(Resource, Default)]
pub struct Actors(pub Vec<Actor>);

#[derive(Clone, Copy)]
pub struct Actor {
    /// Índice en `MapDef::npcs`.
    pub index: usize,
    pub pos: IVec2,
    pub facing: IVec2,
}

impl Actors {
    fn at(&self, pos: IVec2) -> Option<&Actor> {
        self.0.iter().find(|a| a.pos == pos)
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct NpcModel(usize);

/// Poké Ball del suelo, en esa casilla.
#[derive(Component)]
struct ItemModel(IVec2);

/// Todo lo que pertenece al mapa actual (se borra al cambiar de mapa).
#[derive(Component)]
struct MapEntity;

#[derive(Component)]
struct Moving {
    from: IVec2,
    to: IVec2,
    t: f32,
    /// Saltando un saliente (avanza dos casillas por el aire).
    jump: bool,
}

/// Hierba y flores que se mecen con el viento.
#[derive(Component)]
struct Sway(f32);

/// Objetos del suelo que giran y flotan un poco.
#[derive(Component)]
struct Bob(f32);

#[derive(Component)]
struct Wave(f32);

#[derive(Component)]
struct Cloud(f32);

/// El Pokémon que te sigue va siempre a la casilla que acabas de dejar.
#[derive(Resource)]
pub struct Trail {
    pos: IVec2,
    from: IVec2,
    t: f32,
    speed: f32,
}

impl Trail {
    pub fn at(pos: IVec2) -> Self {
        Self { pos, from: pos, t: 1.0, speed: WALK_SPEED }
    }
}

#[derive(Component)]
struct Follower {
    species: usize,
}

/// Hay que (re)construir el mapa.
#[derive(Resource)]
pub struct NeedsSpawn;

/// Cambio de mapa pendiente: se hace cuando la pantalla está en negro.
#[derive(Resource, Clone, Copy)]
pub enum PendingWarp {
    Exit { to_map: usize, digit: u8, index: usize },
    /// Entrar por una puerta (la vuelta queda en `Progress::back`).
    Door(usize),
    /// Ir a un sitio concreto mirando hacia abajo (salir de un interior, Cuerda Huida...).
    Place(Place),
}

/// Fundido a negro: `alpha` va hacia `target`.
#[derive(Resource)]
pub struct Fade {
    alpha: f32,
    pub target: f32,
}

/// Un entrenador te ha visto: "!", se acerca y te reta.
#[derive(Resource)]
struct Challenge {
    actor: usize,
    timer: f32,
    walking: Option<(IVec2, IVec2, f32)>,
}

/// La flecha que se está pulsando y desde cuándo (para girar sin andar con un toque).
#[derive(Resource, Default)]
struct Hold {
    dir: IVec2,
    secs: f32,
    turned: bool,
}

#[derive(Component)]
struct Exclamation;

#[derive(Component)]
struct FadeOverlay;

/// Cartel con el nombre del mapa al entrar.
#[derive(Component)]
struct Banner(f32);

/// El sol (su intensidad cambia en cuevas e interiores).
#[derive(Component)]
pub struct SunLight;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        let start = Place { map: kanto::PLAYER_HOUSE, pos: kanto::house_start() };
        let (map, actors) = enter_map(start.map, &Progress::default());
        app.insert_resource(CurrentMap(map))
            .insert_resource(actors)
            .insert_resource(PlayerPos(start.pos))
            .insert_resource(Facing(S))
            .insert_resource(Trail::at(start.pos + IVec2::X))
            .insert_resource(Fade { alpha: 1.0, target: 0.0 })
            .init_resource::<Hold>()
            .add_systems(Startup, spawn_fade_overlay)
            .add_systems(
                OnEnter(AppState::Overworld),
                (|mut commands: Commands| commands.insert_resource(NeedsSpawn), show_banner),
            )
            .add_systems(OnEnter(AppState::Battle), |mut fade: ResMut<Fade>| *fade = Fade { alpha: 1.0, target: 0.0 })
            .add_systems(
                Update,
                (
                    spawn_world.run_if(resource_exists::<NeedsSpawn>),
                    (interact, start_move)
                        .run_if(not(overlay_open))
                        .run_if(not(resource_exists::<PendingWarp>))
                        .run_if(not(resource_exists::<Challenge>)),
                    animate_move,
                    approach.run_if(resource_exists::<Challenge>),
                    do_warp.run_if(resource_exists::<PendingWarp>),
                    (turn_player, place_npcs, sync_follower, animate_follower, surf_visuals, camera_follow),
                )
                    .chain()
                    .after(DialogSet)
                    .after(PartySet)
                    .after(MenuSet)
                    .run_if(in_state(AppState::Overworld)),
            )
            .add_systems(Update, (sway, bob, waves, clouds, fade, animate_banner, models::walk_cycle));
    }
}

/// Casilla del mapa -> posición 3D. "Arriba" en el mapa es -Z.
pub fn to_world(pos: IVec2) -> Vec3 {
    Vec3::new(pos.x as f32, 0.0, -pos.y as f32)
}

fn dir_to_rotation(dir: IVec2) -> Quat {
    let d = to_world(dir);
    Quat::from_rotation_y(d.x.atan2(d.z))
}

fn can_walk(map: &LoadedMap, actors: &Actors, pos: IVec2) -> bool {
    map.get(pos).walkable() && actors.at(pos).is_none()
}

/// Carga un mapa: quita los objetos ya recogidos y coloca a la gente (apartada si ya toca).
pub fn enter_map(id: usize, progress: &Progress) -> (LoadedMap, Actors) {
    let mut map = LoadedMap::load(id);
    for (i, pos) in map.items.clone().into_iter().enumerate() {
        if progress.taken.get(kanto::item_id(id, i)) {
            map.set(pos, Tile::Ground);
        }
    }
    let actors = map
        .npcs
        .iter()
        .enumerate()
        .map(|(index, &pos)| {
            let npc = &map.def().npcs[index];
            let moved = progress.done.get(npc_id(id, index))
                || matches!(npc.role, Role::Guard { badges, .. } if progress.badge_count() >= badges as u32);
            let pos = match npc.aside {
                Some(aside) if moved => pos + aside,
                _ => pos,
            };
            Actor { index, pos, facing: npc.facing }
        })
        .collect();
    (map, Actors(actors))
}

// ---------------------------------------------------------------- construir el mapa

#[allow(clippy::too_many_arguments)]
fn spawn_world(
    mut commands: Commands,
    mut kit: Kit,
    map: Res<CurrentMap>,
    actors: Res<Actors>,
    player_pos: Res<PlayerPos>,
    facing: Res<Facing>,
    mut trail: ResMut<Trail>,
    old: Query<Entity, With<MapEntity>>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<&mut DirectionalLight, With<SunLight>>,
    mut fog: Query<&mut DistanceFog, With<MainCamera>>,
) {
    commands.remove_resource::<NeedsSpawn>();
    for e in &old {
        commands.entity(e).despawn();
    }
    let scoped = || (MapEntity, DespawnOnExit(AppState::Overworld));
    let m = &map.0;
    let def = m.def();
    let theme = def.theme;
    let size = m.size();
    apply_theme(theme, &mut clear, &mut ambient, &mut sun, &mut fog);

    // El acompañante se pone a tu espalda si no estaba a tu lado
    if (trail.pos - player_pos.0).abs().element_sum() != 1 || !can_walk(m, &actors, trail.pos) {
        let around = [-facing.0, IVec2::Y, IVec2::NEG_Y, IVec2::X, IVec2::NEG_X];
        trail.pos = around.iter().map(|d| player_pos.0 + *d).find(|p| can_walk(m, &actors, *p)).unwrap_or(player_pos.0);
    }
    trail.from = trail.pos;
    trail.t = 1.0;

    // Alrededor del mapa: bosque, roca o mar, para que nunca se vea el vacío (en interiores, negro)
    let center = to_world(size / 2);
    if !theme.indoors() {
        let outside = if theme == Theme::Sea { [40, 90, 160] } else { darken(def.ground, 0.9) };
        let ground = kit.piece(&part(Shape::Cube, outside, center - Vec3::Y * 0.26, Vec3::new(120.0, 0.3, 120.0)));
        commands.spawn((ground, scoped()));
        const RING: i32 = 7;
        for y in -RING..size.y + RING {
            for x in -RING..size.x + RING {
                let inside = (-1..=size.x).contains(&x) && (-1..=size.y).contains(&y);
                let seed = ((x + 50) * 31 + (y + 50) * 17) as u32;
                if inside || seed % 4 == 0 {
                    continue;
                }
                let pos = to_world(IVec2::new(x, y));
                match theme {
                    Theme::Sea => {}
                    Theme::Cave => {
                        let t = Transform::from_translation(pos);
                        kit.spawn_model(&mut commands, (t, scoped()), &models::cave_wall(seed, def.ground));
                    }
                    _ => {
                        let offset = Vec3::new((seed % 7) as f32 * 0.07 - 0.2, 0.0, (seed % 5) as f32 * 0.07 - 0.14);
                        let transform = Transform::from_translation(pos + offset - Vec3::Y * 0.1)
                            .with_rotation(Quat::from_rotation_y(seed as f32))
                            .with_scale(Vec3::splat(0.95 + (seed % 9) as f32 * 0.07));
                        kit.spawn_model(&mut commands, (transform, scoped()), &models::tree());
                    }
                }
            }
        }
    }

    for y in 0..size.y {
        for x in 0..size.x {
            let tile_pos = IVec2::new(x, y);
            let tile = m.get(tile_pos);
            let pos = to_world(tile_pos);
            let checker = (x + y) % 2 == 0;
            let seed = (x * 31 + y * 17 + m.id as i32 * 7) as u32;
            let ground = if checker { def.ground } else { darken(def.ground, 0.95) };
            let color = match tile {
                Tile::Path | Tile::Exit(_) if theme.indoors() => ground,
                Tile::Path | Tile::Exit(_) if theme.underground() => darken(def.ground, 0.85),
                Tile::Path | Tile::Exit(_) => if checker { [226, 206, 150] } else { [218, 198, 142] },
                Tile::Grass => darken(def.ground, 0.8),
                Tile::Water | Tile::Bridge => [40, 70, 120],
                Tile::Building => [200, 190, 170],
                _ => ground,
            };
            let height = if matches!(tile, Tile::Water | Tile::Bridge) { 0.05 } else { 0.2 };
            let floor = kit.piece(&part(Shape::Cube, color, pos + Vec3::Y * (height / 2.0 - 0.2), Vec3::new(1.0, height, 1.0)));
            commands.spawn((floor, scoped()));

            let here = Transform::from_translation(pos);
            let jitter = here.with_rotation(Quat::from_rotation_y(seed as f32));
            // Lo que hay encima de la casilla: (dónde, animación, piezas)
            let (transform, parts): (Transform, Vec<models::Part>) = match tile {
                Tile::Water | Tile::Bridge => {
                    commands.spawn((
                        Mesh3d(kit.mesh(Shape::Cube)),
                        MeshMaterial3d(kit.water()),
                        Transform::from_translation(pos - Vec3::Y * 0.12).with_scale(Vec3::new(1.0, 0.1, 1.0)),
                        Wave(x as f32 * 0.9 + y as f32 * 0.6),
                        scoped(),
                    ));
                    if tile == Tile::Water {
                        continue;
                    }
                    for side in [IVec2::X, IVec2::NEG_X] {
                        if m.get(tile_pos + side) == Tile::Water && (x + y) % 2 == 0 {
                            let p = pos + to_world(side) * 0.46;
                            kit.spawn_model(&mut commands, (Transform::from_translation(p), scoped()), &models::post());
                        }
                    }
                    (here, models::bridge(seed))
                }
                Tile::Tree if theme.indoors() => {
                    // La pared del sur, bajita, para que no tape la habitación
                    let height = if y == 0 { 0.45 } else { 1.7 };
                    let wall = if theme == Theme::Tower { darken(def.ground, 0.6) } else { [235, 225, 205] };
                    (here, models::wall(wall, height))
                }
                Tile::Tree if theme == Theme::Cave => (here, models::cave_wall(seed, darken(def.ground, 1.15))),
                Tile::Tree => (jitter.with_scale(Vec3::splat(0.85 + (seed % 7) as f32 * 0.05)), models::tree()),
                Tile::Mountain => (here, models::mountain(seed)),
                Tile::Ledge => (here, models::ledge(def.ground)),
                Tile::Grass => {
                    kit.spawn_model(&mut commands, (here, Sway(x as f32 * 0.7 + y as f32 * 1.3), scoped()), &models::grass_blades(seed + 1));
                    continue;
                }
                Tile::Flowers => {
                    kit.spawn_model(&mut commands, (here, Sway(x as f32 * 0.5 + y as f32), scoped()), &models::flowers(seed + 3));
                    continue;
                }
                Tile::Item => {
                    kit.spawn_model(&mut commands, (here, ItemModel(tile_pos), Bob(seed as f32), scoped()), &models::item_ball());
                    continue;
                }
                Tile::Rock => (jitter, models::rock(seed)),
                Tile::Bush => (jitter, models::bush()),
                Tile::Sign if theme.indoors() => (here, models::statue()),
                Tile::Sign => (here, models::sign()),
                Tile::Ground if seed % 3 == 0 && !theme.indoors() && !theme.underground() => (here, models::tufts(seed)),
                Tile::Ground if seed % 5 == 0 && theme == Theme::Cave => (here, models::pebbles(seed)),
                Tile::Path if seed % 4 == 0 && !theme.indoors() => (here, models::pebbles(seed)),
                Tile::Fence => {
                    let is_fence = |d: IVec2| m.get(tile_pos + d) == Tile::Fence;
                    (here, models::fence(is_fence(IVec2::NEG_X), is_fence(IVec2::X), is_fence(IVec2::Y), is_fence(IVec2::NEG_Y)))
                }
                Tile::Counter => (here, models::counter()),
                Tile::Shelf => (here, models::shelf(seed)),
                Tile::Table if theme == Theme::Tower => (here, models::tombstone(seed)),
                Tile::Table => (here, models::table()),
                Tile::Plant => (here, models::plant()),
                Tile::Machine => (here, models::machine()),
                Tile::Carpet => (here, models::rug()),
                Tile::Exit(digit) => {
                    let Some(exit) = m.exit(digit) else { continue };
                    let parts = match exit.look {
                        Look::Edge => continue,
                        Look::Cave => models::cave_mouth(),
                        Look::LadderDown => models::ladder(true),
                        Look::LadderUp => models::ladder(false),
                        Look::Mat => models::doormat(),
                    };
                    // Mirando hacia dentro del mapa (la escalera queda pegada a la pared)
                    (here.with_rotation(dir_to_rotation(exit.inward)), parts)
                }
                _ => continue,
            };
            kit.spawn_model(&mut commands, (transform, scoped()), &parts);
        }
    }

    for b in &m.buildings {
        let center = (to_world(b.min) + to_world(b.min + b.size - IVec2::ONE)) / 2.0;
        let door_x = to_world(b.door()).x - center.x;
        let parts = models::building(b.kind, b.size.x as f32, b.size.y as f32, door_x, b.roof);
        kit.spawn_model(&mut commands, (Transform::from_translation(center), scoped()), &parts);
    }

    for (i, actor) in actors.0.iter().enumerate() {
        let npc = &def.npcs[actor.index];
        let transform = Transform::from_translation(to_world(actor.pos)).with_rotation(dir_to_rotation(actor.facing));
        kit.spawn_character(&mut commands, (transform, NpcModel(i), scoped()), &NPC_OUTFITS[npc.look]);
    }

    if !theme.indoors() && !theme.underground() {
        // Nubes que cruzan el cielo
        for i in 0..7 {
            let pos = Vec3::new(i as f32 * 9.0 - 10.0, 9.0 + (i % 3) as f32 * 1.2, -(i as f32 * 7.0 % size.y as f32) - 2.0);
            kit.spawn_model(&mut commands, (Transform::from_translation(pos), Cloud(0.4 + (i % 3) as f32 * 0.15), scoped()), &models::cloud(i + 1));
        }
    }

    let transform = Transform::from_translation(to_world(player_pos.0)).with_rotation(dir_to_rotation(facing.0));
    let player = kit.spawn_character(&mut commands, (Player, transform, scoped()), &PLAYER_OUTFIT);
    let raft = kit.spawn_model(&mut commands, (Transform::default(), Raft), &models::surfboard());
    commands.entity(raft).insert((ChildOf(player), Visibility::Hidden));
}

/// Tabla de surf bajo el jugador (solo se ve en el agua).
#[derive(Component)]
struct Raft;

/// Al surfear: se ve la tabla y el acompañante espera escondido hasta volver a tierra.
fn surf_visuals(
    map: Res<CurrentMap>,
    trail: Res<Trail>,
    player: Single<&Transform, With<Player>>,
    mut raft: Query<&mut Visibility, (With<Raft>, Without<Follower>)>,
    mut follower: Query<&mut Visibility, (With<Follower>, Without<Raft>)>,
) {
    let here = IVec2::new(player.translation.x.round() as i32, (-player.translation.z).round() as i32);
    let on_water = map.0.get(here) == Tile::Water;
    for mut v in &mut raft {
        v.set_if_neq(if on_water { Visibility::Inherited } else { Visibility::Hidden });
    }
    let hide = map.0.get(trail.pos) == Tile::Water || map.0.get(trail.from) == Tile::Water;
    for mut v in &mut follower {
        v.set_if_neq(if hide { Visibility::Hidden } else { Visibility::Inherited });
    }
}

/// Luz y cielo según el sitio: sol fuera, penumbra en cuevas, luz cálida en interiores.
fn apply_theme(
    theme: Theme,
    clear: &mut ClearColor,
    ambient: &mut GlobalAmbientLight,
    sun: &mut Query<&mut DirectionalLight, With<SunLight>>,
    fog: &mut Query<&mut DistanceFog, With<MainCamera>>,
) {
    let (sky, ambient_light, sun_light, fog_range) = match theme {
        Theme::Cave => (Color::srgb(0.08, 0.06, 0.06), 380.0, 2600.0, (8.0, 19.0)),
        Theme::Tower => (Color::srgb(0.06, 0.04, 0.09), 420.0, 2300.0, (9.0, 21.0)),
        Theme::Inside | Theme::Gym => (Color::srgb(0.05, 0.05, 0.07), 900.0, 5200.0, (24.0, 44.0)),
        _ => (crate::SKY, 500.0, light_consts::lux::AMBIENT_DAYLIGHT * 0.4, (14.0, 32.0)),
    };
    clear.0 = sky;
    ambient.brightness = ambient_light;
    for mut light in sun.iter_mut() {
        light.illuminance = sun_light;
    }
    for mut f in fog.iter_mut() {
        f.color = sky;
        f.falloff = FogFalloff::Linear { start: fog_range.0, end: fog_range.1 };
    }
}

fn darken(c: models::Rgb, k: f32) -> models::Rgb {
    c.map(|x| (x as f32 * k).min(255.0) as u8)
}

// ---------------------------------------------------------------- hablar y mirar

#[allow(clippy::too_many_arguments)]
fn interact(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut map: ResMut<CurrentMap>,
    mut actors: ResMut<Actors>,
    player_pos: Res<PlayerPos>,
    facing: Res<Facing>,
    trail: Res<Trail>,
    companion: Res<Companion>,
    mut party: ResMut<Party>,
    mut bag: ResMut<Bag>,
    mut progress: ResMut<Progress>,
    moving: Query<(), (With<Player>, With<Moving>)>,
    items: Query<(Entity, &ItemModel)>,
) {
    if !input.any_just_pressed([KeyCode::KeyZ, KeyCode::Space]) || !moving.is_empty() {
        return;
    }
    let mut target = player_pos.0 + facing.0;
    // Se habla por encima de los mostradores
    if map.0.get(target) == Tile::Counter {
        target += facing.0;
    }
    let say = |commands: &mut Commands, lines: Vec<String>| commands.insert_resource(Dialog::message(lines));

    if let Some(c) = companion.0.and_then(|i| party.0.get(i)).filter(|_| trail.pos == target) {
        let name = c.name();
        let line = if c.fainted() {
            format!("{name} está agotado... Llévalo al Centro Pokémon.")
        } else {
            let lines = [
                format!("{name} te mira muy contento."),
                format!("{name} da saltitos a tu alrededor."),
                format!("{name} tiene ganas de combatir."),
                format!("{name} se acurruca junto a ti."),
            ];
            lines[rand::rng().random_range(0..lines.len())].clone()
        };
        say(&mut commands, vec![line]);
        return;
    }

    if let Some(slot) = actors.0.iter().position(|a| a.pos == target) {
        // La persona se gira hacia ti
        actors.0[slot].facing = -facing.0;
        let index = actors.0[slot].index;
        let id = npc_id(map.0.id, index);
        let npc = &map.0.def().npcs[index];
        let line = |s: &str| speak(npc.name, s);
        let dialog = match &npc.role {
            Role::Talk(lines) => Dialog::message(lines.iter().map(|l| line(l)).collect()),
            Role::Quiz(_) => Dialog::quiz(id, &progress),
            Role::Trainer(t) if progress.done.get(id) => Dialog::message(vec![line(t.after)]),
            Role::Trainer(t) => Dialog::message(t.intro.iter().map(|l| line(l)).collect()).then(After::Battle(id)),
            Role::Nurse => {
                party.heal_all();
                progress.last_center = progress.back;
                Dialog::message(vec![
                    line("¡Hola! Bienvenido al Centro Pokémon."),
                    line("Vamos a curar a tus Pokémon... Un momento, por favor."),
                    line("¡Listo! Tus Pokémon se han recuperado por completo. ¡Vuelve cuando quieras!"),
                ])
            }
            Role::Mom => {
                party.heal_all();
                Dialog::message(vec![line("¡Hola, cariño! Pareces cansado. Descansa un poco."), line("...¡Ya está! Tus Pokémon están como nuevos. ¡Ánimo!")])
            }
            Role::Clerk => Dialog::message(vec![line("¡Hola! ¿En qué puedo ayudarte?")]).then(After::Shop),
            Role::Gift { lines, item, qty, after } => {
                if progress.done.get(id) {
                    Dialog::message(vec![line(after)])
                } else {
                    progress.done.set(id);
                    bag.add(*item, *qty);
                    let mut all: Vec<String> = lines.iter().map(|l| line(l)).collect();
                    all.push(format!("¡Has recibido {}!", item_text(*item, *qty)));
                    Dialog::message(all)
                }
            }
            Role::Guard { badges, blocked, ok } => {
                if progress.badge_count() >= *badges as u32 {
                    if let Some(aside) = npc.aside {
                        actors.0[slot].pos += aside;
                    }
                    Dialog::message(vec![line(ok)])
                } else {
                    Dialog::message(vec![line(blocked), format!("Tienes {} de {} medallas.", progress.badge_count(), badges)])
                }
            }
        };
        commands.insert_resource(dialog);
        return;
    }

    match map.0.get(target) {
        Tile::Sign => {
            let i = map.0.signs.iter().position(|p| *p == target).unwrap_or(0);
            if let Some(text) = map.0.def().signs.get(i) {
                say(&mut commands, vec![text.to_string()]);
            }
        }
        Tile::Item => {
            let Some(i) = map.0.items.iter().position(|p| *p == target) else { return };
            let (item, qty) = map.0.def().items[i];
            bag.add(item, qty);
            progress.taken.set(kanto::item_id(map.0.id, i));
            map.0.set(target, Tile::Ground);
            for (e, model) in &items {
                if model.0 == target {
                    commands.entity(e).despawn();
                }
            }
            say(&mut commands, vec![format!("¡Has encontrado {}!", item_text(item, qty)), format!("Guardas {} en la mochila.", ITEMS[item].name)]);
        }
        Tile::Machine => say(&mut commands, vec!["Es un aparato lleno de luces y botones. Mejor no tocar nada.".to_string()]),
        Tile::Water if !can_surf(&progress, &party) => say(
            &mut commands,
            vec!["El agua es profunda y azul...".into(), "Con la Medalla Alma y un Pokémon de tipo Agua podrías surfear.".into()],
        ),
        Tile::Shelf => say(&mut commands, vec!["Una estantería llena de libros sobre Pokémon.".to_string()]),
        Tile::Building => {
            if map.0.building_at(target).is_some_and(|b| b.door() == target) {
                enter_door(&mut commands, &map.0, target, &mut progress);
            }
        }
        _ => {}
    }
}

fn item_text(item: usize, qty: u16) -> String {
    if qty > 1 { format!("{} × {}", ITEMS[item].name, qty) } else { ITEMS[item].name.to_string() }
}

/// Si aún no puedes entrar en ese mapa, lo que te dicen.
fn locked(to: usize, progress: &Progress) -> Option<Vec<String>> {
    let need = kanto::required_badges(to);
    if progress.badge_count() >= need as u32 {
        return None;
    }
    let badge = crate::world::npcs::BADGES[need as usize - 1].name;
    let def = &kanto::MAPS[to];
    Some(if def.theme == Theme::Gym {
        vec![format!("La puerta del gimnasio está cerrada. Solo admiten retadores con la {badge}.")]
    } else {
        vec![
            speak("Guardia", &format!("¡Alto! El camino hacia {} es peligroso.", def.name)),
            speak("Guardia", &format!("Vuelve cuando tengas la {badge} ({need} medallas).")),
        ]
    })
}

/// Si la casilla es una salida a un mapa al que aún no puedes ir.
fn exit_locked(map: &LoadedMap, pos: IVec2, progress: &Progress) -> Option<Vec<String>> {
    let Tile::Exit(digit) = map.get(pos) else { return None };
    match map.exit(digit)?.to {
        Link::Map(to, _) => locked(to, progress),
        Link::Back => None,
    }
}

fn enter_door(commands: &mut Commands, map: &LoadedMap, door: IVec2, progress: &mut Progress) {
    let Some(b) = map.building_at(door) else { return };
    let inside = match map.door(b) {
        Door::Map(to) => to,
        Door::Badges(n, to, _) if progress.badge_count() >= n as u32 => to,
        Door::Text(text) | Door::Badges(_, _, text) => {
            commands.insert_resource(Dialog::message(vec![text.to_string()]));
            return;
        }
        Door::Default => return,
    };
    if let Some(lines) = locked(inside, progress) {
        commands.insert_resource(Dialog::message(lines));
        return;
    }
    progress.back = Place { map: map.id, pos: b.door() + S };
    if b.kind == BuildingKind::Center {
        progress.last_center = progress.back;
    }
    commands.insert_resource(PendingWarp::Door(inside));
}

// ---------------------------------------------------------------- andar

/// Qué dirección de la pantalla se está pulsando.
fn held_direction(input: &ButtonInput<KeyCode>) -> Option<IVec2> {
    if input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(IVec2::Y)
    } else if input.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(IVec2::NEG_Y)
    } else if input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(IVec2::NEG_X)
    } else if input.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(IVec2::X)
    } else {
        None
    }
}

fn running(input: &ButtonInput<KeyCode>) -> bool {
    input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
}

fn walk_speed(input: &ButtonInput<KeyCode>) -> f32 {
    if running(input) { RUN_SPEED } else { WALK_SPEED }
}

/// Se puede surfear con la Medalla Alma y un Pokémon de tipo Agua en el equipo.
pub fn can_surf(progress: &Progress, party: &Party) -> bool {
    progress.has_badge(4) && party.0.iter().any(|c| c.spec().types().any(|t| t == crate::pokemon::Type::Water))
}

/// Pokémon que salen al surfear.
const SURF_TABLE: [(usize, u32); 3] = [(crate::pokemon::MAGIKARP, 50), (crate::pokemon::STARYU, 25), (crate::pokemon::SEEL, 25)];

/// Qué pasa al intentar dar un paso hacia `dir`.
enum Step {
    Walk(IVec2),
    /// Saltar un saliente: caes dos casillas más allá.
    Jump(IVec2),
    /// Entrar por la puerta de un edificio.
    Enter(IVec2),
    Blocked,
}

fn step(map: &LoadedMap, actors: &Actors, from: IVec2, dir: IVec2, surf: bool) -> Step {
    let target = from + dir;
    match map.get(target) {
        Tile::Water if surf && actors.at(target).is_none() => Step::Walk(target),
        Tile::Ledge if dir == S && can_walk(map, actors, target + S) => Step::Jump(target + S),
        Tile::Building if dir == IVec2::Y && map.building_at(target).is_some_and(|b| b.door() == target) => Step::Enter(target),
        _ if can_walk(map, actors, target) => Step::Walk(target),
        _ => Step::Blocked,
    }
}

#[allow(clippy::too_many_arguments)]
fn start_move(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    rig: Res<CameraRig>,
    map: Res<CurrentMap>,
    actors: Res<Actors>,
    player_pos: Res<PlayerPos>,
    mut facing: ResMut<Facing>,
    mut trail: ResMut<Trail>,
    mut hold: ResMut<Hold>,
    mut progress: ResMut<Progress>,
    party: Res<Party>,
    player: Query<Entity, (With<Player>, Without<Moving>)>,
) {
    let Ok(entity) = player.single() else {
        return;
    };
    // Las flechas son relativas a la pantalla: "arriba" es hacia donde mira la cámara
    let Some(screen) = held_direction(&input) else {
        hold.dir = IVec2::ZERO;
        return;
    };
    let dir = rig.screen_to_map(screen);
    if hold.dir != dir {
        // Nueva dirección: primero se gira; si sigues pulsando, anda
        *hold = Hold { dir, secs: 0.0, turned: facing.0 != dir };
        facing.0 = dir;
    }
    hold.secs += time.delta_secs();
    if hold.turned && hold.secs < TURN_GRACE {
        return;
    }
    match step(&map.0, &actors, player_pos.0, dir, can_surf(&progress, &party)) {
        Step::Walk(to) if exit_locked(&map.0, to, &progress).is_some() => {
            commands.insert_resource(Dialog::message(exit_locked(&map.0, to, &progress).unwrap()));
            hold.dir = IVec2::ZERO;
        }
        Step::Walk(to) => {
            commands.entity(entity).insert(Moving { from: player_pos.0, to, t: 0.0, jump: false });
            *trail = Trail { from: trail.pos, pos: player_pos.0, t: 0.0, speed: walk_speed(&input) };
        }
        Step::Jump(to) => {
            commands.entity(entity).insert(Moving { from: player_pos.0, to, t: 0.0, jump: true });
            *trail = Trail { from: trail.pos, pos: player_pos.0, t: 0.0, speed: JUMP_SPEED };
        }
        Step::Enter(door) => {
            enter_door(&mut commands, &map.0, door, &mut progress);
            hold.dir = IVec2::ZERO;
        }
        Step::Blocked => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn animate_move(
    mut commands: Commands,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    rig: Res<CameraRig>,
    map: Res<CurrentMap>,
    actors: Res<Actors>,
    party: Res<Party>,
    dialog: Option<Res<Dialog>>,
    mut progress: ResMut<Progress>,
    mut player_pos: ResMut<PlayerPos>,
    mut facing: ResMut<Facing>,
    mut trail: ResMut<Trail>,
    mut next_state: ResMut<NextState<AppState>>,
    mut fade: ResMut<Fade>,
    mut player: Query<(Entity, &mut Transform, Option<&mut Moving>, &mut Walker), With<Player>>,
) {
    let Ok((entity, mut transform, moving, mut walker)) = player.single_mut() else {
        return;
    };
    let Some(mut moving) = moving else {
        walker.moving = false;
        return;
    };
    let run = running(&input) && !moving.jump;
    let speed = if moving.jump { JUMP_SPEED } else if run { RUN_SPEED } else { WALK_SPEED };
    let dt = time.delta_secs();
    // El salto recorre dos casillas
    moving.t += dt * speed / if moving.jump { 2.0 } else { 1.0 };
    walker.moving = true;
    walker.running = run;
    // Media zancada por casilla: los pies no patinan
    walker.phase += dt * speed * std::f32::consts::PI;

    // Llegada a la casilla: eventos y, si sigues pulsando, el siguiente paso sin pararse
    let mut stop = false;
    while moving.t >= 1.0 {
        let arrived = moving.to;
        player_pos.0 = arrived;
        progress.steps += 1;
        if progress.repel > 0 {
            progress.repel -= 1;
            if progress.repel == 0 {
                commands.insert_resource(Dialog::message(vec!["Se ha acabado el efecto del Repelente.".into()]));
                stop = true;
            }
        }
        if stop || arrive(&mut commands, &map.0, &actors, arrived, &party, &mut progress, &mut next_state, &mut fade) {
            stop = true;
            break;
        }
        let next = held_direction(&input).map(|s| rig.screen_to_map(s)).filter(|_| dialog.is_none());
        let surf = can_surf(&progress, &party);
        match next.map(|dir| (dir, step(&map.0, &actors, arrived, dir, surf))) {
            Some((dir, Step::Walk(to))) if exit_locked(&map.0, to, &progress).is_none() => {
                facing.0 = dir;
                let leftover = moving.t - 1.0;
                *moving = Moving { from: arrived, to, t: leftover, jump: false };
                *trail = Trail { from: trail.pos, pos: arrived, t: leftover, speed: walk_speed(&input) };
            }
            _ => {
                stop = true;
                break;
            }
        }
    }

    if stop {
        commands.entity(entity).remove::<Moving>();
        walker.moving = false;
        transform.translation = to_world(player_pos.0);
        return;
    }
    let t = moving.t;
    let mut pos = to_world(moving.from).lerp(to_world(moving.to), t);
    if moving.jump {
        pos.y += (t * std::f32::consts::PI).sin() * 0.55;
    } else {
        // Un balanceo mínimo al pisar (sin saltitos)
        pos.y += walker.phase.sin().abs() * 0.02;
    }
    transform.translation = pos;
}

/// Al pisar una casilla: salidas, entrenadores que te ven y Pokémon salvajes.
/// Devuelve `true` si hay que dejar de andar.
#[allow(clippy::too_many_arguments)]
fn arrive(
    commands: &mut Commands,
    map: &LoadedMap,
    actors: &Actors,
    pos: IVec2,
    party: &Party,
    progress: &mut Progress,
    next_state: &mut NextState<AppState>,
    fade: &mut Fade,
) -> bool {
    let def = map.def();
    let tile = map.get(pos);
    if let Tile::Exit(digit) = tile {
        if let Some(exit) = map.exit(digit) {
            let warp = match exit.to {
                Link::Map(to_map, to_digit) => {
                    let index = map.exit_tiles(digit).iter().position(|p| *p == pos).unwrap_or(0);
                    PendingWarp::Exit { to_map, digit: to_digit, index }
                }
                Link::Back => PendingWarp::Place(progress.back),
            };
            commands.insert_resource(warp);
            fade.target = 1.0;
            return true;
        }
    }

    // Entrenadores que te ven: estás en su línea de visión y no hay nada en medio
    for (slot, actor) in actors.0.iter().enumerate() {
        let npc = &def.npcs[actor.index];
        let Some(t) = npc.trainer() else { continue };
        if t.sight == 0 || progress.done.get(npc_id(map.id, actor.index)) {
            continue;
        }
        let seen = (1..=t.sight).find(|k| actor.pos + actor.facing * *k == pos).is_some_and(|k| {
            (1..k).all(|j| {
                let p = actor.pos + actor.facing * j;
                map.get(p).walkable() && actors.at(p).is_none()
            })
        });
        if seen {
            commands.insert_resource(Challenge { actor: slot, timer: 0.0, walking: None });
            return true;
        }
    }

    if tile == Tile::Water && rand::rng().random_bool(ENCOUNTER_CHANCE * 0.6) {
        let levels = if def.encounters.is_empty() { (20, 30) } else { def.levels };
        let wild = random_wild(&SURF_TABLE, levels);
        commands.insert_resource(PendingBattle::Wild(wild));
        next_state.set(AppState::Battle);
        return true;
    }
    let wild_here = tile == Tile::Grass || (def.theme.wild_everywhere() && matches!(tile, Tile::Ground | Tile::Path));
    if wild_here && !def.encounters.is_empty() && rand::rng().random_bool(ENCOUNTER_CHANCE) {
        let wild = random_wild(def.encounters, def.levels);
        let lead = party.first_alive().map_or(0, |i| party.0[i].level);
        // Con Repelente no salen los más débiles que tu primer Pokémon
        if progress.repel == 0 || wild.level >= lead {
            commands.insert_resource(PendingBattle::Wild(wild));
            next_state.set(AppState::Battle);
            return true;
        }
    }
    false
}

/// El entrenador que te ha visto: "!", se acerca andando y te habla.
#[allow(clippy::too_many_arguments)]
fn approach(
    mut commands: Commands,
    mut kit: Kit,
    time: Res<Time>,
    map: Res<CurrentMap>,
    mut challenge: ResMut<Challenge>,
    mut actors: ResMut<Actors>,
    player_pos: Res<PlayerPos>,
    mut facing: ResMut<Facing>,
    npcs: Query<(Entity, &NpcModel)>,
    marks: Query<Entity, With<Exclamation>>,
    mut walkers: Query<&mut Walker, Without<Player>>,
) {
    let slot = challenge.actor;
    let actor = actors.0[slot];
    let entity = npcs.iter().find(|(_, m)| m.0 == slot).map(|(e, _)| e);
    if challenge.timer == 0.0 {
        if let Some(e) = entity {
            let mark = kit.spawn_model(&mut commands, (Transform::default(), Exclamation), &models::exclamation());
            commands.entity(mark).insert(ChildOf(e));
        }
    }
    challenge.timer += time.delta_secs();
    if challenge.timer < 0.7 {
        return;
    }
    for m in &marks {
        commands.entity(m).despawn();
    }

    // Anda hasta quedarse justo delante del jugador
    let target = player_pos.0 - actor.facing;
    let walker = entity.and_then(|e| walkers.get_mut(e).ok());
    if actor.pos != target {
        let (from, to, t) = challenge.walking.unwrap_or((actor.pos, actor.pos + actor.facing, 0.0));
        let t = t + time.delta_secs() * WALK_SPEED;
        if let Some(mut w) = walker {
            w.moving = true;
            w.phase += time.delta_secs() * WALK_SPEED * std::f32::consts::PI;
        }
        if t >= 1.0 {
            actors.0[slot].pos = to;
            challenge.walking = None;
        } else {
            challenge.walking = Some((from, to, t));
        }
        return;
    }
    if let Some(mut w) = walker {
        w.moving = false;
    }
    facing.0 = -actor.facing;
    let npc = &map.0.def().npcs[actor.index];
    let t = npc.trainer().expect("solo los entrenadores retan");
    let lines = t.intro.iter().map(|l| speak(npc.name, l)).collect();
    commands.insert_resource(Dialog::message(lines).then(After::Battle(npc_id(map.0.id, actor.index))));
    commands.remove_resource::<Challenge>();
}

/// Coloca a la gente donde dice `Actors` (y anima al que se acerca o se aparta).
fn place_npcs(
    time: Res<Time>,
    actors: Res<Actors>,
    challenge: Option<Res<Challenge>>,
    mut npcs: Query<(&NpcModel, &mut Transform)>,
) {
    let k = 1.0 - (-12.0 * time.delta_secs()).exp();
    for (model, mut transform) in &mut npcs {
        let Some(actor) = actors.0.get(model.0) else { continue };
        let walking = challenge.as_ref().filter(|c| c.actor == model.0).and_then(|c| c.walking);
        if let Some((from, to, t)) = walking {
            transform.translation = to_world(from).lerp(to_world(to), t);
        } else {
            let pos = to_world(actor.pos);
            let snap = transform.translation.distance(pos) > 2.5;
            transform.translation = if snap { pos } else { transform.translation.lerp(pos, k) };
        }
        transform.rotation = transform.rotation.slerp(dir_to_rotation(actor.facing), k);
    }
}

/// Cuando la pantalla ya está en negro: cambia de mapa y vuelve a aclarar.
#[allow(clippy::too_many_arguments)]
fn do_warp(
    mut commands: Commands,
    warp: Res<PendingWarp>,
    mut fade: ResMut<Fade>,
    mut map: ResMut<CurrentMap>,
    mut actors: ResMut<Actors>,
    progress: Res<Progress>,
    mut player_pos: ResMut<PlayerPos>,
    mut facing: ResMut<Facing>,
    mut trail: ResMut<Trail>,
    mut hold: ResMut<Hold>,
    banner: Query<Entity, With<Banner>>,
) {
    fade.target = 1.0;
    if fade.alpha < 1.0 {
        return;
    }
    let (to_map, arrival, dir) = match *warp {
        PendingWarp::Exit { to_map, digit, index } => {
            let next = LoadedMap::load(to_map);
            let exit = next.exit(digit).expect("la salida existe en los dos mapas");
            let tiles = next.exit_tiles(digit);
            (to_map, tiles[index.min(tiles.len() - 1)] + exit.inward, exit.inward)
        }
        PendingWarp::Door(to_map) => {
            let next = LoadedMap::load(to_map);
            let mat = next.def().exits.iter().find(|e| e.to == Link::Back).expect("los interiores tienen salida");
            let tiles = next.exit_tiles(mat.digit);
            (to_map, tiles[0] + mat.inward, mat.inward)
        }
        PendingWarp::Place(place) => (place.map, place.pos, S),
    };
    let (next, people) = enter_map(to_map, &progress);
    player_pos.0 = arrival;
    facing.0 = dir;
    hold.dir = dir;
    *trail = Trail::at(arrival - dir);
    map.0 = next;
    *actors = people;
    commands.remove_resource::<PendingWarp>();
    commands.insert_resource(NeedsSpawn);
    fade.target = 0.0;
    for e in &banner {
        commands.entity(e).despawn();
    }
    let theme = map.0.def().theme;
    if !matches!(theme, Theme::Inside) {
        spawn_banner(&mut commands, map.0.def().name);
    }
}

/// Gira al jugador suavemente hacia donde mira.
fn turn_player(time: Res<Time>, facing: Res<Facing>, mut player: Single<&mut Transform, With<Player>>) {
    let t = 1.0 - (-16.0 * time.delta_secs()).exp();
    player.rotation = player.rotation.slerp(dir_to_rotation(facing.0), t);
}

/// Crea o cambia el modelo del acompañante cuando cambia quién te sigue.
fn sync_follower(
    mut commands: Commands,
    mut kit: Kit,
    companion: Res<Companion>,
    party: Res<Party>,
    trail: Res<Trail>,
    facing: Res<Facing>,
    followers: Query<(Entity, &Follower)>,
) {
    let wanted = companion.0.and_then(|i| party.0.get(i)).map(|c| c.species);
    let mut have = false;
    for (entity, follower) in &followers {
        if Some(follower.species) == wanted {
            have = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let (Some(species), false) = (wanted, have) {
        let transform = Transform::from_translation(to_world(trail.pos))
            .with_rotation(dir_to_rotation(facing.0))
            .with_scale(Vec3::splat(FOLLOWER_SCALE));
        let root = (Follower { species }, transform, MapEntity, DespawnOnExit(AppState::Overworld));
        kit.spawn_creature(&mut commands, root, species);
    }
}

fn animate_follower(time: Res<Time>, mut trail: ResMut<Trail>, mut follower: Query<&mut Transform, With<Follower>>) {
    trail.t = (trail.t + time.delta_secs() * trail.speed).min(1.0);
    let Ok(mut transform) = follower.single_mut() else {
        return;
    };
    let mut pos = to_world(trail.from).lerp(to_world(trail.pos), trail.t);
    // Pasitos cortos y suaves mientras se mueve
    if trail.t < 1.0 {
        pos.y += (trail.t * std::f32::consts::TAU).sin().abs() * 0.035;
    }
    transform.translation = pos;

    let dir = trail.pos - trail.from;
    if dir != IVec2::ZERO {
        let k = 1.0 - (-12.0 * time.delta_secs()).exp();
        transform.rotation = transform.rotation.slerp(dir_to_rotation(dir), k);
    }
    let breathe = 1.0 + (time.elapsed_secs() * 3.0).sin() * 0.025;
    transform.scale = Vec3::new(FOLLOWER_SCALE, FOLLOWER_SCALE * breathe, FOLLOWER_SCALE);
}

/// La cámara sigue al jugador con suavidad; su orientación solo la cambia el usuario
/// (así no se balancea al andar).
fn camera_follow(
    time: Res<Time>,
    rig: Res<CameraRig>,
    map: Res<CurrentMap>,
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let ahead = if map.0.def().theme.indoors() { 0.6 } else { 1.3 };
    let focus = player.translation.with_y(0.0) + rig.forward() * ahead;
    let target = focus + rig.offset();
    // Al empezar, al cambiar de mapa o al volver de un combate la cámara está lejos: de golpe
    let t = if camera.translation.distance(target) > 6.0 { 1.0 } else { 1.0 - (-6.0 * time.delta_secs()).exp() };
    camera.translation = camera.translation.lerp(target, t);
    let look = camera.translation - rig.offset() + Vec3::Y * 0.3;
    camera.look_at(look, Vec3::Y);
}

// ---------------------------------------------------------------- animaciones del escenario

fn sway(time: Res<Time>, mut q: Query<(&Sway, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (sway, mut transform) in &mut q {
        transform.rotation =
            Quat::from_rotation_z((t * 1.8 + sway.0).sin() * 0.08) * Quat::from_rotation_x((t * 1.3 + sway.0).cos() * 0.05);
    }
}

fn bob(time: Res<Time>, mut q: Query<(&Bob, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (b, mut transform) in &mut q {
        transform.translation.y = (t * 2.2 + b.0).sin().abs() * 0.06;
        transform.rotation = Quat::from_rotation_y(t * 0.8 + b.0);
    }
}

fn waves(time: Res<Time>, mut q: Query<(&Wave, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (wave, mut transform) in &mut q {
        transform.translation.y = -0.12 + (t * 1.6 + wave.0).sin() * 0.025;
    }
}

fn clouds(time: Res<Time>, map: Option<Res<CurrentMap>>, mut q: Query<(&Cloud, &mut Transform)>) {
    let width = map.map_or(30.0, |m| m.0.size().x as f32);
    for (cloud, mut transform) in &mut q {
        transform.translation.x += cloud.0 * time.delta_secs();
        if transform.translation.x > width + 15.0 {
            transform.translation.x = -15.0;
        }
    }
}

// ---------------------------------------------------------------- fundido y cartel de zona

fn spawn_fade_overlay(mut commands: Commands) {
    commands.spawn((
        FadeOverlay,
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(100),
        Pickable::IGNORE,
    ));
}

fn fade(time: Res<Time>, mut fade: ResMut<Fade>, mut overlay: Query<&mut BackgroundColor, With<FadeOverlay>>) {
    let step = FADE_SPEED * time.delta_secs();
    fade.alpha = if fade.alpha < fade.target { (fade.alpha + step).min(fade.target) } else { (fade.alpha - step).max(fade.target) };
    for mut bg in &mut overlay {
        bg.0 = Color::BLACK.with_alpha(fade.alpha);
    }
}

fn show_banner(mut commands: Commands, map: Res<CurrentMap>, banner: Query<Entity, With<Banner>>) {
    for e in &banner {
        commands.entity(e).despawn();
    }
    spawn_banner(&mut commands, map.0.def().name);
}

fn spawn_banner(commands: &mut Commands, name: &str) {
    commands.spawn((
        Banner(0.0),
        DespawnOnExit(AppState::Overworld),
        Node { position_type: PositionType::Absolute, width: percent(100), top: px(-80), justify_content: JustifyContent::Center, ..default() },
        GlobalZIndex(20),
        children![(
            ui::panel(Node { padding: UiRect::axes(px(34), px(10)), align_items: AlignItems::Center, column_gap: px(12), ..default() }),
            children![ui::pokeball(24.0), ui::bold(name.to_string(), 26.0)],
        )],
    ));
}

/// El cartel baja, se queda un momento y vuelve a subir.
fn animate_banner(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Banner, &mut Node)>) {
    for (entity, mut banner, mut node) in &mut q {
        banner.0 += time.delta_secs();
        let t = banner.0;
        let shown = if t < 0.4 { t / 0.4 } else if t < 2.6 { 1.0 } else { 1.0 - (t - 2.6) / 0.4 };
        let ease = 1.0 - (1.0 - shown.clamp(0.0, 1.0)).powi(3);
        node.top = px(-80.0 + ease * 100.0);
        if t > 3.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Lleva al jugador a un sitio (Cuerda Huida, cargar partida...) con fundido.
pub fn warp_to(commands: &mut Commands, place: Place) {
    commands.insert_resource(PendingWarp::Place(place));
}
