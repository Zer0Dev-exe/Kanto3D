//! Kanto3D: un juego de Pokémon en 3D hecho con Bevy. Ver README.md.

mod battle;
mod dev;
mod game;
mod pokemon;
mod render;
mod ui;
mod world;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Title,
    Overworld,
    Battle,
    /// Solo para el modo captura: los Pokémon en fila.
    Gallery,
}

pub const FONT: &[u8] = include_bytes!("../assets/fonts/AdwaitaSans-Regular.ttf");

pub const SKY: Color = Color::srgb(0.55, 0.78, 0.98);

fn main() {
    let snapshot = dev::snapshot::Snapshot::from_env();
    let mut app = App::new();

    if snapshot.is_some() {
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, ..default() })
                .set(RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
                .disable::<WinitPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)),
        ));
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kanto3D".into(),
                resolution: (dev::snapshot::WIDTH, dev::snapshot::HEIGHT).into(),
                ..default()
            }),
            ..default()
        }));
    }

    app.insert_resource(ClearColor(SKY))
        .insert_resource(GlobalAmbientLight { color: Color::WHITE, brightness: 500.0, ..default() })
        .add_plugins(MaterialPlugin::<render::models::Toon>::default())
        .init_resource::<render::models::KitCache>()
        .init_resource::<render::models::CreatureMeshes>()
        .add_systems(Update, render::models::flicker)
        .insert_resource(pokemon::Party(vec![pokemon::Creature::new(pokemon::CHARMANDER, 5)]))
        .init_resource::<game::items::Bag>()
        .init_resource::<game::progress::Progress>()
        .add_systems(Update, game::progress::tick_time.run_if(in_state(AppState::Overworld).or_else(in_state(AppState::Battle))))
        .insert_state(snapshot.as_ref().map(|s| s.initial_state()).unwrap_or_default())
        .add_systems(Startup, setup)
        .add_plugins((
            world::overworld::OverworldPlugin,
            battle::BattlePlugin,
            ui::dialog::DialogPlugin,
            ui::party::PartyPlugin,
            ui::menu::MenuPlugin,
            ui::title::TitlePlugin,
            render::camera::CameraPlugin,
            render::portraits::PortraitsPlugin,
        ));

    // Fuente con todos los caracteres del español (la de Bevy por defecto no tiene ni la ñ)
    app.world_mut()
        .resource_mut::<Assets<Font>>()
        .insert(AssetId::default(), Font::from_bytes(FONT.to_vec()))
        .expect("no se pudo registrar la fuente");

    if let Some(snapshot) = snapshot {
        app.add_plugins(dev::snapshot::SnapshotPlugin(snapshot));
    }
    app.run();
}

#[cfg(test)]
mod tests {
    /// Todos los caracteres de los textos del juego tienen que existir en la fuente,
    /// si no saldrían como cuadrados vacíos.
    #[test]
    fn font_has_every_character_used_in_the_game() {
        let face = ttf_parser::Face::parse(super::FONT, 0).unwrap();
        let mut dirs = vec![std::path::PathBuf::from("src")];
        let mut files = vec![];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() { dirs.push(path) } else { files.push(path) }
            }
        }
        for path in files {
            let source = std::fs::read_to_string(&path).unwrap();
            for line in source.lines().filter(|l| !l.trim_start().starts_with("//")) {
                let code = line.split(" //").next().unwrap();
                for (i, literal) in code.split('"').enumerate() {
                    if i % 2 == 1 {
                        for c in literal.chars().filter(|c| !c.is_control() && *c != '\\') {
                            assert!(face.glyph_index(c).is_some(), "{}: '{c}' no está en la fuente ({line})", path.display());
                        }
                    }
                }
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        render::camera::MainCamera,
        Transform::default(),
        IsDefaultUiCamera,
        // Sin tonemapping: el sombreado toon ya da los colores finales, así salen tal cual
        Tonemapping::None,
        DebandDither::Disabled,
        // Solo brilla lo que emite luz (llamas, mejillas...)
        Bloom {
            intensity: 0.25,
            prefilter: BloomPrefilter { threshold: 1.2, threshold_softness: 0.2 },
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        },
        DistanceFog {
            color: SKY,
            falloff: FogFalloff::Linear { start: 14.0, end: 32.0 },
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, ..default() },
        world::overworld::SunLight,
        Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
