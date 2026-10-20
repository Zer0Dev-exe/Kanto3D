//! Los Pokémon esculpidos con SDF (ver `sdf.rs`). Todos miran hacia +Z con los pies en y = 0.
//! El cuerpo es una sola malla lisa; ojos, boca, garras y llamas son piezas aparte
//! que se pegan a la superficie lanzando rayos contra el modelo.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::render::models::{Part, Rgb, Shape, part};
use crate::pokemon::sdf::{Sdf, ellipsoid, ellipsoid_rot, limb};

pub struct CreatureModel {
    pub body: Sdf,
    pub details: Vec<Part>,
    /// Llamas (posición de la base y tamaño): la cola de Charmander, las crines de Ponyta...
    pub flames: Vec<(Vec3, f32)>,
}

/// Llama en forma de gota: amarilla abajo, naranja en medio y roja en la punta.
pub fn flame() -> Sdf {
    Sdf::new()
        .add(ellipsoid(v(0.0, 0.09, 0.0), Vec3::splat(0.11)), [255, 235, 120], 0.0)
        .add(limb(v(0.0, 0.1, 0.0), v(0.0, 0.36, 0.0), 0.1, 0.008), [255, 235, 120], 0.04)
        .paint(ellipsoid(v(0.0, 0.25, 0.0), v(0.16, 0.12, 0.16)), [255, 150, 30])
        .paint(ellipsoid(v(0.0, 0.37, 0.0), v(0.12, 0.1, 0.12)), [255, 60, 20])
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

const WHITE: Rgb = [255, 255, 255];
const INK: Rgb = [28, 22, 30];
const MOUTH: Rgb = [90, 25, 35];

/// Punto de la superficie visto de frente en (x, y), y la rotación que pone +Z sobre su normal.
fn surf(sdf: &Sdf, x: f32, y: f32) -> (Vec3, Quat) {
    surf_dir(sdf, v(x, y, 3.0), Vec3::NEG_Z)
}

fn surf_dir(sdf: &Sdf, from: Vec3, dir: Vec3) -> (Vec3, Quat) {
    let (p, n) = sdf.hit(from, dir).unwrap_or_else(|| panic!("el rayo desde {from} no toca el modelo"));
    (p, Quat::from_rotation_arc(Vec3::Z, n))
}

/// Pieza plana pegada a la superficie, girada `roll` alrededor de la normal.
fn decal(sdf: &Sdf, x: f32, y: f32, size: Vec3, color: Rgb, roll: f32) -> Part {
    let (p, rot) = surf(sdf, x, y);
    let rot = rot * Quat::from_rotation_z(roll);
    part(Shape::Sphere, color, p + rot * v(0.0, 0.0, size.z * 0.2), size).rot(rot).flat()
}

/// Ojo estilo anime: blanco con contorno, iris, pupila y dos brillos.
fn eye(sdf: &Sdf, x: f32, y: f32, w: f32, h: f32, iris: Rgb) -> Vec<Part> {
    let (p, rot) = surf(sdf, x, y);
    let at = |off: Vec3| p + rot * off;
    vec![
        part(Shape::Sphere, INK, at(v(0.0, 0.0, -0.012)), v(w + 0.026, h + 0.026, 0.05)).rot(rot).flat(),
        part(Shape::Sphere, WHITE, at(v(0.0, 0.0, -0.005)), v(w, h, 0.05)).rot(rot).flat(),
        part(Shape::Sphere, iris, at(v(0.0, -h * 0.06, 0.012)), v(w * 0.78, h * 0.8, 0.03)).rot(rot).flat(),
        part(Shape::Sphere, INK, at(v(0.0, -h * 0.1, 0.018)), v(w * 0.42, h * 0.5, 0.024)).rot(rot).flat(),
        part(Shape::Sphere, WHITE, at(v(-w * 0.16, h * 0.2, 0.025)), v(w * 0.32, w * 0.32, 0.02)).rot(rot).flat(),
        part(Shape::Sphere, WHITE, at(v(w * 0.18, -h * 0.22, 0.024)), v(w * 0.14, w * 0.14, 0.02)).rot(rot).flat(),
    ]
}

fn eyes(sdf: &Sdf, x: f32, y: f32, w: f32, h: f32, iris: Rgb) -> Vec<Part> {
    let mut parts = eye(sdf, -x, y, w, h, iris);
    parts.extend(eye(sdf, x, y, w, h, iris));
    parts
}

/// Sonrisa: dos trazos curvados que se juntan en el centro.
fn smile(sdf: &Sdf, y: f32, width: f32) -> Vec<Part> {
    [-1.0, 1.0]
        .map(|s| decal(sdf, s * width * 0.24, y, v(width * 0.55, 0.018, 0.03), MOUTH, s * 0.3))
        .into()
}

/// Garritas cónicas saliendo de la superficie en (x, y) vista de frente.
fn claw(sdf: &Sdf, x: f32, y: f32) -> Part {
    let (p, rot) = surf(sdf, x, y);
    let dir = rot * Vec3::Z;
    let up = Quat::from_rotation_arc(Vec3::Y, (dir + Vec3::NEG_Y * 0.3).normalize());
    part(Shape::Cone, [250, 245, 230], p + dir * 0.012, v(0.035, 0.05, 0.035)).rot(up)
}

// ---------------------------------------------------------------- Charmander

pub fn charmander() -> CreatureModel {
    let (o, cream) = ([240, 128, 48], [252, 222, 150]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.42, 0.0), v(0.27, 0.32, 0.25)), o, 0.0)
        .add(ellipsoid(v(0.0, 0.86, 0.04), v(0.3, 0.27, 0.27)), o, 0.08)
        .add(ellipsoid(v(0.0, 0.79, 0.2), v(0.2, 0.14, 0.15)), o, 0.06);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.2, 0.56, 0.05), v(s * 0.34, 0.42, 0.15), 0.07, 0.055), o, 0.04)
            .add(limb(v(s * 0.14, 0.26, 0.0), v(s * 0.17, 0.08, 0.04), 0.11, 0.09), o, 0.05)
            .add(ellipsoid(v(s * 0.17, 0.05, 0.09), v(0.1, 0.06, 0.13)), o, 0.03);
    }
    let sdf = sdf
        .add(limb(v(0.0, 0.25, -0.18), v(0.1, 0.2, -0.42), 0.1, 0.08), o, 0.05)
        .add(limb(v(0.1, 0.2, -0.42), v(0.28, 0.28, -0.52), 0.08, 0.065), o, 0.03)
        .add(limb(v(0.28, 0.28, -0.52), v(0.37, 0.5, -0.47), 0.065, 0.05), o, 0.03)
        .paint(ellipsoid(v(0.0, 0.38, 0.2), v(0.19, 0.25, 0.15)), cream);

    let mut details = eyes(&sdf, 0.125, 0.9, 0.11, 0.15, [40, 95, 165]);
    details.extend(smile(&sdf, 0.745, 0.17));
    details.push(decal(&sdf, -0.045, 0.83, v(0.018, 0.012, 0.01), INK, 0.0));
    details.push(decal(&sdf, 0.045, 0.83, v(0.018, 0.012, 0.01), INK, 0.0));
    for s in [-1.0, 1.0] {
        for dx in [-0.05, 0.0, 0.05] {
            details.push(claw(&sdf, s * 0.17 + dx, 0.05));
        }
    }
    CreatureModel { body: sdf, details, flames: vec![(v(0.37, 0.52, -0.47), 1.0)] }
}

// ---------------------------------------------------------------- Squirtle

