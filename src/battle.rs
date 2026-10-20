//! Combates: contra Pokémon salvajes y contra entrenadores (con su equipo, su dinero y, si
//! es líder de gimnasio, su medalla). Menú Luchar / Mochila / Pokémon / Huir.

use std::collections::VecDeque;

use bevy::prelude::*;
use rand::Rng;

use crate::AppState;
use crate::render::camera::{CameraRig, MainCamera};
use crate::pokemon::{Creature, MAX_PARTY, Move, Party, effectiveness_text, species_catch};
use crate::game::items::{Bag, Effect, ITEMS};
use crate::world::kanto::CHAMPION;
use crate::render::models::{Kit, NPC_OUTFITS, Shape, part, tree};
use crate::world::npcs::{BADGES, npc};
use crate::world::overworld::{CurrentMap, PlayerPos, enter_map};
use crate::game::progress::Progress;
use crate::ui;
use crate::world::Theme;

const MAIN_MENU: [&str; 4] = ["Luchar", "Mochila", "Pokémon", "Huir"];
/// Filas visibles de las listas (mochila y equipo).
const LIST_ROWS: usize = 6;

/// Contra quién es el próximo combate.
#[derive(Resource, Clone)]
pub enum PendingBattle {
    Wild(Creature),
    /// Entrenador (`npcs::npc_id`).
    Trainer(usize),
}

/// Qué pasa cuando se acaban los mensajes pendientes.
#[derive(Clone, Copy, PartialEq)]
enum Then {
    Choose,
    FoeTurn,
    /// El entrenador saca a su siguiente Pokémon.
    NextFoe,
    /// Se ha debilitado tu Pokémon: elige a otro.
    MustSwitch,
    End,
    Blackout,
}

#[derive(Clone, Copy, PartialEq)]
enum Menu {
    Main,
    Moves,
    Bag,
    Party,
}

#[derive(Resource)]
struct Battle {
    /// El equipo del rival (un solo Pokémon si es salvaje).
    foes: Vec<Creature>,
    foe: usize,
    trainer: Option<usize>,
    /// Índice en el equipo del Pokémon que está luchando.
    active: usize,
    queue: VecDeque<String>,
    then: Then,
    menu: Menu,
    cursor: usize,
    /// El rival se ha debilitado o ha sido capturado: deja de verse.
    foe_gone: bool,
}

impl Battle {
    fn say(&mut self, msg: impl Into<String>) {
        self.queue.push_back(msg.into());
    }

    fn foe(&self) -> &Creature {
        &self.foes[self.foe]
    }

    fn foe_mut(&mut self) -> &mut Creature {
        &mut self.foes[self.foe]
    }

    /// "Rattata salvaje" o "Onix enemigo".
    fn foe_name(&self) -> String {
        let kind = if self.trainer.is_some() { "enemigo" } else { "salvaje" };
        format!("{} {kind}", self.foe().name())
    }
}

#[derive(Component, Clone, Copy)]
enum BattleLabel {
    FoeName,
    FoeType(usize),
    FoeTrainer,
    PlayerName,
    PlayerType(usize),
    PlayerHp,
    Message,
    Option(usize),
    /// Etiqueta de tipo dentro de cada botón de ataque.
    OptionType(usize),
    MoveType,
    MovePower,
    MoveEffect,
}

/// Ficha del ataque seleccionado (solo visible en el menú de ataques).
#[derive(Component)]
struct MoveInfo;

/// Fila de Poké Balls con el equipo del entrenador rival.
#[derive(Component)]
struct FoeTeam;

/// Lista de la mochila o del equipo.
#[derive(Component)]
struct ListPanel;

#[derive(Component)]
enum Bar {
    FoeHp,
    PlayerHp,
    PlayerXp,
    MovePower,
}

/// Dónde está la arena de combate (lejos del mapa).
const ARENA: Vec3 = Vec3::new(200.0, 0.0, 0.0);
const PLAYER_SPOT: Vec3 = Vec3::new(-1.9, 0.0, 1.4);
const FOE_SPOT: Vec3 = Vec3::new(2.1, 0.0, -2.2);

#[derive(Component, Clone, Copy, PartialEq)]
enum Side {
    Player,
    Foe,
}

/// Modelo 3D de una criatura en combate.
#[derive(Component)]
struct BattleModel {
    side: Side,
    species: usize,
    last_hp: i32,
    /// Segundos que le quedan temblando tras recibir un golpe.
    hurt: f32,
    /// Segundos desde que salió (crece desde pequeño).
    age: f32,
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Battle), start_battle).add_systems(
            Update,
            (battle_input, refresh_ui, sync_list, sync_foe_team, sync_models, animate_models, battle_camera)
                .chain()
                .run_if(in_state(AppState::Battle)),
        );
    }
}

// ---------------------------------------------------------------- lógica

