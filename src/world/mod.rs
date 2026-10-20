//! Mapas dibujados como texto (los de Kanto están en `kanto.rs`).
//!
//! Leyenda:
//! `#` árbol (o pared de roca en cuevas, o muro en interiores) · `g` hierba alta · `.` suelo
//! `:` camino · `~` agua · `=` puente · `f` flores · `F` valla · `r` roca · `b` arbusto · `s` cartel
//! `M` montaña · `v` saliente (se salta hacia abajo) · `o` objeto en el suelo · `*` persona
//! `h` casa · `c` Centro Pokémon · `m` Tienda · `G` gimnasio · `l` laboratorio · `B` edificio grande
//! (cada rectángulo de letras es un edificio; la puerta queda en el centro de su lado sur)
//! Interiores: `C` mostrador · `S` estantería · `T` mesa · `K` planta · `U` estatua · `P` máquina · `x` alfombra
//! `0`-`9` salida (ver `Exit`).

pub mod kanto;
pub mod npcs;
pub mod overworld;

use bevy::prelude::*;

use crate::render::models::Rgb;
use crate::world::npcs::Npc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Ground,
    Path,
    Grass,
    Tree,
    Water,
    Bridge,
    Flowers,
    Fence,
    Rock,
    Bush,
    Sign,
    Mountain,
    Ledge,
    Building,
    Item,
    Counter,
    Shelf,
    Table,
    Plant,
    Machine,
    Carpet,
    Exit(u8),
}