pub fn squirtle() -> CreatureModel {
    let (skin, shell, rim, plate, line) =
        ([120, 195, 235], [165, 102, 55], [245, 232, 180], [240, 212, 125], [130, 85, 45]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.4, 0.0), v(0.25, 0.3, 0.24)), skin, 0.0)
        .add(ellipsoid(v(0.0, 0.42, -0.11), v(0.3, 0.33, 0.2)), shell, 0.02)
        .add(ellipsoid(v(0.0, 0.42, -0.04), v(0.32, 0.35, 0.08)), rim, 0.02)
        .add(ellipsoid(v(0.0, 0.88, 0.04), v(0.27, 0.25, 0.25)), skin, 0.06);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.2, 0.53, 0.06), v(s * 0.34, 0.41, 0.14), 0.075, 0.06), skin, 0.04)
            .add(limb(v(s * 0.13, 0.22, 0.02), v(s * 0.15, 0.08, 0.05), 0.1, 0.085), skin, 0.05)
            .add(ellipsoid(v(s * 0.15, 0.05, 0.1), v(0.09, 0.06, 0.12)), skin, 0.03);
    }
    let sdf = sdf
        // Cola rizada: sale hacia atrás y se enrosca hacia arriba
        .add(limb(v(0.0, 0.22, -0.26), v(0.0, 0.13, -0.46), 0.065, 0.07), skin, 0.03)
        .add(limb(v(0.0, 0.13, -0.46), v(0.0, 0.2, -0.64), 0.07, 0.07), skin, 0.02)
        .add(limb(v(0.0, 0.2, -0.64), v(0.0, 0.4, -0.68), 0.07, 0.065), skin, 0.02)
        .add(limb(v(0.0, 0.4, -0.68), v(0.0, 0.52, -0.56), 0.065, 0.055), skin, 0.02)
        .add(limb(v(0.0, 0.52, -0.56), v(0.0, 0.44, -0.47), 0.055, 0.045), skin, 0.02)
        .paint(ellipsoid(v(0.0, 0.36, -0.58), v(0.08, 0.1, 0.08)), [85, 160, 210])
        .paint(ellipsoid(v(0.0, 0.37, 0.17), v(0.2, 0.25, 0.12)), plate)
        .paint(ellipsoid(v(0.0, 0.43, 0.22), v(0.2, 0.011, 0.1)), line)
        .paint(ellipsoid(v(0.0, 0.33, 0.22), v(0.19, 0.011, 0.1)), line)
        .paint(ellipsoid(v(0.0, 0.38, 0.24), v(0.011, 0.22, 0.08)), line)
        // Placas del caparazón
        .paint(ellipsoid(v(0.0, 0.42, -0.3), v(0.013, 0.24, 0.07)), [120, 72, 36])
        .paint(ellipsoid(v(0.0, 0.52, -0.28), v(0.26, 0.013, 0.08)), [120, 72, 36])
        .paint(ellipsoid(v(0.0, 0.32, -0.28), v(0.26, 0.013, 0.08)), [120, 72, 36]);

    let mut details = eyes(&sdf, 0.11, 0.92, 0.1, 0.13, [150, 55, 45]);
    details.extend(smile(&sdf, 0.8, 0.16));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Bulbasaur

pub fn bulbasaur() -> CreatureModel {
    let (body, spot, bulb, seam) = ([115, 198, 168], [62, 140, 118], [92, 178, 82], [50, 128, 58]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.36, -0.08), v(0.3, 0.24, 0.34)), body, 0.0)
        .add(ellipsoid(v(0.0, 0.52, 0.28), v(0.32, 0.24, 0.24)), body, 0.08)
        .add(ellipsoid(v(0.0, 0.44, 0.36), v(0.26, 0.13, 0.16)), body, 0.05);
    for s in [-1.0, 1.0] {
        sdf = sdf.add(limb(v(s * 0.18, 0.66, 0.22), v(s * 0.29, 0.83, 0.17), 0.07, 0.02), body, 0.03);
        for z in [0.12, -0.26] {
            sdf = sdf
                .add(limb(v(s * 0.2, 0.3, z), v(s * 0.22, 0.08, z + 0.02), 0.1, 0.09), body, 0.04)
                .add(ellipsoid(v(s * 0.22, 0.05, z + 0.05), v(0.1, 0.06, 0.11)), body, 0.03);
        }
    }
    sdf = sdf
        .add(ellipsoid(v(0.0, 0.74, -0.14), v(0.31, 0.29, 0.31)), bulb, 0.05)
        .add(limb(v(0.0, 0.86, -0.14), v(0.0, 1.12, -0.17), 0.24, 0.03), bulb, 0.1);
    for i in 0..4 {
        let rot = Quat::from_rotation_y(i as f32 * PI / 4.0 + 0.4) * Quat::from_rotation_x(0.12);
        sdf = sdf.paint(ellipsoid_rot(v(0.0, 0.9, -0.14), v(0.012, 0.36, 0.34), rot), seam);
    }
    for (c, r) in [
        (v(-0.16, 0.64, 0.34), 0.05),
        (v(0.2, 0.6, 0.32), 0.04),
        (v(0.27, 0.42, -0.05), 0.06),
        (v(-0.26, 0.44, -0.14), 0.05),
        (v(0.1, 0.56, -0.38), 0.05),
    ] {
        sdf = sdf.paint(ellipsoid(c, Vec3::splat(r)), spot);
    }

    let mut details = eyes(&sdf, 0.16, 0.56, 0.1, 0.12, [205, 45, 55]);
    details.extend(smile(&sdf, 0.415, 0.26));
    details.push(decal(&sdf, -0.04, 0.5, v(0.016, 0.012, 0.01), INK, 0.0));
    details.push(decal(&sdf, 0.04, 0.5, v(0.016, 0.012, 0.01), INK, 0.0));
    for s in [-1.0, 1.0] {
        for dx in [-0.045, 0.045] {
            details.push(claw(&sdf, s * 0.22 + dx, 0.05));
        }
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Pikachu

pub fn pikachu() -> CreatureModel {
    let (y, tip, brown) = ([250, 214, 50], [35, 30, 30], [150, 95, 40]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.38, 0.0), v(0.25, 0.3, 0.22)), y, 0.0)
        .add(ellipsoid(v(0.0, 0.8, 0.03), v(0.28, 0.25, 0.24)), y, 0.07);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.14, 0.98, 0.0), v(s * 0.33, 1.38, -0.05), 0.07, 0.016), y, 0.03)
            .add(limb(v(s * 0.17, 0.5, 0.1), v(s * 0.22, 0.42, 0.2), 0.055, 0.045), y, 0.03)
            .add(limb(v(s * 0.13, 0.2, 0.0), v(s * 0.13, 0.07, 0.04), 0.09, 0.07), y, 0.04)
            .add(ellipsoid(v(s * 0.13, 0.05, 0.08), v(0.08, 0.05, 0.12)), y, 0.03)
            .paint(ellipsoid(v(s * 0.3, 1.33, -0.045), Vec3::splat(0.085)), tip);
    }
    // Cola en forma de rayo, plana en X
    let sdf = sdf
        .add(limb(v(0.0, 0.25, -0.18), v(0.0, 0.45, -0.4), 0.045, 0.055), brown, 0.02)
        .add(ellipsoid_rot(v(0.0, 0.58, -0.44), v(0.04, 0.18, 0.08), Quat::from_rotation_x(0.75)), y, 0.01)
        .add(ellipsoid_rot(v(0.0, 0.78, -0.56), v(0.04, 0.17, 0.11), Quat::from_rotation_x(-0.7)), y, 0.01)
        .add(ellipsoid_rot(v(0.0, 1.04, -0.62), v(0.045, 0.24, 0.2), Quat::from_rotation_x(0.4)), y, 0.01)
        .paint(ellipsoid(v(0.0, 0.48, -0.22), v(0.22, 0.025, 0.09)), brown)
        .paint(ellipsoid(v(0.0, 0.37, -0.22), v(0.22, 0.025, 0.09)), brown);

    let mut details = eyes(&sdf, 0.12, 0.84, 0.085, 0.1, [30, 25, 25]);
    for s in [-1.0, 1.0] {
        let (p, rot) = surf(&sdf, s * 0.19, 0.72);
        details.push(part(Shape::Sphere, [235, 60, 50], p, v(0.11, 0.11, 0.035)).rot(rot).glow());
        // Boca en "ω": dos curvas pequeñas bajo la nariz
        details.push(decal(&sdf, s * 0.026, 0.738, v(0.05, 0.016, 0.02), MOUTH, s * 0.55));
    }
    details.push(decal(&sdf, 0.0, 0.775, v(0.02, 0.013, 0.012), INK, 0.0));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Rattata