fn start_battle(
    mut commands: Commands,
    mut kit: Kit,
    party: Res<Party>,
    map: Res<CurrentMap>,
    pending: Option<Res<PendingBattle>>,
    mut progress: ResMut<Progress>,
) {
    let def = map.0.def();
    let (foes, trainer) = match pending.as_deref().cloned() {
        Some(PendingBattle::Trainer(id)) => {
            let t = npc(id).trainer().expect("es un entrenador");
            (t.team.iter().map(|&(s, l)| Creature::new(s, l)).collect(), Some(id))
        }
        Some(PendingBattle::Wild(c)) => (vec![c], None),
        None => (vec![crate::pokemon::random_wild(def.encounters, def.levels.max((2, 4)))], None),
    };
    commands.remove_resource::<PendingBattle>();
    let active = party.first_alive().unwrap_or(0);
    spawn_arena(&mut commands, &mut kit, def.theme, trainer);

    let mut battle = Battle { foes, foe: 0, trainer, active, queue: VecDeque::new(), then: Then::Choose, menu: Menu::Main, cursor: 0, foe_gone: false };
    progress.see(battle.foe().species);
    match trainer {
        Some(id) => {
            battle.say(format!("¡{} quiere luchar!", npc(id).name));
            battle.say(format!("{} saca a {} (Nv. {}).", npc(id).name, battle.foe().name(), battle.foe().level));
        }
        None => battle.say(format!("¡Un {} de nivel {} apareció!", battle.foe_name(), battle.foe().level)),
    }
    battle.say(format!("¡Adelante, {}!", party.0[active].name()));
    commands.insert_resource(battle);
    spawn_ui(&mut commands);
}

/// Los objetos de la mochila que se pueden usar en combate.
fn battle_items(bag: &Bag) -> Vec<usize> {
    (0..ITEMS.len()).filter(|&i| bag.count(i) > 0 && ITEMS[i].usable_in_battle()).collect()
}

#[allow(clippy::too_many_arguments)]
fn battle_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut battle: ResMut<Battle>,
    mut party: ResMut<Party>,
    mut bag: ResMut<Bag>,
    mut progress: ResMut<Progress>,
    mut player_pos: ResMut<PlayerPos>,
    mut map: ResMut<CurrentMap>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let confirm = input.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::KeyZ]);
    let back = input.any_just_pressed([KeyCode::KeyX, KeyCode::Escape, KeyCode::Backspace]);
    let b = &mut *battle;

    // Mostrando mensajes: cada pulsación pasa al siguiente
    if !b.queue.is_empty() {
        if !confirm {
            return;
        }
        b.queue.pop_front();
        if b.queue.is_empty() {
            match b.then {
                Then::Choose => {}
                Then::FoeTurn => foe_turn(b, &mut party),
                Then::NextFoe => {
                    b.foe += 1;
                    b.foe_gone = false;
                    progress.see(b.foe().species);
                    let name = npc(b.trainer.unwrap()).name;
                    b.say(format!("{name} saca a {} (Nv. {}).", b.foe().name(), b.foe().level));
                    b.then = Then::Choose;
                }
                Then::MustSwitch => {
                    b.menu = Menu::Party;
                    b.cursor = party.first_alive().unwrap_or(0);
                }
                Then::End => next_state.set(AppState::Overworld),
                Then::Blackout => {
                    party.heal_all();
                    if b.trainer.is_some() {
                        progress.money /= 2;
                    }
                    // Vuelves al último Centro Pokémon
                    let place = progress.last_center;
                    let (m, actors) = enter_map(place.map, &progress);
                    map.0 = m;
                    commands.insert_resource(actors);
                    player_pos.0 = place.pos;
                    next_state.set(AppState::Overworld);
                }
            }
        }
        return;
    }

    let up = input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]);
    let down = input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]);
    match b.menu {
        Menu::Main | Menu::Moves => {
            // Menú 2x2: izquierda/derecha cambian columna, arriba/abajo cambian fila
            let options = if b.menu == Menu::Main { MAIN_MENU.len() } else { party.0[b.active].moves().len() };
            let mut next = b.cursor;
            if input.any_just_pressed([KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::KeyA, KeyCode::KeyD]) {
                next ^= 1;
            }
            if up || down {
                next ^= 2;
            }
            if next < options {
                b.cursor = next;
            }
        }
        Menu::Bag | Menu::Party => {
            let len = if b.menu == Menu::Bag { battle_items(&bag).len() } else { party.0.len() };
            if len > 0 {
                if down {
                    b.cursor = (b.cursor + 1) % len;
                }
                if up {
                    b.cursor = (b.cursor + len - 1) % len;
                }
            }
        }
    }

    match b.menu {
        Menu::Moves if back => go(b, Menu::Main, 0),
        Menu::Moves if confirm => {
            let mv = party.0[b.active].moves()[b.cursor];
            go(b, Menu::Main, 0);
            player_attack(b, &mut party, &mut progress, mv);
        }
        Menu::Bag if back => go(b, Menu::Main, 1),
        Menu::Bag if confirm => {
            if let Some(&item) = battle_items(&bag).get(b.cursor) {
                go(b, Menu::Main, 1);
                use_item(b, &mut party, &mut bag, &mut progress, item);
            }
        }
        // Si tu Pokémon se ha debilitado no se puede volver atrás: hay que elegir a otro
        Menu::Party if back && !party.0[b.active].fainted() => go(b, Menu::Main, 2),
        Menu::Party if confirm => {
            let i = b.cursor;
            let forced = party.0[b.active].fainted();
            if party.0[i].fainted() {
                b.say(format!("¡{} no puede luchar!", party.0[i].name()));
                b.then = if forced { Then::MustSwitch } else { Then::Choose };
            } else if i == b.active {
                b.say(format!("¡{} ya está luchando!", party.0[i].name()));
                b.then = Then::Choose;
            } else {
                if !forced {
                    b.say(format!("¡Vuelve, {}!", party.0[b.active].name()));
                }
                b.say(format!("¡Adelante, {}!", party.0[i].name()));
                b.active = i;
                b.then = if forced { Then::Choose } else { Then::FoeTurn };
            }
            go(b, Menu::Main, 0);
        }
        Menu::Main if confirm => match b.cursor {
            0 => go(b, Menu::Moves, 0),
            1 => go(b, Menu::Bag, 0),
            2 => go(b, Menu::Party, b.active),
            _ => run_away(b),
        },
        _ => {}
    }
}