impl Tile {
    pub fn walkable(self) -> bool {
        matches!(
            self,
            Tile::Ground | Tile::Path | Tile::Grass | Tile::Flowers | Tile::Bridge | Tile::Carpet | Tile::Exit(_)
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Town,
    Route,
    Forest,
    Cave,
    Inside,
    Gym,
    /// La Torre Pokémon: interior oscuro con fantasmas.
    Tower,
    Sea,
}

impl Theme {
    pub fn indoors(self) -> bool {
        matches!(self, Theme::Inside | Theme::Gym | Theme::Tower)
    }

    pub fn underground(self) -> bool {
        matches!(self, Theme::Cave | Theme::Tower)
    }

    /// Aparecen Pokémon salvajes en cualquier casilla, no solo en la hierba alta.
    pub fn wild_everywhere(self) -> bool {
        matches!(self, Theme::Cave | Theme::Tower)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildingKind {
    House,
    Center,
    Mart,
    Gym,
    Lab,
    Big,
}

#[derive(Clone, Copy, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    /// Esquina suroeste (en casillas) y tamaño.
    pub min: IVec2,
    pub size: IVec2,
    pub roof: Rgb,
    /// Posición en orden de lectura (para `MapDef::doors`).
    pub index: usize,
}

impl Building {
    /// Casilla de la puerta (dentro del edificio, en su lado sur).
    pub fn door(&self) -> IVec2 {
        IVec2::new(self.min.x + self.size.x / 2, self.min.y)
    }
}

/// A dónde lleva una salida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Link {
    /// A la salida con ese número de otro mapa.
    Map(usize, u8),
    /// Sales por la puerta por la que entraste (interiores).
    Back,
}

/// Cómo se ve la salida.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Look {
    /// Camino que sigue fuera del mapa.
    Edge,
    /// Boca de cueva en la montaña.
    Cave,
    LadderDown,
    LadderUp,
    /// Felpudo de salida de un interior.
    Mat,
}

pub struct Exit {
    pub digit: u8,
    pub to: Link,
    /// Hacia dónde das el primer paso al llegar por esta salida (hacia dentro del mapa).
    pub inward: IVec2,
    pub look: Look,
}

pub const N: IVec2 = IVec2::Y;
pub const S: IVec2 = IVec2::NEG_Y;
pub const E: IVec2 = IVec2::X;
pub const W: IVec2 = IVec2::NEG_X;

pub const fn edge(digit: u8, map: usize, to_digit: u8, inward: IVec2) -> Exit {
    Exit { digit, to: Link::Map(map, to_digit), inward, look: Look::Edge }
}

pub const fn cave(digit: u8, map: usize, to_digit: u8, inward: IVec2) -> Exit {
    Exit { digit, to: Link::Map(map, to_digit), inward, look: Look::Cave }
}

pub const fn ladder(digit: u8, map: usize, to_digit: u8, inward: IVec2, down: bool) -> Exit {
    Exit { digit, to: Link::Map(map, to_digit), inward, look: if down { Look::LadderDown } else { Look::LadderUp } }
}

/// La salida de un interior (felpudo al sur).
pub const fn mat() -> Exit {
    Exit { digit: 0, to: Link::Back, inward: N, look: Look::Mat }
}

/// Qué hay tras la puerta de un edificio.
#[derive(Clone, Copy, Debug)]
pub enum Door {
    /// El tipo de edificio decide (Centro, Tienda, casa...).
    Default,
    Map(usize),
    Text(&'static str),
    /// Cerrado hasta tener tantas medallas.
    Badges(u8, usize, &'static str),
}

pub struct MapDef {
    pub name: &'static str,
    pub rows: &'static str,
    pub theme: Theme,
    pub ground: Rgb,
    pub exits: &'static [Exit],
    /// Una por edificio en orden de lectura (las que falten: `Door::Default`).
    pub doors: &'static [Door],
    /// Texto de cada cartel y estatua, en orden de lectura.
    pub signs: &'static [&'static str],
    /// Cada `*` del mapa, en orden de lectura.
    pub npcs: &'static [Npc],
    /// Cada `o` del mapa, en orden de lectura: (objeto, cantidad).
    pub items: &'static [(usize, u16)],
    /// (especie, peso) de los Pokémon salvajes.
    pub encounters: &'static [(usize, u32)],
    pub levels: (u32, u32),
    /// Color del tejado del gimnasio (cada ciudad tiene el suyo).
    pub roof: Rgb,
}

pub const BASE: MapDef = MapDef {
    name: "",
    rows: "",
    theme: Theme::Route,
    ground: [104, 184, 84],
    exits: &[],
    doors: &[],
    signs: &[],
    npcs: &[],
    items: &[],
    encounters: &[],
    levels: (2, 4),
    roof: [200, 70, 60],
};

pub use crate::world::kanto::MAPS;

/// El mapa cargado: casillas, edificios, carteles y personas ya interpretados.
#[derive(Clone)]
pub struct LoadedMap {
    pub id: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub buildings: Vec<Building>,
    /// Posición de cada cartel/estatua, en el mismo orden que `MapDef::signs`.
    pub signs: Vec<IVec2>,
    pub npcs: Vec<IVec2>,
    pub items: Vec<IVec2>,
}

const ROOFS: [Rgb; 5] = [[200, 70, 60], [70, 110, 190], [90, 150, 80], [170, 110, 60], [150, 90, 170]];

impl LoadedMap {
    pub fn load(id: usize) -> Self {
        let lines: Vec<&str> = MAPS[id].rows.trim_matches('\n').lines().collect();
        let h = lines.len() as i32;
        let chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        let at = |pos: IVec2| chars[(h - 1 - pos.y) as usize][pos.x as usize];

        let tiles = chars
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| match c {
                        '#' => Tile::Tree,
                        'g' => Tile::Grass,
                        ':' => Tile::Path,
                        '~' => Tile::Water,
                        '=' => Tile::Bridge,
                        'f' => Tile::Flowers,
                        'F' => Tile::Fence,
                        'r' => Tile::Rock,
                        'b' => Tile::Bush,
                        's' | 'U' => Tile::Sign,
                        'M' => Tile::Mountain,
                        'v' => Tile::Ledge,
                        'o' => Tile::Item,
                        'C' => Tile::Counter,
                        'S' => Tile::Shelf,
                        'T' => Tile::Table,
                        'K' => Tile::Plant,
                        'P' => Tile::Machine,
                        'x' => Tile::Carpet,
                        'h' | 'c' | 'm' | 'G' | 'l' | 'B' => Tile::Building,
                        d if d.is_ascii_digit() => Tile::Exit(*d as u8 - b'0'),
                        _ => Tile::Ground,
                    })
                    .collect()
            })
            .collect();