pub fn rattata() -> CreatureModel {
    let (fur, cream, pink) = ([150, 105, 175], [240, 225, 190], [240, 160, 180]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.32, -0.08), v(0.24, 0.25, 0.33)), fur, 0.0)
        .add(ellipsoid(v(0.0, 0.58, 0.22), v(0.25, 0.24, 0.24)), fur, 0.07)
        .add(ellipsoid(v(0.0, 0.5, 0.4), v(0.13, 0.1, 0.11)), fur, 0.04);
    for s in [-1.0, 1.0] {
        let ear = Quat::from_rotation_z(-s * 0.3);
        sdf = sdf
            .add(ellipsoid_rot(v(s * 0.22, 0.84, 0.16), v(0.13, 0.14, 0.035), ear), fur, 0.02)
            .paint(ellipsoid_rot(v(s * 0.22, 0.84, 0.19), v(0.09, 0.1, 0.03), ear), pink);
        for (z, foot_z) in [(0.12, 0.16), (-0.26, -0.24)] {
            sdf = sdf
                .add(limb(v(s * 0.15, 0.2, z), v(s * 0.15, 0.06, foot_z), 0.07, 0.05), fur, 0.04)
                .add(ellipsoid(v(s * 0.15, 0.04, foot_z + 0.03), v(0.07, 0.045, 0.1)), fur, 0.02);
        }
    }
    let sdf = sdf
        .add(limb(v(0.0, 0.25, -0.38), v(0.0, 0.16, -0.58), 0.04, 0.032), fur, 0.02)
        .add(limb(v(0.0, 0.16, -0.58), v(0.0, 0.18, -0.76), 0.032, 0.028), fur, 0.02)
        .add(limb(v(0.0, 0.18, -0.76), v(0.0, 0.31, -0.86), 0.028, 0.025), fur, 0.02)
        .add(limb(v(0.0, 0.31, -0.86), v(0.0, 0.44, -0.82), 0.025, 0.022), fur, 0.02)
        .add(limb(v(0.0, 0.44, -0.82), v(0.0, 0.49, -0.71), 0.022, 0.02), fur, 0.02)
        .paint(ellipsoid(v(0.0, 0.25, 0.08), v(0.18, 0.17, 0.24)), cream)
        .paint(ellipsoid(v(0.0, 0.46, 0.43), v(0.12, 0.08, 0.1)), cream);

    let mut details = eyes(&sdf, 0.11, 0.65, 0.085, 0.105, [200, 30, 60]);
    details.push(decal(&sdf, 0.0, 0.53, v(0.05, 0.04, 0.04), pink, 0.0));
    for s in [-1.0, 1.0] {
        let (p, rot) = surf(&sdf, s * 0.018, 0.425);
        details.push(part(Shape::Cube, WHITE, p + rot * v(0.0, -0.025, 0.0), v(0.03, 0.05, 0.015)).rot(rot).flat());
        for (dy, angle) in [(0.01, 0.12), (-0.02, -0.08)] {
            let (p, _) = surf(&sdf, s * 0.11, 0.5 + dy);
            let rot = Quat::from_rotation_z(s * angle);
            details.push(part(Shape::Cube, INK, p + v(s * 0.07, 0.0, 0.0), v(0.16, 0.01, 0.01)).rot(rot).flat());
        }
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Pidgey

pub fn pidgey() -> CreatureModel {
    let (brown, dark, cream, mark) = ([178, 128, 78], [140, 95, 55], [245, 225, 180], [40, 30, 30]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.4, -0.02), v(0.25, 0.27, 0.28)), brown, 0.0)
        .add(ellipsoid(v(0.0, 0.76, 0.08), v(0.22, 0.2, 0.21)), brown, 0.07)
        .add(limb(v(0.0, 0.92, 0.12), v(0.0, 1.08, 0.08), 0.045, 0.015), brown, 0.03)
        .add(limb(v(0.0, 0.93, 0.05), v(0.0, 1.05, -0.04), 0.04, 0.012), brown, 0.03)
        .add(ellipsoid_rot(v(0.0, 0.3, -0.36), v(0.12, 0.04, 0.17), Quat::from_rotation_x(-0.5)), dark, 0.03);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(
                ellipsoid_rot(v(s * 0.2, 0.44, -0.08), v(0.06, 0.15, 0.22), Quat::from_rotation_x(0.35) * Quat::from_rotation_z(s * 0.15)),
                dark,
                0.03,
            )
            .paint(
                ellipsoid_rot(v(s * 0.15, 0.79, 0.04), v(0.035, 0.03, 0.11), Quat::from_rotation_y(s * 0.5)),
                mark,
            );
    }
    let sdf = sdf
        .paint(ellipsoid(v(0.0, 0.35, 0.15), v(0.18, 0.22, 0.16)), cream)
        .paint(ellipsoid(v(0.0, 0.72, 0.22), v(0.15, 0.11, 0.1)), cream);

    let mut details = eyes(&sdf, 0.09, 0.8, 0.07, 0.08, [30, 25, 25]);
    let (p, rot) = surf(&sdf, 0.0, 0.73);
    let forward = rot * Vec3::Z;
    details.push(
        part(Shape::Cone, [75, 65, 65], p + forward * 0.04, v(0.075, 0.12, 0.06))
            .rot(Quat::from_rotation_arc(Vec3::Y, forward) * Quat::from_rotation_y(FRAC_PI_2)),
    );
    for s in [-1.0, 1.0] {
        let feet = [230, 150, 160];
        details.push(part(Shape::Cylinder, feet, v(s * 0.08, 0.1, 0.04), v(0.035, 0.16, 0.035)));
        details.push(part(Shape::Sphere, feet, v(s * 0.08, 0.02, 0.08), v(0.09, 0.035, 0.12)));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- ayudas para los nuevos

/// Cuerpo de oruga: una fila de bolas que van menguando hacia la cola.
fn segments(mut sdf: Sdf, color: Rgb, start: Vec3, count: usize, r: f32) -> Sdf {
    for i in 0..count {
        let t = i as f32;
        let c = start + v(0.0, (t * 1.3).sin() * 0.02, -t * r * 1.15);
        sdf = sdf.add(ellipsoid(c, Vec3::splat(r * (1.0 - t * 0.07))), color, 0.05);
    }
    sdf
}

/// Estrella de `n` puntas en el plano XY (mirando a +Z).
fn star(mut sdf: Sdf, center: Vec3, radius: f32, turn: f32, color: Rgb, thick: f32) -> Sdf {
    sdf = sdf.add(ellipsoid(center, v(radius * 0.4, radius * 0.4, thick)), color, 0.0);
    for i in 0..5 {
        let a = std::f32::consts::FRAC_PI_2 + turn + i as f32 * std::f32::consts::TAU / 5.0;
        let tip = center + v(a.cos(), a.sin(), 0.0) * radius;
        sdf = sdf.add(limb(center, tip, thick * 1.2, 0.025), color, 0.05);
    }
    sdf
}

/// Gema del centro de Staryu y Starmie: roja con aro dorado.
fn gem(sdf: &Sdf, y: f32, size: f32) -> Vec<Part> {
    let (p, rot) = surf(sdf, 0.0, y);
    vec![
        part(Shape::Cylinder, [250, 205, 70], p + rot * v(0.0, 0.0, 0.01), v(size * 1.35, 0.04, size * 1.35))
            .rot(rot * Quat::from_rotation_x(FRAC_PI_2)),
        part(Shape::Sphere, [235, 40, 60], p + rot * v(0.0, 0.0, 0.03), v(size, size, size * 0.6)).rot(rot).glow(),
    ]
}

fn feet(mut sdf: Sdf, color: Rgb, x: f32, z: f32, size: Vec3) -> Sdf {
    for s in [-1.0, 1.0] {
        sdf = sdf.add(ellipsoid(v(s * x, size.y * 0.8, z), size), color, 0.04);
    }
    sdf
}

/// Pinta de `color` todo lo que quede por encima (`up`) o por debajo de la altura `y`.
fn paint_level(sdf: Sdf, y: f32, up: bool, color: Rgb) -> Sdf {
    let center = if up { y + 4.0 } else { y - 4.0 };
    sdf.paint(ellipsoid(v(0.0, center, 0.0), v(30.0, 4.0, 30.0)), color)
}

fn fang(sdf: &Sdf, x: f32, y: f32) -> Part {
    let (p, rot) = surf(sdf, x, y);
    part(Shape::Cone, WHITE, p + rot * v(0.0, -0.02, 0.01), v(0.04, 0.06, 0.03)).rot(rot * Quat::from_rotation_x(PI))
}

// ---------------------------------------------------------------- Caterpie

pub fn caterpie() -> CreatureModel {
    let (green, belly, red) = ([130, 205, 80], [245, 230, 140], [230, 60, 60]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.56, 0.16), v(0.22, 0.22, 0.2)), green, 0.0);
    sdf = sdf.add(ellipsoid(v(0.0, 0.34, 0.02), Vec3::splat(0.18)), green, 0.06);
    sdf = segments(sdf, green, v(0.0, 0.17, -0.16), 4, 0.15);
    let sdf = sdf
        .add(limb(v(0.0, 0.74, 0.1), v(0.0, 0.86, 0.06), 0.035, 0.03), red, 0.02)
        .add(limb(v(0.0, 0.86, 0.06), v(-0.08, 0.96, 0.08), 0.03, 0.02), red, 0.01)
        .add(limb(v(0.0, 0.86, 0.06), v(0.08, 0.96, 0.08), 0.03, 0.02), red, 0.01)
        .paint(ellipsoid(v(0.0, 0.3, 0.16), v(0.14, 0.12, 0.08)), belly)
        .paint(ellipsoid(v(0.0, 0.48, 0.34), v(0.15, 0.08, 0.06)), belly);
    let mut details = eyes(&sdf, 0.11, 0.6, 0.1, 0.12, INK);
    details.extend(smile(&sdf, 0.47, 0.12));
    for z in [-0.16, -0.34, -0.52] {
        for s in [-1.0, 1.0] {
            details.push(part(Shape::Sphere, [250, 235, 120], v(s * 0.1, 0.04, z + 0.03), v(0.07, 0.05, 0.07)).flat());
        }
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Weedle

pub fn weedle() -> CreatureModel {
    let (brown, pink, grey) = ([215, 150, 70], [240, 120, 140], [200, 200, 210]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.46, 0.14), v(0.2, 0.2, 0.19)), brown, 0.0);
    sdf = sdf.add(ellipsoid(v(0.0, 0.28, 0.0), Vec3::splat(0.16)), brown, 0.06);
    sdf = segments(sdf, brown, v(0.0, 0.15, -0.16), 4, 0.14);
    let sdf = sdf
        .add(ellipsoid(v(0.0, 0.4, 0.32), v(0.1, 0.08, 0.07)), pink, 0.03)
        .add(limb(v(0.0, 0.62, 0.1), v(0.0, 0.88, 0.04), 0.06, 0.006), grey, 0.02)
        .add(limb(v(0.0, 0.2, -0.6), v(0.0, 0.38, -0.72), 0.05, 0.006), grey, 0.02);
    let mut details = eyes(&sdf, 0.1, 0.52, 0.06, 0.07, INK);
    details.extend(smile(&sdf, 0.34, 0.1));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Geodude

