//! Modelos low-poly hechos con primitivas: cada modelo es una lista de piezas
//! (forma unitaria + color + transform) que cuelgan de una entidad raíz.
//! Se pintan con sombreado cel (`assets/shaders/toon.wesl`) y contorno negro
//! (técnica del "casco invertido": una copia algo mayor de la pieza, negra y vista por dentro).

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Face};
use bevy::shader::ShaderRef;

use crate::pokemon::SPECIES;

/// Grosor del contorno, en unidades del modelo.
const OUTLINE: f32 = 0.05;

pub type Toon = ExtendedMaterial<StandardMaterial, ToonExt>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct ToonExt {
    /// x: intensidad del rim light, y: brillo general
    #[uniform(100)]
    params: Vec4,
}

impl MaterialExtension for ToonExt {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wesl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/toon.wesl".into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    Cube,
    Sphere,
    Cylinder,
    Cone,
}

pub type Rgb = [u8; 3];

#[derive(Clone)]
pub struct Part {
    shape: Shape,
    color: Rgb,
    transform: Transform,
    outline: bool,
    glow: bool,
}

/// Pieza de tamaño `size` (las formas son de tamaño 1) colocada en `pos`.
pub fn part(shape: Shape, color: Rgb, pos: Vec3, size: Vec3) -> Part {
    Part { shape, color, transform: Transform::from_translation(pos).with_scale(size), outline: true, glow: false }
}

impl Part {
    pub fn rot(mut self, rotation: Quat) -> Self {
        self.transform.rotation = rotation;
        self
    }

    /// Sin contorno (detalles pequeños: pupilas, brillos, coloretes...).
    pub fn flat(mut self) -> Self {
        self.outline = false;
        self
    }

    /// Emite luz propia (llamas, chispas).
    pub fn glow(mut self) -> Self {
        self.glow = true;
        self.outline = false;
        self
    }
}

/// Malla del cuerpo, su casco de contorno y los detalles de cada especie (mismo orden que `SPECIES`).
#[derive(Resource)]
pub struct CreatureMeshes {
    species: Vec<CreatureMesh>,
    flame: Handle<Mesh>,
    flame_mat: Handle<StandardMaterial>,
}

struct CreatureMesh {
    body: Handle<Mesh>,
    hull: Handle<Mesh>,
    details: Vec<Part>,
    flames: Vec<(Vec3, f32)>,
}

impl FromWorld for CreatureMeshes {
    fn from_world(world: &mut World) -> Self {
        // Esculpir cada Pokémon cuesta unas décimas de segundo: en paralelo
        let built: Vec<_> = std::thread::scope(|scope| {
            let jobs: Vec<_> = SPECIES
                .iter()
                .map(|s| {
                    scope.spawn(move || {
                        let model = (s.model)();
                        let (body, hull) = model.body.build_meshes(0.016, OUTLINE * 0.5);
                        (body, hull, model.details, model.flames)
                    })
                })
                .collect();
            jobs.into_iter().map(|j| j.join().unwrap()).collect()
        });
        let (flame, _) = crate::pokemon::sculpt::flame().build_meshes(0.008, 0.0);
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let species = built
            .into_iter()
            .map(|(body, hull, details, flames)| CreatureMesh { body: meshes.add(body), hull: meshes.add(hull), details, flames })
            .collect();
        let flame = meshes.add(flame);
        // Sin luz y con brillo HDR (>1): se ve igual desde todos lados y el bloom la hace resplandecer
        let flame_mat = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color: LinearRgba::rgb(2.2, 2.2, 2.2).into(),
            unlit: true,
            ..default()
        });
        Self { species, flame, flame_mat }
    }
}

/// Las partes luminosas (llamas, mejillas) laten un poco.
#[derive(Component)]
pub struct Flicker {
    base: Vec3,
    seed: f32,
}

pub fn flicker(time: Res<Time>, mut q: Query<(&Flicker, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (f, mut transform) in &mut q {
        let wave = (t * 13.0 + f.seed).sin() * 0.5 + (t * 7.3 + f.seed * 2.0).sin() * 0.5;
        transform.scale = f.base * Vec3::new(1.0 - wave * 0.05, 1.0 + wave * 0.12, 1.0 - wave * 0.05);
    }
}

#[derive(Resource, Default)]
pub struct KitCache {
    meshes: HashMap<Shape, Handle<Mesh>>,
    materials: HashMap<(Rgb, bool), Handle<Toon>>,
    outline: Option<Handle<StandardMaterial>>,
    water: Option<Handle<Toon>>,
}

/// Acceso cómodo a mallas y materiales compartidos.
#[derive(SystemParam)]
pub struct Kit<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<Toon>>,
    std_materials: ResMut<'w, Assets<StandardMaterial>>,
    cache: ResMut<'w, KitCache>,
    creatures: Res<'w, CreatureMeshes>,
}