fn go(b: &mut Battle, menu: Menu, cursor: usize) {
    b.menu = menu;
    b.cursor = cursor;
}

fn player_attack(b: &mut Battle, party: &mut Party, progress: &mut Progress, mv: Move) {
    let me = &mut party.0[b.active];
    let (dmg, eff) = me.damage_to(&mv, &b.foes[b.foe]);
    let me_name = me.name();
    let foe = b.foe_mut();
    foe.hp = (foe.hp - dmg).max(0);
    b.say(format!("¡{} usó {}!", me_name, mv.name));
    if let Some(t) = effectiveness_text(eff) {
        b.say(t);
    }
    if b.foe().fainted() {
        foe_fainted(b, party, progress);
    } else {
        b.then = Then::FoeTurn;
    }
}

/// El Pokémon rival se ha debilitado: experiencia y, si era el último, fin del combate.
fn foe_fainted(b: &mut Battle, party: &mut Party, progress: &mut Progress) {
    b.say(format!("¡{} se debilitó!", b.foe_name()));
    b.foe_gone = true;
    let bonus = if b.trainer.is_some() { 1.5 } else { 1.0 };
    let xp = (b.foe().xp_yield() as f32 * bonus) as u32;
    for msg in party.0[b.active].gain_xp(xp) {
        b.say(msg);
    }
    let Some(id) = b.trainer else {
        b.then = Then::End;
        return;
    };
    if b.foe + 1 < b.foes.len() {
        b.then = Then::NextFoe;
        return;
    }
    // Victoria contra un entrenador
    let person = npc(id);
    let t = person.trainer().unwrap();
    progress.done.set(id);
    b.say(format!("¡Has vencido a {}!", person.name));
    b.say(format!("{}: {}", person.name, t.lose));
    let top = b.foes.iter().map(|c| c.level).max().unwrap_or(1);
    let money = t.pay * top;
    progress.earn(money);
    b.say(format!("¡Has ganado {money} ₽!"));
    if let Some(badge) = t.badge {
        progress.badges |= 1 << badge;
        b.say(format!("¡Has conseguido la {}!", BADGES[badge as usize].name));
    }
    if person.name == CHAMPION {
        progress.champion = true;
        b.say("¡Enhorabuena! ¡Eres el nuevo Campeón de la Liga Pokémon!");
        b.say(format!("Tu nombre, {}, quedará para siempre en la Sala de la Fama.", progress.name));
    }
    b.then = Then::End;
}

fn foe_turn(b: &mut Battle, party: &mut Party) {
    let me = &party.0[b.active];
    let mv = if b.trainer.is_some() {
        b.foe().best_move(me)
    } else {
        let moves = b.foe().moves();
        moves[rand::rng().random_range(0..moves.len())]
    };
    let (dmg, eff) = b.foe().damage_to(&mv, me);
    let me = &mut party.0[b.active];
    me.hp = (me.hp - dmg).max(0);
    let me_name = me.name();
    let fainted = me.fainted();

    b.say(format!("¡{} usó {}!", b.foe_name(), mv.name));
    if let Some(t) = effectiveness_text(eff) {
        b.say(t);
    }
    b.then = Then::Choose;

    if fainted {
        b.say(format!("¡{me_name} se debilitó!"));
        if party.first_alive().is_some() {
            b.say("¿A quién sacarás ahora?");
            b.then = Then::MustSwitch;
        } else {
            b.say("No te quedan Pokémon en pie...");
            if b.trainer.is_some() {
                b.say("Pagas la mitad de tu dinero al rival.");
            }
            b.say("¡Corres al último Centro Pokémon a curar a tu equipo!");
            b.then = Then::Blackout;
        }
    }
}

