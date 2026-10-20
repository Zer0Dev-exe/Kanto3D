//! Tipos, ataques, especies y Pokémon concretos (con nivel, experiencia y vida).

use bevy::prelude::*;
use rand::Rng;

pub mod sculpt;
pub mod sdf;

use sculpt::CreatureModel;

/// Los 15 tipos de la primera generación.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type {
    Normal,
    Fire,
    Water,
    Grass,
    Electric,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
}

impl Type {
    pub fn name(self) -> &'static str {
        match self {
            Type::Normal => "Normal",
            Type::Fire => "Fuego",
            Type::Water => "Agua",
            Type::Grass => "Planta",
            Type::Electric => "Eléctrico",
            Type::Ice => "Hielo",
            Type::Fighting => "Lucha",
            Type::Poison => "Veneno",
            Type::Ground => "Tierra",
            Type::Flying => "Volador",
            Type::Psychic => "Psíquico",
            Type::Bug => "Bicho",
            Type::Rock => "Roca",
            Type::Ghost => "Fantasma",
            Type::Dragon => "Dragón",
        }
    }

    pub fn color(self) -> Color {
        // Los colores de tipo de los juegos oficiales
        let [r, g, b] = match self {
            Type::Normal => [168, 167, 122],
            Type::Fire => [238, 129, 48],
            Type::Water => [99, 144, 240],
            Type::Grass => [122, 199, 76],
            Type::Electric => [240, 190, 30],
            Type::Ice => [110, 200, 196],
            Type::Fighting => [194, 46, 40],
            Type::Poison => [163, 62, 161],
            Type::Ground => [214, 176, 80],
            Type::Flying => [169, 143, 243],
            Type::Psychic => [249, 85, 135],
            Type::Bug => [166, 185, 26],
            Type::Rock => [182, 161, 54],
            Type::Ghost => [115, 87, 151],
            Type::Dragon => [111, 53, 252],
        };
        Color::srgb_u8(r, g, b)
    }

    /// Multiplicador de daño de un ataque de tipo `self` contra un defensor de tipo `def`.
    pub fn effectiveness(self, def: Type) -> f32 {
        use Type::*;
        let (double, half, immune): (&[Type], &[Type], &[Type]) = match self {
            Normal => (&[], &[Rock], &[Ghost]),
            Fire => (&[Grass, Ice, Bug], &[Fire, Water, Rock, Dragon], &[]),
            Water => (&[Fire, Ground, Rock], &[Water, Grass, Dragon], &[]),
            Electric => (&[Water, Flying], &[Electric, Grass, Dragon], &[Ground]),
            Grass => (&[Water, Ground, Rock], &[Fire, Grass, Poison, Flying, Bug, Dragon], &[]),
            Ice => (&[Grass, Ground, Flying, Dragon], &[Fire, Water, Ice], &[]),
            Fighting => (&[Normal, Ice, Rock], &[Poison, Flying, Psychic, Bug], &[Ghost]),
            Poison => (&[Grass], &[Poison, Ground, Rock, Ghost], &[]),
            Ground => (&[Fire, Electric, Poison, Rock], &[Grass, Bug], &[Flying]),
            Flying => (&[Grass, Fighting, Bug], &[Electric, Rock], &[]),
            Psychic => (&[Fighting, Poison], &[Psychic], &[]),
            Bug => (&[Grass, Psychic], &[Fire, Fighting, Poison, Flying, Ghost], &[]),
            Rock => (&[Fire, Ice, Flying, Bug], &[Fighting, Ground], &[]),
            Ghost => (&[Psychic, Ghost], &[], &[Normal]),
            Dragon => (&[Dragon], &[], &[]),
        };
        if double.contains(&def) {
            2.0
        } else if half.contains(&def) {
            0.5
        } else if immune.contains(&def) {
            0.0
        } else {
            1.0
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Move {
    pub name: &'static str,
    pub ty: Type,
    pub power: i32,
}

const fn mv(name: &'static str, ty: Type, power: i32) -> Move {
    Move { name, ty, power }
}

// ---------------------------------------------------------------- ataques

use Type::*;

const PLACAJE: Move = mv("Placaje", Normal, 40);
const ARANAZO: Move = mv("Arañazo", Normal, 40);
const ATAQUE_RAPIDO: Move = mv("Ataque Rápido", Normal, 40);
const MORDISCO: Move = mv("Mordisco", Normal, 60);
const HIPERDIENTE: Move = mv("Hiperdiente", Normal, 80);
const GOLPE_CUERPO: Move = mv("Golpe Cuerpo", Normal, 85);
const DESTRUCTOR: Move = mv("Destructor", Normal, 40);
const PISOTON: Move = mv("Pisotón", Normal, 65);
const DERRIBO: Move = mv("Derribo", Normal, 90);
const HIPERRAYO: Move = mv("Hiperrayo", Normal, 150);
const ATIZAR: Move = mv("Atizar", Normal, 80);
const AUTODESTRUCCION: Move = mv("Autodestrucción", Normal, 130);
const ASCUAS: Move = mv("Ascuas", Fire, 40);
const COLMILLO_IGNEO: Move = mv("Colmillo Ígneo", Fire, 65);
const RUEDA_FUEGO: Move = mv("Rueda Fuego", Fire, 60);
const LANZALLAMAS: Move = mv("Lanzallamas", Fire, 90);
const LLAMARADA: Move = mv("Llamarada", Fire, 110);
const PISTOLA_AGUA: Move = mv("Pistola Agua", Water, 40);
const BURBUJA: Move = mv("Burbuja", Water, 40);
const RAYO_BURBUJA: Move = mv("Rayo Burbuja", Water, 65);
const SURF: Move = mv("Surf", Water, 90);
const HIDROBOMBA: Move = mv("Hidrobomba", Water, 110);
const SALPICADURA: Move = mv("Salpicadura", Water, 20);
const LATIGO_CEPA: Move = mv("Látigo Cepa", Grass, 45);
const HOJA_AFILADA: Move = mv("Hoja Afilada", Grass, 55);
const MEGAAGOTAR: Move = mv("Megaagotar", Grass, 40);
const DANZA_PETALO: Move = mv("Danza Pétalo", Grass, 110);
const RAYO_SOLAR: Move = mv("Rayo Solar", Grass, 120);
const IMPACTRUENO: Move = mv("Impactrueno", Electric, 40);
const CHISPA: Move = mv("Chispa", Electric, 65);
const RAYO: Move = mv("Rayo", Electric, 90);
const TRUENO: Move = mv("Trueno", Electric, 110);
const VIENTO_HIELO: Move = mv("Viento Hielo", Ice, 55);
const RAYO_AURORA: Move = mv("Rayo Aurora", Ice, 65);
const RAYO_HIELO: Move = mv("Rayo Hielo", Ice, 90);
const VENTISCA: Move = mv("Ventisca", Ice, 110);
const GOLPE_KARATE: Move = mv("Golpe Kárate", Fighting, 50);
const PATADA_BAJA: Move = mv("Patada Baja", Fighting, 60);
const SUMISION: Move = mv("Sumisión", Fighting, 80);
const TAJO_CRUZADO: Move = mv("Tajo Cruzado", Fighting, 100);
const PICOTAZO_VENENO: Move = mv("Picotazo Veneno", Poison, 35);
const ACIDO: Move = mv("Ácido", Poison, 40);
const POLUCION: Move = mv("Polución", Poison, 40);
const RESIDUOS: Move = mv("Residuos", Poison, 65);
const BOMBA_LODO: Move = mv("Bomba Lodo", Poison, 90);
const BOFETON_LODO: Move = mv("Bofetón Lodo", Ground, 40);
const MAGNITUD: Move = mv("Magnitud", Ground, 70);
const EXCAVAR: Move = mv("Excavar", Ground, 80);
const TERREMOTO: Move = mv("Terremoto", Ground, 100);
const TORNADO: Move = mv("Tornado", Flying, 40);
const PICOTAZO: Move = mv("Picotazo", Flying, 35);
const ATAQUE_ALA: Move = mv("Ataque Ala", Flying, 60);
const ATAQUE_AEREO: Move = mv("Ataque Aéreo", Flying, 75);
const CONFUSION: Move = mv("Confusión", Psychic, 50);
const PSICORRAYO: Move = mv("Psicorrayo", Psychic, 65);
const PSIQUICO: Move = mv("Psíquico", Psychic, 90);
const PICADURA: Move = mv("Picadura", Bug, 60);
const DOBLE_ATAQUE: Move = mv("Doble Ataque", Bug, 50);
const CHUPAVIDAS: Move = mv("Chupavidas", Bug, 40);
const LANZARROCAS: Move = mv("Lanzarrocas", Rock, 50);
const AVALANCHA: Move = mv("Avalancha", Rock, 75);
const ROCA_AFILADA: Move = mv("Roca Afilada", Rock, 100);
const LENGUETAZO: Move = mv("Lengüetazo", Ghost, 40);
const TINIEBLAS: Move = mv("Tinieblas", Ghost, 55);
const BOLA_SOMBRA: Move = mv("Bola Sombra", Ghost, 80);
const FURIA_DRAGON: Move = mv("Furia Dragón", Dragon, 50);
const COLA_DRAGON: Move = mv("Cola Dragón", Dragon, 65);
const GARRA_DRAGON: Move = mv("Garra Dragón", Dragon, 85);
const ENVITE_IGNEO: Move = mv("Envite Ígneo", Fire, 100);
const VOLTIO_CRUEL: Move = mv("Voltio Cruel", Electric, 100);

// ---------------------------------------------------------------- especies

pub struct Species {
    pub name: &'static str,
    /// Número de la Pokédex nacional.
    pub dex: u16,
    pub ty: Type,
    pub ty2: Option<Type>,
    /// "Pokémon Lagartija", "Pokémon Ratón"...
    pub kind: &'static str,
    /// Descripción de la Pokédex (con nuestras palabras).
    pub entry: &'static str,
    base_hp: i32,
    base_atk: i32,
    base_def: i32,
    /// Más alto = más fácil de capturar (1.0 normal).
    pub catch: f32,
    /// (nivel en el que se aprende, ataque)
    learnset: &'static [(u32, Move)],
    /// Evoluciona a la especie `.1` al llegar al nivel `.0`.
    pub evolves: Option<(u32, usize)>,
    pub model: fn() -> CreatureModel,
}

impl Species {
    pub fn types(&self) -> impl Iterator<Item = Type> + '_ {
        std::iter::once(self.ty).chain(self.ty2)
    }
}

pub const CHARMANDER: usize = 0;
pub const SQUIRTLE: usize = 1;
pub const BULBASAUR: usize = 2;
pub const PIKACHU: usize = 3;
pub const RATTATA: usize = 4;
pub const PIDGEY: usize = 5;
pub const CATERPIE: usize = 6;
pub const WEEDLE: usize = 7;
pub const GEODUDE: usize = 8;
pub const ONIX: usize = 9;
pub const ZUBAT: usize = 10;
pub const CLEFAIRY: usize = 11;
pub const STARYU: usize = 12;
pub const STARMIE: usize = 13;
pub const VOLTORB: usize = 14;
pub const ODDISH: usize = 15;
pub const TANGELA: usize = 16;
pub const GRIMER: usize = 17;
pub const KOFFING: usize = 18;
pub const ABRA: usize = 19;
pub const GROWLITHE: usize = 20;
pub const PONYTA: usize = 21;
pub const DIGLETT: usize = 22;
pub const DUGTRIO: usize = 23;
pub const RHYHORN: usize = 24;
pub const GASTLY: usize = 25;
pub const MACHOP: usize = 26;
pub const MAGIKARP: usize = 27;
pub const DRATINI: usize = 28;
pub const SEEL: usize = 29;

const fn sp(
    name: &'static str,
    dex: u16,
    (ty, ty2): (Type, Option<Type>),
    kind: &'static str,
    entry: &'static str,
    [base_hp, base_atk, base_def]: [i32; 3],
    learnset: &'static [(u32, Move)],
    model: fn() -> CreatureModel,
) -> Species {
    Species { name, dex, ty, ty2, kind, entry, base_hp, base_atk, base_def, catch: 1.0, learnset, evolves: None, model }
}

const fn one(t: Type) -> (Type, Option<Type>) {
    (t, None)
}

const fn two(a: Type, b: Type) -> (Type, Option<Type>) {
    (a, Some(b))
}

pub const SPECIES: [Species; 30] = [
    sp(
        "Charmander", 4, one(Fire), "Pokémon Lagartija",
        "La llama de su cola indica cómo se siente: si arde con fuerza, está listo para luchar.",
        [22, 11, 9],
        &[(1, ARANAZO), (1, ASCUAS), (9, MORDISCO), (13, COLMILLO_IGNEO), (22, RUEDA_FUEGO), (30, LANZALLAMAS), (42, LLAMARADA)],
        sculpt::charmander,
    ),
    sp(
        "Squirtle", 7, one(Water), "Pokémon Tortuguita",
        "Se esconde en su caparazón para protegerse y contraataca lanzando chorros de agua.",
        [24, 9, 11],
        &[(1, PLACAJE), (1, PISTOLA_AGUA), (9, BURBUJA), (15, MORDISCO), (22, RAYO_BURBUJA), (31, SURF), (42, HIDROBOMBA)],
        sculpt::squirtle,
    ),
    sp(
        "Bulbasaur", 1, two(Grass, Poison), "Pokémon Semilla",
        "Lleva una semilla en el lomo desde que nace. Crece a la vez que él, tomando el sol.",
        [26, 9, 10],
        &[(1, PLACAJE), (1, LATIGO_CEPA), (9, ACIDO), (14, HOJA_AFILADA), (24, GOLPE_CUERPO), (33, BOMBA_LODO), (42, RAYO_SOLAR)],
        sculpt::bulbasaur,
    ),
    sp(
        "Pikachu", 25, one(Electric), "Pokémon Ratón",
        "Guarda electricidad en las mejillas. Cuando se enfada, la suelta de golpe.",
        [20, 12, 8],
        &[(1, ATAQUE_RAPIDO), (1, IMPACTRUENO), (10, CHISPA), (20, PISOTON), (28, RAYO), (40, TRUENO)],
        sculpt::pikachu,
    ),
    sp(
        "Rattata", 19, one(Normal), "Pokémon Ratón",
        "Muerde lo que sea con sus colmillos, que no paran de crecer. Vive donde haya comida.",
        [20, 9, 8],
        &[(1, PLACAJE), (6, ATAQUE_RAPIDO), (10, MORDISCO), (17, HIPERDIENTE), (28, EXCAVAR)],
        sculpt::rattata,
    ),
    sp(
        "Pidgey", 16, two(Normal, Flying), "Pokémon Pajarito",
        "Muy dócil. Prefiere esconderse entre la hierba alta antes que pelear.",
        [19, 10, 8],
        &[(1, PLACAJE), (5, TORNADO), (9, PICOTAZO), (12, ATAQUE_RAPIDO), (19, ATAQUE_ALA), (31, ATAQUE_AEREO)],
        sculpt::pidgey,
    ),
    sp(
        "Caterpie", 10, one(Bug), "Pokémon Gusano",
        "Sus antenas sueltan un olor muy fuerte que espanta a los pájaros que quieren comérselo.",
        [21, 7, 8],
        &[(1, PLACAJE), (6, PICADURA), (14, CHUPAVIDAS)],
        sculpt::caterpie,
    ),
    sp(
        "Weedle", 13, two(Bug, Poison), "Pokémon Oruga",
        "El aguijón de su cabeza es muy venenoso. Suele vivir en bosques espesos.",
        [20, 8, 7],
        &[(1, PICOTAZO_VENENO), (6, PICADURA), (14, DOBLE_ATAQUE)],
        sculpt::weedle,
    ),
    sp(
        "Geodude", 74, two(Rock, Ground), "Pokémon Roca",
        "Vive en montañas y cuevas. Muchos lo pisan sin darse cuenta porque parece una piedra.",
        [22, 11, 14],
        &[(1, PLACAJE), (6, LANZARROCAS), (11, BOFETON_LODO), (16, MAGNITUD), (26, AVALANCHA), (36, TERREMOTO), (45, AUTODESTRUCCION)],
        sculpt::geodude,
    ),
    sp(
        "Onix", 95, two(Rock, Ground), "Pokémon Serpiente Roca",
        "Avanza bajo tierra a gran velocidad, abriendo túneles con su cuerpo de rocas.",
        [24, 10, 18],
        &[(1, PLACAJE), (1, LANZARROCAS), (12, ATIZAR), (20, AVALANCHA), (30, EXCAVAR), (40, ROCA_AFILADA)],
        sculpt::onix,
    ),
    sp(
        "Zubat", 41, two(Poison, Flying), "Pokémon Murciélago",
        "No tiene ojos. Se orienta con ultrasonidos y odia la luz del sol.",
        [21, 9, 8],
        &[(1, CHUPAVIDAS), (6, MORDISCO), (12, ATAQUE_ALA), (20, PICOTAZO_VENENO), (32, ATAQUE_AEREO)],
        sculpt::zubat,
    ),
    sp(
        "Clefairy", 35, one(Normal), "Pokémon Hada",
        "Las noches de luna llena se reúne con otros y bailan en círculo en lo alto de las montañas.",
        [27, 9, 10],
        &[(1, DESTRUCTOR), (8, ATAQUE_RAPIDO), (16, PSICORRAYO), (25, GOLPE_CUERPO), (35, PSIQUICO)],
        sculpt::clefairy,
    ),
    sp(
        "Staryu", 120, one(Water), "Pokémon Estrellada",
        "Si pierde una de sus puntas, le vuelve a crecer. Su joya brilla por la noche.",
        [20, 10, 11],
        &[(1, PLACAJE), (1, PISTOLA_AGUA), (12, RAYO_BURBUJA), (20, CONFUSION), (30, SURF)],
        sculpt::staryu,
    ),
    sp(
        "Starmie", 121, two(Water, Psychic), "Pokémon Misterioso",
        "Su núcleo central brilla con los colores del arcoíris. Algunos creen que viene del espacio.",
        [26, 13, 13],
        &[(1, PISTOLA_AGUA), (1, CONFUSION), (18, RAYO_BURBUJA), (26, PSICORRAYO), (34, SURF), (44, PSIQUICO), (50, HIDROBOMBA)],
        sculpt::starmie,
    ),
    sp(
        "Voltorb", 100, one(Electric), "Pokémon Bola",
        "Se parece muchísimo a una Pokéball. Explota en cuanto alguien lo toca.",
        [21, 9, 10],
        &[(1, PLACAJE), (1, IMPACTRUENO), (13, CHISPA), (22, RAYO), (32, AUTODESTRUCCION), (40, VOLTIO_CRUEL)],
        sculpt::voltorb,
    ),
    sp(
        "Oddish", 43, two(Grass, Poison), "Pokémon Hierbajo",
        "De día se entierra en la tierra fresca. De noche pasea plantando semillas.",
        [23, 9, 10],
        &[(1, MEGAAGOTAR), (1, ACIDO), (15, HOJA_AFILADA), (24, BOMBA_LODO), (34, DANZA_PETALO)],
        sculpt::oddish,
    ),
    sp(
        "Tangela", 114, one(Grass), "Pokémon Enredadera",
        "Su cuerpo está cubierto de lianas azules que no paran de moverse. Nadie ha visto lo que hay debajo.",
        [26, 11, 14],
        &[(1, LATIGO_CEPA), (10, MEGAAGOTAR), (20, ATIZAR), (29, HOJA_AFILADA), (38, RAYO_SOLAR)],
        sculpt::tangela,
    ),
    sp(
        "Grimer", 88, one(Poison), "Pokémon Lodo",
        "Nació del lodo de las alcantarillas. Por donde pasa no vuelve a crecer la hierba.",
        [26, 11, 10],
        &[(1, DESTRUCTOR), (1, BOFETON_LODO), (14, RESIDUOS), (26, GOLPE_CUERPO), (36, BOMBA_LODO)],
        sculpt::grimer,
    ),
    sp(
        "Koffing", 109, one(Poison), "Pokémon Gas Venenoso",
        "Flota gracias a los gases que lleva dentro. Huele fatal.",
        [22, 10, 13],
        &[(1, PLACAJE), (1, POLUCION), (16, RESIDUOS), (28, BOMBA_LODO), (38, AUTODESTRUCCION)],
        sculpt::koffing,
    ),
    sp(
        "Abra", 63, one(Psychic), "Pokémon Psi",
        "Duerme dieciocho horas al día. Aun dormido, se teletransporta si nota peligro.",
        [18, 14, 7],
        &[(1, CONFUSION), (15, PSICORRAYO), (26, ATAQUE_RAPIDO), (34, PSIQUICO)],
        sculpt::abra,
    ),
    sp(
        "Growlithe", 58, one(Fire), "Pokémon Perrito",
        "Muy leal a su entrenador. Ladra para alejar a cualquiera que se acerque a su territorio.",
        [24, 13, 10],
        &[(1, MORDISCO), (1, ASCUAS), (16, COLMILLO_IGNEO), (26, DERRIBO), (36, LANZALLAMAS), (46, ENVITE_IGNEO)],
        sculpt::growlithe,
    ),
    sp(
        "Ponyta", 77, one(Fire), "Pokémon Caballo Fuego",
        "Nada más nacer ya sabe correr. Sus crines de fuego no queman a quien confía en él.",
        [23, 13, 10],
        &[(1, PLACAJE), (1, ASCUAS), (15, PISOTON), (24, RUEDA_FUEGO), (34, LLAMARADA)],
        sculpt::ponyta,
    ),
    sp(
        "Diglett", 50, one(Ground), "Pokémon Topo",
        "Vive bajo tierra y casi nunca sale. Los campos donde excava dan las mejores cosechas.",
        [16, 11, 7],
        &[(1, ARANAZO), (1, BOFETON_LODO), (15, MAGNITUD), (22, EXCAVAR), (32, TERREMOTO)],
        sculpt::diglett,
    ),
    sp(
        "Dugtrio", 51, one(Ground), "Pokémon Topo",
        "Tres Diglett que se mueven a la vez. Pueden cavar hasta noventa kilómetros bajo tierra.",
        [20, 15, 10],
        &[(1, ARANAZO), (1, MAGNITUD), (26, EXCAVAR), (32, AVALANCHA), (40, TERREMOTO)],
        sculpt::dugtrio,
    ),
    sp(
        "Rhyhorn", 111, two(Ground, Rock), "Pokémon Clavos",
        "Corre en línea recta y lo destroza todo. Una vez que arranca, no sabe frenar.",
        [27, 14, 16],
        &[(1, PISOTON), (1, LANZARROCAS), (20, DERRIBO), (30, AVALANCHA), (40, TERREMOTO), (50, ROCA_AFILADA)],
        sculpt::rhyhorn,
    ),
    sp(
        "Gastly", 92, two(Ghost, Poison), "Pokémon Gas",
        "Su cuerpo es casi todo gas. Envuelve a sus presas para dejarlas dormidas.",
        [18, 13, 6],
        &[(1, LENGUETAZO), (1, TINIEBLAS), (18, CONFUSION), (28, BOLA_SOMBRA), (38, BOMBA_LODO)],
        sculpt::gastly,
    ),
    sp(
        "Machop", 66, one(Fighting), "Pokémon Superpoder",
        "Entrena cada día levantando rocas. Es tan fuerte que puede tumbar a cien personas.",
        [25, 14, 10],
        &[(1, GOLPE_KARATE), (1, PATADA_BAJA), (20, SUMISION), (30, AVALANCHA), (40, TAJO_CRUZADO)],
        sculpt::machop,
    ),
    sp(
        "Magikarp", 129, one(Water), "Pokémon Pez",
        "Famoso por ser el Pokémon más débil. Solo sabe chapotear... de momento.",
        [16, 5, 9],
        &[(1, SALPICADURA), (15, PLACAJE)],
        sculpt::magikarp,
    ),
    sp(
        "Dratini", 147, one(Dragon), "Pokémon Dragón",
        "Durante mucho tiempo se pensó que era una leyenda. Muda la piel varias veces al crecer.",
        [24, 13, 11],
        &[(1, FURIA_DRAGON), (1, PLACAJE), (20, COLA_DRAGON), (30, GOLPE_CUERPO), (42, GARRA_DRAGON), (52, HIPERRAYO)],
        sculpt::dratini,
    ),
    sp(
        "Seel", 86, two(Water, Ice), "Pokémon León Marino",
        "Le encanta nadar entre témpanos de hielo. Rompe el hielo con el cuerno de la frente.",
        [27, 10, 12],
        &[(1, DESTRUCTOR), (1, PISTOLA_AGUA), (14, VIENTO_HIELO), (24, RAYO_AURORA), (34, RAYO_HIELO), (44, VENTISCA)],
        sculpt::seel,
    ),
];

/// Especies que evolucionan por nivel.
const EVOLUTIONS: [(usize, u32, usize); 2] = [(DIGLETT, 26, DUGTRIO), (STARYU, 30, STARMIE)];

/// Más difíciles de atrapar.
const HARD_TO_CATCH: [(usize, f32); 4] = [(DRATINI, 0.45), (ONIX, 0.6), (STARMIE, 0.6), (RHYHORN, 0.7)];

pub fn species_catch(species: usize) -> f32 {
    HARD_TO_CATCH.iter().find(|(s, _)| *s == species).map_or(SPECIES[species].catch, |(_, c)| *c)
}

pub fn evolution(species: usize) -> Option<(u32, usize)> {
    EVOLUTIONS.iter().find(|(s, _, _)| *s == species).map(|(_, l, t)| (*l, *t)).or(SPECIES[species].evolves)
}

// ---------------------------------------------------------------- Pokémon concretos

#[derive(Clone, Debug, PartialEq)]
pub struct Creature {
    pub species: usize,
    pub level: u32,
    pub xp: u32,
    pub hp: i32,
}

pub const MAX_LEVEL: u32 = 100;

impl Creature {
    pub fn new(species: usize, level: u32) -> Self {
        let mut c = Self { species, level, xp: 0, hp: 0 };
        c.hp = c.max_hp();
        c
    }

    pub fn spec(&self) -> &'static Species {
        &SPECIES[self.species]
    }

    pub fn name(&self) -> &'static str {
        self.spec().name
    }

    pub fn max_hp(&self) -> i32 {
        self.spec().base_hp + self.level as i32 * 3
    }

    pub fn attack(&self) -> i32 {
        self.spec().base_atk + self.level as i32 * 2
    }

    pub fn defense(&self) -> i32 {
        self.spec().base_def + self.level as i32 * 2
    }

    pub fn fainted(&self) -> bool {
        self.hp <= 0
    }

    pub fn heal(&mut self) {
        self.hp = self.max_hp();
    }

    /// Los últimos 4 ataques aprendidos hasta su nivel.
    pub fn moves(&self) -> Vec<Move> {
        let known: Vec<Move> = self
            .spec()
            .learnset
            .iter()
            .filter(|(lvl, _)| *lvl <= self.level)
            .map(|(_, m)| *m)
            .collect();
        known[known.len().saturating_sub(4)..].to_vec()
    }

    pub fn xp_to_next(&self) -> u32 {
        self.level * 12
    }

    pub fn xp_yield(&self) -> u32 {
        self.level * 9
    }

    /// Sube un nivel y devuelve los mensajes (ataques nuevos, evolución).
    pub fn level_up(&mut self) -> Vec<String> {
        let mut log = vec![];
        if self.level >= MAX_LEVEL {
            return log;
        }
        let old_max = self.max_hp();
        self.level += 1;
        self.hp += self.max_hp() - old_max;
        log.push(format!("¡{} subió al nivel {}!", self.name(), self.level));
        for (lvl, m) in self.spec().learnset {
            if *lvl == self.level {
                log.push(format!("¡{} aprendió {}!", self.name(), m.name));
            }
        }
        if let Some((lvl, into)) = evolution(self.species).filter(|(lvl, _)| self.level >= *lvl) {
            let _ = lvl;
            let old = self.name();
            let missing = self.max_hp() - self.hp;
            self.species = into;
            self.hp = self.max_hp() - missing;
            log.push(format!("¿Eh? ¡{old} está evolucionando!"));
            log.push(format!("¡Enhorabuena! ¡Tu {old} se ha convertido en {}!", self.name()));
        }
        log
    }

    /// Suma experiencia y devuelve los mensajes de subida de nivel / ataques nuevos.
    pub fn gain_xp(&mut self, amount: u32) -> Vec<String> {
        let mut log = vec![format!("¡{} ganó {} puntos de experiencia!", self.name(), amount)];
        self.xp += amount;
        while self.xp >= self.xp_to_next() && self.level < MAX_LEVEL {
            self.xp -= self.xp_to_next();
            log.extend(self.level_up());
        }
        log
    }

    /// Multiplicador de un ataque de tipo `ty` contra este Pokémon (tipos dobles multiplican).
    pub fn weakness(&self, ty: Type) -> f32 {
        self.spec().types().map(|t| ty.effectiveness(t)).product()
    }

    /// Devuelve (daño, efectividad).
    pub fn damage_to(&self, mv: &Move, target: &Creature) -> (i32, f32) {
        let eff = target.weakness(mv.ty);
        if eff == 0.0 {
            return (0, 0.0);
        }
        let stab = if self.spec().types().any(|t| t == mv.ty) { 1.5 } else { 1.0 };
        let base = ((2 * self.level as i32 / 5 + 2) * mv.power * self.attack() / target.defense()) as f32
            / 50.0
            + 2.0;
        let roll = rand::rng().random_range(0.85..=1.0);
        (((base * stab * eff * roll) as i32).max(1), eff)
    }

    /// El ataque que más daño haría (para los entrenadores rivales), con algo de azar.
    pub fn best_move(&self, target: &Creature) -> Move {
        let moves = self.moves();
        let mut rng = rand::rng();
        if rng.random_bool(0.2) {
            return moves[rng.random_range(0..moves.len())];
        }
        *moves
            .iter()
            .max_by_key(|m| {
                let stab = if self.spec().types().any(|t| t == m.ty) { 3 } else { 2 };
                (m.power as f32 * target.weakness(m.ty) * stab as f32) as i32
            })
            .unwrap()
    }
}

