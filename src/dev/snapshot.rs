//! Modo captura (herramienta de desarrollo): renderiza una escena sin abrir ventana,
//! la guarda como PNG y sale. Se activa con la variable de entorno
//! `POKEGAME_SNAPSHOT=<escena>:<ruta.png>`.
//!
//! Escenas: `title`, `overworld`, `menu` (equipo), `start`, `bag`, `profile`, `dex`, `save`,
//! `shop`, `quiz`, `battle`, `trainer`, `gallery`, `gallery-back` y `p<especie>a<grados>`
//! (primer plano de un Pokémon girado).
//!
//! Más variables: `POKEGAME_WILD=<especie>` (salvaje del combate), `POKEGAME_TRAINER=<mapa>:<n>`
//! (entrenador del combate), `POKEGAME_MAP=<n>` (empezar en ese mapa, junto a su primera
//! salida), `POKEGAME_PLACE=<mapa>,<x>,<y>`, `POKEGAME_KEYS=enter,down,z` (simula esas teclas,
//! una cada pocos fotogramas), `POKEGAME_FRAMES=<n>` (fotograma de la captura) y
//! `POKEGAME_TRACE=<archivo>` (posición del jugador y de la cámara en cada fotograma) y
//! `POKEGAME_BADGES=<bits>` (medallas de la partida de prueba). Más detalles en el README.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::AppState;
use crate::battle::PendingBattle;
use crate::render::camera::MainCamera;
use crate::pokemon::*;
use crate::ui::dialog::Dialog;
use crate::game::items::*;
use crate::world::kanto;
use crate::ui::menu::{GameMenu, Shop};
use crate::render::models::Kit;
use crate::world::npcs::npc_id;
use crate::world::overworld::{CurrentMap, PlayerPos, Trail, enter_map};
use crate::ui::party::PartyMenu;
use crate::game::progress::Progress;
use crate::world::{LoadedMap, Link};

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 800;
const GALLERY: Vec3 = Vec3::new(-500.0, 0.0, 0.0);

#[derive(Resource, Clone)]
pub struct Snapshot {
    scene: String,
    out: String,
    frame: u32,
    capture_at: u32,
    target: Handle<Image>,
}