fn use_item(b: &mut Battle, party: &mut Party, bag: &mut Bag, progress: &mut Progress, item: usize) {
    let it = &ITEMS[item];
    match it.effect {
        Effect::Ball(power) => {
            if b.trainer.is_some() {
                b.say("¡El entrenador desvía la Poké Ball! ¡No se pueden robar Pokémon ajenos!");
                b.then = Then::Choose;
                return;
            }
            bag.take(item);
            b.say(format!("¡Lanzas una {}!", it.name));
            let foe = b.foe();
            let hp_left = foe.hp as f32 / foe.max_hp() as f32;
            let chance = (0.25 + 0.6 * (1.0 - hp_left)) * power * species_catch(foe.species);
            if rand::rng().random_bool(chance.clamp(0.0, 1.0) as f64) {
                let caught = foe.clone();
                b.say(format!("¡Ya está! ¡Has atrapado a {}!", caught.name()));
                progress.catch(caught.species);
                if party.0.len() < MAX_PARTY {
                    b.say(format!("{} se une a tu equipo.", caught.name()));
                    party.0.push(caught);
                } else {
                    b.say(format!("Tu equipo está lleno: {} se ha enviado al PC.", caught.name()));
                    progress.boxed.push(caught);
                }
                b.foe_gone = true;
                b.then = Then::End;
            } else {
                let shake = ["¡Oh, no! ¡Se ha escapado!", "¡Vaya! ¡Parecía que lo tenías!", "¡Casi! ¡Por muy poco!"];
                b.say(shake[rand::rng().random_range(0..shake.len())]);
                b.then = Then::FoeTurn;
            }
        }
        Effect::Heal(amount) => {
            let me = &mut party.0[b.active];
            if me.fainted() || me.hp >= me.max_hp() {
                b.say("No tendrá ningún efecto.");
                b.then = Then::Choose;
                return;
            }
            bag.take(item);
            let before = me.hp;
            me.hp = me.hp.saturating_add(amount).min(me.max_hp());
            b.say(format!("Usas {}. {} recupera {} PS.", it.name, me.name(), me.hp - before));
            b.then = Then::FoeTurn;
        }
        Effect::Revive => {
            let Some(target) = party.0.iter().position(Creature::fainted) else {
                b.say("No tendrá ningún efecto.");
                b.then = Then::Choose;
                return;
            };
            bag.take(item);
            let c = &mut party.0[target];
            c.hp = c.max_hp() / 2;
            b.say(format!("¡{} se ha reanimado!", c.name()));
            b.then = Then::FoeTurn;
        }
        _ => {
            b.say("Ahora no puedes usar eso.");
            b.then = Then::Choose;
        }
    }
}

fn run_away(b: &mut Battle) {
    if b.trainer.is_some() {
        b.say("¡No puedes huir de un combate contra un entrenador!");
        b.then = Then::Choose;
    } else if rand::rng().random_bool(0.75) {
        b.say("¡Escapaste sin problemas!");
        b.then = Then::End;
    } else {
        b.say("¡No puedes escapar!");
        b.then = Then::FoeTurn;
    }
}

// ---------------------------------------------------------------- interfaz

const GREYED: Color = Color::srgb(0.68, 0.69, 0.73);

fn label(label: BattleLabel, size: f32, weight: FontWeight) -> impl Bundle {
    (label, Text::new(""), TextFont { font_size: FontSize::Px(size), weight, ..default() }, TextColor(ui::INK))
}