        // Cada rectángulo de letras de edificio es un edificio: lo recorremos desde su esquina suroeste
        let mut buildings = vec![];
        let (mut signs, mut npcs, mut items) = (vec![], vec![], vec![]);
        let w = chars[0].len() as i32;
        for y in 0..h {
            for x in 0..w {
                let pos = IVec2::new(x, y);
                let c = at(pos);
                match c {
                    's' | 'U' => signs.push(pos),
                    '*' => npcs.push(pos),
                    'o' => items.push(pos),
                    _ => {}
                }
                let kind = match c {
                    'h' => BuildingKind::House,
                    'c' => BuildingKind::Center,
                    'm' => BuildingKind::Mart,
                    'G' => BuildingKind::Gym,
                    'l' => BuildingKind::Lab,
                    'B' => BuildingKind::Big,
                    _ => continue,
                };
                let is_corner = (x == 0 || at(pos - IVec2::X) != c) && (y == 0 || at(pos - IVec2::Y) != c);
                if !is_corner {
                    continue;
                }
                let mut size = IVec2::ONE;
                while x + size.x < w && at(IVec2::new(x + size.x, y)) == c {
                    size.x += 1;
                }
                while y + size.y < h && at(IVec2::new(x, y + size.y)) == c {
                    size.y += 1;
                }
                buildings.push(Building { kind, min: pos, size, roof: [0; 3], index: 0 });
            }
        }
        // Orden de lectura (de arriba abajo y de izquierda a derecha), como aparecen en el texto
        let reading = |p: &IVec2| (-p.y, p.x);
        signs.sort_by_key(reading);
        npcs.sort_by_key(reading);
        items.sort_by_key(reading);
        buildings.sort_by_key(|b| (-(b.min.y + b.size.y - 1), b.min.x));
        let mut houses = 0;
        for (i, b) in buildings.iter_mut().enumerate() {
            b.index = i;
            b.roof = match b.kind {
                BuildingKind::Center => [225, 60, 70],
                BuildingKind::Mart => [60, 120, 210],
                BuildingKind::Lab => [120, 130, 150],
                BuildingKind::Gym => MAPS[id].roof,
                BuildingKind::Big => [130, 125, 140],
                BuildingKind::House => {
                    houses += 1;
                    ROOFS[(houses + id) % ROOFS.len()]
                }
            };
        }
        Self { id, tiles, buildings, signs, npcs, items }
    }

    pub fn def(&self) -> &'static MapDef {
        &MAPS[self.id]
    }

    pub fn size(&self) -> IVec2 {
        IVec2::new(self.tiles[0].len() as i32, self.tiles.len() as i32)
    }

    pub fn get(&self, pos: IVec2) -> Tile {
        // y crece hacia arriba en el mapa, pero hacia abajo en el texto
        let row = self.tiles.len() as i32 - 1 - pos.y;
        if row < 0 || pos.x < 0 {
            return Tile::Tree;
        }
        self.tiles.get(row as usize).and_then(|r| r.get(pos.x as usize)).copied().unwrap_or(Tile::Tree)
    }

    pub fn set(&mut self, pos: IVec2, tile: Tile) {
        let row = self.tiles.len() as i32 - 1 - pos.y;
        if let Some(t) = self.tiles.get_mut(row as usize).and_then(|r| r.get_mut(pos.x as usize)) {
            *t = tile;
        }
    }

    /// Las casillas de una salida, de oeste a este (y de sur a norte).
    pub fn exit_tiles(&self, digit: u8) -> Vec<IVec2> {
        let size = self.size();
        let mut tiles: Vec<IVec2> = (0..size.y)
            .flat_map(|y| (0..size.x).map(move |x| IVec2::new(x, y)))
            .filter(|p| self.get(*p) == Tile::Exit(digit))
            .collect();
        tiles.sort_by_key(|p| (p.x, p.y));
        tiles
    }

    pub fn exit(&self, digit: u8) -> Option<&'static Exit> {
        self.def().exits.iter().find(|e| e.digit == digit)
    }

    pub fn building_at(&self, pos: IVec2) -> Option<&Building> {
        self.buildings.iter().find(|b| {
            let max = b.min + b.size;
            pos.x >= b.min.x && pos.y >= b.min.y && pos.x < max.x && pos.y < max.y
        })
    }

    pub fn door(&self, b: &Building) -> Door {
        match self.def().doors.get(b.index).copied().unwrap_or(Door::Default) {
            Door::Default => match b.kind {
                BuildingKind::Center => Door::Map(crate::world::kanto::CENTER_IN),
                BuildingKind::Mart => Door::Map(crate::world::kanto::MART_IN),
                BuildingKind::House => Door::Map(crate::world::kanto::HOUSE_IN),
                BuildingKind::Lab => Door::Text("Está cerrado."),
                BuildingKind::Gym => Door::Text("El gimnasio está cerrado. El líder no está."),
                BuildingKind::Big => Door::Text("La puerta está cerrada con llave."),
            },
            d => d,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_are_rectangular_and_exits_match() {
        for (id, def) in MAPS.iter().enumerate() {
            let map = LoadedMap::load(id);
            let w = map.tiles[0].len();
            assert!(map.tiles.iter().all(|r| r.len() == w), "{}: filas de distinto ancho", def.name);
            assert_eq!(map.signs.len(), def.signs.len(), "{}: carteles", def.name);
            assert_eq!(map.npcs.len(), def.npcs.len(), "{}: personas", def.name);
            assert_eq!(map.items.len(), def.items.len(), "{}: objetos", def.name);
            assert!(def.npcs.len() <= crate::world::npcs::NPCS_PER_MAP, "{}: demasiadas personas", def.name);
            assert!(map.buildings.len() >= def.doors.len(), "{}: más puertas que edificios", def.name);
            if !def.encounters.is_empty() {
                assert!(def.levels.0 <= def.levels.1);
            }
            for exit in def.exits {
                let here = map.exit_tiles(exit.digit);
                assert!(!here.is_empty(), "{}: falta la salida {}", def.name, exit.digit);
                for tile in &here {
                    assert!(map.get(*tile + exit.inward).walkable(), "{}: al llegar por {} caes en un obstáculo", def.name, exit.digit);
                }
                if let Link::Map(to, to_digit) = exit.to {
                    let other = LoadedMap::load(to);
                    let there = other.exit_tiles(to_digit);
                    assert!(!there.is_empty(), "{}: la salida {} lleva a {} pero allí no existe la {}", def.name, exit.digit, MAPS[to].name, to_digit);
                    let back = other.exit(to_digit).unwrap_or_else(|| panic!("{}: falta definir la salida {}", MAPS[to].name, to_digit));
                    assert_eq!(back.to, Link::Map(id, exit.digit), "{} -> {}: la vuelta no coincide", def.name, MAPS[to].name);
                }
            }
            for row_tile in map.tiles.iter().flatten() {
                if let Tile::Exit(d) = row_tile {
                    assert!(map.exit(*d).is_some(), "{}: la salida {} no está definida", def.name, d);
                }
            }
            for b in &map.buildings {
                assert!(map.get(b.door() + S).walkable(), "{}: la puerta de {:?} da a un obstáculo", def.name, b.kind);
                if let Door::Map(to) | Door::Badges(_, to, _) = map.door(b) {
                    assert!(MAPS[to].exits.iter().any(|e| e.to == Link::Back), "{}: el interior no tiene salida", MAPS[to].name);
                }
            }
        }
    }
}
