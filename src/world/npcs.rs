//! La gente de los mapas: vecinos que hablan, entrenadores, líderes de gimnasio, la enfermera
//! del Centro, el dependiente de la tienda, guardias, regalos y las preguntas de tipos.

use bevy::prelude::*;

/// Máximo de personas por mapa (para numerarlas en la partida guardada).
pub const NPCS_PER_MAP: usize = 16;

pub struct Npc {
    pub name: &'static str,
    /// Índice en `models::NPC_OUTFITS`.
    pub look: usize,
    pub facing: IVec2,
    pub role: Role,
    /// Cuando ya has hablado con él (o le has ganado) se aparta tanto.
    pub aside: Option<IVec2>,
}

pub enum Role {
    Talk(&'static [&'static str]),
    Quiz(Quiz),
    Trainer(Trainer),
    /// Cura a tu equipo (Centro Pokémon).
    Nurse,
    /// Abre la tienda.
    Clerk,
    /// Cura a tu equipo en casa.
    Mom,
    Gift { lines: &'static [&'static str], item: usize, qty: u16, after: &'static str },
    /// Te deja pasar si tienes tantas medallas.
    Guard { badges: u8, blocked: &'static str, ok: &'static str },
}

pub struct Trainer {
    pub team: &'static [(usize, u32)],
    pub intro: &'static [&'static str],
    /// Lo que dice al perder (en el combate).
    pub lose: &'static str,
    /// Lo que dice si le vuelves a hablar.
    pub after: &'static str,
    /// Casillas que ve delante de él (0 = solo si le hablas).
    pub sight: i32,
    /// Medalla que da (líderes de gimnasio).
    pub badge: Option<u8>,
    /// Dinero por nivel de su Pokémon más fuerte.
    pub pay: u32,
}

pub enum Reward {
    Item(usize, u16),
    Xp(u32),
}

pub struct Quiz {
    pub intro: &'static [&'static str],
    pub question: &'static str,
    pub options: &'static [&'static str],
    /// Índices de las respuestas correctas, ordenados
    pub correct: &'static [usize],
    pub reward: Reward,
    pub beaten: &'static str,
}

impl Npc {
    pub fn trainer(&self) -> Option<&Trainer> {
        match &self.role {
            Role::Trainer(t) => Some(t),
            _ => None,
        }
    }
}

/// Identificador único de una persona en todo el juego.
pub fn npc_id(map: usize, index: usize) -> usize {
    map * NPCS_PER_MAP + index
}

pub fn npc(id: usize) -> &'static Npc {
    &crate::world::MAPS[id / NPCS_PER_MAP].npcs[id % NPCS_PER_MAP]
}

// ---------------------------------------------------------------- atajos para escribir los mapas

pub const fn talk(name: &'static str, look: usize, facing: IVec2, lines: &'static [&'static str]) -> Npc {
    Npc { name, look, facing, role: Role::Talk(lines), aside: None }
}

pub const fn trainer(
    name: &'static str,
    look: usize,
    facing: IVec2,
    sight: i32,
    team: &'static [(usize, u32)],
    intro: &'static [&'static str],
    lose: &'static str,
    after: &'static str,
) -> Npc {
    Npc { name, look, facing, role: Role::Trainer(Trainer { team, intro, lose, after, sight, badge: None, pay: 20 }), aside: None }
}

pub const fn leader(
    name: &'static str,
    look: usize,
    badge: u8,
    team: &'static [(usize, u32)],
    intro: &'static [&'static str],
    lose: &'static str,
    after: &'static str,
) -> Npc {
    Npc {
        name,
        look,
        facing: IVec2::NEG_Y,
        role: Role::Trainer(Trainer { team, intro, lose, after, sight: 0, badge: Some(badge), pay: 100 }),
        aside: None,
    }
}

/// Miembro del Alto Mando o rival: se aparta al perder para dejarte pasar.
pub const fn boss(
    name: &'static str,
    look: usize,
    team: &'static [(usize, u32)],
    intro: &'static [&'static str],
    lose: &'static str,
    after: &'static str,
    aside: IVec2,
) -> Npc {
    Npc {
        name,
        look,
        facing: IVec2::NEG_Y,
        role: Role::Trainer(Trainer { team, intro, lose, after, sight: 0, badge: None, pay: 120 }),
        aside: Some(aside),
    }
}

pub const fn nurse() -> Npc {
    Npc { name: "Enfermera Joy", look: crate::render::models::look::NURSE, facing: IVec2::NEG_Y, role: Role::Nurse, aside: None }
}

pub const fn clerk() -> Npc {
    Npc { name: "Dependiente", look: crate::render::models::look::CLERK, facing: IVec2::NEG_Y, role: Role::Clerk, aside: None }
}

pub const fn guard(badges: u8, blocked: &'static str, ok: &'static str, aside: IVec2) -> Npc {
    Npc { name: "Guardia", look: crate::render::models::look::GUARD, facing: IVec2::NEG_Y, role: Role::Guard { badges, blocked, ok }, aside: Some(aside) }
}

// ---------------------------------------------------------------- medallas

pub struct Badge {
    pub name: &'static str,
    pub leader: &'static str,
    pub color: [u8; 3],
}

pub const BADGES: [Badge; 8] = [
    Badge { name: "Medalla Roca", leader: "Brock", color: [150, 150, 160] },
    Badge { name: "Medalla Cascada", leader: "Misty", color: [80, 170, 240] },
    Badge { name: "Medalla Trueno", leader: "Teniente Surge", color: [245, 160, 40] },
    Badge { name: "Medalla Arcoíris", leader: "Erika", color: [120, 200, 90] },
    Badge { name: "Medalla Alma", leader: "Koga", color: [235, 100, 170] },
    Badge { name: "Medalla Pantano", leader: "Sabrina", color: [235, 200, 50] },
    Badge { name: "Medalla Volcán", leader: "Blaine", color: [230, 60, 50] },
    Badge { name: "Medalla Tierra", leader: "Giovanni", color: [70, 160, 90] },
];