impl Snapshot {
    pub fn from_env() -> Option<Self> {
        let value = std::env::var("POKEGAME_SNAPSHOT").ok()?;
        let (scene, out) = value.split_once(':')?;
        let capture_at = std::env::var("POKEGAME_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(90);
        Some(Self { scene: scene.into(), out: out.into(), frame: 0, capture_at, target: Handle::default() })
    }

    pub fn initial_state(&self) -> AppState {
        match self.scene.as_str() {
            "title" => AppState::Title,
            "battle" | "trainer" => AppState::Battle,
            "gallery" | "gallery-back" => AppState::Gallery,
            _ if self.closeup().is_some() => AppState::Gallery,
            _ => AppState::Overworld,
        }
    }

    /// `p<especie>a<grados>`: primer plano de un Pokémon girado esos grados.
    fn closeup(&self) -> Option<(usize, f32)> {
        let rest = self.scene.strip_prefix('p')?;
        let (species, angle) = rest.split_once('a')?;
        Some((species.parse().ok()?, angle.parse::<f32>().ok()?.to_radians()))
    }
}

pub struct SnapshotPlugin(pub Snapshot);

/// Una partida a medias para las capturas de menús.
fn demo(app: &mut App) {
    let mut team = vec![Creature::new(CHARMANDER, 24), Creature::new(PIKACHU, 21), Creature::new(GEODUDE, 19), Creature::new(STARYU, 18)];
    team[1].hp = 20;
    team[2].hp = 6;
    team[1].xp = 120;
    let mut bag = Bag::default();
    for (item, n) in [(SUPER_BALL, 4), (ULTRA_BALL, 1), (SUPER_POTION, 3), (REVIVE, 1), (REPEL, 2), (ESCAPE_ROPE, 1), (RARE_CANDY, 1)] {
        bag.add(item, n);
    }
    let badges = std::env::var("POKEGAME_BADGES").ok().and_then(|v| v.parse().ok()).unwrap_or(0b0000_0111);
    let mut progress = Progress { money: 8420, badges, seconds: 3.0 * 3600.0 + 25.0 * 60.0, steps: 5210, ..Progress::default() };
    for s in [CHARMANDER, PIKACHU, GEODUDE, STARYU, ONIX, ZUBAT, CLEFAIRY, PIDGEY, RATTATA, CATERPIE, WEEDLE, STARMIE, ODDISH] {
        progress.see(s);
    }
    for s in [CHARMANDER, PIKACHU, GEODUDE, STARYU, PIDGEY, ZUBAT] {
        progress.catch(s);
    }
    app.insert_resource(Party(team)).insert_resource(bag).insert_resource(progress).insert_resource(Companion(Some(1)));
}

impl Plugin for SnapshotPlugin {
    fn build(&self, app: &mut App) {
        let mut snapshot = self.0.clone();
        let mut image = Image::new_target_texture(WIDTH, HEIGHT, TextureFormat::Rgba8UnormSrgb, None);
        image.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
        snapshot.target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let scene = snapshot.scene.as_str();
        if !matches!(scene, "battle" | "title") && snapshot.closeup().is_none() {
            demo(app);
        }

        // Dónde empezar
        let place = std::env::var("POKEGAME_PLACE").ok().and_then(|v| {
            let mut it = v.split(',').map(|n| n.trim().parse::<i32>());
            Some((it.next()?.ok()? as usize, IVec2::new(it.next()?.ok()?, it.next()?.ok()?)))
        });
        let map = std::env::var("POKEGAME_MAP").ok().and_then(|v| v.parse::<usize>().ok());
        let start = place.or_else(|| {
            let id = map.or(match scene {
                "overworld" | "menu" | "start" | "bag" | "profile" | "dex" | "save" => Some(kanto::PALLET),
                "shop" => Some(kanto::MART_IN),
                "quiz" => Some(kanto::LAB_IN),
                _ => None,
            })?;
            let m = LoadedMap::load(id);
            let exit = m.def().exits.iter().find(|e| e.to == Link::Back).unwrap_or(&m.def().exits[0]);
            let steps = if id == kanto::PALLET { 6 } else { 2 };
            Some((id, m.exit_tiles(exit.digit)[0] + exit.inward * steps))
        });
        if let Some((id, pos)) = start {
            let progress = app.world().get_resource::<Progress>().cloned().unwrap_or_default();
            let (m, actors) = enter_map(id, &progress);
            app.insert_resource(PlayerPos(pos)).insert_resource(CurrentMap(m)).insert_resource(actors).insert_resource(Trail::at(pos + IVec2::NEG_Y));
        }

        match scene {
            "menu" => {
                app.insert_resource(PartyMenu::at(1));
            }
            "start" => {
                app.insert_resource(GameMenu::Start(2));
            }
            "bag" => {
                app.insert_resource(GameMenu::Bag(Default::default()));
            }
            "profile" => {
                app.insert_resource(GameMenu::Profile);
            }
            "dex" => {
                app.insert_resource(GameMenu::Dex(3));
            }
            "save" => {
                app.insert_resource(GameMenu::Save(crate::ui::menu::SaveScreen::Asking(true)));
            }
            "shop" => {
                app.insert_resource(Shop::default());
            }
            "quiz" => {
                app.insert_resource(Dialog::quiz(npc_id(kanto::LAB_IN, 1), &Progress::default()));
            }
            "trainer" => {
                let (map, index) = std::env::var("POKEGAME_TRAINER")
                    .ok()
                    .and_then(|v| v.split_once(':').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?))))
                    .unwrap_or((kanto::GYM_PEWTER, 0));
                let (m, actors) = enter_map(map, &Progress::default());
                app.insert_resource(CurrentMap(m)).insert_resource(actors).insert_resource(PendingBattle::Trainer(npc_id(map, index)));
            }
            _ => {}
        }
        app.insert_resource(snapshot)
            .add_systems(Update, (attach_target, capture))
            .add_systems(PostUpdate, trace.after(bevy::transform::TransformSystems::Propagate))
            .add_systems(PreUpdate, press_keys.after(bevy::input::InputSystems))
            .add_systems(OnEnter(AppState::Gallery), spawn_gallery)
            .add_systems(Update, gallery_camera.run_if(in_state(AppState::Gallery)));
    }
}

fn attach_target(mut commands: Commands, snapshot: Res<Snapshot>, cameras: Query<Entity, Added<MainCamera>>) {
    for camera in &cameras {
        commands.entity(camera).insert(RenderTarget::Image(snapshot.target.clone().into()));
    }
}

