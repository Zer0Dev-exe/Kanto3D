//! Kanto: los pueblos, rutas, cuevas, gimnasios e interiores, con su gente y sus Pokémon.
//! Las conexiones siguen el mapa de Pokémon Rojo; la distribución de cada zona es propia.

use bevy::prelude::*;

use crate::pokemon::*;
use crate::game::items::*;
use crate::render::models::look::*;
use crate::world::npcs::*;
use crate::game::progress::Place;
use crate::world::*;

pub const PALLET: usize = 0;
pub const R1: usize = 1;
pub const VIRIDIAN: usize = 2;
pub const R2: usize = 3;
pub const FOREST: usize = 4;
pub const PEWTER: usize = 5;
pub const R3: usize = 6;
pub const MOON1: usize = 7;
pub const MOON2: usize = 8;
pub const R4: usize = 9;
pub const CERULEAN: usize = 10;
pub const R24: usize = 11;
pub const R25: usize = 12;
pub const R5: usize = 13;
pub const SAFFRON: usize = 14;
pub const R6: usize = 15;
pub const VERMILION: usize = 16;
pub const R11: usize = 17;
pub const DIGLETT_CAVE: usize = 18;
pub const R9: usize = 19;
pub const R10: usize = 20;
pub const TUNNEL: usize = 21;
pub const LAVENDER: usize = 22;
pub const R8: usize = 23;
pub const R7: usize = 24;
pub const CELADON: usize = 25;
pub const R12: usize = 26;
pub const R13: usize = 27;
pub const R14: usize = 28;
pub const R15: usize = 29;
pub const FUCHSIA: usize = 30;
pub const R16: usize = 31;
pub const R17: usize = 32;
pub const R18: usize = 33;
pub const R19: usize = 34;
pub const R20: usize = 35;
pub const SEAFOAM: usize = 36;
pub const CINNABAR: usize = 37;
pub const R21: usize = 38;
pub const R22: usize = 39;
pub const R23: usize = 40;
pub const VICTORY: usize = 41;
pub const INDIGO: usize = 42;
pub const CENTER_IN: usize = 43;
pub const MART_IN: usize = 44;
pub const HOUSE_IN: usize = 45;
pub const PLAYER_HOUSE: usize = 46;
pub const LAB_IN: usize = 47;
pub const GYM_PEWTER: usize = 48;
pub const GYM_CERULEAN: usize = 49;
pub const GYM_VERMILION: usize = 50;
pub const GYM_CELADON: usize = 51;
pub const GYM_FUCHSIA: usize = 52;
pub const GYM_SAFFRON: usize = 53;
pub const GYM_CINNABAR: usize = 54;
pub const GYM_VIRIDIAN: usize = 55;
pub const LEAGUE: usize = 56;
pub const TOWER: usize = 57;
pub const SAFARI: usize = 58;
pub const SILPH: usize = 59;
pub const CASINO: usize = 60;
pub const MANSION: usize = 61;
pub const DOJO: usize = 62;
pub const MUSEUM: usize = 63;

/// Delante de la puerta de tu casa.
pub fn home_door() -> Place {
    let town = LoadedMap::load(PALLET);
    Place { map: PALLET, pos: town.buildings[0].door() + S }
}

/// Dónde empiezas la partida: en tu casa.
pub fn house_start() -> IVec2 {
    IVec2::new(3, 5)
}

/// Número único de cada objeto del suelo (para la partida guardada).
pub fn item_id(map: usize, index: usize) -> usize {
    map * 16 + index
}

/// Medallas necesarias para entrar en un mapa: así Kanto se recorre en el orden de Pokémon
/// Rojo (Brock, Misty, Surge, Erika, Koga, Sabrina, Blaine y Giovanni).
pub fn required_badges(map: usize) -> u8 {
    match map {
        R3 | MOON1 | MOON2 | R4 | CERULEAN | R24 | R25 | GYM_CERULEAN => 1,
        R5 | SAFFRON | R6 | VERMILION | GYM_VERMILION | DOJO => 2,
        R11 | DIGLETT_CAVE | R9 | R10 | TUNNEL | LAVENDER | R8 | R7 | CELADON | GYM_CELADON | CASINO | TOWER => 3,
        R12 | R13 | R14 | R15 | FUCHSIA | R16 | R17 | R18 | GYM_FUCHSIA | SAFARI => 4,
        R19 | R20 | SEAFOAM | CINNABAR | R21 | GYM_SAFFRON | SILPH => 5,
        GYM_CINNABAR | MANSION => 6,
        GYM_VIRIDIAN => 7,
        R23 | VICTORY | INDIGO | LEAGUE => 8,
        _ => 0,
    }
}

/// El último Centro Pokémon para una ciudad (el del propio mapa o, en interiores, el de fuera).
pub const CHAMPION: &str = "Campeón Azul";

const TOWN: MapDef = MapDef { theme: Theme::Town, ground: [110, 188, 88], ..BASE };
const CAVE: MapDef = MapDef { theme: Theme::Cave, ground: [120, 100, 82], ..BASE };
const INSIDE: MapDef = MapDef { theme: Theme::Inside, ground: [226, 210, 178], ..BASE };
const GYM: MapDef = MapDef { theme: Theme::Gym, ground: [200, 190, 170], ..BASE };
const SEA: MapDef = MapDef { theme: Theme::Sea, ground: [232, 214, 160], ..BASE };