/// Un Pokémon salvaje de la tabla `(especie, peso)` de la zona, con nivel en `levels`.
pub fn random_wild(table: &[(usize, u32)], levels: (u32, u32)) -> Creature {
    let mut rng = rand::rng();
    // Para capturas de desarrollo (ver snapshot.rs)
    if let Some(species) = std::env::var("KANTO3D_WILD").ok().and_then(|v| v.parse().ok()) {
        return Creature::new(species, 5);
    }
    let total: u32 = table.iter().map(|(_, w)| w).sum();
    let mut roll = rng.random_range(0..total.max(1));
    let species = table
        .iter()
        .find(|(_, w)| {
            let hit = roll < *w;
            roll = roll.saturating_sub(*w);
            hit
        })
        .map_or(RATTATA, |(s, _)| *s);
    Creature::new(species, rng.random_range(levels.0..=levels.1))
}

pub fn effectiveness_text(eff: f32) -> Option<&'static str> {
    if eff == 0.0 {
        Some("No afecta...")
    } else if eff > 1.0 {
        Some("¡Es muy eficaz!")
    } else if eff < 1.0 {
        Some("No es muy eficaz...")
    } else {
        None
    }
}

pub const MAX_PARTY: usize = 6;

#[derive(Resource, Clone)]
pub struct Party(pub Vec<Creature>);