impl Kit<'_> {
    pub fn mesh(&mut self, shape: Shape) -> Handle<Mesh> {
        let meshes = &mut self.meshes;
        self.cache
            .meshes
            .entry(shape)
            .or_insert_with(|| match shape {
                Shape::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                Shape::Sphere => meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap()),
                Shape::Cylinder => meshes.add(Cylinder::new(0.5, 1.0).mesh().resolution(20)),
                Shape::Cone => meshes.add(Cone { radius: 0.5, height: 1.0 }.mesh().resolution(20)),
            })
            .clone()
    }

    pub fn mat(&mut self, color: Rgb, glow: bool) -> Handle<Toon> {
        let materials = &mut self.materials;
        self.cache
            .materials
            .entry((color, glow))
            .or_insert_with(|| {
                let base_color = Color::srgb_u8(color[0], color[1], color[2]);
                materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color,
                        emissive: if glow { base_color.to_linear() * 2.5 } else { LinearRgba::BLACK },
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..default()
                    },
                    extension: ToonExt { params: Vec4::new(0.18, 1.0, 0.0, 0.0) },
                })
            })
            .clone()
    }

    fn outline_mat(&mut self) -> Handle<StandardMaterial> {
        let materials = &mut self.std_materials;
        self.cache
            .outline
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(0.06, 0.05, 0.08),
                    unlit: true,
                    cull_mode: Some(Face::Front),
                    ..default()
                })
            })
            .clone()
    }

    pub fn water(&mut self) -> Handle<Toon> {
        let materials = &mut self.materials;
        self.cache
            .water
            .get_or_insert_with(|| {
                materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        base_color: Color::srgba(0.3, 0.62, 0.98, 0.82),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    },
                    extension: ToonExt { params: Vec4::new(0.5, 1.0, 0.0, 0.0) },
                })
            })
            .clone()
    }

    /// Una pieza suelta, sin contorno (suelo, plataformas...).
    pub fn piece(&mut self, p: &Part) -> impl Bundle + use<> {
        (Mesh3d(self.mesh(p.shape)), MeshMaterial3d(self.mat(p.color, p.glow)), p.transform)
    }

    /// Crea `root` y le cuelga todas las piezas del modelo, con su contorno.
    pub fn spawn_model(&mut self, commands: &mut Commands, root: impl Bundle, parts: &[Part]) -> Entity {
        let root = commands.spawn((root, Visibility::default())).id();
        self.attach_parts(commands, root, parts);
        root
    }

    /// Un Pokémon: cuerpo esculpido (con colores por vértice) + contorno + detalles.
    pub fn spawn_creature(&mut self, commands: &mut Commands, root: impl Bundle, species: usize) -> Entity {
        let root = commands.spawn((root, Visibility::default())).id();
        let c = &self.creatures.species[species];
        let (body, hull, details, flames) = (c.body.clone(), c.hull.clone(), c.details.clone(), c.flames.clone());
        commands.spawn((Mesh3d(body), MeshMaterial3d(self.mat([255, 255, 255], false)), ChildOf(root)));
        commands.spawn((Mesh3d(hull), MeshMaterial3d(self.outline_mat()), NotShadowCaster, ChildOf(root)));
        for (i, (base, size)) in flames.into_iter().enumerate() {
            commands.spawn((
                Mesh3d(self.creatures.flame.clone()),
                MeshMaterial3d(self.creatures.flame_mat.clone()),
                Transform::from_translation(base).with_scale(Vec3::splat(size)),
                Flicker { base: Vec3::splat(size), seed: i as f32 * 2.3 },
                NotShadowCaster,
                ChildOf(root),
            ));
        }
        self.attach_parts(commands, root, &details);
        root
    }

    fn attach_parts(&mut self, commands: &mut Commands, root: Entity, parts: &[Part]) {
        for (i, p) in parts.iter().enumerate() {
            let piece = self.piece(p);
            let id = commands.spawn((piece, ChildOf(root))).id();
            if p.glow {
                commands.entity(id).insert(Flicker { base: p.transform.scale, seed: i as f32 * 1.7 });
            }
            if p.outline {
                let mut hull = p.transform;
                hull.scale += Vec3::splat(OUTLINE);
                commands.spawn((
                    Mesh3d(self.mesh(p.shape)),
                    MeshMaterial3d(self.outline_mat()),
                    hull,
                    NotShadowCaster,
                    ChildOf(root),
                ));
            }
        }
    }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

const WHITE: Rgb = [255, 255, 255];
const INK: Rgb = [25, 20, 35];

/// Ojos estilo anime: blanco ovalado, iris grande y un brillo.
fn eyes(y: f32, z: f32, spread: f32, size: f32, iris: Rgb) -> Vec<Part> {
    let mut parts = vec![];
    for side in [-1.0, 1.0] {
        let x = side * spread;
        parts.push(part(Shape::Sphere, WHITE, v(x, y, z), v(size * 0.8, size * 1.1, size * 0.5)));
        parts.push(part(Shape::Sphere, iris, v(x, y - size * 0.05, z + size * 0.17), v(size * 0.55, size * 0.8, size * 0.25)).flat());
        parts.push(part(Shape::Sphere, INK, v(x, y - size * 0.08, z + size * 0.24), v(size * 0.3, size * 0.45, size * 0.15)).flat());
        parts.push(part(Shape::Sphere, WHITE, v(x - size * 0.12, y + size * 0.2, z + size * 0.3), Vec3::splat(size * 0.2)).flat());
    }
    parts
}

fn mouth(pos: Vec3, width: f32) -> Part {
    part(Shape::Sphere, [120, 30, 40], pos, v(width, width * 0.3, width * 0.3)).flat()
}

// ---------------------------------------------------------------- personajes

pub struct Outfit {
    /// Sin gorra se ve el pelo.
    pub cap: Option<Rgb>,
    pub shirt: Rgb,
    pub pants: Rgb,
    pub skin: Rgb,
    pub hair: Rgb,
    /// Melena hasta los hombros.
    pub long_hair: bool,
}

const fn fit(cap: Option<Rgb>, shirt: Rgb, pants: Rgb, skin: Rgb, hair: Rgb, long_hair: bool) -> Outfit {
    Outfit { cap, shirt, pants, skin, hair, long_hair }
}

pub const PLAYER_OUTFIT: Outfit = fit(Some([220, 50, 50]), [60, 100, 210], [55, 60, 80], [250, 205, 165], [70, 45, 30], false);

/// Índices en `NPC_OUTFITS`.
pub mod look {
    pub const YOUNGSTER: usize = 0;
    pub const OAK: usize = 1;
    pub const HIKER: usize = 2;
    pub const NURSE: usize = 3;
    pub const CLERK: usize = 4;
    pub const LASS: usize = 5;
    pub const BUG: usize = 6;
    pub const FISHER: usize = 7;
    pub const MOM: usize = 8;
    pub const RIVAL: usize = 9;
    pub const BROCK: usize = 10;
    pub const MISTY: usize = 11;
    pub const SURGE: usize = 12;
    pub const ERIKA: usize = 13;
    pub const KOGA: usize = 14;
    pub const SABRINA: usize = 15;
    pub const BLAINE: usize = 16;
    pub const GIOVANNI: usize = 17;
    pub const ROCKET: usize = 18;
    pub const GENTLEMAN: usize = 19;
    pub const CHANNELER: usize = 20;
    pub const GUARD: usize = 21;
    pub const SWIMMER: usize = 22;
    pub const ACE: usize = 23;
    pub const LORELEI: usize = 24;
    pub const BRUNO: usize = 25;
    pub const AGATHA: usize = 26;
    pub const LANCE: usize = 27;
    pub const SCIENTIST: usize = 28;
    pub const BIKER: usize = 29;
    pub const OLDMAN: usize = 30;
    pub const GIRL: usize = 31;
}