pub const MAPS: [MapDef; 64] = [
    // ------------------------------------------------------------ Pueblo Paleta
    MapDef {
        name: "Pueblo Paleta",
        rows: r"
##########11############
#f........::.........f.#
#..hhhh...::...hhhh....#
#..hhhh...::...hhhh....#
#..hhhh...::...hhhh....#
#....:....::.....:.....#
#....::::::::::::::....#
#.........::.......*...#
#.f.f.....::...........#
#.........::...lllll...#
#.FFFFF.s.::...lllll...#
#.f.f.f...::...lllll...#
#.f.f.f...::.....:.....#
#.........::::::::.....#
#..*......::........b..#
#.........::...........#
#.b.~~~~~~==~~~~~~~~...#
#...~~~~~~==~~~~~~~~...#
####~~~~~~22~~~~~~~~####",
        exits: &[edge(1, R1, 2, S), edge(2, R21, 1, N)],
        doors: &[Door::Map(PLAYER_HOUSE), Door::Default, Door::Map(LAB_IN)],
        signs: &["PUEBLO PALETA: un lugar tranquilo y de colores puros."],
        npcs: &[
            talk("Vecina", GIRL, S, &["La ciencia avanza muchísimo...", "¡Ahora puedes guardar tus Pokémon y tu partida en cualquier momento!"]),
            talk("Vecino", GENTLEMAN, E, &["El Profe Roble investiga a los Pokémon en su laboratorio.", "Si vas a ir a la hierba alta, ¡lleva siempre un Pokémon contigo!"]),
        ],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 1
    MapDef {
        name: "Ruta 1",
        rows: r"
########11########
#gggg...::...gggg#
#gggg...::...gggg#
#.......::.......#
#vvvv.*.::..vvvvv#
#.......::.......#
#..##...::...ggg.#
#..##...::...ggg.#
#.......::...ggg.#
#vvvvvv.::.vvvvvv#
#.......::.......#
#.ggg...::.......#
#.ggg...:::::....#
#.ggg.......:..s.#
#...........:....#
#..gggg.....:..b.#
#..gggg.:::::....#
#..gggg.::.......#
#vvvvvvv::..vvvv.#
#.......::.......#
#.gg....::....gg.#
#.gg....::....gg.#
#.......::.......#
########22########",
        exits: &[edge(1, VIRIDIAN, 2, S), edge(2, PALLET, 1, N)],
        signs: &["RUTA 1: Pueblo Paleta - Ciudad Verde."],
        npcs: &[Npc {
            name: "Dependiente",
            look: CLERK,
            facing: S,
            role: Role::Gift {
                lines: &["¡Hola! Trabajo en la Tienda Pokémon de Ciudad Verde.", "Toma, una muestra gratis para que nos conozcas."],
                item: POTION,
                qty: 1,
                after: "¡Pásate por la tienda de Ciudad Verde cuando quieras!",
            },
            aside: None,
        }],
        encounters: &[(PIDGEY, 50), (RATTATA, 50)],
        levels: (2, 5),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Verde
    MapDef {
        name: "Ciudad Verde",
        rows: r"
###########11#############
#f.........::...........f#
#..GGGGG...::....hhhh....#
#..GGGGG...::....hhhh....#
#..GGGGG...::....hhhh....#
#....:.....::......:.....#
#....:::::::::::::::.....#
#..........::............#
#..cccc....::...mmmm.....#
#..cccc....::...mmmm.....#
#..cccc....::...mmmm.....#
#....:.....::.....:......#
#....::::::::::::::......#
4..........::............#
4....s.....::.......*....#
#.ff.......::......ff....#
#..........::............#
###########22#############",
        exits: &[edge(1, R2, 2, S), edge(2, R1, 1, N), edge(4, R22, 3, E)],
        doors: &[Door::Badges(7, GYM_VIRIDIAN, "El gimnasio está cerrado a cal y canto. Dicen que el líder solo vuelve para los entrenadores con 7 medallas."), Door::Default, Door::Default, Door::Default],
        signs: &["CIUDAD VERDE: la eterna ciudad verde. Al oeste, Ruta 22 y la Liga Pokémon."],
        npcs: &[talk("Anciano", OLDMAN, W, &["Ah, ¿vas a por las medallas?", "Para entrar en la Liga Pokémon necesitas las 8 medallas de Kanto.", "Empieza por Brock, en Ciudad Plateada, al norte del Bosque Verde."])],
        roof: [60, 140, 90],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 2
    MapDef {
        name: "Ruta 2",
        rows: r"
#########11#########
#........::........#
#.ggg....::....MMMM#
#.ggg....::....M7MM#
#........::.......o#
#.vvvvv..::..vvvvv.#
#........::........#
#.....*..::........#
#..ggg...::...ggg..#
#..ggg...::...ggg..#
#........:::::.....#
#............::....#
#.............::...#
##############66####
####55##############
#...::.............#
#...::..ggggg......#
#...::..ggggg..s...#
#...:::::::::......#
#...........::.....#
#vvvvvvvvv..::.vvvv#
#...........::.....#
#.gggg......::..gg.#
#.gggg....::....gg.#
#.gggg...::........#
#........::........#
#..b.....::.....b..#
#........::........#
#........::........#
#########22#########",
        exits: &[
            edge(1, PEWTER, 2, S),
            edge(2, VIRIDIAN, 1, N),
            edge(5, FOREST, 2, S),
            edge(6, FOREST, 1, N),
            cave(7, DIGLETT_CAVE, 1, S),
        ],
        signs: &["RUTA 2: Ciudad Verde - Ciudad Plateada. Para seguir al norte hay que cruzar el Bosque Verde."],
        npcs: &[Npc {
            name: "Montañero Iker",
            look: HIKER,
            facing: S,
            role: Role::Quiz(Quiz {
                intro: &["Dicen que en el bosque hay criaturas eléctricas.", "¿Sabes contra qué no les va bien?"],
                question: "¿Contra qué DOS tipos es poco eficaz Eléctrico?",
                options: &["Agua", "Planta", "Eléctrico", "Fuego"],
                correct: &[1, 2],
                reward: Reward::Item(POKE_BALL, 5),
                beaten: "Con esas Poké Balls podrás capturar un buen equipo.",
            }),
            aside: None,
        }],
        items: &[(POTION, 1)],
        encounters: &[(PIDGEY, 35), (RATTATA, 35), (CATERPIE, 15), (WEEDLE, 15)],
        levels: (3, 5),
        ..BASE
    },
    // ------------------------------------------------------------ Bosque Verde
    MapDef {
        name: "Bosque Verde",
        rows: r"
##########11##########
#gggg.....::....gggg.#
#gggg.....::....gggg.#
#..##.....:.....##...#
#..##..*..:.....##.o.#
#.gggg....:....gggg..#
#.gggg....:....gggg..#
#.gggg....::::::.....#
#...##.........:..##.#
#...##.........:..##.#
#.......ggg....:.....#
#.......ggg....:..*..#
#..s....ggg....:.....#
#.##..::::::::::.....#
#.##..:..........gg..#
#gggg.:...##.....gg..#
#gggg.:...##..*..gg..#
#gggg.:.........o....#
#.....:::::::........#
#..gggg.....:.gggg...#
#..gggg.....:.gggg...#
#...........::.......#
#..b.........:....b..#
##########22##########",
        theme: Theme::Forest,
        ground: [86, 160, 72],
        exits: &[edge(1, R2, 6, S), edge(2, R2, 5, N)],
        signs: &["BOSQUE VERDE: dicen que aquí vive un Pikachu muy travieso. ¡No te pierdas!"],
        npcs: &[
            trainer("Cazabichos Rick", BUG, E, 3, &[(WEEDLE, 6), (CATERPIE, 6)], &["¡Eh! ¡Tú! ¡Mira mis bichos!"], "¡No! ¡Mis bichos!", "Los Pokémon Bicho evolucionan muy rápido."),
            trainer("Cazabichos Doug", BUG, W, 3, &[(WEEDLE, 7), (CATERPIE, 7), (WEEDLE, 7)], &["¡Los bichos son lo más! ¡A luchar!"], "Uf, qué fuerte eres.", "Algún día atraparé un Pikachu."),
            trainer("Cazabichos Sammy", BUG, W, 4, &[(WEEDLE, 9)], &["¡Mi Weedle tiene veneno!"], "¡Mi Weedle!", "Vuelve al bosque cuando quieras."),
        ],
        items: &[(POTION, 1), (POKE_BALL, 2)],
        encounters: &[(CATERPIE, 30), (WEEDLE, 30), (PIDGEY, 20), (PIKACHU, 8), (ODDISH, 12)],
        levels: (3, 6),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Plateada
    MapDef {
        name: "Ciudad Plateada",
        rows: r"
##########################
#.....BBBBBBBB...........#
#.....BBBBBBBB.....MMM...#
#.....BBBBBBBB.....MMM...#
#.........:..............#
#..GGGGG..:.....hhhh.....#
#..GGGGG..:.....hhhh.....#
#..GGGGG..:.....hhhh.....#
#....:....:.......:......#
#....::::::::::::::::::::3
#..........:........*...:3
#..cccc....:....mmmm.....#
#..cccc....:....mmmm.....#
#..cccc....:....mmmm.....#
#....:.....:......:......#
#....:::::::::::::::.....#
#.s.......::.............#
#f.f......::.........f.f.#
#.........::.............#
##########22##############",
        exits: &[edge(2, R2, 1, N), edge(3, R3, 4, W)],
        doors: &[Door::Map(MUSEUM), Door::Map(GYM_PEWTER)],
        signs: &["CIUDAD PLATEADA: una ciudad gris como la piedra."],
        npcs: &[talk("Chico", YOUNGSTER, W, &["¿Vas a retar a Brock? Sus Pokémon de tipo Roca son durísimos.", "¡Los ataques de tipo Agua y Planta les hacen mucho daño!"])],
        roof: [150, 120, 90],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 3
    MapDef {
        name: "Ruta 3",
        rows: r"
############################
#....gggg.......MMMMMMMMMMMM
#....gggg...*...MMMMMMMMMMMM
#..............MMMMMMMMMMMMM
#.......vvvv......MMMMMMMMMM
4:::::::::::::::::::...MMMMM
4.........*........:::::::5M
#..gggg..........gggg..MMMMM
#..gggg...vvvvv..gggg..MMMMM
#..gggg..........gggg...MMMM
#........*.....s.........MMM
#..o.....................MMM
#.ggggg........ggggg.....MMM
############################",
        exits: &[edge(4, PEWTER, 3, E), cave(5, MOON1, 2, W)],
        signs: &["RUTA 3: Mt. Moon al este. Las cuevas están llenas de Zubat, lleva Repelentes."],
        npcs: &[
            trainer("Joven Ben", YOUNGSTER, S, 3, &[(RATTATA, 10), (PIDGEY, 10)], &["¡Eh! ¡Me has mirado! ¡Eso es un combate!"], "¡Qué pasada!", "Mis pantalones cortos son los mejores para luchar."),
            trainer("Chica Janice", LASS, N, 2, &[(PIDGEY, 9), (PIDGEY, 9)], &["¿Sabes lo bonitos que son los Pidgey?"], "Jo, has ganado.", "Mis Pidgey necesitan descansar."),
            trainer("Cazabichos Colton", BUG, N, 4, &[(CATERPIE, 10), (WEEDLE, 10), (CATERPIE, 10)], &["¡Atrapé estos bichos en el Bosque Verde!"], "¡Mis bichos no pueden más!", "Los bichos se crían rápido."),
        ],
        items: &[(SUPER_REPEL, 1)],
        encounters: &[(PIDGEY, 30), (RATTATA, 30), (CLEFAIRY, 5), (GEODUDE, 15), (MACHOP, 20)],
        levels: (6, 10),
        ..BASE
    },
    // ------------------------------------------------------------ Mt. Moon
    MapDef {
        name: "Mt. Moon",
        rows: r"
######################
#....#.....r.....o...#
#.5..#..........#....#
#....#..*.......#....#
#....#.....######....#
#..................r.#
###.####..r.........##
#......#......*......#
#..r...#..###........#
#......#..###...~~~..#
#.o....#.......~~~~..#
#..........r.........#
####...######...######
#..........*.........#
#..###..........###..#
#..###..........###..#
#....................#
#########2############",
        exits: &[edge(2, R3, 5, N), ladder(5, MOON2, 5, S, true)],
        npcs: &[
            trainer("Montañero Marcos", HIKER, W, 2, &[(GEODUDE, 10), (GEODUDE, 10), (ONIX, 10)], &["¡Uf! ¡Has asustado a mis Pokémon de roca!"], "Vaya, eres duro de roer.", "Mt. Moon tiene muchas salidas. ¡No te pierdas!"),
            trainer("Chica Iris", LASS, S, 3, &[(CLEFAIRY, 12), (ODDISH, 12)], &["¿Has visto los Clefairy? ¡Bailan en la cima!"], "Uy, has ganado...", "Los Clefairy vienen de la luna, ¿sabías?"),
            trainer("Recluta Rocket", ROCKET, S, 3, &[(RATTATA, 13), (ZUBAT, 13)], &["¡Somos el Team Rocket! Buscamos fósiles en esta cueva.", "¡Y no vas a impedirlo!"], "¡Rayos y centellas!", "El Team Rocket volverá..."),
        ],
        items: &[(RARE_CANDY, 1), (ESCAPE_ROPE, 1)],
        encounters: &[(ZUBAT, 50), (GEODUDE, 30), (CLEFAIRY, 8), (MACHOP, 12)],
        levels: (7, 11),
        ..CAVE
    },
    // ------------------------------------------------------------ Mt. Moon (sótano)
    MapDef {
        name: "Mt. Moon - Sótano",
        rows: r"
######################
#.5....#.........o...#
#......#..###........#
#..r...#..###...*....#
#......#............##
#....................3
####...#####..r......#
#...........#........#
#..~~~~.....#...r....#
#..~~~~..*..#........#
#...........#..o.....#
#..r.................#
#######...############
#....o...............#
#....................#
######################",
        exits: &[ladder(5, MOON1, 5, S, false), edge(3, R4, 5, W)],
        npcs: &[
            trainer("Súper Nerd Miguel", SCIENTIST, W, 4, &[(GRIMER, 12), (VOLTORB, 12), (KOFFING, 12)], &["¡Los fósiles de esta cueva son míos!"], "¡Vale, vale! ¡Quédatelos!", "Algún día podré revivir un fósil..."),
            trainer("Recluta Rocket", ROCKET, E, 3, &[(GRIMER, 14), (KOFFING, 14)], &["¡Alto ahí! ¡Zona del Team Rocket!"], "¡Imposible!", "Nuestro jefe es el mejor entrenador del mundo..."),
        ],
        items: &[(SUPER_POTION, 1), (REVIVE, 1), (SUPER_BALL, 2)],
        encounters: &[(ZUBAT, 50), (GEODUDE, 30), (CLEFAIRY, 10), (MACHOP, 10)],
        levels: (8, 12),
        ..CAVE
    },
    // ------------------------------------------------------------ Ruta 4
    MapDef {
        name: "Ruta 4",
        rows: r"
########################
MMMMMM.....ggggg.......#
MMMMMM.*...ggggg.......#
MMMMM5:::::::::::::::::3
MMMMMM.........vvvvv...3
#.......gggg...........#
#...s...gggg......o....#
#.......gggg...........#
########################",
        exits: &[cave(5, MOON2, 3, E), edge(3, CERULEAN, 4, W)],
        signs: &["RUTA 4: Mt. Moon - Ciudad Celeste."],
        npcs: &[trainer("Chica Lis", LASS, S, 1, &[(ODDISH, 14), (CLEFAIRY, 13)], &["¡Uy, un entrenador! ¡Vamos a luchar!"], "¡Has ganado!", "Ciudad Celeste es preciosa, ya verás.")],
        items: &[(POTION, 2)],
        encounters: &[(RATTATA, 35), (PIDGEY, 35), (MACHOP, 20), (GEODUDE, 10)],
        levels: (8, 12),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Celeste
    MapDef {
        name: "Ciudad Celeste",
        rows: r"
############11##############
#~~~~~......::......hhhh...#
#~~~~~......::......hhhh...#
#~~~~~..*...::......hhhh...#
#...........::........:....#
#..GGGGGG...:::::::::::....#
#..GGGGGG...::.............#
#..GGGGGG...::.............#
#.....:.....::.............#
4::::::::::::::::::::::::::3
4.....s.....::.............3
#..cccc.....::.....mmmm....#
#..cccc.....::.....mmmm....#
#..cccc.....::.....mmmm....#
#....:......::.......:.....#
#....::::::::::::::::::....#
#..BBBB.....::.....hhhh....#
#..BBBB.....::.....hhhh....#
#....:......::.......:...*.#
#....::::::::::::::::::....#
#f..........::...........f.#
############22##############",
        exits: &[edge(1, R24, 2, S), edge(2, R5, 1, N), edge(3, R9, 4, W), edge(4, R4, 3, E)],
        doors: &[Door::Default, Door::Map(GYM_CERULEAN), Door::Default, Door::Default, Door::Text("Tienda de Bicis: ¡una bici por solo 1.000.000! ...demasiado cara.")],
        signs: &["CIUDAD CELESTE: una ciudad misteriosa y llena de agua."],
        npcs: &[
            Npc {
                name: "Pescadora Ana",
                look: FISHER,
                facing: S,
                role: Role::Quiz(Quiz {
                    intro: &["¡Eh! No me espantes los peces.", "Si aciertas esto, te enseño un truco."],
                    question: "¿Qué tipo resiste los ataques de tipo Fuego?",
                    options: &["Planta", "Agua", "Normal", "Eléctrico"],
                    correct: &[1],
                    reward: Reward::Xp(120),
                    beaten: "Los peces de agua no temen al fuego, ¿eh?",
                }),
                aside: None,
            },
            talk("Chica", GIRL, W, &["Misty, la líder del gimnasio, es una experta en Pokémon de tipo Agua.", "Los Pokémon Eléctricos y de Planta son los que mejor le van."]),
        ],
        roof: [70, 150, 220],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 24 (Puente Pepita)
    MapDef {
        name: "Ruta 24 - Puente Pepita",
        rows: r"
################
#..gggg......::3
#..gggg......::3
#..gggg.....::.#
#..o........::.#
#.......*...::.#
#~~~~~~~~~~~*=~#
#~~~~~~~~~~~==~#
#~~~~~~~~~~~*=~#
#~~~~~~~~~~~==~#
#~~~~~~~~~~~*=~#
#~~~~~~~~~~~==~#
#~~~~~~~~~~~*=~#
#~~~~~~~~~~~==~#
#~~~~~~~~~~~*=~#
#~~~~~~~~~~~==~#
#......s....::.#
#.gggg......::.#
#.gggg......::.#
############22##",
        exits: &[edge(2, CERULEAN, 1, N), edge(3, R25, 4, W)],
        signs: &["PUENTE PEPITA: ¡vence a los 5 entrenadores del puente y llévate un premio!"],
        npcs: &[
            trainer("Recluta Rocket", ROCKET, E, 4, &[(RATTATA, 15), (ZUBAT, 15), (GRIMER, 15)], &["¡Felicidades por cruzar el puente!", "¿Quieres unirte al Team Rocket? ¿No? ¡Pues a luchar!"], "¡Arrgh!", "Te arrepentirás de no unirte a nosotros..."),
            trainer("Joven Tim", YOUNGSTER, E, 1, &[(RATTATA, 14), (PIDGEY, 14)], &["¡Este es el Puente Pepita! ¡Primer combate!"], "¡Vaya!", "Quedan cuatro más."),
            trainer("Chica Ali", LASS, E, 1, &[(ODDISH, 15), (PIDGEY, 15)], &["¡Segundo combate! ¡No te dejaré pasar!"], "¡Ay, no!", "Ánimo con los demás."),
            trainer("Joven Luis", YOUNGSTER, E, 1, &[(RATTATA, 15), (CATERPIE, 15), (WEEDLE, 15)], &["¡Tercer combate! ¡Vamos!"], "¡Qué fuerte!", "Ya casi lo tienes."),
            trainer("Chica Rosa", LASS, E, 1, &[(CLEFAIRY, 16), (PIDGEY, 15)], &["¡Cuarto combate! ¡A por todas!"], "¡Has ganado!", "Solo queda uno."),
            trainer("Campista Jorge", YOUNGSTER, E, 1, &[(MACHOP, 18)], &["¡El último del puente! ¡Mi Machop es imbatible!"], "¡Increíble!", "¡Has superado el Puente Pepita!"),
        ],
        items: &[(SUPER_POTION, 1)],
        encounters: &[(ODDISH, 30), (PIDGEY, 20), (CATERPIE, 15), (WEEDLE, 15), (ABRA, 10), (BULBASAUR, 10)],
        levels: (8, 14),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 25
    MapDef {
        name: "Ruta 25",
        rows: r"
########################
#.....ggg.........hhhh.#
#.....ggg.........hhhh.#
4::::::::::::.....hhhh.#
4.....*.....::::::::::.#
#..gggg...*.......ggg..#
#..gggg.........o.ggg..#
########################",
        exits: &[edge(4, R24, 3, E)],
        doors: &[Door::Default],
        npcs: &[
            trainer("Excursionista Sara", LASS, N, 1, &[(ODDISH, 17), (BULBASAUR, 17)], &["¿Vas a ver a Bill? ¡Primero, un combate!"], "¡Qué bien luchas!", "Bill es un genio de los Pokémon."),
            trainer("Montañero Paco", HIKER, N, 2, &[(GEODUDE, 16), (MACHOP, 17)], &["¡Mis Pokémon han entrenado en las montañas!"], "¡Increíble!", "Hay que entrenar más."),
        ],
        items: &[(ULTRA_BALL, 1)],
        encounters: &[(ODDISH, 30), (PIDGEY, 25), (CATERPIE, 15), (WEEDLE, 15), (ABRA, 15)],
        levels: (10, 14),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 5
    MapDef {
        name: "Ruta 5",
        rows: r"
######11########
#.....::.......#
#.ggg.::..gggg.#
#.ggg.::..gggg.#
#.....::...*...#
#FFFF.::.FFFFF.#
#ffff.::.fffff.#
#.....::.......#
#..o..::..s....#
#vvvvv::.vvvvvv#
#.....::.......#
#.gggg::gggg...#
#.gggg::gggg...#
#.....::.......#
######22########",
        exits: &[edge(1, CERULEAN, 2, S), edge(2, SAFFRON, 1, N)],
        signs: &["RUTA 5: Ciudad Celeste - Ciudad Azafrán."],
        npcs: &[talk("Señora", GIRL, W, &["Ciudad Azafrán está justo en el centro de Kanto.", "Allí está la sede de Silph S.A., que fabrica las Poké Balls."])],
        items: &[(POTION, 2)],
        encounters: &[(ODDISH, 30), (PIDGEY, 30), (RATTATA, 20), (ABRA, 10), (GROWLITHE, 10)],
        levels: (13, 16),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Azafrán
    MapDef {
        name: "Ciudad Azafrán",
        rows: r"
#############11#############
#.............::...........#
#..hhhhh.....::.....GGGGG..#
#..hhhhh.....::.....GGGGG..#
#..hhhhh.....::.....GGGGG..#
#....:.......::.......:....#
#....::::::::::::::::::....#
#.............::...........#
#.....BBBBBBBBBBBBBBBB.....#
#.....BBBBBBBBBBBBBBBB.....#
#.....BBBBBBBBBBBBBBBB.....#
#.....BBBBBBBBBBBBBBBB.....#
4.............:............3
4::::::::::::::::::::::::::3
#.............::...........#
#..cccc.......::......mmmm.#
#..cccc...*...::......mmmm.#
#..cccc.......::......mmmm.#
#....:........::........:..#
#....::::::::::::::::::::..#
#.s...........::.........f.#
#############22#############",
        exits: &[edge(1, R5, 2, S), edge(2, R6, 1, N), edge(3, R8, 4, W), edge(4, R7, 3, E)],
        doors: &[
            Door::Map(DOJO),
            Door::Map(GYM_SAFFRON),
            Door::Map(SILPH),
        ],
        signs: &["CIUDAD AZAFRÁN: la ciudad que brilla como el oro."],
        npcs: &[talk("Ejecutivo", GENTLEMAN, E, &["Sabrina, la líder del gimnasio, tiene poderes psíquicos desde niña.", "Los Pokémon de tipo Bicho y Fantasma son su punto débil."])],
        roof: [220, 180, 60],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 6
    MapDef {
        name: "Ruta 6",
        rows: r"
######11########
#.....::.......#
#.ggg.::.*.ggg.#
#.ggg.::...ggg.#
#.....::.......#
#~~~~.::.......#
#~~~~.::..*....#
#~~~~.::.......#
#.....::..s....#
#vvvv.::.vvvvvv#
#..gg.::.gggg..#
#..gg.::.gggg..#
#.....::.......#
######22########",
        exits: &[edge(1, SAFFRON, 2, S), edge(2, VERMILION, 1, N)],
        signs: &["RUTA 6: Ciudad Azafrán - Ciudad Carmín."],
        npcs: &[
            trainer("Campista Rubén", YOUNGSTER, W, 2, &[(RATTATA, 18), (ODDISH, 18)], &["¡Vengo de acampar y quiero luchar!"], "¡Uf, me has ganado!", "Voy a seguir entrenando."),
            trainer("Chica Isa", LASS, W, 3, &[(PIDGEY, 18), (CLEFAIRY, 19)], &["¡Qué día tan bonito para un combate!"], "¡Oh, no!", "Vermilion huele a mar, ¿verdad?"),
        ],
        encounters: &[(ODDISH, 30), (PIDGEY, 30), (RATTATA, 20), (ABRA, 10), (MAGIKARP, 10)],
        levels: (13, 17),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Carmín
    MapDef {
        name: "Ciudad Carmín",
        rows: r"
######11####################
#.....::..............ffff.#
#.hhhh::.....BBBBB.........#
#.hhhh::.....BBBBB.........#
#.hhhh::.....BBBBB...*.....#
#...:.::.......:...........#
#...:::::::::::::::::::::::3
#.....::...................3
#..cccc..........mmmm......#
#..cccc..........mmmm......#
#..cccc..........mmmm......#
#....:.............:.......#
#....:::::::::::::::.......#
#.........GGGGG...s........#
#.........GGGGG............#
#.........GGGGG............#
#...........:..............#
#.....:::::::::::..........#
#~~~~~~~~~~~~~~~~~~~~~~~~~~#
############################",
        theme: Theme::Town,
        exits: &[edge(1, R6, 2, S), edge(3, R11, 4, W)],
        doors: &[Door::Default, Door::Default, Door::Default, Door::Default, Door::Map(GYM_VERMILION)],
        signs: &["CIUDAD CARMÍN: el puerto del atardecer. Gimnasio del Teniente Surge."],
        npcs: &[talk("Marinero", SWIMMER, S, &["El barco S.S. Anne ya ha zarpado.", "El Teniente Surge usa Pokémon Eléctricos. ¡Los de tipo Tierra son inmunes a la electricidad!"])],
        roof: [230, 150, 50],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 11
    MapDef {
        name: "Ruta 11",
        rows: r"
########################
#gggg...MM5MM....gggg..#
#gggg...MM.MM....gggg..#
#.......:::......*.....#
4::::::::::::::::::::::3
4....*..........*.....:3
#..gggggg....s...gggg..#
#..gggggg.......gggg...#
########################",
        exits: &[edge(4, VERMILION, 3, E), edge(3, R12, 4, W), cave(5, DIGLETT_CAVE, 2, S)],
        signs: &["RUTA 11: la Cueva Diglett lleva hasta la Ruta 2."],
        npcs: &[
            trainer("Mecánico Dani", SCIENTIST, S, 1, &[(VOLTORB, 19), (KOFFING, 19)], &["¡Estoy probando inventos! ¡Prueba esto!"], "¡Cortocircuito!", "Mis inventos necesitan ajustes."),
            trainer("Joven Eddy", YOUNGSTER, N, 1, &[(RATTATA, 18), (DIGLETT, 18)], &["¡Mi Diglett sale de la cueva!"], "¡Se metió bajo tierra!", "Los Diglett cavan muy rápido."),
            trainer("Jugador Hugo", GENTLEMAN, N, 1, &[(GROWLITHE, 19), (PONYTA, 19)], &["¡Apuesto a que gano yo!"], "¡He perdido la apuesta!", "Mejor no volver a apostar."),
        ],
        encounters: &[(RATTATA, 30), (PIDGEY, 30), (MACHOP, 15), (DIGLETT, 10), (GROWLITHE, 15)],
        levels: (13, 17),
        ..BASE
    },
    // ------------------------------------------------------------ Cueva Diglett
    MapDef {
        name: "Cueva Diglett",
        rows: r"
####################
#1.........r.......#
#..####.......###..#
#..####..r....###..#
#..........##......#
#.r...###...##..r..#
#.....###..........#
#..##.......####...#
#..##...r.......o..#
#..............r..2#
####################",
        exits: &[edge(1, R2, 7, E), edge(2, R11, 5, W)],
        items: &[(MAX_POTION, 1)],
        encounters: &[(DIGLETT, 90), (DUGTRIO, 10)],
        levels: (15, 22),
        ..CAVE
    },
    // ------------------------------------------------------------ Ruta 9
    MapDef {
        name: "Ruta 9",
        rows: r"
##########################
#..gggg.....MMMMM...gggg.#
#..gggg..*..MMMMM...gggg.#
#..........vvvvvvv.......#
4::::::::::::::::::::::::3
4.....*.............*...:3
#.gggggg....o.....gggggg.#
#.gggggg..........gggggg.#
#..........s.............#
##########################",
        exits: &[edge(4, CERULEAN, 3, E), edge(3, R10, 4, W)],
        signs: &["RUTA 9: Ciudad Celeste - Túnel Roca."],
        npcs: &[
            trainer("Montañero Alan", HIKER, S, 2, &[(GEODUDE, 18), (MACHOP, 18)], &["¡Voy al Túnel Roca! ¡Pero antes, un combate!"], "¡Me has machacado!", "Dentro del túnel está muy oscuro."),
            trainer("Chica Alicia", LASS, N, 1, &[(ODDISH, 18), (CLEFAIRY, 18)], &["¡No me mires así! ¡Lucha!"], "¡Jo!", "Lleva Cuerdas Huida si entras en cuevas."),
            trainer("Cazabichos Brent", BUG, N, 1, &[(CATERPIE, 19), (WEEDLE, 19)], &["¡Mis bichos ya son mayores!"], "¡No puede ser!", "Algún día evolucionarán."),
        ],
        items: &[(REVIVE, 1)],
        encounters: &[(RATTATA, 35), (PIDGEY, 30), (GEODUDE, 15), (MACHOP, 20)],
        levels: (14, 18),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 10
    MapDef {
        name: "Ruta 10",
        rows: r"
####################
#......cccc....MMMM#
#......cccc....MMMM#
4........:.....M5MM#
4::::::::::::::::..#
#.ggg.....*......M.#
MMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMMMMMMMMM
MMMMMMMMMMMMM6MMMMMM
#............:.....#
#..ggg.......:..o..#
#..ggg...*...:.....#
#............:.....#
#.....::::::::.....#
#.....:......s.....#
#.ggg.:.....ggg....#
#.ggg.:.....ggg....#
#.....::...........#
#vvvv..:...vvvvvvvv#
#......::..........#
#.......::.........#
#.......::.........#
########22##########",
        exits: &[edge(4, R9, 3, E), edge(2, LAVENDER, 1, N), cave(5, TUNNEL, 1, S), cave(6, TUNNEL, 2, S)],
        signs: &["RUTA 10: Túnel Roca al norte, Pueblo Lavanda al sur."],
        npcs: &[
            trainer("Pokémaníaco Mark", SCIENTIST, N, 1, &[(GRIMER, 20), (KOFFING, 20)], &["¡Colecciono Pokémon raros! ¡Mira!"], "¡Mis Pokémon raros!", "Algún día los tendré todos."),
            trainer("Montañera Clara", HIKER, E, 4, &[(ONIX, 20), (GEODUDE, 21)], &["¡Acabo de salir del Túnel Roca!"], "¡Qué potencia!", "Pueblo Lavanda está aquí al lado."),
        ],
        items: &[(SUPER_POTION, 2)],
        encounters: &[(VOLTORB, 30), (RATTATA, 25), (PIDGEY, 20), (MACHOP, 25)],
        levels: (14, 18),
        ..BASE
    },
    // ------------------------------------------------------------ Túnel Roca
    MapDef {
        name: "Túnel Roca",
        rows: r"
######################
#1....#.......#....o.#
#.....#..*....#......#
#.###.#..###..#..###.#
#.....r.......r......#
####.#####.####.######
#.....#.....*........#
#..r..#..........##..#
#.....#####.....##...#
#...........r........#
######.####.#####.####
#o.......#...........#
#..##....#....*...##.#
#..##.........r...##.#
#.......r............#
#...................2#
######################",
        exits: &[edge(1, R10, 5, E), edge(2, R10, 6, W)],
        npcs: &[
            trainer("Montañero Lenny", HIKER, S, 2, &[(GEODUDE, 20), (MACHOP, 20), (ONIX, 21)], &["¡En la oscuridad no se ve nada! ¡Pero a ti sí!"], "¡Me has deslumbrado!", "Sigue hacia el sur para salir."),
            trainer("Chica Dana", LASS, W, 4, &[(ODDISH, 21), (CLEFAIRY, 21)], &["¡Qué miedo da este túnel!"], "¡Ay!", "¿Dónde está la salida?"),
            trainer("Montañero Oliver", HIKER, S, 2, &[(ONIX, 22), (GEODUDE, 22)], &["¡Mis Pokémon de roca son los mejores!"], "¡Vaya golpe!", "Volveré a entrenar."),
        ],
        items: &[(ESCAPE_ROPE, 1), (HYPER_POTION, 1)],
        encounters: &[(ZUBAT, 40), (GEODUDE, 30), (MACHOP, 20), (ONIX, 10)],
        levels: (15, 20),
        ..CAVE
    },
    // ------------------------------------------------------------ Pueblo Lavanda
    MapDef {
        name: "Pueblo Lavanda",
        rows: r"
#########11#############
#........::............#
#.hhhh...::...BBBBBB...#
#.hhhh...::...BBBBBB...#
#.hhhh...::...BBBBBB...#
#...:....::...BBBBBB...#
#...:::::::::::::::....#
4........::............#
4::::::::::...*........#
#..cccc..::....mmmm....#
#..cccc..::....mmmm....#
#..cccc..::....mmmm....#
#....:...::.......:....#
#....::::::::::::::....#
#.s......::...........f#
#........::...........f#
#########22#############",
        exits: &[edge(1, R10, 2, S), edge(2, R12, 1, N), edge(4, R8, 3, E)],
        doors: &[Door::Default, Door::Map(TOWER)],
        signs: &["PUEBLO LAVANDA: el pueblo noble y púrpura. Aquí está la Torre Pokémon."],
        npcs: &[talk("Chica", CHANNELER, W, &["La Torre Pokémon es donde descansan los Pokémon que nos dejaron.", "Dicen que por las noches se ven fantasmas..."])],
        ground: [150, 170, 120],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 8
    MapDef {
        name: "Ruta 8",
        rows: r"
########################
#..gggg....*.....gggg..#
#..gggg..........gggg..#
4::::::::::::::::::::::3
4......*.......*......:3
#.ggggg....o....ggggg..#
#.ggggg...s.....ggggg..#
########################",
        exits: &[edge(4, SAFFRON, 3, E), edge(3, LAVENDER, 4, W)],
        signs: &["RUTA 8: Ciudad Azafrán - Pueblo Lavanda."],
        npcs: &[
            trainer("Motorista Rafa", BIKER, S, 2, &[(KOFFING, 22), (GRIMER, 22)], &["¡Brrrum! ¡Apártate o lucha!"], "¡Me has gripado el motor!", "Necesito una moto nueva."),
            trainer("Jugador Sergio", GENTLEMAN, N, 1, &[(GROWLITHE, 22), (VOLTORB, 22)], &["¡Apostemos! ¡Gano yo seguro!"], "¡He perdido otra vez!", "La suerte cambiará."),
            trainer("Chica Julia", LASS, N, 1, &[(CLEFAIRY, 23), (ODDISH, 22)], &["¿Me ayudas a entrenar?"], "¡Gracias por el combate!", "Ha sido divertido."),
        ],
        items: &[(SUPER_REPEL, 1)],
        encounters: &[(GROWLITHE, 20), (PIDGEY, 25), (RATTATA, 25), (ABRA, 15), (ODDISH, 15)],
        levels: (18, 22),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 7
    MapDef {
        name: "Ruta 7",
        rows: r"
################
#..gggg...ggg..#
#..gggg...ggg..#
4::::::::::::::3
4.....*.......:3
#.gggg...s..gg.#
################",
        exits: &[edge(4, CELADON, 3, E), edge(3, SAFFRON, 4, W)],
        signs: &["RUTA 7: Ciudad Azulona - Ciudad Azafrán."],
        npcs: &[talk("Chica", GIRL, N, &["En el Centro Comercial de Azulona venden de todo.", "¡Hasta Poción Máxima, si tienes medallas suficientes!"])],
        encounters: &[(GROWLITHE, 20), (PIDGEY, 30), (RATTATA, 20), (ODDISH, 20), (ABRA, 10)],
        levels: (18, 22),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Azulona
    MapDef {
        name: "Ciudad Azulona",
        rows: r"
##############################
#..BBBBBBBB.....hhhh...ffff..#
#..BBBBBBBB.....hhhh...ffff..#
#..BBBBBBBB.....hhhh...ffff..#
#......:..........:..........#
#......::::::::::::.....*....#
#............:...............#
4::::::::::::::::::::::::::::3
4.......:..........:.........3
#..cccc.:.......BBBBBBB......#
#..cccc.:.......BBBBBBB......#
#..cccc.:.......BBBBBBB......#
#....::::..........:.........#
#.......:..........:.........#
#.......::::::::::::.........#
#..ffff......:.....GGGGGGG...#
#..ffff......:.....GGGGGGG...#
#..ffff..s...:.....GGGGGGG...#
#.............::::::::::.....#
##############################",
        exits: &[edge(3, R7, 4, W), edge(4, R16, 3, E)],
        doors: &[
            Door::Map(MART_IN),
            Door::Default,
            Door::Default,
            Door::Map(CASINO),
            Door::Map(GYM_CELADON),
        ],
        signs: &["CIUDAD AZULONA: la ciudad de los sueños arcoíris. Gimnasio de Erika."],
        npcs: &[talk("Señor", GENTLEMAN, W, &["Erika, la líder, cuida un jardín precioso de Pokémon Planta.", "El fuego, el hielo y los Pokémon voladores le hacen mucho daño."])],
        roof: [110, 190, 100],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 12
    MapDef {
        name: "Ruta 12",
        rows: r"
#######11#######
#~~~~..::..~~~~#
#~~~~..::..~~~~#
#~~~~*.::..~~~~#
#~~~~..::..~~~~#
#~~~~..::.*~~~~#
#......::......#
#.gg...::...gg.#
4:::::::::.....#
4......::......#
#..s...::..o...#
#~~~...::...~~~#
#~~~*..::...~~~#
#~~~...::...~~~#
#......::......#
#.ggg..::..ggg.#
#.ggg..::..ggg.#
#......::......#
#######22#######",
        exits: &[edge(1, LAVENDER, 2, S), edge(4, R11, 3, E), edge(2, R13, 1, N)],
        signs: &["RUTA 12: el muelle de pesca de Lavanda."],
        npcs: &[
            trainer("Pescador Ned", FISHER, E, 2, &[(MAGIKARP, 22), (MAGIKARP, 22), (STARYU, 22)], &["¡Shhh! ¡Que se escapan los peces! ...Bueno, ya que estás, ¡lucha!"], "¡Ni un pez!", "Los Magikarp no sirven de mucho... ¿o sí?"),
            trainer("Pescador Elliot", FISHER, W, 2, &[(SEEL, 23)], &["¡He pescado un Seel!"], "¡Mi Seel!", "Vuelve a pescar conmigo."),
            trainer("Pescador Andrés", FISHER, E, 3, &[(MAGIKARP, 23), (SEEL, 23)], &["¡Mira qué capturas!"], "¡Uf!", "Hoy no pican."),
        ],
        items: &[(ULTRA_BALL, 1)],
        encounters: &[(ODDISH, 35), (PIDGEY, 30), (RATTATA, 20), (TANGELA, 15)],
        levels: (22, 26),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 13
    MapDef {
        name: "Ruta 13",
        rows: r"
##################11####
#..gg.....gg......::...#
#..gg..*..gg....FF::FF.#
4:::::::::::::::::::...#
4.......*.......*......#
#.ggggg.....s....gggg..#
#.ggggg..o.......gggg..#
########################",
        exits: &[edge(1, R12, 2, S), edge(4, R14, 3, E)],
        signs: &["RUTA 13: el laberinto de vallas. Sigue al oeste."],
        npcs: &[
            trainer("Ornitólogo Pepe", YOUNGSTER, S, 2, &[(PIDGEY, 24), (PIDGEY, 24), (ZUBAT, 25)], &["¡Mis pájaros vuelan alto!"], "¡Han caído!", "Los Pokémon voladores son preciosos."),
            trainer("Chica Mari", LASS, N, 1, &[(CLEFAIRY, 26)], &["¿Luchamos?"], "¡Oh!", "Me gusta tu equipo."),
            trainer("Motorista Iván", BIKER, N, 1, &[(GRIMER, 25), (KOFFING, 25)], &["¡Esta ruta es mía!"], "¡Pfff!", "Me voy al Camino de Bicis."),
        ],
        items: &[(HYPER_POTION, 1)],
        encounters: &[(ODDISH, 35), (PIDGEY, 30), (TANGELA, 15), (GROWLITHE, 20)],
        levels: (22, 26),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 14
    MapDef {
        name: "Ruta 14",
        rows: r"
##############
#..gggg......#
#..gggg......3
#::::::::::::3
#::.....*....#
#::.gggg.....#
#::.gggg..*..#
#::..........#
#::...s......#
#::.....gggg.#
#::..o..gggg.#
#::..........#
#::.ggg..*...#
#::.ggg......#
#::..........#
#22###########",
        exits: &[edge(3, R13, 4, W), edge(2, R15, 1, N)],
        signs: &["RUTA 14: al sur, Ciudad Fucsia."],
        npcs: &[
            trainer("Ornitólogo Carlos", YOUNGSTER, N, 1, &[(PIDGEY, 26), (ZUBAT, 26)], &["¡Mira cómo vuelan!"], "¡Aterrizaje forzoso!", "Volveré a volar."),
            trainer("Motorista Lucas", BIKER, W, 8, &[(KOFFING, 27), (KOFFING, 27)], &["¡Brrrum! ¡A luchar!"], "¡Me he quedado sin gasolina!", "Ciudad Fucsia está al sur."),
            trainer("Motorista Mario", BIKER, W, 7, &[(GRIMER, 28)], &["¡Mi Grimer apesta! ¡Jajaja!"], "¡Puaj!", "Necesito una ducha."),
        ],
        items: &[(RARE_CANDY, 1)],
        encounters: &[(ODDISH, 30), (PIDGEY, 30), (TANGELA, 20), (GROWLITHE, 20)],
        levels: (24, 28),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 15
    MapDef {
        name: "Ruta 15",
        rows: r"
####################11##
#..gggg......gggg...::.#
#..gggg..*...gggg...::.#
4:::::::::::::::::::::.#
4......*........*......#
#.gggg....o...s...gggg.#
########################",
        exits: &[edge(1, R14, 2, S), edge(4, FUCHSIA, 3, E)],
        signs: &["RUTA 15: Ciudad Fucsia al oeste."],
        npcs: &[
            trainer("Chica Natalia", LASS, S, 2, &[(CLEFAIRY, 28), (ODDISH, 28)], &["¡Mis Pokémon son monísimos y fortísimos!"], "¡No puede ser!", "Bueno, monísimos sí son."),
            trainer("Ornitóloga Eva", LASS, N, 1, &[(PIDGEY, 28), (ZUBAT, 28), (PIDGEY, 28)], &["¡A volar!"], "¡Plumas por todas partes!", "Me encantan los pájaros."),
            trainer("Bella Clara", GIRL, N, 1, &[(PONYTA, 29)], &["¡Mi Ponyta es el más rápido!"], "¡Uf, qué calor!", "Ponyta necesita correr más."),
        ],
        items: &[(SUPER_BALL, 3)],
        encounters: &[(ODDISH, 30), (PIDGEY, 30), (TANGELA, 20), (GROWLITHE, 20)],
        levels: (24, 28),
        ..BASE
    },
    // ------------------------------------------------------------ Ciudad Fucsia
    MapDef {
        name: "Ciudad Fucsia",
        rows: r"
############################
#....BBBBBBBBBB.......ffff.#
#....BBBBBBBBBB.......ffff.#
#....BBBBBBBBBB....*..ffff.#
#.........:................#
#..hhhh...:.......GGGGG....#
#..hhhh...:.......GGGGG....#
#..hhhh...:.......GGGGG....#
#....:....:.........:......#
4::::::::::::::::::::::::::3
4.........:................3
#..cccc...:......mmmm......#
#..cccc...:......mmmm......#
#..cccc...:......mmmm......#
#....:....:........:.......#
#....:::::::::::::::.......#
#..........:.......s.......#
#~~~~......::.........~~~~.#
#~~~~.....::..........~~~~.#
#..........::..............#
#..........::..............#
###########22###############",
        exits: &[edge(4, R18, 3, E), edge(3, R15, 4, W), edge(2, R19, 1, N)],
        doors: &[Door::Map(SAFARI), Door::Default, Door::Map(GYM_FUCHSIA)],
        signs: &["CIUDAD FUCSIA: ¡detrás de la Zona Safari se esconden muchos secretos!"],
        npcs: &[talk("Guarda", GUARD, S, &["El líder Koga es un maestro ninja.", "Sus Pokémon Veneno no soportan los ataques de tipo Psíquico y Tierra."])],
        roof: [200, 80, 170],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 16
    MapDef {
        name: "Ruta 16",
        rows: r"
####################
#..ggggg....gggg...#
#..ggggg.*..gggg...#
#..................3
#::::::::::::::::::3
#::.....*..........#
#::..gggg...o..ggg.#
#::..gggg......ggg.#
#::..........s.....#
#22#################",
        exits: &[edge(3, CELADON, 4, W), edge(2, R17, 1, N)],
        signs: &["RUTA 16: al sur empieza el Camino de Bicis."],
        npcs: &[
            trainer("Motorista Rico", BIKER, S, 2, &[(GRIMER, 28), (KOFFING, 28)], &["¡Esta es nuestra zona!"], "¡Vaya paliza!", "Vete ya."),
            trainer("Motorista Jaime", BIKER, N, 1, &[(KOFFING, 29), (VOLTORB, 29)], &["¡Brrrrum!"], "¡Pinchazo!", "Mi moto no arranca."),
        ],
        items: &[(HYPER_POTION, 1)],
        encounters: &[(RATTATA, 35), (PIDGEY, 30), (GRIMER, 15), (KOFFING, 20)],
        levels: (24, 28),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 17 (Camino de Bicis)
    MapDef {
        name: "Ruta 17 - Camino de Bicis",
        rows: r"
#11#########
#::........#
#::.ggg....#
#::.ggg..*.#
#::........#
#::.....r..#
#::...*....#
#::.gggg...#
#::.gggg...#
#::........#
#::...s....#
#::.....*..#
#::.ggg....#
#::.ggg..o.#
#::........#
#::..*.....#
#::.gggg...#
#::.gggg...#
#::........#
#::......*.#
#::.ggg....#
#::.ggg....#
#::........#
#22#########",
        exits: &[edge(1, R16, 2, S), edge(2, R18, 1, N)],
        signs: &["CAMINO DE BICIS: ¡cuidado con las bandas de moteros!"],
        npcs: &[
            trainer("Motorista Ángel", BIKER, W, 7, &[(KOFFING, 29), (GRIMER, 29)], &["¡Esta cuesta es nuestra!"], "¡Me caigo!", "Bájate la cuesta con cuidado."),
            trainer("Motorista Toni", BIKER, W, 4, &[(VOLTORB, 30), (KOFFING, 30)], &["¿Quién es este pringado?"], "Vale, no eres un pringado.", "Me retiro."),
            trainer("Motorista Rober", BIKER, W, 6, &[(GRIMER, 31)], &["¡Mi Grimer no se lava nunca!"], "¡Puaj!", "Vete de aquí."),
            trainer("Motorista Óscar", BIKER, W, 3, &[(KOFFING, 30), (GROWLITHE, 30)], &["¡Brrrum, brrrum!"], "¡Se me ha calado!", "..."),
            trainer("Jefe Motorista", BIKER, W, 7, &[(GRIMER, 32), (KOFFING, 32), (GROWLITHE, 32)], &["¡Soy el jefe de esta banda!"], "¡Imposible!", "Respeto tu fuerza."),
        ],
        items: &[(RARE_CANDY, 1)],
        encounters: &[(RATTATA, 25), (PIDGEY, 25), (GRIMER, 15), (KOFFING, 15), (PONYTA, 20)],
        levels: (26, 30),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 18
    MapDef {
        name: "Ruta 18",
        rows: r"
#11#################
#::...ggggg.....gg.#
#::...ggggg..*..gg.#
#::::::::::::::::::3
#.........*.......:3
#..gggg.......s....#
####################",
        exits: &[edge(1, R17, 2, S), edge(3, FUCHSIA, 4, W)],
        signs: &["RUTA 18: Ciudad Fucsia al este."],
        npcs: &[
            trainer("Ornitólogo Wilton", YOUNGSTER, S, 2, &[(PIDGEY, 30), (ZUBAT, 30)], &["¡Los pájaros de aquí son fortísimos!"], "¡Ni una pluma!", "Ciudad Fucsia está muy cerca."),
            trainer("Ornitólogo Ramiro", YOUNGSTER, N, 1, &[(PIDGEY, 31), (PIDGEY, 31)], &["¡Mis Pidgey vuelan rapidísimo!"], "¡Caída libre!", "Volveré a entrenar."),
        ],
        encounters: &[(RATTATA, 30), (PIDGEY, 35), (GROWLITHE, 15), (PONYTA, 20)],
        levels: (26, 30),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 19 (mar)
    MapDef {
        name: "Ruta 19",
        rows: r"
######11########
#.....::.......#
#..b..::...s...#
#~~~~~==~~~~~~~#
#~~~~~==~~~~~~~#
#~~~~*==~~~~~~~#
#~~~~~==~~~~~~~#
#~~~~~==*~~~~~~#
#~~~~~==~~~~~~~#
#~~~~~==~~~~~~~#
#~~~~*==~~~~~~~#
#~~~~~==~~~~~~~#
4=======~~~~~~~#
4=======~~~~~~~#
#~~~~~~~~~~~~~~#
################",
        exits: &[edge(1, FUCHSIA, 2, S), edge(4, R20, 3, E)],
        signs: &["RUTA 19: la pasarela marina hasta las Islas Espuma."],
        npcs: &[
            trainer("Nadador Ricardo", SWIMMER, E, 1, &[(STARYU, 30), (MAGIKARP, 30)], &["¡El agua está buenísima! ¡A luchar!"], "¡Glu, glu!", "Nadar es lo mejor."),
            trainer("Nadadora Lucía", SWIMMER, W, 1, &[(SEEL, 31), (STARYU, 30)], &["¡Mis Pokémon nadan más rápido que tú!"], "¡Me hundo!", "Ten cuidado en la pasarela."),
            trainer("Nadador Tomás", SWIMMER, E, 1, &[(MAGIKARP, 31), (SEEL, 31)], &["¡Splash!"], "¡Plaf!", "Seguiré nadando."),
        ],
        ..SEA
    },
    // ------------------------------------------------------------ Ruta 20 (mar)
    MapDef {
        name: "Ruta 20",
        rows: r"
##############################
#~~~~~~~~~~~~~~~~~~~~~~~~~~~~#
#~~~~~~~~~~MMMMMMMM~~~~~~~~~~#
#~~~~~~~~~MMMMMMMMMM~~~~~~~~~#
#~~~~~~~~=6MMMMMMMM5=~~~~~~~~#
4=========MMMMMMMMMM=========3
4=========MMMMMMMMMM=========3
#~~~~*~~~~~MMMMMMMM~~~~*~~~~~#
#~~~~~~~~~~~MMMMMM~~~~~~~~~~~#
#~~~~~~~~~~~~~~~~~~~~~~~~~~~~#
##############################",
        exits: &[edge(3, R19, 4, W), edge(4, CINNABAR, 3, E), cave(5, SEAFOAM, 1, E), cave(6, SEAFOAM, 2, W)],
        npcs: &[
            trainer("Nadadora Marta", SWIMMER, N, 1, &[(STARYU, 32), (STARMIE, 32)], &["¡Las Islas Espuma están heladas!"], "¡Brrr!", "Abrígate si entras."),
            trainer("Nadador Javi", SWIMMER, N, 1, &[(SEEL, 33), (MAGIKARP, 33)], &["¡Último combate antes de la isla!"], "¡Arrastrado por la corriente!", "Isla Canela está al oeste."),
        ],
        ..SEA
    },
    // ------------------------------------------------------------ Islas Espuma
    MapDef {
        name: "Islas Espuma",
        rows: r"
####################
#....~~~~.....r....#
#.r..~~~~..........#
#.......o.....##...#
2.....r.......##...1
#..~~~~~...........#
#..~~~~~....r......#
#..........~~~~....#
#.o..r.....~~~~....#
####################",
        exits: &[edge(1, R20, 5, W), edge(2, R20, 6, E)],
        items: &[(ULTRA_BALL, 2), (MAX_POTION, 1)],
        encounters: &[(SEEL, 40), (ZUBAT, 30), (STARYU, 15), (GEODUDE, 15)],
        levels: (28, 34),
        ground: [175, 205, 220],
        ..CAVE
    },
    // ------------------------------------------------------------ Isla Canela
    MapDef {
        name: "Isla Canela",
        rows: r"
#########11#############
#~~~.....::......~~~~~~#
#.BBBBBB.::..GGGGG.....#
#.BBBBBB.::..GGGGG.....#
#.BBBBBB.::..GGGGG.....#
#....:...::....:.......#
#....::::::::::::......#
#.........::...........3
#.........:::::::::::::3
#..cccc...::..mmmm.....#
#..cccc...::..mmmm.....#
#..cccc...::..mmmm.....#
#....:....::....:.....s#
#....:::::::::::::.....#
#.....BBBB.............#
#.....BBBB..*..........#
#.......:..............#
########################",
        exits: &[edge(1, R21, 2, S), edge(3, R20, 4, W)],
        doors: &[
            Door::Map(MANSION),
            Door::Map(GYM_CINNABAR),
            Door::Default,
            Door::Default,
            Door::Text("Laboratorio Pokémon de Canela: «Investigamos fósiles. Prohibido el paso.»"),
        ],
        signs: &["ISLA CANELA: la ardiente ciudad del deseo. Gimnasio de Blaine."],
        npcs: &[talk("Científico", SCIENTIST, N, &["Blaine, el líder, es un hombre de fuego... y de acertijos.", "El agua, la tierra y la roca le hacen mucho daño a sus Pokémon."])],
        roof: [220, 70, 50],
        ..TOWN
    },
    // ------------------------------------------------------------ Ruta 21 (mar)
    MapDef {
        name: "Ruta 21",
        rows: r"
#####11#######
#~~~~==~~~~~~#
#~~~~==~~~~~~#
#~~~~==*~~~~~#
#~~~~==~~~~~~#
#~~gg==gg~~~~#
#~~gg==gg~~~~#
#~~~~==~~~~~~#
#~~~*==~~~~~~#
#~~~~==~~~~~~#
#~~~~==~~~~~~#
#~~~~==*~~~~~#
#~~~~==~~~~~~#
#~~gg==gg~~~~#
#~~gg==gg~~~~#
#~~~~==~~~~~~#
#~~~~==~~~~~~#
#####22#######",
        exits: &[edge(1, PALLET, 2, S), edge(2, CINNABAR, 1, N)],
        npcs: &[
            trainer("Pescador Ronaldo", FISHER, W, 1, &[(MAGIKARP, 28), (STARYU, 28)], &["¡Aquí pescan los mejores!"], "¡Se me ha roto la caña!", "El mar es inmenso."),
            trainer("Nadadora Sonia", SWIMMER, E, 1, &[(SEEL, 30), (STARMIE, 30)], &["¡Nado desde Pueblo Paleta hasta Canela!"], "¡Uf, qué cansancio!", "Sigue la pasarela."),
            trainer("Pescador Wade", FISHER, W, 1, &[(MAGIKARP, 30), (MAGIKARP, 30), (SEEL, 30)], &["¡Mira mi colección de Magikarp!"], "¡Splash!", "Algún día evolucionarán..."),
        ],
        encounters: &[(TANGELA, 20), (PIDGEY, 30), (RATTATA, 30), (MAGIKARP, 20)],
        levels: (21, 30),
        ..SEA
    },
    // ------------------------------------------------------------ Ruta 22
    MapDef {
        name: "Ruta 22",
        rows: r"
##11####################
#.::...........gggggg..#
#.::.MMMM......gggggg..#
#.::.MMMM..*.......vvvv#
#.::::::::::::::::.....#
#...gggg......:::::::::3
#...gggg....s.........:3
#~~~~.....gggg.........#
#~~~~.....gggg..b......#
########################",
        exits: &[edge(3, VIRIDIAN, 4, W), edge(1, R23, 2, S)],
        signs: &["RUTA 22: entrada a la Liga Pokémon. Solo para entrenadores con medallas."],
        npcs: &[trainer(
            "Rival Azul",
            RIVAL,
            S,
            3,
            &[(PIDGEY, 9), (SQUIRTLE, 8)],
            &["¡Eh, tú! ¿Vas a la Liga Pokémon? ¡Ja!", "Seguro que no tienes ni una medalla. ¡Veamos de qué eres capaz!"],
            "¿Qué? ¡Habrá sido suerte!",
            "Me voy a por las medallas. ¡Te espero en la Liga!",
        )],
        encounters: &[(RATTATA, 45), (PIDGEY, 35), (MAGIKARP, 5), (MACHOP, 15)],
        levels: (3, 6),
        ..BASE
    },
    // ------------------------------------------------------------ Ruta 23
    MapDef {
        name: "Ruta 23",
        rows: r"
MMMMMM11MMMMMMMM
M.....::.......M
M.ggg.::.ggg...M
M.ggg.::.ggg...M
M.....::Mo.....M
MMMMMMM*.MMMMMMM
M......:MMMMMMMM
M..ggg.:.ggg...M
M..ggg.:.ggg...M
M......:M......M
MMMMMMM*.MMMMMMM
M......:M......M
M.ggg..:..ggg..M
M.ggg..:..ggg..M
M......:...s...M
M......::......M
MMMMMMM22MMMMMMM",
        exits: &[cave(1, VICTORY, 2, S), edge(2, R22, 1, N)],
        signs: &["RUTA 23: Calle Victoria al norte. Solo pasan los entrenadores con las 8 medallas."],
        npcs: &[
            guard(8, "Solo pueden pasar los entrenadores con las 8 medallas de Kanto.", "¡Tienes las 8 medallas! Adelante, la Calle Victoria te espera.", E),
            guard(4, "Para seguir necesitas al menos 4 medallas.", "¡4 medallas! Puedes pasar.", E),
        ],
        items: &[(MAX_POTION, 1)],
        encounters: &[(MACHOP, 25), (PIDGEY, 25), (RHYHORN, 20), (PONYTA, 15), (GROWLITHE, 15)],
        levels: (38, 43),
        ..BASE
    },
    // ------------------------------------------------------------ Calle Victoria
    MapDef {
        name: "Calle Victoria",
        rows: r"
########11##########
#.......::...#..o..#
#.###...::...#.....#
#.###..........##..#
#......*..r........#
####.####.#####.####
#...........r......#
#..~~~~.......###..#
#..~~~~...*...###..#
#.........r........#
#.####.######.####.#
#o.....r...........#
#....###....###....#
#.......:::........#
#.......::.........#
########22##########",
        exits: &[edge(1, INDIGO, 2, S), edge(2, R23, 1, N)],
        npcs: &[
            trainer("Entrenador Guay Naomi", ACE, E, 3, &[(PONYTA, 42), (STARMIE, 42), (TANGELA, 42)], &["Solo los mejores llegan hasta aquí."], "Eres de los mejores, sí.", "Suerte en la Liga."),
            trainer("Entrenador Guay Rolando", ACE, W, 4, &[(RHYHORN, 43), (GROWLITHE, 43), (MACHOP, 43)], &["¡La Liga está al otro lado! ¡Pero primero yo!"], "¡Increíble!", "Ve con todo."),
        ],
        items: &[(ULTRA_BALL, 3), (MAX_POTION, 1)],
        encounters: &[(MACHOP, 25), (GEODUDE, 20), (ONIX, 20), (ZUBAT, 20), (RHYHORN, 15)],
        levels: (40, 45),
        ..CAVE
    },
    // ------------------------------------------------------------ Meseta Añil
    MapDef {
        name: "Meseta Añil",
        rows: r"
####################
#..BBBBBBBBBBBBBB..#
#..BBBBBBBBBBBBBB..#
#..BBBBBBBBBBBBBB..#
#..BBBBBBBBBBBBBB..#
#.........:........#
#.........:........#
#..cccc...:...s....#
#..cccc...:........#
#..cccc...:........#
#....:....:........#
#....::::::........#
#f.......::.......f#
#########22#########",
        exits: &[edge(2, VICTORY, 1, N)],
        doors: &[Door::Map(LEAGUE)],
        signs: &["LIGA POKÉMON: el Alto Mando y el Campeón te esperan. ¡No hay vuelta atrás!"],
        roof: [90, 70, 160],
        ..TOWN
    },
    // ------------------------------------------------------------ Centro Pokémon
    MapDef {
        name: "Centro Pokémon",
        rows: r"
############
#P...*....S#
#..CCCCCC..#
#..........#
#K........K#
#..T....T..#
#..T..*.T..#
#....xx....#
#####00#####",
        exits: &[mat()],
        npcs: &[nurse(), talk("Entrenador", ACE, N, &["Cuando tu equipo esté cansado, ven al Centro Pokémon.", "¡La enfermera Joy los cura gratis!", "Pulsa Enter para abrir el menú y guardar la partida."])],
        ..INSIDE
    },
    // ------------------------------------------------------------ Tienda Pokémon
    MapDef {
        name: "Tienda Pokémon",
        rows: r"
############
#SS.....SSS#
#*C.......K#
#.C........#
#....SS.SS.#
#....SS.SS.#
#......*...#
#....xx....#
#####00#####",
        exits: &[mat()],
        npcs: &[clerk(), talk("Cliente", GIRL, N, &["Con más medallas, las tiendas venden objetos mejores.", "¡Las Super Ball atrapan Pokémon mucho más fácilmente!"])],
        ground: [205, 225, 240],
        ..INSIDE
    },
    // ------------------------------------------------------------ Casa
    MapDef {
        name: "Casa",
        rows: r"
##########
#S.SS..K.#
#........#
#..TT..*.#
#..TT....#
#K.......#
#...xx...#
####00####",
        exits: &[mat()],
        npcs: &[talk("Vecino", GENTLEMAN, W, &["¡Hola! ¿Eres un entrenador de Pokémon?", "Cuida bien de tus Pokémon y ellos cuidarán de ti."])],
        ..INSIDE
    },
    // ------------------------------------------------------------ Tu casa
    MapDef {
        name: "Tu casa",
        rows: r"
##########
#SS.P..KK#
#........#
#..TT.*..#
#..TT....#
#K.......#
#...xx...#
####00####",
        exits: &[mat()],
        npcs: &[Npc { name: "Mamá", look: MOM, facing: W, role: Role::Mom, aside: None }],
        ..INSIDE
    },
    // ------------------------------------------------------------ Laboratorio
    MapDef {
        name: "Laboratorio del Profe Roble",
        rows: r"
##############
#SSS.PP...SSS#
#.....*......#
#....TTT.....#
#............#
#SS.......*.S#
#SS..........#
#K..........K#
#.....xx.....#
######00######",
        exits: &[mat()],
        npcs: &[
            Npc {
                name: "Profe Roble",
                look: OAK,
                facing: S,
                role: Role::Gift {
                    lines: &[
                        "¡Hola! ¡Bienvenido al mundo de los Pokémon!",
                        "Tu misión es completar la Pokédex y conseguir las 8 medallas de Kanto.",
                        "Toma, estas Poké Balls te ayudarán a atrapar Pokémon salvajes.",
                    ],
                    item: POKE_BALL,
                    qty: 5,
                    after: "Habla con mi ayudante si quieres poner a prueba lo que sabes de tipos.",
                },
                aside: None,
            },
            Npc {
                name: "Ayudante",
                look: SCIENTIST,
                facing: W,
                role: Role::Quiz(Quiz {
                    intro: &["Conocer los tipos es la clave para ganar combates.", "A ver si estabas atento..."],
                    question: "¿Qué DOS tipos son muy eficaces contra Agua?",
                    options: &["Fuego", "Planta", "Eléctrico", "Normal"],
                    correct: &[1, 2],
                    reward: Reward::Item(SUPER_POTION, 2),
                    beaten: "¡Sabes mucho de tipos! Sigue así.",
                }),
                aside: None,
            },
        ],
        ground: [230, 230, 225],
        ..INSIDE
    },
    // ------------------------------------------------------------ Gimnasio de Plateada (Brock)
    MapDef {
        name: "Gimnasio de Plateada",
        rows: r"
############
#....*.....#
#.r......r.#
#..........#
#.rr....rr.#
#....*.....#
#.r......r.#
#..........#
#.rr....rr.#
#U.......U.#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_1, GYM_SIGN_1],
        npcs: &[
            leader("Líder Brock", BROCK, 0, &[(GEODUDE, 12), (ONIX, 14)], &["¡Soy Brock, el líder del gimnasio de Plateada!", "Creo en la defensa de piedra y en la determinación. ¡Mis Pokémon de tipo Roca no se rinden!"], "¡Te he subestimado! Toma la Medalla Roca.", "La Medalla Roca demuestra que eres fuerte. ¡Sigue hacia Ciudad Celeste!"),
            trainer("Campista Jerry", HIKER, S, 4, &[(GEODUDE, 10), (DIGLETT, 10)], &["¡Eh, para! ¡Estás a años luz de poder vencer a Brock!"], "¡Increíble!", "Brock es mucho más fuerte que yo."),
        ],
        ground: [190, 160, 120],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Celeste (Misty)
    MapDef {
        name: "Gimnasio de Celeste",
        rows: r"
############
#....*.....#
#~~~~..~~~~#
#~~~~..~~~~#
#~~~*..~~~~#
#~~~~..~~~~#
#~~~~..*~~~#
#~~~~..~~~~#
#..........#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_2, GYM_SIGN_2],
        npcs: &[
            leader("Líder Misty", MISTY, 1, &[(STARYU, 18), (STARMIE, 21)], &["¡Hola! Soy Misty, ¡la sirena de Celeste!", "Mi táctica es atacar con Pokémon de tipo Agua. ¡A ver qué tal lo haces!"], "¡Vaya! ¡Eres muy bueno! Toma la Medalla Cascada.", "Con la Medalla Cascada, tus Pokémon te obedecerán mejor."),
            trainer("Nadador Luis", SWIMMER, E, 2, &[(MAGIKARP, 16), (STARYU, 16)], &["¡Misty es la mejor nadadora!"], "¡Glub!", "Misty te espera al fondo."),
            trainer("Nadadora Diana", SWIMMER, W, 2, &[(STARYU, 17), (SEEL, 17)], &["¡Soy la mejor alumna de Misty!"], "¡Ay!", "No te confíes con Starmie."),
        ],
        ground: [190, 220, 240],
        roof: [70, 150, 220],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Carmín (Teniente Surge)
    MapDef {
        name: "Gimnasio de Carmín",
        rows: r"
############
#....*.....#
#FFFF..FFFF#
#P........P#
#...*......#
#FF..FF..FF#
#......*...#
#P........P#
#..........#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_3, GYM_SIGN_3],
        npcs: &[
            leader("Teniente Surge", SURGE, 2, &[(VOLTORB, 21), (PIKACHU, 18), (VOLTORB, 24)], &["¡Eh, chaval! ¡Bienvenido al gimnasio de Carmín!", "¡Los Pokémon Eléctricos me salvaron en la guerra! ¡Te voy a electrocutar!"], "¡Uau! ¡Eres un auténtico soldado! Toma la Medalla Trueno.", "¡Con la Medalla Trueno, tus Pokémon serán más rápidos!"),
            trainer("Marinero Dwayne", SWIMMER, E, 4, &[(PIKACHU, 21), (VOLTORB, 21)], &["¡El Teniente nos entrena duro!"], "¡Cortocircuito!", "El tipo Tierra es inmune a la electricidad."),
            trainer("Mecánico Baily", SCIENTIST, W, 4, &[(VOLTORB, 21), (VOLTORB, 21), (MAGIKARP, 21)], &["¡Bzzzt! ¡Electricidad a tope!"], "¡Se fue la luz!", "Prueba con Pokémon de tipo Tierra."),
        ],
        ground: [200, 200, 180],
        roof: [230, 150, 50],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Azulona (Erika)
    MapDef {
        name: "Gimnasio de Azulona",
        rows: r"
############
#f...*...f.#
#bb.....bbb#
#ff..*...ff#
#bbb....bbb#
#..........#
#ff...*..ff#
#bbbb..bbbb#
#ff......ff#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_4, GYM_SIGN_4],
        npcs: &[
            leader("Líder Erika", ERIKA, 3, &[(TANGELA, 29), (ODDISH, 24), (BULBASAUR, 29)], &["Hola... Qué buen día hace, ¿verdad?", "Soy Erika, líder de Azulona. Me encantan las flores y los Pokémon Planta. ¿Luchamos?"], "Oh... He perdido. Toma, la Medalla Arcoíris.", "Aprenderé de esta derrota. Sigue cuidando de tus Pokémon."),
            trainer("Chica Ana", LASS, S, 4, &[(ODDISH, 23), (BULBASAUR, 23)], &["¡No pises las flores!"], "¡Ay, mis plantas!", "Erika es tan elegante..."),
            trainer("Belleza Tamia", GIRL, W, 3, &[(TANGELA, 24), (ODDISH, 24)], &["¡Este gimnasio solo admite chicas! Bueno... te dejo pasar si me ganas."], "¡Qué grosería ganar!", "Erika te espera."),
        ],
        ground: [180, 220, 150],
        roof: [110, 190, 100],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Fucsia (Koga)
    MapDef {
        name: "Gimnasio de Fucsia",
        rows: r"
############
#....*.....#
#SSSS..SSSS#
#...*......#
#.SSSSSSSS.#
#.......*..#
#SSSS.SSSSS#
#..........#
#..*.......#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_5, GYM_SIGN_5],
        npcs: &[
            leader("Líder Koga", KOGA, 4, &[(KOFFING, 37), (GRIMER, 39), (KOFFING, 37), (GRIMER, 43)], &["¡Fua ja ja! ¡Un mocoso osa retarme!", "Soy Koga, maestro ninja. ¡Conocerás el terror del veneno!"], "¡Humm! Has demostrado tu valía. Toma la Medalla Alma.", "La Medalla Alma te honra. Sigue tu camino, joven."),
            trainer("Malabarista Kirk", GENTLEMAN, E, 4, &[(VOLTORB, 34), (KOFFING, 34)], &["¡Los ninjas no se ven venir!"], "¡Me has visto!", "Este gimnasio es un laberinto."),
            trainer("Domador Phil", BIKER, W, 6, &[(GROWLITHE, 34), (PONYTA, 34)], &["¡Mis fieras están hambrientas!"], "¡Qué miedo!", "Koga es muy astuto."),
            trainer("Malabarista Nate", GENTLEMAN, E, 6, &[(GRIMER, 35), (KOFFING, 35)], &["¡Veneno por todas partes!"], "¡Puaj!", "Usa Pokémon de tipo Psíquico."),
        ],
        ground: [190, 170, 200],
        roof: [200, 80, 170],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Azafrán (Sabrina)
    MapDef {
        name: "Gimnasio de Azafrán",
        rows: r"
############
#....*.....#
#..P....P..#
#..........#
#.*......*.#
#..........#
#..P....P..#
#....*.....#
#..........#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_6, GYM_SIGN_6],
        npcs: &[
            leader("Líder Sabrina", SABRINA, 5, &[(ABRA, 38), (STARMIE, 37), (GASTLY, 38), (ABRA, 43)], &["Sabía que vendrías...", "Tengo poderes psíquicos desde que era pequeña. Ya sé cómo va a acabar este combate."], "No lo había previsto... Toma la Medalla Pantano.", "Tu poder supera a mis visiones. Enhorabuena."),
            trainer("Médium Tasha", CHANNELER, E, 6, &[(GASTLY, 34), (ABRA, 34)], &["Puedo leer tu mente..."], "¡No lo vi venir!", "Sabrina lo ve todo."),
            trainer("Médium Cameron", CHANNELER, W, 6, &[(ABRA, 34), (KOFFING, 34)], &["Sé lo que vas a hacer."], "¡Mis visiones fallaron!", "El tipo Bicho es su punto débil."),
            trainer("Psíquico Johan", SCIENTIST, S, 2, &[(ABRA, 36), (STARMIE, 36)], &["¡Doblaré tu cuchara con la mente!"], "¡Se me dobló a mí!", "Sabrina te espera."),
        ],
        ground: [220, 200, 160],
        roof: [220, 180, 60],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Canela (Blaine)
    MapDef {
        name: "Gimnasio de Canela",
        rows: r"
############
#....*.....#
#rr......rr#
#...*......#
#.rrr..rrr.#
#......*...#
#rr......rr#
#..........#
#P........P#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_7, GYM_SIGN_7],
        npcs: &[
            leader("Líder Blaine", BLAINE, 6, &[(GROWLITHE, 42), (PONYTA, 40), (PONYTA, 42), (GROWLITHE, 47)], &["¡Jajajá! ¡Soy Blaine, el líder ardiente!", "¡Mis Pokémon de Fuego te van a dejar chamuscado! ¿Tienes Antiquemar? ¡Pues lo vas a necesitar!"], "¡Me he quemado yo! Toma la Medalla Volcán.", "La Medalla Volcán es tuya. ¡Apágame el fuego otra vez cuando quieras!"),
            trainer("Científico Ramón", SCIENTIST, E, 5, &[(GROWLITHE, 36), (VOLTORB, 36)], &["¡Pregunta de examen: el Fuego quema la Planta!"], "¡Suspenso!", "Blaine pone exámenes muy difíciles."),
            trainer("Superfriki Erik", YOUNGSTER, W, 5, &[(PONYTA, 37), (KOFFING, 37)], &["¡Calor, calor!"], "¡Me he derretido!", "Usa Agua, Tierra o Roca."),
        ],
        ground: [210, 150, 120],
        roof: [220, 70, 50],
        ..GYM
    },
    // ------------------------------------------------------------ Gimnasio de Verde (Giovanni)
    MapDef {
        name: "Gimnasio de Verde",
        rows: r"
############
#....*.....#
#.rr....rr.#
#..........#
#...*......#
#.rr....rr.#
#......*...#
#.rr....rr.#
#..........#
#U........U#
#....xx....#
#####00#####",
        exits: &[mat()],
        signs: &[GYM_SIGN_8, GYM_SIGN_8],
        npcs: &[
            leader("Líder Giovanni", GIOVANNI, 7, &[(RHYHORN, 45), (DUGTRIO, 42), (ONIX, 44), (RHYHORN, 50)], &["¡Ja! Así que has llegado hasta aquí.", "Soy Giovanni, el líder de Ciudad Verde... y el jefe del Team Rocket. ¡Prepárate!"], "¡Ha! Has ganado... Toma la Medalla Tierra.", "El Team Rocket se disuelve. Quizá vuelva algún día..."),
            trainer("Entrenador Guay Yuji", ACE, E, 5, &[(RHYHORN, 38), (DIGLETT, 38)], &["¡El jefe es invencible!"], "¡Imposible!", "Giovanni es muy fuerte."),
            trainer("Entrenadora Guay Sara", ACE, W, 5, &[(DUGTRIO, 40), (MACHOP, 40)], &["¡Bienvenido al gimnasio más difícil de Kanto!"], "¡Qué nivel!", "Usa Agua, Planta o Hielo."),
        ],
        ground: [160, 180, 140],
        roof: [60, 140, 90],
        ..GYM
    },
    // ------------------------------------------------------------ Liga Pokémon
    MapDef {
        name: "Liga Pokémon",
        rows: r"
#########
#.......#
#K..U..K#
#.......#
####.####
####*.###
#.......#
#P..x..P#
#.......#
####.####
####*.###
#.......#
#K.....K#
#.......#
####.####
####*.###
#.......#
#K.....K#
#.......#
####.####
####*.###
#.......#
#K..x..K#
#.......#
####.####
####*.###
#.......#
#K.....K#
#...x...#
####0####",
        exits: &[mat()],
        signs: &["SALA DE LA FAMA: aquí se guardan los nombres de los Campeones de la Liga Pokémon."],
        npcs: &[
            boss(CHAMPION, RIVAL, &[(PIDGEY, 61), (RHYHORN, 59), (GROWLITHE, 61), (STARMIE, 61), (DRATINI, 63), (SQUIRTLE, 65)], &["¡Hola! ¡Por fin llegas!", "Mientras tú te paseabas, yo vencí al Alto Mando. ¡Ahora soy el Campeón!", "¡Voy a demostrarte quién es el mejor entrenador del mundo!"], "¡NO! ¡No puede ser! ¡He perdido!", "Eres el nuevo Campeón... Enhorabuena, de verdad.", E),
            boss("Alto Mando Lance", LANCE, &[(DRATINI, 56), (DRATINI, 56), (RHYHORN, 58), (DRATINI, 60)], &["Soy Lance, maestro de los dragones.", "Los dragones son Pokémon legendarios, casi imposibles de vencer. ¡Prepárate!"], "¡Increíble! Me has vencido...", "El Campeón te espera. Alguien a quien conoces...", E),
            boss("Alto Mando Agatha", AGATHA, &[(GASTLY, 54), (KOFFING, 54), (GASTLY, 56), (GRIMER, 56)], &["¿Así que tú eres el niño del que habla Roble?", "Ese viejo ya no sabe lo que es un combate de verdad. ¡Te lo enseñaré yo!"], "Vaya, vaya... Eres bueno, niño.", "Sigue adelante. Lance te espera.", E),
            boss("Alto Mando Bruno", BRUNO, &[(ONIX, 53), (MACHOP, 55), (MACHOP, 55), (ONIX, 56)], &["¡Soy Bruno, del Alto Mando!", "¡Mis Pokémon de lucha y yo hemos entrenado hasta el límite! ¡Hoo hah!"], "¿Por qué...? ¿Cómo he perdido?", "Mi entrenamiento no ha sido suficiente. Adelante.", E),
            boss("Alto Mando Lorelei", LORELEI, &[(SEEL, 54), (STARMIE, 53), (SEEL, 54), (MAGIKARP, 56), (DRATINI, 56)], &["Bienvenido a la Liga Pokémon. Soy Lorelei, del Alto Mando.", "Nadie aguanta mis Pokémon de hielo. ¡Te dejaré congelado!"], "¡Increíble! Eres buenísimo.", "Adelante. El siguiente es Bruno.", E),
        ],
        ground: [210, 200, 230],
        ..GYM
    },
    // ------------------------------------------------------------ Torre Pokémon
    MapDef {
        name: "Torre Pokémon",
        rows: r"
##############
#..T.T.T.T..*#
#............#
#.T.T.*.T.T..#
#............#
#..T.T...T.T.#
#.o..........#
#.T.T..*.T.T.#
#............#
#..T.T...T.T.#
#.....xx.....#
######00######",
        theme: Theme::Tower,
        exits: &[mat()],
        npcs: &[
            trainer("Médium Patricia", CHANNELER, W, 3, &[(GASTLY, 24)], &["¡Uuuuh! ¡Fuera de aquíii!"], "¿Eh? ¿Qué me ha pasado?", "Estaba poseída..."),
            trainer("Médium Hope", CHANNELER, S, 3, &[(GASTLY, 23), (GASTLY, 23)], &["¡Los espíritus te echarán!"], "¡Desaparecieron!", "Que descansen en paz."),
            trainer("Médium Ruth", CHANNELER, S, 2, &[(GASTLY, 25), (KOFFING, 24)], &["¡Vete... vete...!"], "¡Ya me he calmado!", "Gracias."),
        ],
        items: &[(REVIVE, 1)],
        encounters: &[(GASTLY, 90), (ZUBAT, 10)],
        levels: (18, 25),
        ground: [140, 120, 160],
        ..INSIDE
    },
    // ------------------------------------------------------------ Zona Safari
    MapDef {
        name: "Zona Safari",
        rows: r"
####################
#gggg....:....gggg.#
#gggg..o.:....gggg.#
#......:::::.......#
#.~~~~.:...:..ggg..#
#.~~~~.:.*.:..ggg..#
#.~~~~.:...:.......#
#......:...:...r...#
#.ggg..:...:..gggg.#
#.ggg..:...::.gggg.#
#......:....:......#
#..s...:....:..o...#
#gggg..:....:..ggg.#
#gggg..::::::..ggg.#
#.........:........#
#.........x........#
#########00#########",
        theme: Theme::Forest,
        ground: [120, 176, 80],
        exits: &[mat()],
        signs: &["ZONA SAFARI: aquí viven Pokémon que no se ven en ningún otro sitio de Kanto."],
        npcs: &[talk("Guarda", GUARD, W, &["¡Bienvenido a la Zona Safari!", "En la hierba alta hay Rhyhorn, Tangela, Ponyta... ¡y dicen que hasta algún Dratini!", "Con Surf podrás cruzar el estanque."])],
        items: &[(ULTRA_BALL, 2), (RARE_CANDY, 1)],
        encounters: &[(RHYHORN, 20), (TANGELA, 15), (PONYTA, 15), (GROWLITHE, 15), (ODDISH, 15), (CLEFAIRY, 14), (DRATINI, 6)],
        levels: (22, 30),
        ..BASE
    },
    // ------------------------------------------------------------ Silph S.A.
    MapDef {
        name: "Silph S.A.",
        rows: r"
##############
#S..P.*..P..S#
#............#
#.TT..*...TT.#
#.TT......TT.#
#............#
#SS..*....SSS#
#............#
#.K..*...*.K.#
#............#
#P....xx....P#
######00######",
        theme: Theme::Inside,
        ground: [215, 222, 232],
        exits: &[mat()],
        npcs: &[
            Npc {
                name: "Presidente de Silph",
                look: GENTLEMAN,
                facing: S,
                role: Role::Gift {
                    lines: &["¡Gracias por echar al Team Rocket de Silph S.A.!", "Toma, nuestro mejor invento: la Master Ball. Atrapa cualquier Pokémon sin fallar."],
                    item: MASTER_BALL,
                    qty: 1,
                    after: "Úsala con el Pokémon que más desees.",
                },
                aside: None,
            },
            trainer("Ejecutivo Rocket", ROCKET, S, 3, &[(RATTATA, 36), (KOFFING, 37), (ZUBAT, 38), (GRIMER, 38)], &["¡El Team Rocket ha tomado Silph S.A.!", "¡Ni se te ocurra acercarte al presidente!"], "¡Retirada! ¡Retirada!", "El jefe no estará contento..."),
            trainer("Recluta Rocket", ROCKET, E, 4, &[(KOFFING, 32), (ZUBAT, 32)], &["¡Alto! ¡Aquí no se pasa!"], "¡Uf!", "Hay más compañeros arriba."),
            trainer("Recluta Rocket", ROCKET, W, 3, &[(MACHOP, 33), (GRIMER, 33)], &["¡Este edificio es nuestro!"], "¡Imposible!", "..."),
            trainer("Científico", SCIENTIST, W, 3, &[(VOLTORB, 33), (KOFFING, 33), (VOLTORB, 34)], &["¡Me obligaron a trabajar para ellos!... Bueno, y me pagan bien."], "¡Vale, me rindo!", "Las Poké Balls se inventaron aquí, ¿sabías?"),
        ],
        ..INSIDE
    },
    // ------------------------------------------------------------ Casino de Azulona
    MapDef {
        name: "Casino de Azulona",
        rows: r"
##############
#PP.PP.PP.PP.#
#............#
#PP.PP*PP.PP.#
#............#
#PP.PP.PP.PP.#
#.*........*.#
#.....xx.....#
######00######",
        theme: Theme::Inside,
        ground: [190, 80, 90],
        exits: &[mat()],
        npcs: &[
            talk("Jugador", GENTLEMAN, S, &["¡Esta máquina está a punto de dar premio! Lo noto...", "...Llevo así tres días."]),
            trainer("Recluta Rocket", ROCKET, E, 9, &[(RATTATA, 29), (KOFFING, 29), (ZUBAT, 30)], &["¡Eh! ¿Qué haces mirando el póster de la pared?", "¡Lárgate o te enseño lo que es bueno!"], "¡Me has descubierto!", "No le digas a nadie que hay un escondite aquí..."),
            talk("Empleada", GIRL, W, &["¡Bienvenido al Casino de Azulona!", "Las máquinas están en mantenimiento, pero puedes mirar."]),
        ],
        ..INSIDE
    },
    // ------------------------------------------------------------ Mansión Pokémon
    MapDef {
        name: "Mansión Pokémon",
        rows: r"
################
#S..S.S...o..S.#
#..............#
#.##...*...###.#
#.##.......###.#
#.......r......#
#r.r..r...*..r.#
#..............#
#.###.....###..#
#..*...........#
#......xx......#
#######00#######",
        theme: Theme::Tower,
        ground: [150, 128, 105],
        exits: &[mat()],
        npcs: &[
            trainer("Científico Ted", SCIENTIST, S, 3, &[(GRIMER, 34), (KOFFING, 34), (VOLTORB, 35)], &["¡Aquí se hacían experimentos con Pokémon!"], "¡Mis notas!", "En el diario hablan de un Pokémon nuevo... muy poderoso."),
            trainer("Ladrón Simón", BIKER, W, 4, &[(GROWLITHE, 34), (PONYTA, 34)], &["¡Esta mansión está llena de tesoros!"], "¡Me pillaste!", "Me llevo solo un recuerdo..."),
            trainer("Científico Iván", SCIENTIST, E, 5, &[(VOLTORB, 36), (GRIMER, 36)], &["¡No toques nada!"], "¡Cuidado con los experimentos!", "Aquí pasaron cosas raras."),
        ],
        items: &[(RARE_CANDY, 1)],
        encounters: &[(GRIMER, 30), (KOFFING, 30), (GROWLITHE, 15), (PONYTA, 15), (RATTATA, 10)],
        levels: (30, 36),
        ..INSIDE
    },
    // ------------------------------------------------------------ Dojo Kárate
    MapDef {
        name: "Dojo Kárate",
        rows: r"
############
#U...*....U#
#..........#
#.*......*.#
#..........#
#.*......*.#
#..........#
#....xx....#
#####00#####",
        theme: Theme::Inside,
        ground: [215, 185, 135],
        exits: &[mat()],
        signs: &["DOJO KÁRATE: ¡el auténtico gimnasio de lucha de Azafrán!", "Lema del Dojo: ¡el cuerpo fuerte forja una mente fuerte!"],
        npcs: &[
            trainer("Maestro Kárate", BRUNO, S, 0, &[(MACHOP, 37), (MACHOP, 39), (ONIX, 37)], &["¡Jo! ¿Vienes a retar al Dojo Kárate?", "¡Mis puños son de acero!"], "¡Uooh! ¡Qué derrota tan honrosa!", "Entrena cada día y serás invencible."),
            trainer("Karateka Hideki", HIKER, E, 6, &[(MACHOP, 31), (MACHOP, 31)], &["¡Hiyaa!"], "¡Ay!", "El maestro es aún más fuerte."),
            trainer("Karateka Aaron", HIKER, W, 6, &[(MACHOP, 32)], &["¡Mis puños echan humo!"], "¡Uf!", "Sigue entrenando."),
            trainer("Karateka Mike", HIKER, E, 6, &[(MACHOP, 32), (GEODUDE, 31)], &["¡Hoo-hah!"], "¡Me rindo!", "..."),
            trainer("Karateka Hitoshi", HIKER, W, 6, &[(MACHOP, 33)], &["¡A por ti!"], "¡Increíble!", "El maestro te espera."),
        ],
        ..INSIDE
    },
    // ------------------------------------------------------------ Museo de Ciencias
    MapDef {
        name: "Museo de Ciencias",
        rows: r"
##############
#U..U..U..U..#
#............#
#..T.T..*....#
#............#
#S..........S#
#C..*.......K#
#.....xx.....#
######00######",
        theme: Theme::Inside,
        ground: [225, 215, 195],
        exits: &[mat()],
        signs: &[
            "Fósil Hélix: un Pokémon prehistórico con concha en espiral.",
            "Fósil Domo: el caparazón de un Pokémon que vivió en el mar hace millones de años.",
            "Ámbar Viejo: en su interior hay algo... ¿será ADN de un Pokémon volador?",
            "Transbordador espacial: así viajan los científicos más allá de la luna.",
        ],
        npcs: &[
            talk("Guía", SCIENTIST, W, &["Los fósiles de Mt. Moon son de Pokémon que vivieron hace muchísimo tiempo.", "En Isla Canela investigan cómo devolverlos a la vida."]),
            talk("Recepcionista", GIRL, E, &["¡Bienvenido al Museo de Ciencias de Ciudad Plateada!", "Hoy la entrada es gratis."]),
        ],
        ..INSIDE
    },
];

const GYM_SIGN_1: &str = "GIMNASIO DE CIUDAD PLATEADA. Líder: Brock, el entrenador de roca dura.";
const GYM_SIGN_2: &str = "GIMNASIO DE CIUDAD CELESTE. Líder: Misty, la sirena de Celeste.";
const GYM_SIGN_3: &str = "GIMNASIO DE CIUDAD CARMÍN. Líder: Teniente Surge, el americano relámpago.";
const GYM_SIGN_4: &str = "GIMNASIO DE CIUDAD AZULONA. Líder: Erika, la princesa de la naturaleza.";
const GYM_SIGN_5: &str = "GIMNASIO DE CIUDAD FUCSIA. Líder: Koga, el maestro ninja del veneno.";
const GYM_SIGN_6: &str = "GIMNASIO DE CIUDAD AZAFRÁN. Líder: Sabrina, la maestra psíquica.";
const GYM_SIGN_7: &str = "GIMNASIO DE ISLA CANELA. Líder: Blaine, el exaltado hombre de fuego.";
const GYM_SIGN_8: &str = "GIMNASIO DE CIUDAD VERDE. Líder: ...¿quién será?";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_stand_on_walkable_tiles_and_trainers_have_teams() {
        for (id, def) in MAPS.iter().enumerate() {
            let map = LoadedMap::load(id);
            for (i, npc) in def.npcs.iter().enumerate() {
                let pos = map.npcs[i];
                assert!(map.get(pos).walkable(), "{}: {} está sobre un obstáculo", def.name, npc.name);
                if let Some(aside) = npc.aside {
                    assert!(map.get(pos + aside).walkable(), "{}: {} no tiene dónde apartarse", def.name, npc.name);
                }
                if let Some(t) = npc.trainer() {
                    assert!(!t.team.is_empty() && t.team.len() <= MAX_PARTY, "{}: {} equipo inválido", def.name, npc.name);
                }
            }
        }
    }

    #[test]
    fn the_player_starts_inside_their_house() {
        let house = LoadedMap::load(PLAYER_HOUSE);
        assert!(house.get(house_start()).walkable());
        let home = home_door();
        assert!(LoadedMap::load(home.map).get(home.pos).walkable());
    }

    /// Con cada medalla se abre justo lo necesario para llegar al siguiente gimnasio.
    #[test]
    fn badges_open_kanto_in_order() {
        let gyms = [GYM_PEWTER, GYM_CERULEAN, GYM_VERMILION, GYM_CELADON, GYM_FUCHSIA, GYM_SAFFRON, GYM_CINNABAR, GYM_VIRIDIAN, LEAGUE];
        for (badges, gym) in gyms.into_iter().enumerate() {
            let mut seen = vec![false; MAPS.len()];
            let mut stack = vec![PALLET];
            while let Some(id) = stack.pop() {
                if required_badges(id) as usize > badges || std::mem::replace(&mut seen[id], true) {
                    continue;
                }
                let map = LoadedMap::load(id);
                stack.extend(MAPS[id].exits.iter().filter_map(|e| match e.to {
                    Link::Map(to, _) => Some(to),
                    Link::Back => None,
                }));
                for b in &map.buildings {
                    if let Door::Map(to) | Door::Badges(_, to, _) = map.door(b) {
                        stack.push(to);
                    }
                }
            }
            assert!(seen[gym], "con {badges} medallas no se llega a {}", MAPS[gym].name);
            if let Some(next) = gyms.get(badges + 1) {
                assert!(!seen[*next], "con {badges} medallas ya se puede entrar en {}", MAPS[*next].name);
            }
        }
    }

    #[test]
    fn every_map_is_reachable_from_pallet_town() {
        let mut seen = vec![false; MAPS.len()];
        let mut stack = vec![PALLET];
        while let Some(id) = stack.pop() {
            if std::mem::replace(&mut seen[id], true) {
                continue;
            }
            let map = LoadedMap::load(id);
            for exit in MAPS[id].exits {
                if let Link::Map(to, _) = exit.to {
                    stack.push(to);
                }
            }
            for b in &map.buildings {
                if let Door::Map(to) | Door::Badges(_, to, _) = map.door(b) {
                    stack.push(to);
                }
            }
        }
        for (id, def) in MAPS.iter().enumerate() {
            assert!(seen[id], "no se puede llegar a {}", def.name);
        }
    }
}