fn type_tag(label: BattleLabel) -> impl Bundle {
    (
        label,
        Text::new(""),
        TextFont { font_size: FontSize::Px(14.0), weight: FontWeight::BOLD, ..default() },
        TextColor(Color::WHITE),
        Node {
            padding: UiRect::axes(px(8), px(2)),
            align_self: AlignSelf::FlexStart,
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    )
}

fn bar(kind: Bar, height: f32, color: Color) -> impl Bundle {
    ui::bar(kind, 1.0, height, color)
}

fn info_panel() -> impl Bundle {
    ui::panel(Node { width: px(340), flex_direction: FlexDirection::Column, row_gap: px(7), padding: UiRect::all(px(16)), ..default() })
}

/// Hueco del menú: un botón redondeado que se resalta al seleccionarlo.
#[derive(Component)]
struct OptionBox(usize);

fn option_slot(i: usize) -> impl Bundle {
    (
        OptionBox(i),
        Node {
            width: percent(48),
            align_items: AlignItems::Center,
            column_gap: px(8),
            padding: UiRect::axes(px(12), px(8)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        children![type_tag(BattleLabel::OptionType(i)), label(BattleLabel::Option(i), 22.0, FontWeight::BOLD)],
    )
}

fn move_info() -> impl Bundle {
    (
        MoveInfo,
        Node { display: Display::None, flex_grow: 1.0, flex_basis: px(0), flex_direction: FlexDirection::Column, row_gap: px(10), ..default() },
        children![
            (
                Node { align_items: AlignItems::Center, column_gap: px(10), ..default() },
                children![type_tag(BattleLabel::MoveType), label(BattleLabel::MoveEffect, 18.0, FontWeight::BOLD)],
            ),
            (
                Node { align_items: AlignItems::Center, column_gap: px(10), ..default() },
                children![
                    ui::muted("Potencia", 16.0),
                    (Node { width: px(150), ..default() }, children![bar(Bar::MovePower, 8.0, ui::ACCENT)]),
                    label(BattleLabel::MovePower, 18.0, FontWeight::BOLD),
                ],
            ),
            ui::muted("Z: usar   ·   X: volver", 15.0),
        ],
    )
}

fn types_row(first: BattleLabel, second: BattleLabel) -> impl Bundle {
    (Node { column_gap: px(6), ..default() }, children![type_tag(first), type_tag(second)])
}

fn spawn_ui(commands: &mut Commands) {
    commands.spawn((
        DespawnOnExit(AppState::Battle),
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(24)),
            row_gap: px(12),
            ..default()
        },
        children![
            // Rival: arriba a la izquierda
            (
                info_panel(),
                children![
                    (
                        Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() },
                        children![label(BattleLabel::FoeName, 24.0, FontWeight::BOLD), (FoeTeam, Node { column_gap: px(3), ..default() })],
                    ),
                    label(BattleLabel::FoeTrainer, 15.0, FontWeight::NORMAL),
                    types_row(BattleLabel::FoeType(0), BattleLabel::FoeType(1)),
                    bar(Bar::FoeHp, 10.0, Color::NONE),
                ],
            ),
            // Tu Pokémon: datos abajo a la derecha, sobre la caja de mensajes
            (
                Node { flex_grow: 1.0, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::FlexEnd, ..default() },
                children![
                    (ListPanel, Node { flex_direction: FlexDirection::Column, ..default() }),
                    (
                        info_panel(),
                        children![
                            label(BattleLabel::PlayerName, 24.0, FontWeight::BOLD),
                            types_row(BattleLabel::PlayerType(0), BattleLabel::PlayerType(1)),
                            bar(Bar::PlayerHp, 10.0, Color::NONE),
                            label(BattleLabel::PlayerHp, 18.0, FontWeight::NORMAL),
                            bar(Bar::PlayerXp, 5.0, ui::XP),
                        ],
                    ),
                ],
            ),
            // Caja de mensajes + menú
            (
                ui::panel(Node { height: px(160), padding: UiRect::all(px(20)), column_gap: px(20), ..default() }),
                children![
                    (label(BattleLabel::Message, 24.0, FontWeight::NORMAL), Node { flex_grow: 1.0, flex_basis: px(0), ..default() }),
                    move_info(),
                    (
                        Node {
                            width: px(440),
                            flex_wrap: FlexWrap::Wrap,
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            row_gap: px(10),
                            ..default()
                        },
                        children![option_slot(0), option_slot(1), option_slot(2), option_slot(3)],
                    ),
                ],
            ),
        ],
    ));
}