const SKIN: Rgb = [250, 210, 175];
const TAN: Rgb = [215, 160, 120];
const DARK: Rgb = [150, 100, 70];

pub const NPC_OUTFITS: [Outfit; 32] = [
    fit(Some([60, 160, 80]), [240, 200, 60], [90, 70, 50], TAN, [30, 25, 20], false),
    fit(None, [245, 245, 245], [130, 110, 90], SKIN, [210, 210, 215], false),
    fit(Some([40, 40, 50]), [230, 110, 40], [40, 60, 90], DARK, [20, 15, 15], false),
    fit(Some([255, 255, 255]), [250, 170, 200], [250, 250, 250], SKIN, [240, 120, 160], true),
    fit(Some([60, 110, 200]), [80, 140, 220], [50, 60, 90], SKIN, [60, 40, 30], false),
    fit(None, [240, 120, 150], [60, 70, 140], SKIN, [200, 90, 50], true),
    fit(Some([240, 220, 120]), [250, 250, 250], [80, 150, 80], TAN, [60, 40, 25], false),
    fit(Some([230, 230, 200]), [90, 150, 210], [70, 80, 100], TAN, [40, 30, 20], false),
    fit(None, [240, 150, 170], [170, 120, 190], SKIN, [140, 80, 40], true),
    fit(None, [140, 90, 200], [60, 60, 70], SKIN, [150, 100, 50], false),
    fit(None, [200, 120, 60], [70, 110, 70], DARK, [50, 35, 25], false),
    fit(None, [250, 210, 70], [220, 60, 50], SKIN, [240, 120, 50], false),
    fit(None, [100, 120, 70], [80, 90, 60], SKIN, [250, 220, 90], false),
    fit(None, [90, 170, 90], [200, 80, 100], SKIN, [25, 20, 30], true),
    fit(Some([80, 50, 110]), [100, 60, 140], [50, 40, 70], TAN, [30, 25, 40], false),
    fit(None, [180, 40, 60], [30, 30, 40], SKIN, [30, 25, 45], true),
    fit(None, [245, 245, 245], [90, 90, 100], TAN, [250, 250, 250], false),
    fit(None, [230, 130, 50], [60, 50, 50], SKIN, [30, 25, 20], false),
    fit(Some([30, 30, 35]), [35, 35, 40], [30, 30, 35], SKIN, [30, 25, 20], false),
    fit(None, [120, 80, 60], [70, 55, 45], SKIN, [140, 140, 145], false),
    fit(None, [245, 245, 240], [140, 100, 170], SKIN, [60, 40, 70], true),
    fit(Some([40, 60, 120]), [50, 75, 140], [40, 45, 70], TAN, [30, 25, 20], false),
    fit(Some([250, 250, 250]), [60, 160, 220], [40, 120, 200], TAN, [80, 50, 30], false),
    fit(Some([220, 60, 60]), [250, 250, 250], [50, 60, 120], SKIN, [80, 50, 30], false),
    fit(None, [150, 60, 150], [80, 40, 90], SKIN, [210, 50, 50], true),
    fit(None, [60, 60, 60], [70, 60, 50], DARK, [20, 15, 15], false),
    fit(None, [120, 80, 150], [60, 50, 70], SKIN, [180, 180, 190], true),
    fit(None, [60, 70, 90], [40, 40, 60], SKIN, [210, 60, 40], false),
    fit(None, [240, 240, 235], [80, 90, 110], SKIN, [90, 70, 50], false),
    fit(Some([50, 50, 55]), [60, 60, 65], [40, 40, 45], TAN, [30, 25, 20], false),
    fit(None, [180, 150, 110], [110, 90, 70], SKIN, [230, 230, 230], false),
    fit(None, [250, 200, 90], [90, 140, 200], SKIN, [120, 70, 40], true),
];

/// Un personaje por piezas: el cuerpo y las extremidades, que se mueven al andar.
pub struct Character {
    pub body: Vec<Part>,
    /// Piernas izquierda y derecha (relativas a la cadera).
    pub legs: [Vec<Part>; 2],
    /// Brazos izquierdo y derecho (relativos al hombro).
    pub arms: [Vec<Part>; 2],
}

pub const HIP: Vec3 = Vec3::new(0.085, 0.31, 0.0);
pub const SHOULDER: Vec3 = Vec3::new(0.205, 0.565, 0.0);

