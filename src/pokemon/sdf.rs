//! Escultura con campos de distancia (SDF): las piezas se funden suavemente entre sí
//! como arcilla, y luego se convierten en una malla lisa con "surface nets".
//! Las capas de pintura colorean la superficie sin cambiar la forma (barriga, manchas, rayas).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::render::models::Rgb;

#[derive(Clone, Copy)]
pub enum Prim {
    Ellipsoid { center: Vec3, radii: Vec3, rot: Quat },
    /// Cono redondeado de `a` (radio `ra`) a `b` (radio `rb`): brazos, colas, orejas...
    Limb { a: Vec3, b: Vec3, ra: f32, rb: f32 },
}

pub fn ellipsoid(center: Vec3, radii: Vec3) -> Prim {
    Prim::Ellipsoid { center, radii, rot: Quat::IDENTITY }
}

pub fn ellipsoid_rot(center: Vec3, radii: Vec3, rot: Quat) -> Prim {
    Prim::Ellipsoid { center, radii, rot }
}

pub fn limb(a: Vec3, b: Vec3, ra: f32, rb: f32) -> Prim {
    Prim::Limb { a, b, ra, rb }
}

impl Prim {
    fn dist(&self, p: Vec3) -> f32 {
        match *self {
            Prim::Ellipsoid { center, radii, rot } => {
                let q = rot.inverse() * (p - center);
                let k0 = (q / radii).length();
                let k1 = (q / (radii * radii)).length();
                if k1 < 1e-6 { -radii.min_element() } else { k0 * (k0 - 1.0) / k1 }
            }
            Prim::Limb { a, b, ra, rb } => round_cone(p, a, b, ra, rb),
        }
    }

    fn bounds(&self) -> (Vec3, Vec3) {
        match *self {
            Prim::Ellipsoid { center, radii, .. } => {
                let r = Vec3::splat(radii.max_element());
                (center - r, center + r)
            }
            Prim::Limb { a, b, ra, rb } => {
                let r = ra.max(rb);
                (a.min(b) - r, a.max(b) + r)
            }
        }
    }
}

fn sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Distancia a un cono redondeado (Íñigo Quílez).
fn round_cone(p: Vec3, a: Vec3, b: Vec3, r1: f32, r2: f32) -> f32 {
    let ba = b - a;
    let l2 = ba.dot(ba);
    let rr = r1 - r2;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pa = p - a;
    let y = pa.dot(ba);
    let z = y - l2;
    let x2 = (pa * l2 - ba * y).length_squared();
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let k = sign(rr) * rr * rr * x2;
    if sign(z) * a2 * z2 > k {
        return (x2 + z2).sqrt() * il2 - r2;
    }
    if sign(y) * a2 * y2 < k {
        return (x2 + y2).sqrt() * il2 - r1;
    }
    ((x2 * a2 * il2).sqrt() + y * rr) * il2 - r1
}

enum Layer {
    /// Forma que se funde con lo anterior; `k` es cuánto se suaviza la unión.
    Solid { prim: Prim, k: f32 },
    /// Solo pinta la superficie que cae dentro de la forma.
    Paint { prim: Prim },
}

pub struct Sdf {
    layers: Vec<(Layer, Vec3)>,
}

fn linear(c: Rgb) -> Vec3 {
    let l = Color::srgb_u8(c[0], c[1], c[2]).to_linear();
    Vec3::new(l.red, l.green, l.blue)
}

impl Sdf {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn add(mut self, prim: Prim, color: Rgb, k: f32) -> Self {
        self.layers.push((Layer::Solid { prim, k }, linear(color)));
        self
    }

    pub fn paint(mut self, prim: Prim, color: Rgb) -> Self {
        self.layers.push((Layer::Paint { prim }, linear(color)));
        self
    }

    pub fn dist(&self, p: Vec3) -> f32 {
        let mut d = f32::MAX;
        for (layer, _) in &self.layers {
            if let Layer::Solid { prim, k } = layer {
                d = smin(prim.dist(p), d, *k).0;
            }
        }
        d
    }

