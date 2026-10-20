//! Todo lo que avanza en la partida (y se guarda): dinero, medallas, Pokédex, a quién has
//! ganado, qué objetos has recogido, dónde estás...

use bevy::prelude::*;

use crate::pokemon::Creature;
use crate::world::kanto;

/// Conjunto de banderas numeradas, compacto (un bit cada una).
#[derive(Clone, Default, PartialEq, Debug)]
pub struct Flags(pub Vec<u64>);

impl Flags {
    pub fn get(&self, i: usize) -> bool {
        self.0.get(i / 64).is_some_and(|w| w >> (i % 64) & 1 == 1)
    }

    pub fn set(&mut self, i: usize) {
        if self.0.len() <= i / 64 {
            self.0.resize(i / 64 + 1, 0);
        }
        self.0[i / 64] |= 1 << (i % 64);
    }

    pub fn count(&self) -> u32 {
        self.0.iter().map(|w| w.count_ones()).sum()
    }
}

/// Dónde estás: mapa, casilla y hacia dónde miras.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Place {
    pub map: usize,
    pub pos: IVec2,
}

#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Progress {
    pub name: String,
    /// Número de entrenador (como en la ficha de los juegos).
    pub id: u16,
    pub money: u32,
    /// Una bit por medalla.
    pub badges: u8,
    pub seconds: f64,
    /// Personas con las que ya has terminado (entrenadores ganados, regalos, preguntas...).
    pub done: Flags,
    /// Objetos del suelo recogidos (`kanto::item_id`).
    pub taken: Flags,
    pub seen: Flags,
    pub caught: Flags,
    /// Dónde reapareces si se debilita todo tu equipo.
    pub last_center: Place,
    /// Por dónde volver al salir de un interior.
    pub back: Place,
    pub repel: u32,
    pub steps: u32,
    pub champion: bool,
    /// Pokémon guardados en el PC (los que no caben en el equipo).
    pub boxed: Vec<Creature>,
}

pub const MAX_MONEY: u32 = 999_999;

impl Default for Progress {
    fn default() -> Self {
        let home = kanto::home_door();
        Self {
            name: "Rojo".into(),
            id: rand::random(),
            money: 3000,
            badges: 0,
            seconds: 0.0,
            done: Flags::default(),
            taken: Flags::default(),
            seen: Flags::default(),
            caught: Flags::default(),
            last_center: Place { map: kanto::PLAYER_HOUSE, pos: kanto::house_start() },
            back: home,
            repel: 0,
            steps: 0,
            champion: false,
            boxed: vec![],
        }
    }
}

impl Progress {
    pub fn badge_count(&self) -> u32 {
        self.badges.count_ones()
    }

    pub fn has_badge(&self, i: u8) -> bool {
        self.badges >> i & 1 == 1
    }

    pub fn earn(&mut self, money: u32) {
        self.money = (self.money + money).min(MAX_MONEY);
    }

    pub fn see(&mut self, species: usize) {
        self.seen.set(species);
    }

    pub fn catch(&mut self, species: usize) {
        self.seen.set(species);
        self.caught.set(species);
    }

    /// "12:05" (horas:minutos).
    pub fn time_text(&self) -> String {
        let minutes = (self.seconds / 60.0) as u64;
        format!("{}:{:02}", minutes / 60, minutes % 60)
    }
}

/// Cuenta el tiempo de juego.
pub fn tick_time(time: Res<Time>, mut progress: ResMut<Progress>) {
    progress.bypass_change_detection().seconds += time.delta_secs_f64();
}