/// Personaje de ~1 casilla de alto, cabezón estilo chibi, mirando hacia +Z.
pub fn character(o: &Outfit) -> Character {
    let shoe = [40, 40, 45];
    let leg = || {
        vec![
            part(Shape::Cube, o.pants, v(0.0, -0.13, 0.0), v(0.12, 0.27, 0.14)),
            part(Shape::Sphere, shoe, v(0.0, -0.27, 0.03), v(0.14, 0.09, 0.2)),
        ]
    };
    let arm = |side: f32| {
        vec![
            part(Shape::Cube, o.shirt, v(0.0, -0.11, 0.0), v(0.09, 0.25, 0.1)).rot(Quat::from_rotation_z(side * 0.12)),
            part(Shape::Sphere, o.skin, v(side * 0.025, -0.26, 0.0), Vec3::splat(0.09)),
        ]
    };
    let mut body = vec![
        // cuerpo y mochila
        part(Shape::Cube, o.shirt, v(0.0, 0.44, 0.0), v(0.32, 0.3, 0.2)),
        part(Shape::Cube, o.pants, v(0.0, 0.31, 0.0), v(0.3, 0.08, 0.19)),
        part(Shape::Cube, [240, 190, 60], v(0.0, 0.46, -0.14), v(0.24, 0.24, 0.1)),
        // cabeza
        part(Shape::Sphere, o.skin, v(0.0, 0.8, 0.0), v(0.42, 0.4, 0.4)),
        part(Shape::Sphere, o.hair, v(0.0, 0.83, -0.05), v(0.44, 0.38, 0.38)),
        mouth(v(0.0, 0.7, 0.2), 0.06),
    ];
    match o.cap {
        Some(cap) => {
            body.push(part(Shape::Sphere, cap, v(0.0, 0.92, -0.01), v(0.45, 0.25, 0.43)));
            body.push(part(Shape::Cube, cap, v(0.0, 0.9, 0.22), v(0.34, 0.035, 0.18)));
            body.push(part(Shape::Sphere, WHITE, v(0.0, 0.99, 0.12), v(0.12, 0.08, 0.04)).flat());
        }
        None => {
            // Flequillo
            body.push(part(Shape::Sphere, o.hair, v(0.0, 0.94, 0.06), v(0.4, 0.2, 0.34)));
            body.push(part(Shape::Sphere, o.hair, v(-0.1, 0.9, 0.15), v(0.18, 0.12, 0.12)).flat());
        }
    }
    if o.long_hair {
        body.push(part(Shape::Sphere, o.hair, v(0.0, 0.66, -0.12), v(0.42, 0.42, 0.26)));
    }
    body.extend(eyes(0.8, 0.16, 0.085, 0.09, [60, 40, 30]));
    Character { body, legs: [leg(), leg()], arms: [arm(-1.0), arm(1.0)] }
}

/// Pone a un personaje a andar: `phase` avanza con la distancia recorrida.
#[derive(Component, Default)]
pub struct Walker {
    pub phase: f32,
    /// 0 = quieto, 1 = andando del todo (se mezcla suavemente).
    pub blend: f32,
    pub moving: bool,
    /// Corriendo: zancadas más largas.
    pub running: bool,
}

/// Una extremidad de un personaje (gira en su articulación).
#[derive(Component)]
pub struct Limb {
    root: Entity,
    /// -1 izquierda, 1 derecha.
    side: f32,
    leg: bool,
}

/// Balanceo de piernas y brazos: piernas opuestas, cada brazo contra su pierna.
pub fn walk_cycle(time: Res<Time>, mut walkers: Query<&mut Walker>, mut limbs: Query<(&Limb, &mut Transform)>) {
    let k = 1.0 - (-12.0 * time.delta_secs()).exp();
    for mut w in &mut walkers {
        let target = if w.moving { 1.0 } else { 0.0 };
        w.blend += (target - w.blend) * k;
        if !w.moving && w.blend < 0.02 {
            // Al pararse termina la zancada en vez de quedarse a medias
            w.phase = 0.0;
        }
    }
    for (limb, mut transform) in &mut limbs {
        let Ok(w) = walkers.get(limb.root) else { continue };
        let reach = if w.running { 1.25 } else { 1.0 };
        let swing = w.phase.sin() * w.blend * reach;
        let angle = if limb.leg { swing * 0.62 * limb.side } else { -swing * 0.55 * limb.side };
        // Al levantar la pierna hacia delante se dobla un poco la rodilla (sube el pie)
        transform.rotation = Quat::from_rotation_x(angle);
    }
}

impl Kit<'_> {
    /// Un personaje con las piernas y los brazos articulados.
    pub fn spawn_character(&mut self, commands: &mut Commands, root: impl Bundle, outfit: &Outfit) -> Entity {
        let c = character(outfit);
        let root = self.spawn_model(commands, (root, Walker::default()), &c.body);
        for (i, side) in [-1.0f32, 1.0].into_iter().enumerate() {
            for (parts, pivot, leg) in [(&c.legs[i], HIP, true), (&c.arms[i], SHOULDER, false)] {
                let joint = commands
                    .spawn((
                        Transform::from_translation(pivot * Vec3::new(side, 1.0, 1.0)),
                        Visibility::default(),
                        Limb { root, side, leg },
                        ChildOf(root),
                    ))
                    .id();
                self.attach_parts(commands, joint, parts);
            }
        }
        root
    }
}

// ---------------------------------------------------------------- escenario

pub fn tree() -> Vec<Part> {
    vec![
        part(Shape::Cylinder, [120, 75, 45], v(0.0, 0.3, 0.0), v(0.24, 0.6, 0.24)),
        part(Shape::Cone, [40, 120, 60], v(0.0, 0.95, 0.0), v(0.95, 0.9, 0.95)),
        part(Shape::Cone, [55, 145, 70], v(0.0, 1.45, 0.0), v(0.72, 0.75, 0.72)),
        part(Shape::Cone, [75, 170, 85], v(0.0, 1.85, 0.0), v(0.46, 0.55, 0.46)),
    ]
}

pub fn grass_blades(seed: u32) -> Vec<Part> {
    // Posiciones pseudoaleatorias pero fijas por casilla
    let mut h = seed.wrapping_mul(2654435761);
    let mut rnd = move || {
        h ^= h << 13;
        h ^= h >> 17;
        h ^= h << 5;
        (h % 1000) as f32 / 1000.0
    };
    (0..6)
        .map(|_| {
            let (x, z, s) = (rnd() * 0.7 - 0.35, rnd() * 0.7 - 0.35, 0.8 + rnd() * 0.4);
            let color = if rnd() > 0.5 { [60, 165, 70] } else { [95, 195, 80] };
            part(Shape::Cone, color, v(x, 0.22 * s, z), v(0.22, 0.45 * s, 0.22)).flat()
        })
        .collect()
}

// ---------------------------------------------------------------- pueblo y decoración