impl Party {
    pub fn first_alive(&self) -> Option<usize> {
        self.0.iter().position(|c| !c.fainted())
    }

    pub fn heal_all(&mut self) {
        self.0.iter_mut().for_each(Creature::heal);
    }
}

/// Índice en el equipo del Pokémon que te sigue por el mapa (o ninguno).
#[derive(Resource)]
pub struct Companion(pub Option<usize>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_chart_works_with_dual_types() {
        let geodude = Creature::new(GEODUDE, 10);
        assert_eq!(geodude.weakness(Water), 4.0);
        assert_eq!(geodude.weakness(Electric), 0.0);
        assert_eq!(Creature::new(GASTLY, 10).weakness(Normal), 0.0);
        assert_eq!(Creature::new(CHARMANDER, 5).weakness(Water), 2.0);
    }

    #[test]
    fn every_species_knows_a_move_at_level_one() {
        for (i, s) in SPECIES.iter().enumerate() {
            assert!(!Creature::new(i, 1).moves().is_empty(), "{} no sabe ningún ataque", s.name);
        }
    }

    #[test]
    fn diglett_evolves() {
        let mut d = Creature::new(DIGLETT, 25);
        d.gain_xp(10_000);
        assert_eq!(d.species, DUGTRIO);
    }
}