pub fn geodude() -> CreatureModel {
    let (rock, dark) = ([155, 150, 140], [120, 115, 108]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.5, 0.0), v(0.36, 0.32, 0.32)), rock, 0.0);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.3, 0.52, 0.0), v(s * 0.5, 0.62, 0.1), 0.09, 0.07), rock, 0.05)
            .add(limb(v(s * 0.5, 0.62, 0.1), v(s * 0.55, 0.45, 0.22), 0.07, 0.07), rock, 0.03)
            .add(ellipsoid(v(s * 0.56, 0.42, 0.26), v(0.13, 0.12, 0.12)), rock, 0.03)
            .paint(ellipsoid_rot(v(s * 0.13, 0.66, 0.26), v(0.12, 0.035, 0.08), Quat::from_rotation_z(s * 0.35)), dark);
    }
    for (c, r) in [(v(0.2, 0.75, 0.1), 0.06), (v(-0.25, 0.35, 0.15), 0.07), (v(0.1, 0.3, -0.25), 0.08), (v(-0.15, 0.7, -0.2), 0.07)] {
        sdf = sdf.paint(ellipsoid(c, Vec3::splat(r)), dark);
    }
    let mut details = eyes(&sdf, 0.13, 0.56, 0.08, 0.06, [40, 30, 30]);
    details.extend(smile(&sdf, 0.42, 0.22));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Onix

pub fn onix() -> CreatureModel {
    let (rock, dark) = ([160, 160, 170], [125, 125, 135]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 1.2, 0.3), v(0.27, 0.23, 0.34)), rock, 0.0);
    let chain = [
        (v(0.0, 0.88, 0.16), 0.2),
        (v(0.0, 0.62, 0.0), 0.22),
        (v(0.0, 0.38, -0.16), 0.21),
        (v(0.0, 0.2, -0.42), 0.19),
        (v(0.12, 0.16, -0.68), 0.16),
        (v(0.3, 0.2, -0.86), 0.13),
        (v(0.42, 0.3, -1.0), 0.1),
    ];
    for (c, r) in chain {
        sdf = sdf.add(ellipsoid(c, Vec3::splat(r)), rock, 0.02);
    }
    sdf = sdf
        .add(limb(v(0.0, 1.38, 0.16), v(0.0, 1.7, -0.04), 0.075, 0.012), rock, 0.03)
        .paint(ellipsoid(v(0.0, 1.1, 0.5), v(0.16, 0.035, 0.06)), dark);
    for (c, _) in chain.iter().step_by(2) {
        sdf = sdf.paint(ellipsoid(*c + v(0.1, 0.1, 0.1), Vec3::splat(0.06)), dark);
    }
    let mut details = eyes(&sdf, 0.12, 1.22, 0.06, 0.06, INK);
    details.extend(smile(&sdf, 1.05, 0.2));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Zubat