    /// Distancia y color (lineal) en el punto.
    pub fn sample(&self, p: Vec3) -> (f32, Vec3) {
        let mut d = f32::MAX;
        let mut color = Vec3::ZERO;
        for (layer, c) in &self.layers {
            match layer {
                Layer::Solid { prim, k } => {
                    let di = prim.dist(p);
                    let (nd, _) = smin(di, d, *k);
                    // El color cambia en una franja más estrecha que la forma, para que no "manche"
                    let (_, h) = smin(di, d, k * 0.3);
                    color = color.lerp(*c, h);
                    d = nd;
                }
                Layer::Paint { prim } => {
                    let w = (0.5 - prim.dist(p) / 0.012).clamp(0.0, 1.0);
                    color = color.lerp(*c, w);
                }
            }
        }
        (d, color)
    }

    pub fn normal(&self, p: Vec3) -> Vec3 {
        let e = 0.002;
        Vec3::new(
            self.dist(p + Vec3::X * e) - self.dist(p - Vec3::X * e),
            self.dist(p + Vec3::Y * e) - self.dist(p - Vec3::Y * e),
            self.dist(p + Vec3::Z * e) - self.dist(p - Vec3::Z * e),
        )
        .normalize_or(Vec3::Y)
    }

    /// Lanza un rayo y devuelve el punto de la superficie donde choca y su normal.
    pub fn hit(&self, from: Vec3, dir: Vec3) -> Option<(Vec3, Vec3)> {
        let dir = dir.normalize();
        let mut t = 0.0;
        for _ in 0..400 {
            let p = from + dir * t;
            let d = self.dist(p);
            if d < 0.0005 {
                return Some((p, self.normal(p)));
            }
            t += (d * 0.7).max(0.0005);
            if t > 10.0 {
                break;
            }
        }
        None
    }

    fn bounds(&self) -> (Vec3, Vec3) {
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        for (layer, _) in &self.layers {
            if let Layer::Solid { prim, k } = layer {
                let (a, b) = prim.bounds();
                lo = lo.min(a - *k);
                hi = hi.max(b + *k);
            }
        }
        (lo, hi)
    }

    /// Malla lisa de la superficie y su "casco" para el contorno (desplazado `outline` hacia fuera).
    pub fn build_meshes(&self, cell: f32, outline: f32) -> (Mesh, Mesh) {
        let data = self.surface_nets(cell);
        let outline_positions: Vec<[f32; 3]> = data
            .positions
            .iter()
            .zip(&data.normals)
            .map(|(p, n)| (Vec3::from(*p) + Vec3::from(*n) * outline).to_array())
            .collect();
        let body = mesh(data.positions, data.normals.clone(), Some(data.colors), data.indices.clone());
        let hull = mesh(outline_positions, data.normals, None, data.indices);
        (body, hull)
    }