/// Edificio de `w`×`d` casillas mirando al sur (+Z), centrado en su huella.
/// `door_x` es dónde cae la puerta respecto al centro.
pub fn building(kind: crate::world::BuildingKind, w: f32, d: f32, door_x: f32, roof: Rgb) -> Vec<Part> {
    use crate::world::BuildingKind;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

    let (wall, wall_h) = match kind {
        BuildingKind::House => ([245, 230, 200], 1.4),
        BuildingKind::Center => ([250, 246, 240], 1.6),
        BuildingKind::Mart => ([240, 244, 250], 1.6),
        BuildingKind::Lab => ([225, 228, 232], 1.7),
        BuildingKind::Gym => ([236, 226, 205], 1.9),
        BuildingKind::Big => ([214, 214, 222], 1.6 + d * 0.35),
    };
    let front = d / 2.0;
    let mut p = vec![
        part(Shape::Cube, [150, 140, 125], v(0.0, 0.08, 0.0), v(w - 0.02, 0.16, d - 0.02)),
        part(Shape::Cube, wall, v(0.0, wall_h / 2.0, 0.0), v(w - 0.15, wall_h, d - 0.15)),
        // Puerta con pomo y escalón
        part(Shape::Cube, [120, 80, 50], v(door_x, 0.5, front - 0.05), v(0.55, 0.9, 0.08)),
        part(Shape::Sphere, [250, 210, 80], v(door_x + 0.17, 0.5, front), Vec3::splat(0.07)).flat(),
        part(Shape::Cube, [170, 160, 145], v(door_x, 0.04, front + 0.2), v(0.8, 0.08, 0.35)),
    ];
    if kind == BuildingKind::Big {
        // Azotea plana con cornisa
        p.push(part(Shape::Cube, roof, v(0.0, wall_h + 0.06, 0.0), v(w + 0.1, 0.14, d + 0.1)));
        p.push(part(Shape::Cube, darker(roof), v(0.0, wall_h + 0.18, 0.0), v(w - 0.6, 0.12, d - 0.6)));
        // Ventanas en varias plantas
        let floors = ((wall_h - 0.6) / 0.85) as i32;
        for f in 0..floors {
            let y = 1.35 + f as f32 * 0.85;
            let mut x = -w / 2.0 + 0.7;
            while x < w / 2.0 - 0.4 {
                if f > 0 || (x - door_x).abs() > 0.75 {
                    p.push(part(Shape::Cube, [150, 205, 245], v(x, y, front - 0.05), v(0.5, 0.45, 0.06)).flat());
                }
                x += 1.0;
            }
        }
        return p;
    }

    // Tejado a dos aguas a 45º: dos placas + el triángulo del hastial (un cubo girado)
    let hd = d / 2.0 + 0.25;
    let slab = hd * SQRT_2;
    p.extend([
        part(Shape::Cube, wall, v(0.0, wall_h, 0.0), v(w - 0.15, (d / 2.0 - 0.08) * SQRT_2, (d / 2.0 - 0.08) * SQRT_2))
            .rot(Quat::from_rotation_x(FRAC_PI_4)),
        part(Shape::Cube, roof, v(0.0, wall_h + hd / 2.0, hd / 2.0), v(w + 0.3, 0.14, slab)).rot(Quat::from_rotation_x(FRAC_PI_4)),
        part(Shape::Cube, roof, v(0.0, wall_h + hd / 2.0, -hd / 2.0), v(w + 0.3, 0.14, slab)).rot(Quat::from_rotation_x(-FRAC_PI_4)),
        part(Shape::Cube, darker(roof), v(0.0, wall_h + hd + 0.02, 0.0), v(w + 0.36, 0.14, 0.18)),
    ]);
    // Filas de tejas: listones más oscuros paralelos a la cumbrera, en las dos aguas
    for side in [1.0, -1.0] {
        let rot = Quat::from_rotation_x(side * FRAC_PI_4);
        let center = v(0.0, wall_h + hd / 2.0, side * hd / 2.0);
        let (down, normal) = (rot * Vec3::Z * side, rot * Vec3::Y);
        for k in [-0.3, -0.1, 0.1, 0.3] {
            let pos = center + down * (k * slab) + normal * 0.075;
            p.push(part(Shape::Cube, darker(roof), pos, v(w + 0.3, 0.025, 0.06)).rot(rot).flat());
        }
    }
    // Ventanas a los lados de la puerta
    let mut x = -w / 2.0 + 0.7;
    while x < w / 2.0 - 0.5 {
        if (x - door_x).abs() > 0.75 {
            p.push(part(Shape::Cube, WHITE, v(x, wall_h * 0.58, front - 0.06), v(0.52, 0.46, 0.06)));
            p.push(part(Shape::Cube, [150, 205, 245], v(x, wall_h * 0.58, front - 0.03), v(0.4, 0.34, 0.04)).flat());
        }
        x += 1.0;
    }
    let sign_y = wall_h + 0.35;
    match kind {
        BuildingKind::House => {
            p.push(part(Shape::Cube, [170, 90, 70], v(w / 2.0 - 0.75, wall_h + hd * 0.75, -hd * 0.3), v(0.32, 0.9, 0.32)));
        }
        BuildingKind::Center => {
            // Emblema de Pokéball sobre la puerta
            p.push(part(Shape::Cylinder, WHITE, v(door_x, sign_y, front + 0.1), v(0.62, 0.06, 0.62)).rot(Quat::from_rotation_x(FRAC_PI_2)));
            p.push(part(Shape::Cube, [225, 50, 60], v(door_x, sign_y + 0.15, front + 0.12), v(0.56, 0.26, 0.05)).flat());
            p.push(part(Shape::Cube, INK, v(door_x, sign_y, front + 0.14), v(0.6, 0.06, 0.05)).flat());
            p.push(part(Shape::Sphere, WHITE, v(door_x, sign_y, front + 0.15), v(0.18, 0.18, 0.08)));
        }
        BuildingKind::Mart => {
            // Letrero azul con franja blanca
            p.push(part(Shape::Cube, [50, 110, 210], v(door_x, sign_y, front + 0.1), v(1.3, 0.42, 0.08)));
            p.push(part(Shape::Cube, WHITE, v(door_x, sign_y, front + 0.15), v(1.1, 0.1, 0.02)).flat());
            p.push(part(Shape::Cube, [50, 110, 210], v(door_x - 0.35, sign_y + 0.08, front + 0.17), v(0.12, 0.12, 0.02)).flat());
        }
        BuildingKind::Gym => {
            // Gran emblema de medalla y columnas a los lados de la puerta
            p.push(part(Shape::Cylinder, [240, 200, 70], v(door_x, sign_y, front + 0.1), v(0.8, 0.08, 0.8)).rot(Quat::from_rotation_x(FRAC_PI_2)));
            p.push(part(Shape::Cylinder, roof, v(door_x, sign_y, front + 0.15), v(0.55, 0.04, 0.55)).rot(Quat::from_rotation_x(FRAC_PI_2)).flat());
            for s in [-1.0, 1.0] {
                p.push(part(Shape::Cylinder, [250, 248, 240], v(door_x + s * 0.55, 0.65, front + 0.05), v(0.18, 1.3, 0.18)));
            }
        }
        BuildingKind::Lab => {
            p.push(part(Shape::Cylinder, [200, 205, 215], v(-w / 2.0 + 0.8, wall_h + hd * 0.6, -hd * 0.4), v(0.14, 0.9, 0.14)));
            p.push(part(Shape::Sphere, [235, 238, 242], v(-w / 2.0 + 0.8, wall_h + hd * 0.6 + 0.5, -hd * 0.4), v(0.6, 0.18, 0.6)));
        }
        BuildingKind::Big => {}
    }
    p
}