pub fn zubat() -> CreatureModel {
    let (blue, purple, inner) = ([80, 135, 215], [160, 110, 200], [205, 160, 225]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.78, 0.0), v(0.17, 0.2, 0.15)), blue, 0.0);
    for s in [-1.0, 1.0] {
        let wing = Quat::from_rotation_z(s * 0.35) * Quat::from_rotation_y(s * 0.25);
        sdf = sdf
            .add(limb(v(s * 0.08, 0.92, 0.0), v(s * 0.17, 1.15, -0.02), 0.06, 0.02), blue, 0.03)
            .paint(ellipsoid(v(s * 0.14, 1.05, 0.03), v(0.04, 0.08, 0.03)), inner)
            .add(ellipsoid_rot(v(s * 0.42, 0.85, -0.02), v(0.3, 0.17, 0.025), wing), purple, 0.04)
            .paint(ellipsoid_rot(v(s * 0.45, 0.83, 0.01), v(0.25, 0.13, 0.02), wing), inner)
            .add(limb(v(s * 0.05, 0.62, 0.0), v(s * 0.07, 0.34, 0.03), 0.026, 0.018), blue, 0.02);
    }
    let mut details = vec![decal(&sdf, 0.0, 0.74, v(0.14, 0.11, 0.04), [90, 25, 45], 0.0)];
    for s in [-1.0, 1.0] {
        details.push(fang(&sdf, s * 0.04, 0.8));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Clefairy

pub fn clefairy() -> CreatureModel {
    let (pink, brown, wing) = ([250, 190, 200], [130, 85, 60], [255, 220, 225]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.38, 0.0), v(0.28, 0.32, 0.26)), pink, 0.0)
        .add(ellipsoid(v(0.0, 0.66, 0.02), v(0.26, 0.22, 0.23)), pink, 0.1);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.13, 0.8, 0.0), v(s * 0.26, 1.04, -0.03), 0.07, 0.015), pink, 0.03)
            .paint(ellipsoid(v(s * 0.24, 1.0, -0.03), Vec3::splat(0.06)), brown)
            .add(limb(v(s * 0.22, 0.42, 0.06), v(s * 0.32, 0.36, 0.14), 0.06, 0.05), pink, 0.03)
            .add(ellipsoid_rot(v(s * 0.18, 0.5, -0.24), v(0.12, 0.08, 0.03), Quat::from_rotation_z(s * 0.5)), wing, 0.02);
    }
    sdf = feet(sdf, pink, 0.13, 0.05, v(0.1, 0.06, 0.13))
        .add(limb(v(0.0, 0.86, 0.12), v(0.0, 0.94, 0.18), 0.03, 0.025), pink, 0.02)
        .add(limb(v(0.0, 0.94, 0.18), v(0.04, 0.9, 0.24), 0.025, 0.02), pink, 0.01);
    let mut details = eyes(&sdf, 0.09, 0.7, 0.06, 0.08, INK);
    details.extend(smile(&sdf, 0.6, 0.1));
    for s in [-1.0, 1.0] {
        details.push(decal(&sdf, s * 0.17, 0.62, v(0.07, 0.04, 0.02), [245, 140, 160], 0.0));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Staryu y Starmie

pub fn staryu() -> CreatureModel {
    let brown = [205, 140, 70];
    let sdf = star(Sdf::new(), v(0.0, 0.42, 0.0), 0.5, 0.0, brown, 0.09);
    let details = gem(&sdf, 0.42, 0.11);
    CreatureModel { body: sdf, details, flames: vec![] }
}

pub fn starmie() -> CreatureModel {
    let purple = [135, 105, 195];
    let sdf = star(Sdf::new(), v(0.0, 0.5, -0.1), 0.5, PI / 5.0, purple, 0.08);
    let sdf = star(sdf, v(0.0, 0.5, 0.02), 0.52, 0.0, purple, 0.09);
    let details = gem(&sdf, 0.5, 0.13);
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Voltorb

pub fn voltorb() -> CreatureModel {
    let sdf = Sdf::new().add(ellipsoid(v(0.0, 0.4, 0.0), Vec3::splat(0.4)), WHITE, 0.0);
    let sdf = paint_level(sdf, 0.41, true, [225, 50, 50]).paint(ellipsoid(v(0.0, 0.41, 0.0), v(0.5, 0.012, 0.5)), INK);
    let mut details = vec![];
    for s in [-1.0, 1.0] {
        details.extend(eye(&sdf, s * 0.15, 0.48, 0.1, 0.08, [255, 255, 255]));
        let (p, rot) = surf(&sdf, s * 0.16, 0.6);
        details.push(part(Shape::Cube, INK, p, v(0.16, 0.03, 0.02)).rot(rot * Quat::from_rotation_z(s * 0.4)).flat());
    }
    details.extend(smile(&sdf, 0.28, 0.25));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Oddish

pub fn oddish() -> CreatureModel {
    let (blue, leaf, vein) = ([80, 110, 195], [85, 175, 80], [55, 135, 60]);
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.32, 0.0), v(0.27, 0.27, 0.25)), blue, 0.0);
    sdf = feet(sdf, blue, 0.12, 0.04, v(0.08, 0.06, 0.11));
    for i in 0..5 {
        let rot = Quat::from_rotation_y(i as f32 * std::f32::consts::TAU / 5.0) * Quat::from_rotation_x(0.55);
        let c = v(0.0, 0.58, 0.0) + rot * v(0.0, 0.26, 0.0);
        sdf = sdf.add(ellipsoid_rot(c, v(0.11, 0.3, 0.035), rot), leaf, 0.04).paint(ellipsoid_rot(c, v(0.012, 0.3, 0.05), rot), vein);
    }
    let mut details = eyes(&sdf, 0.1, 0.36, 0.07, 0.08, [200, 40, 50]);
    details.extend(smile(&sdf, 0.26, 0.1));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Tangela

pub fn tangela() -> CreatureModel {
    let (vine, dark, red) = ([60, 95, 195], [40, 65, 150], [215, 60, 70]);
    let center = v(0.0, 0.46, 0.0);
    let mut sdf = Sdf::new().add(ellipsoid(center, Vec3::splat(0.36)), vine, 0.0);
    // Lianas enrolladas por toda la bola (puntos repartidos en espiral)
    for i in 0..34 {
        let y = 1.0 - (i as f32 + 0.5) / 17.0;
        let r = (1.0 - y * y).sqrt();
        let a = i as f32 * 2.399;
        let dir = v(a.cos() * r, y, a.sin() * r);
        let tangent = dir.cross(Vec3::Y).normalize_or(Vec3::X);
        let p = center + dir * 0.36;
        sdf = sdf.add(limb(p - tangent * 0.1, p + tangent * 0.12 + dir * 0.05, 0.045, 0.035), vine, 0.02);
        if i % 3 == 0 {
            sdf = sdf.paint(ellipsoid(p, Vec3::splat(0.05)), dark);
        }
    }
    sdf = sdf.paint(ellipsoid(v(0.0, 0.5, 0.3), v(0.24, 0.08, 0.12)), [20, 20, 35]);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.25, 0.3, 0.2), v(s * 0.38, 0.12, 0.3), 0.04, 0.03), vine, 0.02)
            .add(ellipsoid(v(s * 0.14, 0.05, 0.06), v(0.1, 0.06, 0.14)), red, 0.04);
    }
    let mut details = vec![];
    for s in [-1.0, 1.0] {
        let (p, rot) = surf(&sdf, s * 0.09, 0.5);
        details.push(part(Shape::Sphere, WHITE, p + rot * v(0.0, 0.0, 0.01), v(0.08, 0.06, 0.03)).rot(rot).flat());
        details.push(part(Shape::Sphere, INK, p + rot * v(0.01 * s, 0.0, 0.025), v(0.04, 0.045, 0.02)).rot(rot).flat());
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Grimer

pub fn grimer() -> CreatureModel {
    let purple = [160, 115, 180];
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.12, 0.0), v(0.46, 0.16, 0.42)), purple, 0.0)
        .add(ellipsoid(v(0.0, 0.46, 0.0), v(0.3, 0.32, 0.26)), purple, 0.14);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.24, 0.48, 0.05), v(s * 0.45, 0.66, 0.15), 0.1, 0.07), purple, 0.06)
            .add(ellipsoid(v(s * 0.47, 0.62, 0.18), v(0.08, 0.1, 0.06)), purple, 0.03)
            .add(ellipsoid(v(s * 0.35, 0.05, 0.3), v(0.1, 0.06, 0.1)), purple, 0.05);
    }
    let mut details = eyes(&sdf, 0.1, 0.62, 0.06, 0.07, INK);
    details.push(decal(&sdf, 0.0, 0.42, v(0.3, 0.14, 0.04), [70, 30, 70], 0.0));
    details.push(decal(&sdf, 0.0, 0.39, v(0.16, 0.06, 0.03), [230, 100, 140], 0.0));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Koffing