#[allow(clippy::type_complexity)]
fn refresh_ui(
    battle: Res<Battle>,
    party: Res<Party>,
    mut labels: Query<(&BattleLabel, &mut Text, &mut TextColor, Option<&mut BackgroundColor>, Option<&mut Node>)>,
    mut bars: Query<(&Bar, &mut Node, &mut BackgroundColor), (Without<BattleLabel>, Without<OptionBox>)>,
    mut boxes: Query<(&OptionBox, &mut BackgroundColor, &mut BorderColor), (Without<BattleLabel>, Without<Bar>)>,
    mut info: Single<&mut Node, (With<MoveInfo>, Without<BattleLabel>, Without<Bar>)>,
) {
    let foe = battle.foe();
    let me = &party.0[battle.active];
    let moves = me.moves();
    let waiting = !battle.queue.is_empty();
    let choosing_move = !waiting && battle.menu == Menu::Moves;
    let current = moves.get(battle.cursor).copied().filter(|_| choosing_move);
    info.display = if choosing_move { Display::Flex } else { Display::None };

    for (label, mut text, mut color, bg, node) in &mut labels {
        match *label {
            BattleLabel::FoeName => text.0 = format!("{}   Nv.{}", foe.name(), foe.level),
            BattleLabel::FoeTrainer => {
                text.0 = battle.trainer.map(|id| npc(id).name.to_string()).unwrap_or_else(|| "Pokémon salvaje".into());
                color.0 = ui::MUTED;
            }
            BattleLabel::PlayerName => text.0 = format!("{}   Nv.{}", me.name(), me.level),
            BattleLabel::PlayerHp => text.0 = format!("PS {}/{}", me.hp, me.max_hp()),
            BattleLabel::FoeType(i) | BattleLabel::PlayerType(i) => {
                let who = if matches!(label, BattleLabel::FoeType(_)) { foe } else { me };
                let ty = who.spec().types().nth(i);
                if let (Some(ty), Some(mut bg)) = (ty, bg) {
                    text.0 = ty.name().to_uppercase();
                    bg.0 = ty.color();
                }
                if let Some(mut node) = node {
                    node.display = if ty.is_some() { Display::Flex } else { Display::None };
                }
            }
            BattleLabel::Message => {
                text.0 = if let Some(msg) = battle.queue.front() {
                    format!("{msg}  ▼")
                } else {
                    match battle.menu {
                        Menu::Moves => String::new(),
                        Menu::Bag => "¿Qué objeto quieres usar?".into(),
                        Menu::Party if me.fainted() => "¿A quién vas a sacar?".into(),
                        Menu::Party => "¿A quién quieres cambiar?".into(),
                        Menu::Main => format!("¿Qué hará {}?", me.name()),
                    }
                };
                if let Some(mut node) = node {
                    node.display = if choosing_move { Display::None } else { Display::Flex };
                }
            }
            BattleLabel::Option(i) => {
                text.0 = match battle.menu {
                    Menu::Moves => moves.get(i).map(|m| m.name.to_string()).unwrap_or_default(),
                    _ => MAIN_MENU[i].to_string(),
                };
                let disabled = i == 3 && battle.trainer.is_some() && battle.menu == Menu::Main;
                color.0 = if waiting || matches!(battle.menu, Menu::Bag | Menu::Party) || disabled { GREYED } else { ui::INK };
            }
            BattleLabel::OptionType(i) => {
                let mv = moves.get(i).filter(|_| battle.menu == Menu::Moves);
                if let (Some(mv), Some(mut bg)) = (mv, bg) {
                    text.0 = mv.ty.name().to_uppercase();
                    bg.0 = if waiting { GREYED } else { mv.ty.color() };
                }
                if let Some(mut node) = node {
                    node.display = if mv.is_some() { Display::Flex } else { Display::None };
                }
            }
            BattleLabel::MoveType => {
                if let (Some(mv), Some(mut bg)) = (current, bg) {
                    text.0 = mv.ty.name().to_uppercase();
                    bg.0 = mv.ty.color();
                }
            }
            BattleLabel::MovePower => text.0 = current.map(|m| m.power.to_string()).unwrap_or_default(),
            BattleLabel::MoveEffect => {
                let Some(mv) = current else { continue };
                let eff = foe.weakness(mv.ty);
                let (msg, c) = if eff == 0.0 {
                    (format!("No afecta a {}", foe.name()), Color::srgb(0.5, 0.5, 0.55))
                } else if eff > 1.0 {
                    (format!("Muy eficaz contra {}", foe.name()), Color::srgb(0.15, 0.6, 0.25))
                } else if eff < 1.0 {
                    (format!("Poco eficaz contra {}", foe.name()), Color::srgb(0.8, 0.25, 0.2))
                } else {
                    (format!("Eficacia normal contra {}", foe.name()), ui::MUTED)
                };
                text.0 = msg;
                color.0 = c;
            }
        }
    }

    for (slot, mut bg, mut border) in &mut boxes {
        let in_grid = matches!(battle.menu, Menu::Main | Menu::Moves);
        let selected = !waiting && in_grid && battle.cursor == slot.0;
        let exists = battle.menu != Menu::Moves || slot.0 < moves.len();
        let (fill, edge) = match (selected && exists, battle.menu) {
            // Seleccionado: borde del color del tipo y fondo de ese color muy suave
            (true, Menu::Moves) => {
                let ty = moves[slot.0].ty.color();
                (ty.mix(&Color::WHITE, 0.82), ty)
            }
            (true, _) => (Color::srgb(0.9, 0.94, 1.0), ui::ACCENT),
            (false, _) if exists && !waiting && in_grid => (Color::NONE, ui::TRACK),
            _ => (Color::NONE, Color::NONE),
        };
        bg.0 = fill;
        *border = BorderColor::all(edge);
    }

    for (bar, mut node, mut bg) in &mut bars {
        let fraction = match bar {
            Bar::FoeHp => foe.hp as f32 / foe.max_hp() as f32,
            Bar::PlayerHp => me.hp as f32 / me.max_hp() as f32,
            Bar::PlayerXp => me.xp as f32 / me.xp_to_next() as f32,
            Bar::MovePower => current.map_or(0.0, |m| m.power as f32 / 120.0),
        };
        node.width = percent(fraction.clamp(0.0, 1.0) * 100.0);
        match bar {
            Bar::FoeHp | Bar::PlayerHp => bg.0 = ui::hp_color(fraction),
            Bar::MovePower => bg.0 = current.map_or(ui::ACCENT, |m| m.ty.color()),
            Bar::PlayerXp => {}
        }
    }
}

