//! Objetos de la mochila: Poké Balls, medicinas y objetos de campo, con su precio en la tienda.

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Effect {
    /// Multiplicador de captura (la Master Ball nunca falla).
    Ball(f32),
    /// Cura tantos PS (`i32::MAX` = todos).
    Heal(i32),
    Revive,
    /// Durante tantos pasos no salen Pokémon salvajes más débiles que el tuyo.
    Repel(u32),
    /// Te saca de una cueva al último Centro Pokémon.
    EscapeRope,
    RareCandy,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pocket {
    Balls,
    Medicine,
    Field,
}

impl Pocket {
    pub const ALL: [Pocket; 3] = [Pocket::Balls, Pocket::Medicine, Pocket::Field];

    pub fn name(self) -> &'static str {
        match self {
            Pocket::Balls => "Poké Balls",
            Pocket::Medicine => "Medicinas",
            Pocket::Field => "Objetos",
        }
    }
}

pub struct Item {
    pub name: &'static str,
    pub effect: Effect,
    /// 0 = no se vende.
    pub price: u32,
    /// Medallas necesarias para que aparezca en las tiendas.
    pub badges: u8,
    pub desc: &'static str,
    /// Color del icono.
    pub color: [u8; 3],
}

impl Item {
    pub fn pocket(&self) -> Pocket {
        match self.effect {
            Effect::Ball(_) => Pocket::Balls,
            Effect::Heal(_) | Effect::Revive => Pocket::Medicine,
            _ => Pocket::Field,
        }
    }

    pub fn usable_in_battle(&self) -> bool {
        matches!(self.effect, Effect::Ball(_) | Effect::Heal(_) | Effect::Revive)
    }

    /// Hay que elegir a qué Pokémon se le da.
    pub fn needs_target(&self) -> bool {
        matches!(self.effect, Effect::Heal(_) | Effect::Revive | Effect::RareCandy)
    }
}

pub const POKE_BALL: usize = 0;
pub const SUPER_BALL: usize = 1;
pub const ULTRA_BALL: usize = 2;
pub const MASTER_BALL: usize = 3;
pub const POTION: usize = 4;
pub const SUPER_POTION: usize = 5;
pub const HYPER_POTION: usize = 6;
pub const MAX_POTION: usize = 7;
pub const REVIVE: usize = 8;
pub const REPEL: usize = 9;
pub const SUPER_REPEL: usize = 10;
pub const ESCAPE_ROPE: usize = 11;
pub const RARE_CANDY: usize = 12;

pub const ITEMS: [Item; 13] = [
    Item {
        name: "Poké Ball",
        effect: Effect::Ball(1.0),
        price: 200,
        badges: 0,
        desc: "Una Poké Ball de toda la vida para atrapar Pokémon salvajes.",
        color: [230, 50, 55],
    },
    Item {
        name: "Super Ball",
        effect: Effect::Ball(1.5),
        price: 600,
        badges: 2,
        desc: "Una Poké Ball de buena calidad: atrapa Pokémon con más facilidad.",
        color: [60, 120, 230],
    },
    Item {
        name: "Ultra Ball",
        effect: Effect::Ball(2.0),
        price: 1200,
        badges: 4,
        desc: "Una Poké Ball de altísimo rendimiento. Muy eficaz.",
        color: [250, 200, 40],
    },
    Item {
        name: "Master Ball",
        effect: Effect::Ball(255.0),
        price: 0,
        badges: 0,
        desc: "La mejor Poké Ball que existe. Atrapa cualquier Pokémon sin fallar.",
        color: [140, 60, 190],
    },
    Item {
        name: "Poción",
        effect: Effect::Heal(20),
        price: 300,
        badges: 0,
        desc: "Espray medicinal que restaura 20 PS a un Pokémon.",
        color: [170, 90, 220],
    },
    Item {
        name: "Superpoción",
        effect: Effect::Heal(50),
        price: 700,
        badges: 2,
        desc: "Espray medicinal que restaura 50 PS a un Pokémon.",
        color: [235, 90, 120],
    },
    Item {
        name: "Hiperpoción",
        effect: Effect::Heal(200),
        price: 1200,
        badges: 4,
        desc: "Espray medicinal muy potente: restaura 200 PS a un Pokémon.",
        color: [240, 140, 200],
    },
    Item {
        name: "Poción Máxima",
        effect: Effect::Heal(i32::MAX),
        price: 2500,
        badges: 6,
        desc: "Restaura todos los PS de un Pokémon.",
        color: [80, 150, 240],
    },
    Item {
        name: "Revivir",
        effect: Effect::Revive,
        price: 1500,
        badges: 3,
        desc: "Reanima a un Pokémon debilitado y le devuelve la mitad de sus PS.",
        color: [250, 210, 60],
    },
    Item {
        name: "Repelente",
        effect: Effect::Repel(100),
        price: 350,
        badges: 0,
        desc: "Durante 100 pasos no aparecerán Pokémon salvajes débiles.",
        color: [90, 190, 120],
    },
    Item {
        name: "Superrepelente",
        effect: Effect::Repel(200),
        price: 500,
        badges: 3,
        desc: "Durante 200 pasos no aparecerán Pokémon salvajes débiles.",
        color: [150, 100, 220],
    },
    Item {
        name: "Cuerda Huida",
        effect: Effect::EscapeRope,
        price: 550,
        badges: 0,
        desc: "Una cuerda larga para salir de cuevas y torres al instante.",
        color: [200, 150, 90],
    },
    Item {
        name: "Caramelo Raro",
        effect: Effect::RareCandy,
        price: 0,
        badges: 0,
        desc: "Un caramelo lleno de energía. Sube un nivel a un Pokémon.",
        color: [120, 200, 240],
    },
];

/// Lo que llevas en la mochila: cuántos tienes de cada objeto (mismo orden que `ITEMS`).
#[derive(Resource, Clone, PartialEq, Debug)]
pub struct Bag(pub [u16; ITEMS.len()]);

impl Default for Bag {
    fn default() -> Self {
        let mut bag = Self([0; ITEMS.len()]);
        bag.0[POKE_BALL] = 5;
        bag.0[POTION] = 2;
        bag
    }
}

pub const MAX_STACK: u16 = 999;

impl Bag {
    pub fn count(&self, item: usize) -> u16 {
        self.0[item]
    }

    pub fn add(&mut self, item: usize, n: u16) {
        self.0[item] = (self.0[item] + n).min(MAX_STACK);
    }

    /// Gasta uno; `false` si no quedaba.
    pub fn take(&mut self, item: usize) -> bool {
        if self.0[item] == 0 {
            return false;
        }
        self.0[item] -= 1;
        true
    }

    /// Los objetos que tienes de un bolsillo, en orden.
    pub fn pocket(&self, pocket: Pocket) -> Vec<usize> {
        (0..ITEMS.len()).filter(|&i| self.0[i] > 0 && ITEMS[i].pocket() == pocket).collect()
    }

    pub fn balls(&self) -> u32 {
        self.pocket(Pocket::Balls).iter().map(|&i| self.0[i] as u32).sum()
    }
}

/// Lo que vende una tienda según tus medallas.
pub fn shop_stock(badges: u32) -> Vec<usize> {
    (0..ITEMS.len()).filter(|&i| ITEMS[i].price > 0 && ITEMS[i].badges as u32 <= badges).collect()
}