pub fn koffing() -> CreatureModel {
    let (purple, crater, cream) = ([140, 110, 175], [110, 85, 145], [235, 225, 175]);
    let center = v(0.0, 0.62, 0.0);
    let mut sdf = Sdf::new().add(ellipsoid(center, Vec3::splat(0.38)), purple, 0.0);
    for dir in [v(0.7, 0.5, 0.3), v(-0.7, 0.5, 0.2), v(0.2, 0.9, -0.3), v(-0.4, -0.4, -0.6), v(0.6, -0.3, -0.5), v(0.0, 0.3, -0.95)] {
        let d = dir.normalize();
        sdf = sdf
            .add(limb(center + d * 0.32, center + d * 0.44, 0.07, 0.06), purple, 0.03)
            .paint(ellipsoid(center + d * 0.46, Vec3::splat(0.05)), crater);
    }
    let sdf = sdf.paint(ellipsoid(v(0.25, 0.85, 0.2), Vec3::splat(0.05)), crater);
    let mut details = eyes(&sdf, 0.13, 0.7, 0.1, 0.09, [60, 30, 40]);
    details.push(decal(&sdf, 0.0, 0.55, v(0.22, 0.07, 0.03), [80, 25, 45], 0.0));
    for s in [-1.0, 1.0] {
        details.push(fang(&sdf, s * 0.06, 0.58));
        details.push(decal(&sdf, 0.0, 0.38, v(0.16, 0.03, 0.02), cream, s * 0.7));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Abra

pub fn abra() -> CreatureModel {
    let (yellow, brown) = ([240, 200, 80], [150, 100, 60]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.36, 0.0), v(0.25, 0.27, 0.22)), yellow, 0.0)
        .add(ellipsoid(v(0.0, 0.76, 0.04), v(0.23, 0.21, 0.21)), yellow, 0.08)
        .add(ellipsoid(v(0.0, 0.7, 0.2), v(0.12, 0.09, 0.1)), yellow, 0.05);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.12, 0.9, 0.0), v(s * 0.22, 1.12, -0.06), 0.06, 0.015), yellow, 0.03)
            .add(limb(v(s * 0.2, 0.46, 0.06), v(s * 0.08, 0.36, 0.24), 0.055, 0.045), yellow, 0.03)
            .add(limb(v(s * 0.15, 0.16, 0.06), v(-s * 0.1, 0.1, 0.24), 0.08, 0.06), yellow, 0.04)
            .add(ellipsoid(v(-s * 0.13, 0.08, 0.26), v(0.07, 0.05, 0.09)), yellow, 0.03);
    }
    let sdf = sdf
        .add(limb(v(0.0, 0.2, -0.2), v(0.15, 0.12, -0.45), 0.08, 0.05), yellow, 0.04)
        .add(limb(v(0.15, 0.12, -0.45), v(0.32, 0.18, -0.5), 0.05, 0.03), yellow, 0.02)
        .paint(ellipsoid(v(0.0, 0.38, 0.1), v(0.22, 0.2, 0.15)), brown)
        .paint(ellipsoid(v(0.0, 0.72, 0.29), v(0.05, 0.03, 0.03)), INK);
    let mut details = vec![];
    for s in [-1.0, 1.0] {
        details.push(decal(&sdf, s * 0.1, 0.79, v(0.1, 0.018, 0.02), INK, s * 0.2));
    }
    details.extend(smile(&sdf, 0.66, 0.1));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Growlithe

pub fn growlithe() -> CreatureModel {
    let (orange, cream, black) = ([240, 135, 55], [250, 235, 180], [40, 35, 40]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.42, -0.06), v(0.22, 0.22, 0.34)), orange, 0.0)
        .add(ellipsoid(v(0.0, 0.48, 0.22), v(0.21, 0.21, 0.16)), cream, 0.06)
        .add(ellipsoid(v(0.0, 0.72, 0.3), v(0.22, 0.2, 0.2)), orange, 0.08)
        .add(ellipsoid(v(0.0, 0.65, 0.46), v(0.11, 0.08, 0.1)), cream, 0.05)
        .add(ellipsoid(v(0.0, 0.9, 0.26), v(0.12, 0.08, 0.12)), cream, 0.05);
    for s in [-1.0, 1.0] {
        sdf = sdf.add(limb(v(s * 0.13, 0.85, 0.24), v(s * 0.19, 1.0, 0.2), 0.065, 0.012), orange, 0.03);
        for z in [0.16, -0.26] {
            sdf = sdf
                .add(limb(v(s * 0.13, 0.32, z), v(s * 0.14, 0.06, z + 0.02), 0.075, 0.065), orange, 0.04)
                .add(ellipsoid(v(s * 0.14, 0.04, z + 0.06), v(0.07, 0.045, 0.09)), cream, 0.02);
        }
        for k in 0..3 {
            let z = -0.25 + k as f32 * 0.15;
            sdf = sdf.paint(ellipsoid_rot(v(s * 0.2, 0.5, z), v(0.05, 0.12, 0.022), Quat::from_rotation_x(0.4)), black);
        }
    }
    let sdf = sdf.add(ellipsoid_rot(v(0.0, 0.6, -0.46), v(0.11, 0.17, 0.12), Quat::from_rotation_x(-0.6)), cream, 0.05);
    let mut details = eyes(&sdf, 0.1, 0.75, 0.07, 0.08, [50, 40, 30]);
    details.push(decal(&sdf, 0.0, 0.68, v(0.06, 0.04, 0.03), INK, 0.0));
    details.extend(smile(&sdf, 0.6, 0.1));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Ponyta