/// La lista de la mochila o del equipo (se rehace solo si cambia algo).
fn sync_list(
    mut commands: Commands,
    battle: Res<Battle>,
    party: Res<Party>,
    bag: Res<Bag>,
    panel: Single<Entity, With<ListPanel>>,
    mut last: Local<Option<(u8, usize, Vec<(usize, i32)>)>>,
) {
    let open = battle.queue.is_empty() && matches!(battle.menu, Menu::Bag | Menu::Party);
    let rows: Vec<(usize, i32)> = match battle.menu {
        Menu::Bag => battle_items(&bag).into_iter().map(|i| (i, bag.count(i) as i32)).collect(),
        _ => party.0.iter().map(|c| (c.species, c.hp)).collect(),
    };
    let key = open.then(|| (battle.menu as u8, battle.cursor, rows.clone()));
    if *last == key {
        return;
    }
    *last = key;
    commands.entity(*panel).despawn_related::<Children>();
    if !open {
        return;
    }
    let window = commands
        .spawn((
            ChildOf(*panel),
            ui::panel(Node { width: px(420), padding: UiRect::all(px(14)), flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }),
        ))
        .id();
    let title = if battle.menu == Menu::Bag { "Mochila" } else { "Tu equipo" };
    commands.spawn((ChildOf(window), ui::bold(title, 20.0)));
    if rows.is_empty() {
        commands.spawn((ChildOf(window), ui::muted("No tienes objetos que usar en combate.", 16.0)));
    }
    let start = battle.cursor.saturating_sub(LIST_ROWS - 1);
    for i in start..rows.len().min(start + LIST_ROWS) {
        let selected = i == battle.cursor;
        let row = commands
            .spawn((
                ChildOf(window),
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                    padding: UiRect::axes(px(10), px(6)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(if selected { Color::srgb(0.9, 0.94, 1.0) } else { Color::NONE }),
                BorderColor::all(if selected { ui::ACCENT } else { Color::NONE }),
            ))
            .id();
        match battle.menu {
            Menu::Bag => {
                let (item, count) = rows[i];
                commands.spawn((ChildOf(row), ui::item_icon(&ITEMS[item], 22.0)));
                commands.spawn((ChildOf(row), ui::text(ITEMS[item].name, 18.0), Node { flex_grow: 1.0, ..default() }));
                commands.spawn((ChildOf(row), ui::bold(format!("× {count}"), 17.0)));
            }
            _ => {
                let c = &party.0[i];
                let hp = c.hp.max(0) as f32 / c.max_hp() as f32;
                commands.spawn((ChildOf(row), ui::bold(c.name(), 18.0), Node { width: px(120), ..default() }));
                commands.spawn((ChildOf(row), ui::muted(format!("Nv.{}", c.level), 15.0), Node { width: px(50), ..default() }));
                commands.spawn((ChildOf(row), Node { flex_grow: 1.0, ..default() }, children![ui::bar((), hp, 8.0, ui::hp_color(hp))]));
                let status = if c.fainted() { "Debilitado".to_string() } else { format!("{}/{}", c.hp, c.max_hp()) };
                commands.spawn((ChildOf(row), ui::muted(status, 14.0), Node { width: px(78), ..default() }));
            }
        }
    }
    commands.spawn((ChildOf(window), ui::muted("Flechas: elegir   ·   Z: aceptar   ·   X: volver", 14.0)));
}

/// Poké Balls del entrenador rival: llenas las que le quedan, grises las debilitadas.
fn sync_foe_team(mut commands: Commands, battle: Res<Battle>, team: Single<Entity, With<FoeTeam>>, mut last: Local<Option<Vec<bool>>>) {
    if battle.trainer.is_none() {
        return;
    }
    let alive: Vec<bool> = battle.foes.iter().map(|c| !c.fainted()).collect();
    if last.as_ref() == Some(&alive) {
        return;
    }
    commands.entity(*team).despawn_related::<Children>();
    for ok in &alive {
        let ball = commands.spawn((ChildOf(*team), ui::pokeball(14.0))).id();
        if !ok {
            commands.entity(ball).insert(BackgroundColor(GREYED));
        }
    }
    *last = Some(alive);
}

// ---------------------------------------------------------------- escena 3D

fn spawn_arena(commands: &mut Commands, kit: &mut Kit, theme: Theme, trainer: Option<usize>) {
    let scoped = || DespawnOnExit(AppState::Battle);
    let (floor, platform) = match theme {
        Theme::Cave => ([110, 92, 78], [150, 130, 110]),
        Theme::Tower => ([95, 80, 115], [140, 125, 160]),
        Theme::Gym | Theme::Inside => ([205, 195, 175], [235, 225, 205]),
        Theme::Sea => ([90, 160, 210], [232, 214, 160]),
        Theme::Forest => ([86, 160, 72], [190, 205, 140]),
        _ => ([104, 184, 84], [200, 215, 150]),
    };
    let ground = kit.piece(&part(Shape::Cylinder, floor, ARENA - Vec3::Y * 0.1, Vec3::new(30.0, 0.2, 30.0)));
    commands.spawn((ground, scoped()));
    for spot in [PLAYER_SPOT, FOE_SPOT] {
        let p = kit.piece(&part(Shape::Cylinder, platform, ARENA + spot, Vec3::new(2.8, 0.12, 2.8)));
        commands.spawn((p, scoped()));
    }
    if matches!(theme, Theme::Gym | Theme::Inside) {
        // Líneas del campo de combate
        let line = part(Shape::Cube, [250, 250, 250], ARENA + Vec3::new(0.1, 0.005, -0.4), Vec3::new(0.08, 0.02, 9.0));
        let line = kit.piece(&line.rot(Quat::from_rotation_y(0.7)));
        commands.spawn((line, scoped()));
    }
    // Alrededor: árboles fuera, rocas en cuevas, columnas dentro
    for i in 0..18 {
        let angle = i as f32 / 18.0 * std::f32::consts::TAU;
        let radius = 9.0 + (i % 3) as f32 * 1.5;
        let pos = ARENA + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        let parts = match theme {
            Theme::Cave => crate::render::models::cave_wall(i as u32 * 7 + 1, floor),
            Theme::Gym | Theme::Inside | Theme::Tower if i % 2 == 0 => crate::render::models::pillar(),
            Theme::Gym | Theme::Inside | Theme::Tower => crate::render::models::plant(),
            Theme::Sea => continue,
            _ => tree(),
        };
        let transform = Transform::from_translation(pos).with_scale(Vec3::splat(1.4 + (i % 4) as f32 * 0.2));
        kit.spawn_model(commands, (transform, scoped()), &parts);
    }
    // El entrenador rival, detrás de su Pokémon
    if let Some(id) = trainer {
        let pos = ARENA + FOE_SPOT + Vec3::new(1.9, 0.0, -1.6);
        let look = ARENA + PLAYER_SPOT;
        let dir = look - pos;
        let t = Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(dir.x.atan2(dir.z))).with_scale(Vec3::splat(1.3));
        kit.spawn_character(commands, (t, scoped()), &NPC_OUTFITS[npc(id).look]);
    }
}