fn darker(c: Rgb) -> Rgb {
    c.map(|x| (x as f32 * 0.75) as u8)
}

/// Tramo de valla: poste en el centro y travesaños hacia las vallas vecinas.
pub fn fence(west: bool, east: bool, north: bool, south: bool) -> Vec<Part> {
    let wood = [205, 165, 115];
    let mut p = vec![part(Shape::Cube, wood, v(0.0, 0.3, 0.0), v(0.12, 0.6, 0.12))];
    for (on, dir) in [(west, -Vec3::X), (east, Vec3::X), (north, -Vec3::Z), (south, Vec3::Z)] {
        if on {
            let size = if dir.x != 0.0 { v(0.52, 0.08, 0.06) } else { v(0.06, 0.08, 0.52) };
            for y in [0.25, 0.47] {
                p.push(part(Shape::Cube, wood, dir * 0.25 + Vec3::Y * y, size));
            }
        }
    }
    p
}

fn rnd(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 1000) as f32 / 1000.0
}

pub fn flowers(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2654435761) | 1;
    let colors = [[240, 70, 90], [255, 220, 70], [255, 255, 255], [240, 140, 200], [140, 120, 240]];
    (0..5)
        .flat_map(|_| {
            let (x, z) = (rnd(&mut s) * 0.7 - 0.35, rnd(&mut s) * 0.7 - 0.35);
            let color = colors[(rnd(&mut s) * colors.len() as f32) as usize % colors.len()];
            [
                part(Shape::Cylinder, [70, 150, 60], v(x, 0.1, z), v(0.03, 0.2, 0.03)).flat(),
                part(Shape::Sphere, color, v(x, 0.22, z), v(0.16, 0.07, 0.16)),
                part(Shape::Sphere, [255, 200, 40], v(x, 0.25, z), Vec3::splat(0.06)).flat(),
            ]
        })
        .collect()
}

/// Matojos sueltos para que el césped no se vea liso.
pub fn tufts(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2654435761) | 1;
    (0..3)
        .map(|_| {
            let (x, z, h) = (rnd(&mut s) * 0.8 - 0.4, rnd(&mut s) * 0.8 - 0.4, 0.12 + rnd(&mut s) * 0.1);
            part(Shape::Cone, [80, 160, 65], v(x, h / 2.0, z), v(0.1, h, 0.1)).flat()
        })
        .collect()
}

/// Piedrecitas del camino.
pub fn pebbles(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2246822519) | 1;
    (0..2)
        .map(|_| {
            let (x, z, r) = (rnd(&mut s) * 0.7 - 0.35, rnd(&mut s) * 0.7 - 0.35, 0.06 + rnd(&mut s) * 0.06);
            part(Shape::Sphere, [180, 165, 130], v(x, 0.0, z), v(r * 1.4, r * 0.6, r)).flat()
        })
        .collect()
}

pub fn rock(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2246822519) | 1;
    let grey = [150, 150, 160];
    vec![
        part(Shape::Sphere, grey, v(0.0, 0.22, 0.0), v(0.8, 0.5, 0.7)).rot(Quat::from_rotation_y(rnd(&mut s) * 3.0)),
        part(Shape::Sphere, [130, 130, 140], v(0.25, 0.15, 0.2), v(0.4, 0.3, 0.35)),
        part(Shape::Sphere, [170, 170, 180], v(-0.2, 0.42, -0.05), v(0.3, 0.18, 0.3)).flat(),
    ]
}

pub fn bush() -> Vec<Part> {
    let g = [60, 140, 60];
    vec![
        part(Shape::Sphere, g, v(0.0, 0.35, 0.0), Vec3::splat(0.75)),
        part(Shape::Sphere, [75, 160, 70], v(-0.22, 0.3, 0.15), Vec3::splat(0.5)),
        part(Shape::Sphere, [75, 160, 70], v(0.25, 0.32, 0.1), Vec3::splat(0.5)),
        part(Shape::Sphere, [240, 90, 110], v(0.1, 0.6, 0.28), Vec3::splat(0.08)).flat(),
        part(Shape::Sphere, [240, 90, 110], v(-0.25, 0.45, 0.33), Vec3::splat(0.07)).flat(),
    ]
}

pub fn sign() -> Vec<Part> {
    let wood = [170, 120, 75];
    vec![
        part(Shape::Cylinder, wood, v(0.0, 0.3, 0.0), v(0.1, 0.6, 0.1)),
        part(Shape::Cube, wood, v(0.0, 0.65, 0.0), v(0.75, 0.45, 0.08)),
        part(Shape::Cube, [235, 215, 175], v(0.0, 0.65, 0.045), v(0.62, 0.32, 0.02)).flat(),
        part(Shape::Cube, [120, 85, 55], v(0.0, 0.72, 0.06), v(0.4, 0.04, 0.01)).flat(),
        part(Shape::Cube, [120, 85, 55], v(0.0, 0.62, 0.06), v(0.48, 0.04, 0.01)).flat(),
    ]
}