fn capture(mut commands: Commands, mut snapshot: ResMut<Snapshot>, mut exit: MessageWriter<AppExit>) {
    snapshot.frame += 1;
    if snapshot.frame == snapshot.capture_at {
        commands.spawn(Screenshot::image(snapshot.target.clone())).observe(save_to_disk(snapshot.out.clone()));
    }
    if snapshot.frame == snapshot.capture_at + 20 {
        exit.write(AppExit::Success);
    }
}

fn press_keys(snapshot: Res<Snapshot>, mut input: ResMut<ButtonInput<KeyCode>>) {
    let Ok(keys) = std::env::var("POKEGAME_KEYS") else {
        return;
    };
    let mut frame = 30;
    for name in keys.split(',') {
        // "up*20": mantener pulsada 20 fotogramas
        let (name, hold) = name.split_once('*').map_or((name, 5), |(n, h)| (n, h.parse().unwrap_or(5)));
        let key = match name {
            "enter" => KeyCode::Enter,
            "esc" => KeyCode::Escape,
            "z" => KeyCode::KeyZ,
            "x" => KeyCode::KeyX,
            "space" => KeyCode::Space,
            "up" => KeyCode::ArrowUp,
            "down" => KeyCode::ArrowDown,
            "left" => KeyCode::ArrowLeft,
            "right" => KeyCode::ArrowRight,
            "tab" => KeyCode::Tab,
            "q" => KeyCode::KeyQ,
            "e" => KeyCode::KeyE,
            "shift" => KeyCode::ShiftLeft,
            "wait" => {
                frame += hold + 1;
                continue;
            }
            other => panic!("tecla desconocida: {other}"),
        };
        if snapshot.frame == frame {
            input.press(key);
        } else if snapshot.frame == frame + hold {
            input.release(key);
        }
        frame += hold + 1;
    }
}

fn spawn_gallery(mut commands: Commands, mut kit: Kit, snapshot: Res<Snapshot>) {
    use crate::render::models::{Shape, part};
    let ground = kit.piece(&part(Shape::Cylinder, [104, 184, 84], GALLERY - Vec3::Y * 0.1, Vec3::new(24.0, 0.2, 24.0)));
    commands.spawn(ground);
    if let Some((species, angle)) = snapshot.closeup() {
        kit.spawn_creature(&mut commands, Transform::from_translation(GALLERY).with_rotation(Quat::from_rotation_y(angle)), species);
        return;
    }
    let back = snapshot.scene == "gallery-back";
    for (i, _) in SPECIES.iter().enumerate() {
        let (col, row) = (i % 6, i / 6);
        let pos = GALLERY + Vec3::new((col as f32 - 2.5) * 1.55 + (row % 2) as f32 * 0.4, 0.0, row as f32 * -1.9);
        let angle = if back { std::f32::consts::PI + 0.5 } else { 0.4 };
        kit.spawn_creature(&mut commands, Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(angle)), i);
    }
}

fn gallery_camera(snapshot: Res<Snapshot>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    **camera = if snapshot.closeup().is_some() {
        Transform::from_translation(GALLERY + Vec3::new(0.0, 0.95, 2.4)).looking_at(GALLERY + Vec3::Y * 0.55, Vec3::Y)
    } else {
        Transform::from_translation(GALLERY + Vec3::new(0.0, 4.6, 6.2)).looking_at(GALLERY + Vec3::new(0.0, 0.3, -3.6), Vec3::Y)
    };
}

/// `POKEGAME_TRACE=<archivo>`: apunta cada fotograma el tiempo, la posición del jugador y la
/// de la cámara (para medir que el movimiento sea fluido).
fn trace(
    time: Res<Time>,
    player: Query<&Transform, With<crate::world::overworld::Player>>,
    camera: Query<&Transform, With<MainCamera>>,
    mut out: Local<Option<std::fs::File>>,
) {
    use std::io::Write;
    let Ok(path) = std::env::var("POKEGAME_TRACE") else { return };
    let file = out.get_or_insert_with(|| std::fs::File::create(path).expect("no se pudo crear la traza"));
    let (Ok(p), Ok(c)) = (player.single(), camera.single()) else { return };
    let (p, c) = (p.translation, c.translation);
    writeln!(file, "{} {} {} {} {} {} {}", time.delta_secs(), p.x, p.y, p.z, c.x, c.y, c.z).ok();
}