fn spawn_creature(commands: &mut Commands, kit: &mut Kit, side: Side, creature: &Creature) {
    let (spot, other) = match side {
        Side::Player => (PLAYER_SPOT, FOE_SPOT),
        Side::Foe => (FOE_SPOT, PLAYER_SPOT),
    };
    let dir = other - spot;
    let transform = Transform::from_translation(ARENA + spot)
        .with_rotation(Quat::from_rotation_y(dir.x.atan2(dir.z)))
        .with_scale(Vec3::splat(0.01));
    let model = BattleModel { side, species: creature.species, last_hp: creature.hp, hurt: 0.0, age: 0.0 };
    kit.spawn_creature(commands, (transform, model, DespawnOnExit(AppState::Battle)), creature.species);
}

/// Crea/cambia los modelos cuando cambia la criatura que lucha y oculta a la que ya no está.
fn sync_models(
    mut commands: Commands,
    mut kit: Kit,
    battle: Res<Battle>,
    party: Res<Party>,
    mut models: Query<(Entity, &mut BattleModel, &mut Visibility)>,
) {
    let me = &party.0[battle.active];
    let mut have = [false, false];
    for (entity, mut model, mut visibility) in &mut models {
        let creature = match model.side {
            Side::Player => me,
            Side::Foe => battle.foe(),
        };
        if model.species != creature.species {
            commands.entity(entity).despawn();
            continue;
        }
        have[model.side as usize] = true;
        if creature.hp < model.last_hp {
            model.hurt = 0.4;
        }
        model.last_hp = creature.hp;
        let gone = match model.side {
            Side::Foe => battle.foe_gone,
            Side::Player => me.fainted(),
        };
        *visibility = if gone { Visibility::Hidden } else { Visibility::Inherited };
    }
    if !have[Side::Player as usize] {
        spawn_creature(&mut commands, &mut kit, Side::Player, me);
    }
    if !have[Side::Foe as usize] {
        spawn_creature(&mut commands, &mut kit, Side::Foe, battle.foe());
    }
}

/// Cámara del combate; con Q/E o arrastrando el ratón gira alrededor de la arena.
fn battle_camera(rig: Res<CameraRig>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    let focus = ARENA + Vec3::new(0.4, 0.6, -0.6);
    let offset = Quat::from_rotation_y(rig.battle_yaw()) * Vec3::new(-4.6, 2.2, 7.4);
    **camera = Transform::from_translation(focus + offset).looking_at(focus, Vec3::Y);
}

/// Aparición (crece al salir de la Poké Ball), respiración y temblor al recibir daño.
fn animate_models(time: Res<Time>, mut models: Query<(&mut BattleModel, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (mut model, mut transform) in &mut models {
        let (spot, phase) = match model.side {
            Side::Player => (PLAYER_SPOT, 0.0),
            Side::Foe => (FOE_SPOT, 1.7),
        };
        model.age += time.delta_secs();
        let grow = (model.age / 0.35).min(1.0);
        let pop = 1.0 + (1.0 - grow) * (grow * std::f32::consts::PI).sin() * 0.3;
        let breathe = 1.0 + (t * 3.0 + phase).sin() * 0.03;
        let s = 1.5 * grow.max(0.01) * pop;
        transform.scale = Vec3::new(s, s * breathe, s);

        let mut pos = ARENA + spot;
        if model.hurt > 0.0 {
            model.hurt -= time.delta_secs();
            pos.x += (t * 60.0).sin() * 0.12;
        }
        transform.translation = pos;
    }
}