    pub fn surface_nets(&self, cell: f32) -> MeshData {
        let (lo, hi) = self.bounds();
        let lo = lo - Vec3::splat(cell * 2.0);
        let hi = hi + Vec3::splat(cell * 2.0);
        let n = ((hi - lo) / cell).ceil().as_uvec3() + UVec3::ONE;
        let (nx, ny, nz) = (n.x as usize, n.y as usize, n.z as usize);
        let point = |x: usize, y: usize, z: usize| lo + Vec3::new(x as f32, y as f32, z as f32) * cell;
        let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

        let mut values = vec![0.0f32; nx * ny * nz];
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    values[idx(x, y, z)] = self.dist(point(x, y, z));
                }
            }
        }

        // Un vértice por celda que corta la superficie: la media de los cortes de sus aristas
        let cidx = |x: usize, y: usize, z: usize| x + (nx - 1) * (y + (ny - 1) * z);
        let mut cell_vertex = vec![u32::MAX; (nx - 1) * (ny - 1) * (nz - 1)];
        let mut data = MeshData::default();
        const CORNERS: [[usize; 3]; 8] =
            [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0], [0, 0, 1], [1, 0, 1], [0, 1, 1], [1, 1, 1]];
        const EDGES: [[usize; 2]; 12] =
            [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]];

        for z in 0..nz - 1 {
            for y in 0..ny - 1 {
                for x in 0..nx - 1 {
                    let v: [f32; 8] = CORNERS.map(|[dx, dy, dz]| values[idx(x + dx, y + dy, z + dz)]);
                    let inside = v.iter().filter(|d| **d < 0.0).count();
                    if inside == 0 || inside == 8 {
                        continue;
                    }
                    let mut sum = Vec3::ZERO;
                    let mut count = 0.0;
                    for [a, b] in EDGES {
                        if (v[a] < 0.0) != (v[b] < 0.0) {
                            let t = v[a] / (v[a] - v[b]);
                            let pa = Vec3::from(CORNERS[a].map(|c| c as f32));
                            let pb = Vec3::from(CORNERS[b].map(|c| c as f32));
                            sum += pa.lerp(pb, t);
                            count += 1.0;
                        }
                    }
                    let p = point(x, y, z) + sum / count * cell;
                    let (_, color) = self.sample(p);
                    cell_vertex[cidx(x, y, z)] = data.positions.len() as u32;
                    data.positions.push(p.to_array());
                    data.normals.push(self.normal(p).to_array());
                    data.colors.push([color.x, color.y, color.z, 1.0]);
                }
            }
        }

        // Una cara por cada arista de la rejilla que corta la superficie
        for z in 1..nz - 1 {
            for y in 1..ny - 1 {
                for x in 1..nx - 1 {
                    let d0 = values[idx(x, y, z)];
                    for axis in 0..3 {
                        let (ex, ey, ez) = [(1, 0, 0), (0, 1, 0), (0, 0, 1)][axis];
                        if x + ex >= nx || y + ey >= ny || z + ez >= nz {
                            continue;
                        }
                        let d1 = values[idx(x + ex, y + ey, z + ez)];
                        if (d0 < 0.0) == (d1 < 0.0) {
                            continue;
                        }
                        // Las 4 celdas que comparten la arista, en orden antihorario visto desde +eje
                        let cells = match axis {
                            0 => [(x, y - 1, z - 1), (x, y, z - 1), (x, y, z), (x, y - 1, z)],
                            1 => [(x - 1, y, z - 1), (x - 1, y, z), (x, y, z), (x, y, z - 1)],
                            _ => [(x - 1, y - 1, z), (x, y - 1, z), (x, y, z), (x - 1, y, z)],
                        };
                        let q = cells.map(|(cx, cy, cz)| cell_vertex[cidx(cx, cy, cz)]);
                        if q.contains(&u32::MAX) {
                            continue;
                        }
                        // La normal sale de dentro (negativo) hacia fuera
                        if d0 < 0.0 {
                            data.indices.extend([q[0], q[1], q[2], q[0], q[2], q[3]]);
                        } else {
                            data.indices.extend([q[0], q[2], q[1], q[0], q[3], q[2]]);
                        }
                    }
                }
            }
        }
        data
    }
}

#[derive(Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

fn smin(a: f32, b: f32, k: f32) -> (f32, f32) {
    if b == f32::MAX {
        return (a, 1.0);
    }
    if k <= 0.0 {
        return if a < b { (a, 1.0) } else { (b, 0.0) };
    }
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    (b + (a - b) * h - k * h * (1.0 - h), h)
}

fn mesh(positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, colors: Option<Vec<[f32; 4]>>, indices: Vec<u32>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices));
    if let Some(colors) = colors {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_mesh_is_closed_and_faces_point_outwards() {
        let sdf = Sdf::new().add(ellipsoid(Vec3::ZERO, Vec3::splat(0.5)), [255, 0, 0], 0.0);
        let data = sdf.surface_nets(0.05);
        assert!(data.indices.len() > 300);
        for p in &data.positions {
            assert!((Vec3::from(*p).length() - 0.5).abs() < 0.03, "vértice fuera de la esfera: {p:?}");
        }
        for tri in data.indices.chunks(3) {
            let [a, b, c] = [0, 1, 2].map(|i| Vec3::from(data.positions[tri[i] as usize]));
            let face = (b - a).cross(c - a);
            assert!(face.dot((a + b + c) / 3.0) > 0.0, "triángulo mirando hacia dentro");
        }
    }

    #[test]
    fn hit_finds_the_surface() {
        let sdf = Sdf::new().add(ellipsoid(Vec3::ZERO, Vec3::new(0.3, 0.5, 0.4)), [0, 0, 0], 0.0);
        let (p, n) = sdf.hit(Vec3::new(0.0, 0.0, 3.0), Vec3::NEG_Z).unwrap();
        assert!((p.z - 0.4).abs() < 0.01);
        assert!(n.z > 0.99);
    }
}
