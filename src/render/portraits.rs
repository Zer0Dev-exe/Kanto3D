//! Retratos para la interfaz: cada especie (y el jugador) posa en un "estudio" lejos del mapa
//! y una cámara propia la renderiza en vivo a una imagen con fondo transparente.
//! Solo se encienden las cámaras de los retratos que se están viendo.

use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::pokemon::{Party, SPECIES};
use crate::ui::menu::GameMenu;
use crate::render::models::{Kit, PLAYER_OUTFIT};
use crate::game::progress::Progress;

const STUDIO: Vec3 = Vec3::new(1000.0, 0.0, 0.0);
const SIZE: u32 = 192;

/// Imagen en vivo de cada especie (mismo orden que `SPECIES`) y, al final, la del jugador.
#[derive(Resource)]
pub struct Portraits(pub Vec<Handle<Image>>);

impl Portraits {
    pub fn player(&self) -> Handle<Image> {
        self.0[SPECIES.len()].clone()
    }
}

#[derive(Component)]
struct Pose(f32);

#[derive(Component)]
struct StudioCamera(usize);

pub struct PortraitsPlugin;

impl Plugin for PortraitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, (pose, switch_cameras));
    }
}

fn setup(mut commands: Commands, mut kit: Kit, mut images: ResMut<Assets<Image>>) {
    let mut handles = vec![];
    for i in 0..=SPECIES.len() {
        let spot = STUDIO + Vec3::X * (i as f32 * 10.0);
        let mut image = Image::new_target_texture(SIZE, SIZE, TextureFormat::Rgba8UnormSrgb, None);
        image.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
        let handle = images.add(image);

        let pose = (Transform::from_translation(spot), Pose(i as f32 * 1.3));
        let (eye, target) = if i < SPECIES.len() {
            kit.spawn_creature(&mut commands, pose, i);
            (Vec3::new(0.0, 0.8, 1.95), Vec3::Y * 0.6)
        } else {
            kit.spawn_character(&mut commands, pose, &PLAYER_OUTFIT);
            (Vec3::new(0.0, 0.75, 1.75), Vec3::Y * 0.55)
        };
        commands.spawn((
            Camera3d::default(),
            Camera { order: -1 - i as isize, clear_color: ClearColorConfig::Custom(Color::NONE), is_active: false, ..default() },
            RenderTarget::Image(handle.clone().into()),
            Tonemapping::None,
            DebandDither::Disabled,
            StudioCamera(i),
            Transform::from_translation(spot + eye).looking_at(spot + target, Vec3::Y),
        ));
        handles.push(handle);
    }
    commands.insert_resource(Portraits(handles));
}

/// Giran despacio de un lado a otro para que el retrato esté vivo.
fn pose(time: Res<Time>, mut q: Query<(&Pose, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (pose, mut transform) in &mut q {
        transform.rotation = Quat::from_rotation_y((t * 0.8 + pose.0).sin() * 0.6 + 0.25);
    }
}

/// Enciende solo las cámaras de los retratos visibles: tu equipo siempre; la Pokédex, los vistos;
/// la ficha, el jugador.
fn switch_cameras(
    party: Res<Party>,
    menu: Option<Res<GameMenu>>,
    progress: Res<Progress>,
    mut cameras: Query<(&StudioCamera, &mut Camera)>,
) {
    let dex = matches!(menu.as_deref(), Some(GameMenu::Dex(_)));
    let profile = matches!(menu.as_deref(), Some(GameMenu::Profile));
    for (studio, mut camera) in &mut cameras {
        let i = studio.0;
        let wanted = if i == SPECIES.len() {
            profile
        } else {
            party.0.iter().any(|c| c.species == i) || (dex && progress.seen.get(i))
        };
        if camera.is_active != wanted {
            camera.is_active = wanted;
        }
    }
}