pub fn ponyta() -> CreatureModel {
    let (body, hoof) = ([250, 240, 205], [110, 100, 95]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.62, -0.05), v(0.19, 0.19, 0.34)), body, 0.0)
        .add(limb(v(0.0, 0.72, 0.2), v(0.0, 0.95, 0.32), 0.11, 0.09), body, 0.06)
        .add(ellipsoid(v(0.0, 1.02, 0.4), v(0.12, 0.13, 0.17)), body, 0.05)
        .add(ellipsoid(v(0.0, 0.96, 0.54), v(0.08, 0.07, 0.08)), body, 0.04);
    for s in [-1.0, 1.0] {
        sdf = sdf.add(limb(v(s * 0.07, 1.12, 0.36), v(s * 0.1, 1.22, 0.33), 0.04, 0.01), body, 0.02);
        for z in [0.16, -0.24] {
            sdf = sdf
                .add(limb(v(s * 0.11, 0.55, z), v(s * 0.12, 0.08, z), 0.065, 0.045), body, 0.04)
                .add(ellipsoid(v(s * 0.12, 0.05, z + 0.01), v(0.055, 0.05, 0.06)), hoof, 0.01);
        }
    }
    let mut details = eyes(&sdf, 0.08, 1.05, 0.06, 0.07, [120, 50, 30]);
    details.push(decal(&sdf, 0.0, 0.95, v(0.04, 0.02, 0.02), INK, 0.0));
    let flames = vec![
        (v(0.0, 1.13, 0.4), 0.8),
        (v(0.0, 1.05, 0.28), 0.9),
        (v(0.0, 0.92, 0.2), 0.85),
        (v(0.0, 0.78, 0.12), 0.75),
        (v(0.0, 0.7, -0.38), 1.0),
        (v(0.0, 0.66, -0.3), 0.7),
    ];
    CreatureModel { body: sdf, details, flames }
}

// ---------------------------------------------------------------- Diglett y Dugtrio

fn diglett_at(sdf: Sdf, base: Vec3, h: f32) -> Sdf {
    sdf.add(limb(base, base + v(0.0, h - 0.18, 0.0), 0.19, 0.18), [175, 115, 75], 0.03)
        .add(ellipsoid(base + v(0.0, h * 0.62, 0.17), v(0.1, 0.07, 0.07)), [240, 130, 150], 0.03)
}

fn diglett_face(sdf: &Sdf, x: f32, h: f32, details: &mut Vec<Part>) {
    for s in [-1.0, 1.0] {
        let (p, rot) = surf(sdf, x + s * 0.065, h * 0.78);
        details.push(part(Shape::Sphere, INK, p, v(0.035, 0.08, 0.02)).rot(rot).flat());
        details.push(part(Shape::Sphere, WHITE, p + rot * v(-0.006, 0.02, 0.012), v(0.014, 0.022, 0.01)).rot(rot).flat());
    }
}