pub fn cloud(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(3266489917) | 1;
    (0..5)
        .map(|i| {
            let x = i as f32 * 0.9 - 1.8 + rnd(&mut s) * 0.3;
            let r = 1.0 + rnd(&mut s) * 0.8 - (i as f32 - 2.0).abs() * 0.2;
            part(Shape::Sphere, WHITE, v(x, rnd(&mut s) * 0.3, rnd(&mut s) * 0.6), v(r * 1.3, r * 0.8, r))
        })
        .collect()
}

// ---------------------------------------------------------------- montaña, cuevas y mar

/// Bloque de montaña: rocas apiladas de alturas distintas.
pub fn mountain(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2654435761) | 1;
    let h = 1.1 + rnd(&mut s) * 0.7;
    let rock = [160, 135, 105];
    vec![
        part(Shape::Cube, rock, v(0.0, h / 2.0 - 0.1, 0.0), v(1.0, h, 1.0)).flat(),
        part(Shape::Cube, [175, 150, 118], v(rnd(&mut s) * 0.3 - 0.15, h - 0.05, rnd(&mut s) * 0.3 - 0.15), v(0.7, 0.3, 0.7))
            .rot(Quat::from_rotation_y(rnd(&mut s))),
        part(Shape::Cube, [120, 165, 90], v(0.0, h - 0.08, 0.0), v(1.02, 0.08, 1.02)).flat(),
        part(Shape::Cube, [140, 115, 90], v(0.0, h * 0.35, 0.5), v(0.9, 0.04, 0.03)).flat(),
    ]
}

/// Pared de roca de una cueva.
pub fn cave_wall(seed: u32, color: Rgb) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2246822519) | 1;
    let h = 1.0 + rnd(&mut s) * 0.6;
    let dark = darker(color);
    vec![
        part(Shape::Cube, color, v(0.0, h / 2.0 - 0.1, 0.0), v(1.0, h, 1.0)).flat(),
        part(Shape::Sphere, dark, v(rnd(&mut s) * 0.4 - 0.2, h - 0.1, rnd(&mut s) * 0.4 - 0.2), v(0.8, 0.45, 0.8)),
    ]
}

/// Boca de cueva excavada en la montaña (mirando hacia `dir`).
pub fn cave_mouth() -> Vec<Part> {
    vec![
        part(Shape::Cube, [25, 20, 25], v(0.0, 0.35, -0.3), v(0.8, 0.8, 0.5)).flat(),
        part(Shape::Cylinder, [25, 20, 25], v(0.0, 0.75, -0.3), v(0.8, 0.5, 0.4)).rot(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)).flat(),
        part(Shape::Cube, [130, 110, 85], v(0.0, 1.2, -0.05), v(1.0, 0.25, 0.5)),
    ]
}

/// Escalera: hacia abajo es un agujero con peldaños, hacia arriba una escalera de mano.
pub fn ladder(down: bool) -> Vec<Part> {
    let wood = [170, 120, 70];
    if down {
        let mut p = vec![part(Shape::Cylinder, [20, 15, 20], v(0.0, 0.01, 0.0), v(0.8, 0.04, 0.8)).flat()];
        for i in 0..3 {
            p.push(part(Shape::Cube, wood, v(0.0, 0.03, -0.2 + i as f32 * 0.2), v(0.5, 0.04, 0.06)).flat());
        }
        p
    } else {
        let mut p = vec![];
        for s in [-1.0, 1.0] {
            p.push(part(Shape::Cube, wood, v(s * 0.22, 0.6, -0.35), v(0.06, 1.2, 0.06)));
        }
        for i in 0..5 {
            p.push(part(Shape::Cube, wood, v(0.0, 0.15 + i as f32 * 0.22, -0.35), v(0.44, 0.04, 0.05)).flat());
        }
        p
    }
}

/// Saliente: un escalón de césped que solo se puede saltar hacia el sur.
pub fn ledge(ground: Rgb) -> Vec<Part> {
    let top = ground;
    let side = [(ground[0] as f32 * 0.7) as u8, (ground[1] as f32 * 0.62) as u8, (ground[2] as f32 * 0.5) as u8];
    vec![
        part(Shape::Cube, top, v(0.0, 0.1, -0.1), v(1.0, 0.2, 0.8)).flat(),
        part(Shape::Cube, side, v(0.0, 0.06, 0.32), v(1.0, 0.12, 0.12)).flat(),
    ]
}

/// Tabla de surf (para cruzar el agua).
pub fn surfboard() -> Vec<Part> {
    vec![
        part(Shape::Sphere, [250, 250, 245], v(0.0, -0.02, 0.05), v(0.55, 0.07, 1.15)),
        part(Shape::Cube, [60, 140, 230], v(0.0, 0.015, 0.05), v(0.12, 0.02, 0.95)).flat(),
        part(Shape::Sphere, [180, 225, 250], v(0.0, -0.06, -0.55), v(0.7, 0.04, 0.35)).flat(),
    ]
}

/// Tablones de un puente o pasarela.
pub fn bridge(seed: u32) -> Vec<Part> {
    let wood = if seed % 2 == 0 { [190, 140, 90] } else { [175, 128, 82] };
    let mut p = vec![part(Shape::Cube, wood, v(0.0, -0.05, 0.0), v(1.0, 0.1, 1.0))];
    for z in [-0.25, 0.0, 0.25] {
        p.push(part(Shape::Cube, [140, 100, 60], v(0.0, 0.003, z + 0.12), v(1.0, 0.01, 0.025)).flat());
    }
    p
}

/// Poste de madera en el borde de la pasarela.
pub fn post() -> Vec<Part> {
    vec![part(Shape::Cylinder, [150, 105, 65], v(0.0, 0.0, 0.0), v(0.14, 0.6, 0.14))]
}

// ---------------------------------------------------------------- interiores

pub fn wall(color: Rgb, height: f32) -> Vec<Part> {
    vec![
        part(Shape::Cube, color, v(0.0, height / 2.0 - 0.2, 0.0), v(1.0, height, 1.0)).flat(),
        part(Shape::Cube, darker(color), v(0.0, 0.0, 0.0), v(1.02, 0.18, 1.02)).flat(),
    ]
}

