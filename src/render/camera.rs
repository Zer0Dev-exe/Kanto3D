//! Cámara orbital: Q/E giran 90º, arrastrar con el botón derecho (o central) del ratón gira
//! libremente y cambia la inclinación, la rueda (o +/-) acerca y aleja, y R la deja como al principio.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

use crate::AppState;

/// La cámara principal (la que ve el jugador; las de los retratos no la llevan).
#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct CameraRig {
    yaw: f32,
    yaw_target: f32,
    pitch: f32,
    distance: f32,
    distance_target: f32,
    /// En combate se gira alrededor de la arena, aparte.
    battle_yaw: f32,
    battle_yaw_target: f32,
}

const PITCH: f32 = 0.86;
const DISTANCE: f32 = 11.0;

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            yaw_target: 0.0,
            pitch: PITCH,
            distance: DISTANCE,
            distance_target: DISTANCE,
            battle_yaw: 0.0,
            battle_yaw_target: 0.0,
        }
    }
}

impl CameraRig {
    /// Desde el punto que se mira hasta la cámara.
    pub fn offset(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * Vec3::new(0.0, self.pitch.sin(), self.pitch.cos()) * self.distance
    }

    /// Hacia dónde mira la cámara, sobre el suelo.
    pub fn forward(&self) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * Vec3::NEG_Z
    }

    pub fn battle_yaw(&self) -> f32 {
        self.battle_yaw
    }

    /// Convierte una dirección de pantalla (arriba = +Y) en una dirección del mapa, según hacia
    /// dónde mire la cámara (redondeado al cuarto de vuelta más cercano).
    pub fn screen_to_map(&self, dir: IVec2) -> IVec2 {
        let quarters = (self.yaw / FRAC_PI_2).round() as i32;
        let mut d = dir;
        for _ in 0..quarters.rem_euclid(4) {
            d = IVec2::new(-d.y, d.x);
        }
        d
    }

    pub fn reset_battle(&mut self) {
        self.battle_yaw = 0.0;
        self.battle_yaw_target = 0.0;
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRig>()
            .add_systems(OnEnter(AppState::Battle), |mut rig: ResMut<CameraRig>| rig.reset_battle())
            .add_systems(Update, rig_input);
    }
}

fn rig_input(
    time: Res<Time>,
    state: Res<State<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    motion: Option<Res<AccumulatedMouseMotion>>,
    scroll: Option<Res<AccumulatedMouseScroll>>,
    mut rig: ResMut<CameraRig>,
) {
    let battle = match state.get() {
        AppState::Battle => true,
        AppState::Overworld => false,
        AppState::Gallery | AppState::Title => return,
    };
    let drag = buttons.is_some_and(|b| b.any_pressed([MouseButton::Right, MouseButton::Middle]));
    let delta = motion.map(|m| m.delta).unwrap_or_default();
    let wheel = scroll.map(|s| s.delta.y).unwrap_or_default();

    let (step, yaw_target) = if battle {
        (0.5, &mut rig.battle_yaw_target)
    } else {
        (FRAC_PI_2, &mut rig.yaw_target)
    };
    if keys.just_pressed(KeyCode::KeyQ) {
        *yaw_target -= step;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        *yaw_target += step;
    }
    if drag {
        *yaw_target -= delta.x * 0.008;
    }
    if battle {
        rig.battle_yaw_target = rig.battle_yaw_target.clamp(-1.4, 1.4);
    }

    if drag && !battle {
        rig.pitch = (rig.pitch + delta.y * 0.006).clamp(0.4, 1.35);
    }
    let zoom = wheel + if keys.pressed(KeyCode::Equal) { 4.0 * time.delta_secs() } else { 0.0 }
        - if keys.pressed(KeyCode::Minus) { 4.0 * time.delta_secs() } else { 0.0 };
    rig.distance_target = (rig.distance_target - zoom * 0.8).clamp(4.5, 15.0);

    if keys.just_pressed(KeyCode::KeyR) {
        // Vuelve a la orientación de inicio más cercana (sin dar vueltas de más)
        let turns = (rig.yaw_target / TAU).round();
        *rig = CameraRig { yaw: rig.yaw, distance: rig.distance, yaw_target: turns * TAU, ..default() };
    }

    let k = 1.0 - (-10.0 * time.delta_secs()).exp();
    rig.yaw += (rig.yaw_target - rig.yaw) * k;
    rig.distance += (rig.distance_target - rig.distance) * k;
    rig.battle_yaw += (rig.battle_yaw_target - rig.battle_yaw) * k;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_follow_the_camera() {
        let mut rig = CameraRig::default();
        assert_eq!(rig.screen_to_map(IVec2::Y), IVec2::Y);
        // Cámara al este mirando al oeste: "arriba" es el oeste y "derecha" es el norte
        rig.yaw = FRAC_PI_2;
        assert_eq!(rig.screen_to_map(IVec2::Y), IVec2::NEG_X);
        assert_eq!(rig.screen_to_map(IVec2::X), IVec2::Y);
        rig.yaw = -FRAC_PI_2;
        assert_eq!(rig.screen_to_map(IVec2::Y), IVec2::X);
    }
}