pub fn diglett() -> CreatureModel {
    let dirt = [110, 80, 60];
    let sdf = Sdf::new().add(ellipsoid(v(0.0, 0.0, 0.0), v(0.42, 0.1, 0.42)), dirt, 0.0);
    let sdf = diglett_at(sdf, v(0.0, 0.0, 0.0), 0.5);
    let mut details = vec![];
    diglett_face(&sdf, 0.0, 0.5, &mut details);
    for (x, z) in [(0.32, 0.18), (-0.3, -0.15), (0.1, -0.33)] {
        details.push(part(Shape::Sphere, [140, 105, 80], v(x, 0.08, z), Vec3::splat(0.08)));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

pub fn dugtrio() -> CreatureModel {
    let dirt = [110, 80, 60];
    let mut sdf = Sdf::new().add(ellipsoid(v(0.0, 0.0, -0.05), v(0.62, 0.12, 0.5)), dirt, 0.0);
    let heads = [(v(0.0, 0.0, -0.18), 0.66), (v(-0.25, 0.0, 0.08), 0.52), (v(0.25, 0.0, 0.08), 0.5)];
    for (base, h) in heads {
        sdf = diglett_at(sdf, base, h);
    }
    let mut details = vec![];
    for (base, h) in heads {
        diglett_face(&sdf, base.x, h, &mut details);
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Rhyhorn

pub fn rhyhorn() -> CreatureModel {
    let (grey, dark, horn) = ([165, 165, 175], [130, 130, 142], [240, 235, 220]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.46, -0.1), v(0.3, 0.27, 0.42)), grey, 0.0)
        .add(ellipsoid(v(0.0, 0.5, 0.36), v(0.22, 0.2, 0.25)), grey, 0.08)
        .add(limb(v(0.0, 0.58, 0.56), v(0.0, 0.84, 0.64), 0.075, 0.012), horn, 0.03)
        .add(limb(v(0.0, 0.68, 0.42), v(0.0, 0.8, 0.42), 0.04, 0.01), horn, 0.02);
    for s in [-1.0, 1.0] {
        sdf = sdf.add(limb(v(s * 0.15, 0.68, 0.3), v(s * 0.2, 0.8, 0.26), 0.04, 0.012), grey, 0.02);
        for z in [0.15, -0.32] {
            sdf = sdf.add(limb(v(s * 0.18, 0.35, z), v(s * 0.2, 0.06, z + 0.02), 0.1, 0.09), grey, 0.04);
        }
    }
    for k in 0..4 {
        let z = 0.12 - k as f32 * 0.16;
        sdf = sdf.add(ellipsoid(v(0.0, 0.72 - k as f32 * 0.02, z), v(0.1, 0.06, 0.07)), dark, 0.04);
    }
    let sdf = sdf.add(limb(v(0.0, 0.4, -0.5), v(0.0, 0.3, -0.64), 0.05, 0.03), grey, 0.02);
    let mut details = eyes(&sdf, 0.15, 0.56, 0.05, 0.06, [200, 40, 40]);
    details.extend(smile(&sdf, 0.42, 0.14));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Gastly

pub fn gastly() -> CreatureModel {
    let (black, gas) = ([55, 40, 70], [165, 120, 200]);
    let center = v(0.0, 0.75, 0.0);
    let mut sdf = Sdf::new().add(ellipsoid(center, Vec3::splat(0.3)), black, 0.0);
    // Nube de gas alrededor (a bultos, pegada por detrás y por los lados)
    for i in 0..9 {
        let a = i as f32 * std::f32::consts::TAU / 9.0;
        let d = v(a.cos(), a.sin() * 0.8, -0.55).normalize();
        sdf = sdf.add(ellipsoid(center + d * 0.32, Vec3::splat(0.17)), gas, 0.08);
    }
    let mut details = eyes(&sdf, 0.12, 0.82, 0.11, 0.1, [255, 255, 255]);
    details.push(decal(&sdf, 0.0, 0.66, v(0.26, 0.1, 0.03), [190, 60, 100], 0.0));
    for s in [-1.0, 1.0] {
        details.push(fang(&sdf, s * 0.08, 0.7));
        let (p, rot) = surf(&sdf, s * 0.12, 0.92);
        details.push(part(Shape::Cube, [20, 15, 25], p, v(0.14, 0.03, 0.02)).rot(rot * Quat::from_rotation_z(-s * 0.35)).flat());
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Machop

pub fn machop() -> CreatureModel {
    let (skin, crest, belt) = ([150, 175, 195], [120, 145, 165], [200, 160, 70]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.45, 0.0), v(0.24, 0.24, 0.19)), skin, 0.0)
        .add(ellipsoid(v(0.0, 0.84, 0.03), v(0.22, 0.2, 0.2)), skin, 0.06);
    for (x, h) in [(-0.09, 0.08), (0.0, 0.12), (0.09, 0.08)] {
        sdf = sdf.add(limb(v(x, 0.98, 0.05), v(x * 1.2, 1.0 + h, 0.0), 0.04, 0.02), crest, 0.02);
    }
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(limb(v(s * 0.22, 0.55, 0.0), v(s * 0.35, 0.38, 0.12), 0.08, 0.07), skin, 0.04)
            .add(ellipsoid(v(s * 0.36, 0.33, 0.16), Vec3::splat(0.08)), skin, 0.02)
            .add(limb(v(s * 0.12, 0.26, 0.0), v(s * 0.15, 0.06, 0.03), 0.09, 0.08), skin, 0.04)
            .add(ellipsoid(v(s * 0.15, 0.04, 0.08), v(0.08, 0.04, 0.12)), skin, 0.02)
            .paint(ellipsoid_rot(v(s * 0.08, 0.56, 0.16), v(0.08, 0.05, 0.03), Quat::from_rotation_z(s * 0.3)), crest);
    }
    let sdf = sdf.paint(ellipsoid(v(0.0, 0.32, 0.0), v(0.3, 0.035, 0.3)), belt);
    let mut details = eyes(&sdf, 0.09, 0.88, 0.065, 0.075, [200, 40, 40]);
    details.extend(smile(&sdf, 0.75, 0.12));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Magikarp

pub fn magikarp() -> CreatureModel {
    let (red, belly, fin) = ([235, 95, 60], [250, 230, 200], [250, 230, 160]);
    let mut sdf = Sdf::new()
        .add(ellipsoid(v(0.0, 0.48, 0.0), v(0.14, 0.33, 0.36)), red, 0.0)
        .add(ellipsoid(v(0.0, 0.4, 0.33), v(0.1, 0.1, 0.07)), fin, 0.04)
        .add(ellipsoid_rot(v(0.0, 0.88, -0.05), v(0.03, 0.14, 0.24), Quat::from_rotation_x(0.3)), fin, 0.03)
        .add(ellipsoid_rot(v(0.0, 0.62, -0.5), v(0.03, 0.22, 0.12), Quat::from_rotation_x(0.6)), fin, 0.02)
        .add(ellipsoid_rot(v(0.0, 0.3, -0.48), v(0.03, 0.2, 0.12), Quat::from_rotation_x(-0.5)), fin, 0.02);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(ellipsoid_rot(v(s * 0.14, 0.22, 0.05), v(0.02, 0.1, 0.16), Quat::from_rotation_z(s * 0.6)), fin, 0.02)
            .add(limb(v(s * 0.06, 0.48, 0.3), v(s * 0.16, 0.32, 0.42), 0.015, 0.008), fin, 0.0);
    }
    let sdf = sdf.paint(ellipsoid(v(0.0, 0.25, 0.08), v(0.16, 0.12, 0.3)), belly);
    let mut details = vec![];
    for s in [-1.0, 1.0] {
        let (p, rot) = surf_dir(&sdf, v(s * 2.0, 0.6, 0.18), v(-s, 0.0, 0.0));
        details.push(part(Shape::Sphere, WHITE, p, v(0.12, 0.12, 0.03)).rot(rot).flat());
        details.push(part(Shape::Sphere, INK, p + rot * v(0.0, 0.0, 0.012), v(0.05, 0.05, 0.02)).rot(rot).flat());
    }
    for s in [-1.0, 1.0] {
        details.push(decal(&sdf, 0.0, 0.4 + s * 0.035, v(0.12, 0.03, 0.03), [240, 170, 140], 0.0));
    }
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Dratini

pub fn dratini() -> CreatureModel {
    let (blue, white) = ([110, 150, 230], [250, 250, 255]);
    let path = [
        (v(0.32, 0.08, -0.5), 0.035),
        (v(0.0, 0.1, -0.58), 0.06),
        (v(-0.32, 0.11, -0.36), 0.08),
        (v(-0.36, 0.12, 0.0), 0.1),
        (v(-0.1, 0.14, 0.2), 0.11),
        (v(0.18, 0.16, 0.1), 0.11),
        (v(0.16, 0.5, 0.12), 0.1),
        (v(0.0, 0.76, 0.2), 0.11),
    ];
    let mut sdf = Sdf::new();
    for w in path.windows(2) {
        sdf = sdf.add(limb(w[0].0, w[1].0, w[0].1, w[1].1), blue, 0.04);
    }
    sdf = sdf
        .add(ellipsoid(v(0.0, 0.82, 0.22), v(0.16, 0.15, 0.19)), blue, 0.06)
        .add(ellipsoid(v(0.0, 0.78, 0.38), v(0.1, 0.08, 0.08)), blue, 0.04)
        .add(ellipsoid(v(0.0, 0.84, 0.43), v(0.04, 0.03, 0.03)), white, 0.01);
    for s in [-1.0, 1.0] {
        for (k, (dy, a)) in [(0.06, 0.5), (0.0, 0.0), (-0.06, -0.5)].into_iter().enumerate() {
            let rot = Quat::from_rotation_z(s * a) * Quat::from_rotation_y(s * 0.3);
            sdf = sdf.add(ellipsoid_rot(v(s * (0.21 + k as f32 * 0.0), 0.84 + dy, 0.18), v(0.09, 0.025, 0.012), rot), white, 0.01);
        }
    }
    let sdf = paint_level(sdf, 0.09, false, white);
    let mut details = eyes(&sdf, 0.09, 0.86, 0.06, 0.07, [60, 40, 90]);
    details.extend(smile(&sdf, 0.74, 0.1));
    CreatureModel { body: sdf, details, flames: vec![] }
}

// ---------------------------------------------------------------- Seel

pub fn seel() -> CreatureModel {
    let (white, horn, tongue) = ([235, 242, 252], [245, 235, 210], [235, 90, 100]);
    let mut sdf = Sdf::new()
        .add(ellipsoid_rot(v(0.0, 0.32, -0.08), v(0.25, 0.27, 0.42), Quat::from_rotation_x(-0.25)), white, 0.0)
        .add(ellipsoid(v(0.0, 0.62, 0.22), v(0.22, 0.2, 0.2)), white, 0.08)
        .add(ellipsoid(v(0.0, 0.55, 0.38), v(0.12, 0.07, 0.08)), white, 0.04)
        .add(limb(v(0.0, 0.78, 0.25), v(0.0, 0.93, 0.29), 0.04, 0.012), horn, 0.02);
    for s in [-1.0, 1.0] {
        sdf = sdf
            .add(ellipsoid_rot(v(s * 0.24, 0.12, 0.18), v(0.07, 0.03, 0.15), Quat::from_rotation_y(s * 0.5)), white, 0.03)
            .add(ellipsoid_rot(v(s * 0.12, 0.08, -0.52), v(0.13, 0.03, 0.08), Quat::from_rotation_y(s * 0.6)), white, 0.03);
    }
    let mut details = eyes(&sdf, 0.09, 0.66, 0.05, 0.06, INK);
    let (p, rot) = surf(&sdf, 0.0, 0.5);
    details.push(part(Shape::Sphere, tongue, p + rot * v(0.0, -0.04, 0.02), v(0.06, 0.09, 0.03)).rot(rot));
    details.push(decal(&sdf, 0.0, 0.58, v(0.04, 0.025, 0.02), INK, 0.0));
    CreatureModel { body: sdf, details, flames: vec![] }
}

#[cfg(test)]
mod tests {
    use crate::pokemon::SPECIES;

    #[test]
    fn every_pokemon_builds_a_closed_mesh() {
        for s in &SPECIES {
            let model = (s.model)();
            let data = model.body.surface_nets(0.03);
            assert!(data.indices.len() > 3000, "{} tiene muy pocos triángulos", s.name);
            assert!(!model.details.is_empty());
        }
    }
}