pub fn counter() -> Vec<Part> {
    vec![
        part(Shape::Cube, [200, 90, 80], v(0.0, 0.32, 0.0), v(1.0, 0.64, 0.8)),
        part(Shape::Cube, [250, 245, 235], v(0.0, 0.67, 0.0), v(1.04, 0.07, 0.86)),
    ]
}

pub fn shelf(seed: u32) -> Vec<Part> {
    let mut s = seed.wrapping_mul(2654435761) | 1;
    let mut p = vec![part(Shape::Cube, [150, 105, 70], v(0.0, 0.6, -0.1), v(0.95, 1.2, 0.6))];
    let colors = [[230, 70, 70], [70, 130, 230], [250, 200, 60], [120, 200, 110], [200, 120, 220]];
    for shelf_y in [0.35, 0.75, 1.1] {
        for i in 0..3 {
            let c = colors[(rnd(&mut s) * 5.0) as usize % 5];
            p.push(part(Shape::Cube, c, v(-0.28 + i as f32 * 0.28, shelf_y, 0.22), v(0.18, 0.22, 0.12)).flat());
        }
    }
    p
}

pub fn table() -> Vec<Part> {
    vec![
        part(Shape::Cylinder, [190, 140, 95], v(0.0, 0.42, 0.0), v(0.9, 0.08, 0.9)),
        part(Shape::Cylinder, [150, 105, 70], v(0.0, 0.2, 0.0), v(0.12, 0.4, 0.12)),
    ]
}

pub fn tombstone(seed: u32) -> Vec<Part> {
    let grey = if seed % 2 == 0 { [170, 165, 180] } else { [150, 145, 165] };
    vec![
        part(Shape::Cube, grey, v(0.0, 0.35, 0.0), v(0.55, 0.7, 0.18)),
        part(Shape::Cylinder, grey, v(0.0, 0.7, 0.0), v(0.55, 0.18, 0.55)).rot(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        part(Shape::Cube, [110, 105, 125], v(0.0, 0.05, 0.25), v(0.7, 0.1, 0.35)).flat(),
    ]
}

pub fn plant() -> Vec<Part> {
    vec![
        part(Shape::Cylinder, [200, 110, 70], v(0.0, 0.18, 0.0), v(0.4, 0.36, 0.4)),
        part(Shape::Sphere, [70, 160, 80], v(0.0, 0.6, 0.0), v(0.6, 0.55, 0.6)),
        part(Shape::Sphere, [90, 185, 95], v(0.1, 0.78, 0.08), v(0.35, 0.3, 0.35)).flat(),
    ]
}

/// Estatua de la entrada de los gimnasios.
pub fn statue() -> Vec<Part> {
    vec![
        part(Shape::Cube, [190, 190, 200], v(0.0, 0.25, 0.0), v(0.6, 0.5, 0.6)),
        part(Shape::Sphere, [215, 215, 225], v(0.0, 0.75, 0.0), v(0.5, 0.55, 0.45)),
        part(Shape::Sphere, [215, 215, 225], v(0.0, 1.1, 0.05), v(0.35, 0.32, 0.32)),
        part(Shape::Cone, [215, 215, 225], v(-0.12, 1.3, 0.0), v(0.1, 0.2, 0.1)),
        part(Shape::Cone, [215, 215, 225], v(0.12, 1.3, 0.0), v(0.1, 0.2, 0.1)),
    ]
}

/// Columna de un gimnasio.
pub fn pillar() -> Vec<Part> {
    vec![
        part(Shape::Cube, [200, 195, 185], v(0.0, 0.1, 0.0), v(0.9, 0.2, 0.9)),
        part(Shape::Cylinder, [240, 235, 225], v(0.0, 1.2, 0.0), v(0.6, 2.2, 0.6)),
        part(Shape::Cube, [200, 195, 185], v(0.0, 2.35, 0.0), v(0.9, 0.2, 0.9)),
    ]
}

/// Máquina: la de curar del Centro Pokémon, un ordenador...
pub fn machine() -> Vec<Part> {
    let mut p = vec![
        part(Shape::Cube, [210, 215, 225], v(0.0, 0.45, -0.05), v(0.85, 0.9, 0.7)),
        part(Shape::Cube, [60, 150, 210], v(0.0, 0.7, 0.31), v(0.6, 0.3, 0.03)).flat(),
    ];
    for i in 0..3 {
        p.push(part(Shape::Sphere, [235, 70, 80], v(-0.22 + i as f32 * 0.22, 0.95, 0.0), Vec3::splat(0.14)).glow());
    }
    p
}

/// Felpudo de salida.
pub fn doormat() -> Vec<Part> {
    vec![part(Shape::Cube, [200, 70, 60], v(0.0, 0.0, 0.0), v(0.9, 0.02, 0.8)).flat()]
}

/// Alfombra (casilla de interior).
pub fn rug() -> Vec<Part> {
    vec![part(Shape::Cube, [180, 60, 70], v(0.0, 0.0, 0.0), v(1.0, 0.015, 1.0)).flat()]
}

/// Objeto en el suelo: una Poké Ball.
pub fn item_ball() -> Vec<Part> {
    vec![
        part(Shape::Sphere, [230, 50, 55], v(0.0, 0.2, 0.0), Vec3::splat(0.34)),
        part(Shape::Sphere, WHITE, v(0.0, 0.17, 0.0), v(0.345, 0.3, 0.345)).flat(),
        part(Shape::Cylinder, INK, v(0.0, 0.2, 0.0), v(0.35, 0.03, 0.35)).flat(),
        part(Shape::Sphere, WHITE, v(0.0, 0.2, 0.16), Vec3::splat(0.09)).flat(),
    ]
}

/// "!" sobre la cabeza del entrenador que te ha visto.
pub fn exclamation() -> Vec<Part> {
    vec![
        part(Shape::Cube, [235, 50, 50], v(0.0, 1.45, 0.0), v(0.1, 0.32, 0.1)),
        part(Shape::Sphere, [235, 50, 50], v(0.0, 1.22, 0.0), Vec3::splat(0.11)),
    ]
}
