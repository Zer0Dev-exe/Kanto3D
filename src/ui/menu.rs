//! Menú de juego (Enter o Esc): Pokédex, Pokémon, Mochila, Perfil (ficha de entrenador) y
//! Guardar. También la Tienda Pokémon (comprar y vender).

use bevy::prelude::*;

use crate::AppState;
use crate::pokemon::{Companion, Creature, Party, SPECIES};
use crate::ui::dialog::{Dialog, DialogSet};
use crate::game::items::{Bag, Effect, ITEMS, Pocket, shop_stock};
use crate::world::npcs::BADGES;
use crate::world::overworld::{CurrentMap, Facing, PlayerPos, warp_to};
use crate::ui::party::PartyMenu;
use crate::render::portraits::Portraits;
use crate::game::progress::{Place, Progress};
use crate::game::save::{self, Saving};
use crate::ui::{self, ACCENT, FRAME, MUTED, TRACK, bold, bold_in, muted, pill, pokeball, text};

const START: [&str; 6] = ["Pokédex", "Pokémon", "Mochila", "Perfil", "Guardar", "Salir"];
const DEX_COLUMNS: usize = 6;
const BAG_ROWS: usize = 8;

#[derive(Resource, Clone, PartialEq, Debug)]
pub enum GameMenu {
    Start(usize),
    Dex(usize),
    Bag(BagScreen),
    Profile,
    Save(SaveScreen),
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct BagScreen {
    pocket: usize,
    cursor: usize,
    /// Eligiendo a qué Pokémon se le da el objeto.
    target: Option<usize>,
    note: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SaveScreen {
    Asking(bool),
    Writing,
    Done(String),
}

/// La tienda abierta (al hablar con el dependiente).
#[derive(Resource, Clone, PartialEq, Debug, Default)]
pub struct Shop {
    stage: ShopStage,
    cursor: usize,
    qty: u16,
    note: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum ShopStage {
    #[default]
    Choose,
    Buy,
    Sell,
}

#[derive(Component)]
struct MenuRoot;

/// Los menús leen el teclado antes que el mundo.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuSet;

/// Hay algo abierto encima del mapa (diálogo, menú, tienda...): el jugador no se mueve.
pub fn overlay_open(
    dialog: Option<Res<Dialog>>,
    party: Option<Res<PartyMenu>>,
    menu: Option<Res<GameMenu>>,
    shop: Option<Res<Shop>>,
) -> bool {
    dialog.is_some() || party.is_some() || menu.is_some() || shop.is_some()
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::Overworld), |mut commands: Commands| {
            commands.remove_resource::<GameMenu>();
            commands.remove_resource::<Shop>();
        })
        .add_systems(
            Update,
            (
                open_menu,
                menu_input.run_if(resource_exists::<GameMenu>),
                shop_input.run_if(resource_exists::<Shop>),
                finish_saving.run_if(resource_exists::<Saving>),
                draw_menu,
                draw_shop,
            )
                .chain()
                .in_set(MenuSet)
                .after(DialogSet)
                .run_if(in_state(AppState::Overworld)),
        );
    }
}

const NAV: [KeyCode; 14] = [
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::KeyW,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::KeyZ,
    KeyCode::KeyX,
    KeyCode::Space,
    KeyCode::Enter,
    KeyCode::Escape,
    KeyCode::Backspace,
];

struct Keys {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    ok: bool,
    back: bool,
}

/// Lee las teclas del menú y las gasta (que no muevan al jugador).
fn keys(input: &mut ButtonInput<KeyCode>) -> Keys {
    let k = Keys {
        up: input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]),
        down: input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]),
        left: input.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]),
        right: input.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]),
        ok: input.any_just_pressed([KeyCode::KeyZ, KeyCode::Space, KeyCode::Enter]),
        back: input.any_just_pressed([KeyCode::KeyX, KeyCode::Escape, KeyCode::Backspace]),
    };
    for key in NAV {
        input.clear_just_pressed(key);
    }
    k
}

fn open_menu(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    dialog: Option<Res<Dialog>>,
    party: Option<Res<PartyMenu>>,
    menu: Option<Res<GameMenu>>,
    shop: Option<Res<Shop>>,
) {
    if dialog.is_some() || party.is_some() || menu.is_some() || shop.is_some() {
        return;
    }
    if input.any_just_pressed([KeyCode::Enter, KeyCode::Escape]) {
        input.clear_just_pressed(KeyCode::Enter);
        input.clear_just_pressed(KeyCode::Escape);
        commands.insert_resource(GameMenu::Start(0));
    }
}

fn wrap(i: usize, delta: isize, len: usize) -> usize {
    if len == 0 { 0 } else { (i as isize + delta).rem_euclid(len as isize) as usize }
}

#[allow(clippy::too_many_arguments)]
fn menu_input(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut menu: ResMut<GameMenu>,
    mut party: ResMut<Party>,
    mut bag: ResMut<Bag>,
    mut progress: ResMut<Progress>,
    companion: Res<Companion>,
    map: Res<CurrentMap>,
    pos: Res<PlayerPos>,
    facing: Res<Facing>,
    saving: Option<Res<Saving>>,
) {
    let k = keys(&mut input);
    let mut next = menu.clone();
    let mut back_to = None;
    match &mut next {
        GameMenu::Start(cursor) => {
            if k.up {
                *cursor = wrap(*cursor, -1, START.len());
            }
            if k.down {
                *cursor = wrap(*cursor, 1, START.len());
            }
            if k.back {
                commands.remove_resource::<GameMenu>();
                return;
            }
            if k.ok {
                next = match *cursor {
                    0 => GameMenu::Dex(0),
                    1 => {
                        commands.remove_resource::<GameMenu>();
                        commands.insert_resource(PartyMenu::at(0));
                        return;
                    }
                    2 => GameMenu::Bag(BagScreen::default()),
                    3 => GameMenu::Profile,
                    4 => GameMenu::Save(SaveScreen::Asking(true)),
                    _ => {
                        commands.remove_resource::<GameMenu>();
                        return;
                    }
                };
            }
        }
        GameMenu::Dex(cursor) => {
            let n = SPECIES.len();
            if k.left {
                *cursor = wrap(*cursor, -1, n);
            }
            if k.right {
                *cursor = wrap(*cursor, 1, n);
            }
            if k.up {
                *cursor = wrap(*cursor, -(DEX_COLUMNS as isize), n);
            }
            if k.down {
                *cursor = wrap(*cursor, DEX_COLUMNS as isize, n);
            }
            if k.back {
                next = GameMenu::Start(0);
            }
        }
        GameMenu::Profile => {
            if k.back || k.ok {
                next = GameMenu::Start(3);
            }
        }
        GameMenu::Save(screen) => match screen {
            SaveScreen::Asking(yes) => {
                if k.left || k.right || k.up || k.down {
                    *yes = !*yes;
                }
                if k.back || (k.ok && !*yes) {
                    next = GameMenu::Start(4);
                } else if k.ok && saving.is_none() {
                    let place = Place { map: map.0.id, pos: pos.0 };
                    save::start_saving(&mut commands, save::snapshot(&progress, &party, &bag, companion.0, place, facing.0));
                    *screen = SaveScreen::Writing;
                }
            }
            SaveScreen::Writing => {}
            SaveScreen::Done(_) => {
                if k.ok || k.back {
                    commands.remove_resource::<GameMenu>();
                    return;
                }
            }
        },
        GameMenu::Bag(screen) => {
            let pocket = Pocket::ALL[screen.pocket];
            let items = bag.pocket(pocket);
            if let Some(target) = screen.target {
                // Elegir el Pokémon
                if k.up {
                    screen.target = Some(wrap(target, -1, party.0.len()));
                }
                if k.down {
                    screen.target = Some(wrap(target, 1, party.0.len()));
                }
                if k.back {
                    screen.target = None;
                }
                if k.ok {
                    if let Some(&item) = items.get(screen.cursor) {
                        screen.note = Some(give_item(&mut party.0[target], &mut bag, item));
                    }
                    screen.target = None;
                }
            } else {
                if k.left {
                    *screen = BagScreen { pocket: wrap(screen.pocket, -1, 3), ..default() };
                } else if k.right {
                    *screen = BagScreen { pocket: wrap(screen.pocket, 1, 3), ..default() };
                } else if k.up {
                    screen.cursor = wrap(screen.cursor, -1, items.len());
                    screen.note = None;
                } else if k.down {
                    screen.cursor = wrap(screen.cursor, 1, items.len());
                    screen.note = None;
                } else if k.back {
                    back_to = Some(GameMenu::Start(2));
                } else if k.ok {
                    if let Some(&item) = items.get(screen.cursor) {
                        match ITEMS[item].effect {
                            _ if ITEMS[item].needs_target() => screen.target = Some(0),
                            Effect::Repel(steps) => {
                                bag.take(item);
                                progress.repel = steps;
                                screen.note = Some(format!("Has usado {}. Los Pokémon débiles no se acercarán.", ITEMS[item].name));
                            }
                            Effect::EscapeRope if map.0.def().theme.underground() => {
                                bag.take(item);
                                warp_to(&mut commands, progress.last_center);
                                commands.remove_resource::<GameMenu>();
                                commands.insert_resource(Dialog::message(vec![format!("¡Usas la {} y sales al exterior!", ITEMS[item].name)]));
                                return;
                            }
                            Effect::EscapeRope => screen.note = Some("Aquí no sirve de nada. Úsala dentro de una cueva.".into()),
                            _ => screen.note = Some("Solo se puede usar durante un combate.".into()),
                        }
                    }
                }
                // Si te quedas sin el objeto, el cursor no se sale de la lista
                let len = bag.pocket(pocket).len();
                screen.cursor = screen.cursor.min(len.saturating_sub(1));
            }
        }
    }
    if let Some(b) = back_to {
        next = b;
    }
    if next != *menu {
        *menu = next;
    }
}

/// Usa un objeto con un Pokémon fuera de combate.
fn give_item(c: &mut Creature, bag: &mut Bag, item: usize) -> String {
    let it = &ITEMS[item];
    match it.effect {
        Effect::Heal(amount) if !c.fainted() && c.hp < c.max_hp() => {
            bag.take(item);
            let before = c.hp;
            c.hp = c.hp.saturating_add(amount).min(c.max_hp());
            format!("{} recupera {} PS.", c.name(), c.hp - before)
        }
        Effect::Revive if c.fainted() => {
            bag.take(item);
            c.hp = c.max_hp() / 2;
            format!("¡{} se ha reanimado!", c.name())
        }
        Effect::RareCandy if c.level < crate::pokemon::MAX_LEVEL => {
            bag.take(item);
            c.xp = 0;
            c.level_up().join(" ")
        }
        _ => "No tendrá ningún efecto.".into(),
    }
}

fn finish_saving(mut commands: Commands, mut saving: ResMut<Saving>, mut menu: Option<ResMut<GameMenu>>) {
    let Some(result) = save::poll_saving(&mut saving) else { return };
    let ms = saving.started.elapsed().as_secs_f32() * 1000.0;
    commands.remove_resource::<Saving>();
    let text = match result {
        Ok(bytes) => format!("¡Partida guardada! ({bytes} bytes en {ms:.1} ms)"),
        Err(e) => format!("No se pudo guardar: {e}"),
    };
    if let Some(menu) = menu.as_deref_mut() {
        *menu = GameMenu::Save(SaveScreen::Done(text));
    }
}

#[allow(clippy::too_many_arguments)]
fn shop_input(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut shop: ResMut<Shop>,
    mut bag: ResMut<Bag>,
    mut progress: ResMut<Progress>,
) {
    let k = keys(&mut input);
    let mut s = shop.clone();
    match s.stage {
        ShopStage::Choose => {
            if k.up {
                s.cursor = wrap(s.cursor, -1, 3);
            }
            if k.down {
                s.cursor = wrap(s.cursor, 1, 3);
            }
            if k.back || (k.ok && s.cursor == 2) {
                commands.remove_resource::<Shop>();
                commands.insert_resource(Dialog::message(vec![crate::ui::dialog::speak("Dependiente", "¡Gracias por tu visita! ¡Vuelve pronto!")]));
                return;
            }
            if k.ok {
                s = Shop { stage: if s.cursor == 0 { ShopStage::Buy } else { ShopStage::Sell }, cursor: 0, qty: 1, note: None };
            }
        }
        ShopStage::Buy | ShopStage::Sell => {
            let buying = s.stage == ShopStage::Buy;
            let list = if buying { shop_stock(progress.badge_count()) } else { sellable(&bag) };
            if k.up {
                s.cursor = wrap(s.cursor, -1, list.len());
                s.qty = 1;
            }
            if k.down {
                s.cursor = wrap(s.cursor, 1, list.len());
                s.qty = 1;
            }
            let max = list.get(s.cursor).map_or(1, |&i| {
                if buying { (progress.money / ITEMS[i].price).clamp(1, 99) as u16 } else { bag.count(i).max(1) }
            });
            if k.right {
                s.qty = if s.qty >= max { 1 } else { s.qty + 1 };
            }
            if k.left {
                s.qty = if s.qty <= 1 { max } else { s.qty - 1 };
            }
            if k.back {
                s = Shop { stage: ShopStage::Choose, cursor: if buying { 0 } else { 1 }, qty: 1, note: None };
            } else if k.ok {
                if let Some(&item) = list.get(s.cursor) {
                    let it = &ITEMS[item];
                    if buying {
                        let total = it.price * s.qty as u32;
                        if total > progress.money {
                            s.note = Some("No tienes dinero suficiente.".into());
                        } else {
                            progress.money -= total;
                            bag.add(item, s.qty);
                            s.note = Some(format!("¡Aquí tienes! {} × {} por {total} ₽.", it.name, s.qty));
                        }
                    } else {
                        let qty = s.qty.min(bag.count(item));
                        let total = it.price / 2 * qty as u32;
                        for _ in 0..qty {
                            bag.take(item);
                        }
                        progress.earn(total);
                        s.note = Some(format!("Has vendido {} × {} por {total} ₽.", it.name, qty));
                        s.cursor = s.cursor.min(sellable(&bag).len().saturating_sub(1));
                    }
                    s.qty = 1;
                }
            }
        }
    }
    if s != *shop {
        *shop = s;
    }
}

fn sellable(bag: &Bag) -> Vec<usize> {
    (0..ITEMS.len()).filter(|&i| bag.count(i) > 0 && ITEMS[i].price > 0).collect()
}

// ---------------------------------------------------------------- dibujar

/// Fondo oscurecido y una ventana centrada.
fn window(commands: &mut Commands, width: f32, top: bool) -> Entity {
    let root = commands
        .spawn((
            MenuRoot,
            DespawnOnExit(AppState::Overworld),
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                justify_content: if top { JustifyContent::FlexEnd } else { JustifyContent::Center },
                align_items: if top { AlignItems::FlexStart } else { AlignItems::Center },
                padding: UiRect::all(px(20)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, if top { 0.2 } else { 0.5 })),
            GlobalZIndex(12),
        ))
        .id();
    commands
        .spawn((
            ChildOf(root),
            ui::panel(Node { width: px(width), padding: UiRect::all(px(22)), flex_direction: FlexDirection::Column, row_gap: px(10), ..default() }),
        ))
        .id()
}

fn header(commands: &mut Commands, parent: Entity, title: &str) {
    commands.spawn((
        ChildOf(parent),
        Node { align_items: AlignItems::Center, column_gap: px(12), margin: UiRect::bottom(px(4)), ..default() },
        children![pokeball(28.0), bold(title.to_string(), 28.0)],
    ));
}

fn row(commands: &mut Commands, parent: Entity, selected: bool) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            Node {
                align_items: AlignItems::Center,
                column_gap: px(12),
                padding: UiRect::axes(px(12), px(7)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(if selected { Color::srgb(0.9, 0.94, 1.0) } else { Color::NONE }),
            BorderColor::all(if selected { ACCENT } else { Color::NONE }),
        ))
        .id()
}

fn portrait(commands: &mut Commands, parent: Entity, image: Option<Handle<Image>>, size: f32, tint: Color) -> Entity {
    let frame = commands
        .spawn((
            ChildOf(parent),
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.0,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(size * 0.18)),
                overflow: Overflow::clip(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(tint.mix(&Color::WHITE, 0.7)),
            BorderColor::all(tint),
        ))
        .id();
    if let Some(image) = image {
        commands.spawn((ChildOf(frame), ImageNode::new(image), Node { width: percent(100), height: percent(100), ..default() }));
    }
    frame
}

#[allow(clippy::too_many_arguments)]
fn draw_menu(
    mut commands: Commands,
    menu: Option<Res<GameMenu>>,
    party: Res<Party>,
    bag: Res<Bag>,
    progress: Res<Progress>,
    map: Res<CurrentMap>,
    portraits: Option<Res<Portraits>>,
    roots: Query<Entity, With<MenuRoot>>,
    shop: Option<Res<Shop>>,
) {
    let Some(menu) = menu else {
        if shop.is_none() {
            for e in &roots {
                commands.entity(e).despawn();
            }
        }
        return;
    };
    if !roots.is_empty() && !(menu.is_changed() || party.is_changed() || bag.is_changed()) {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let portraits = portraits.as_deref();
    match &*menu {
        GameMenu::Start(cursor) => draw_start(&mut commands, *cursor, &progress),
        GameMenu::Dex(cursor) => draw_dex(&mut commands, *cursor, &progress, portraits),
        GameMenu::Bag(screen) => draw_bag(&mut commands, screen, &bag, &party),
        GameMenu::Profile => draw_profile(&mut commands, &progress, portraits),
        GameMenu::Save(screen) => draw_save(&mut commands, screen, &progress, &party, map.0.def().name),
    }
}

fn draw_start(commands: &mut Commands, cursor: usize, progress: &Progress) {
    let w = window(commands, 300.0, true);
    for (i, name) in START.iter().enumerate() {
        let r = row(commands, w, i == cursor);
        let label = if i == 3 { progress.name.clone() } else { name.to_string() };
        commands.spawn((ChildOf(r), Node { width: px(10), height: px(10), border_radius: BorderRadius::all(percent(50)), ..default() }, BackgroundColor(if i == cursor { ACCENT } else { TRACK })));
        commands.spawn((ChildOf(r), bold(label, 22.0)));
    }
    commands.spawn((ChildOf(w), Node { height: px(2), margin: UiRect::vertical(px(4)), ..default() }, BackgroundColor(TRACK)));
    commands.spawn((
        ChildOf(w),
        Node { justify_content: JustifyContent::SpaceBetween, ..default() },
        children![muted("Dinero", 16.0), bold(format!("{} ₽", progress.money), 16.0)],
    ));
    commands.spawn((
        ChildOf(w),
        Node { justify_content: JustifyContent::SpaceBetween, ..default() },
        children![muted("Medallas", 16.0), bold(format!("{} / 8", progress.badge_count()), 16.0)],
    ));
    commands.spawn((ChildOf(w), muted("Z: elegir   ·   X: cerrar", 14.0)));
}

fn draw_dex(commands: &mut Commands, cursor: usize, progress: &Progress, portraits: Option<&Portraits>) {
    let w = window(commands, 1040.0, false);
    header(commands, w, "Pokédex");
    commands.spawn((
        ChildOf(w),
        muted(format!("Vistos: {}   ·   Capturados: {}   ·   Total de Kanto en este juego: {}", progress.seen.count(), progress.caught.count(), SPECIES.len()), 16.0),
    ));
    let body = commands.spawn((ChildOf(w), Node { column_gap: px(18), ..default() })).id();
    let grid = commands
        .spawn((ChildOf(body), Node { width: px(600), flex_wrap: FlexWrap::Wrap, row_gap: px(6), column_gap: px(6), ..default() }))
        .id();
    // En el orden de la Pokédex nacional
    let mut order: Vec<usize> = (0..SPECIES.len()).collect();
    order.sort_by_key(|&i| SPECIES[i].dex);
    let selected = order[cursor.min(order.len() - 1)];
    for (slot, &i) in order.iter().enumerate() {
        let s = &SPECIES[i];
        let seen = progress.seen.get(i);
        let sel = slot == cursor;
        let cell = commands
            .spawn((
                ChildOf(grid),
                Node {
                    width: px(94),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(if sel { Color::srgb(0.9, 0.94, 1.0) } else { Color::NONE }),
                BorderColor::all(if sel { ACCENT } else { TRACK }),
            ))
            .id();
        let image = portraits.filter(|_| seen).map(|p| p.0[i].clone());
        let tint = if seen { s.ty.color() } else { MUTED };
        let pic = portrait(commands, cell, image, 62.0, tint);
        if !seen {
            commands.spawn((ChildOf(pic), bold_in("?", 30.0, Color::WHITE)));
        }
        let top = commands.spawn((ChildOf(cell), Node { align_items: AlignItems::Center, column_gap: px(4), ..default() })).id();
        if progress.caught.get(i) {
            commands.spawn((ChildOf(top), pokeball(11.0)));
        }
        commands.spawn((ChildOf(top), muted(format!("Nº {:03}", s.dex), 12.0)));
        commands.spawn((ChildOf(cell), text(if seen { s.name } else { "???" }, 13.0)));
    }

    // Ficha de la especie elegida
    let s = &SPECIES[selected];
    let seen = progress.seen.get(selected);
    let card = commands
        .spawn((ChildOf(body), Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: px(10), align_items: AlignItems::Center, ..default() }))
        .id();
    let image = portraits.filter(|_| seen).map(|p| p.0[selected].clone());
    let pic = portrait(commands, card, image, 220.0, if seen { s.ty.color() } else { MUTED });
    if !seen {
        commands.spawn((ChildOf(pic), bold_in("?", 90.0, Color::WHITE)));
    }
    commands.spawn((ChildOf(card), bold(format!("Nº {:03}  {}", s.dex, if seen { s.name } else { "???" }), 26.0)));
    if seen {
        commands.spawn((ChildOf(card), muted(s.kind, 17.0)));
        let types = commands.spawn((ChildOf(card), Node { column_gap: px(6), ..default() })).id();
        for t in s.types() {
            commands.spawn((ChildOf(types), pill(t.name().to_uppercase(), t.color())));
        }
        commands.spawn((ChildOf(card), text(s.entry, 17.0), Node { max_width: px(360), ..default() }));
        let status = if progress.caught.get(selected) { "Capturado" } else { "Visto" };
        commands.spawn((ChildOf(card), pill(status, if progress.caught.get(selected) { ACCENT } else { MUTED })));
    } else {
        commands.spawn((ChildOf(card), muted("Aún no has visto este Pokémon.", 17.0)));
    }
    commands.spawn((ChildOf(w), muted("Flechas: moverse   ·   X: volver", 14.0)));
}

fn draw_bag(commands: &mut Commands, screen: &BagScreen, bag: &Bag, party: &Party) {
    let w = window(commands, 860.0, false);
    header(commands, w, "Mochila");
    let tabs = commands.spawn((ChildOf(w), Node { column_gap: px(8), ..default() })).id();
    for (i, pocket) in Pocket::ALL.iter().enumerate() {
        let active = i == screen.pocket;
        commands.spawn((
            ChildOf(tabs),
            bold_in(pocket.name(), 17.0, if active { Color::WHITE } else { ui::INK }),
            Node { padding: UiRect::axes(px(16), px(6)), border_radius: BorderRadius::all(px(12)), ..default() },
            BackgroundColor(if active { FRAME } else { TRACK }),
        ));
    }
    let body = commands.spawn((ChildOf(w), Node { column_gap: px(18), min_height: px(360), ..default() })).id();
    let list = commands.spawn((ChildOf(body), Node { width: px(420), flex_direction: FlexDirection::Column, row_gap: px(4), ..default() })).id();
    let items = bag.pocket(Pocket::ALL[screen.pocket]);
    if items.is_empty() {
        commands.spawn((ChildOf(list), muted("Este bolsillo está vacío.", 17.0)));
    }
    let start = screen.cursor.saturating_sub(BAG_ROWS - 1);
    for (i, &item) in items.iter().enumerate().skip(start).take(BAG_ROWS) {
        let r = row(commands, list, i == screen.cursor);
        commands.spawn((ChildOf(r), ui::item_icon(&ITEMS[item], 26.0)));
        commands.spawn((ChildOf(r), text(ITEMS[item].name, 19.0), Node { flex_grow: 1.0, ..default() }));
        commands.spawn((ChildOf(r), bold(format!("× {}", bag.count(item)), 18.0)));
    }

    let side = commands.spawn((ChildOf(body), Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: px(10), ..default() })).id();
    if let Some(&item) = items.get(screen.cursor) {
        let it = &ITEMS[item];
        commands.spawn((ChildOf(side), Node { align_items: AlignItems::Center, column_gap: px(10), ..default() }, children![ui::item_icon(it, 44.0), bold(it.name, 24.0)]));
        commands.spawn((ChildOf(side), text(it.desc, 17.0)));
    }
    if let Some(target) = screen.target {
        commands.spawn((ChildOf(side), bold("¿A qué Pokémon se lo das?", 18.0)));
        for (i, c) in party.0.iter().enumerate() {
            let r = row(commands, side, i == target);
            let hp = c.hp.max(0) as f32 / c.max_hp() as f32;
            commands.spawn((ChildOf(r), bold(c.name(), 17.0), Node { width: px(110), ..default() }));
            commands.spawn((ChildOf(r), muted(format!("Nv.{}", c.level), 14.0)));
            commands.spawn((ChildOf(r), Node { flex_grow: 1.0, ..default() }, children![ui::bar((), hp, 7.0, ui::hp_color(hp))]));
        }
    }
    if let Some(note) = &screen.note {
        commands.spawn((ChildOf(side), bold_in(note.clone(), 17.0, ACCENT)));
    }
    commands.spawn((ChildOf(w), muted("Izquierda/derecha: bolsillo   ·   Z: usar   ·   X: volver", 14.0)));
}

fn info_line(commands: &mut Commands, parent: Entity, label: &str, value: String) {
    commands.spawn((
        ChildOf(parent),
        Node { justify_content: JustifyContent::SpaceBetween, padding: UiRect::vertical(px(6)), border: UiRect::bottom(px(2)), ..default() },
        BorderColor::all(TRACK),
        children![muted(label.to_string(), 18.0), bold(value, 20.0)],
    ));
}

fn draw_profile(commands: &mut Commands, progress: &Progress, portraits: Option<&Portraits>) {
    let w = window(commands, 820.0, false);
    // Franja de la ficha
    commands.spawn((
        ChildOf(w),
        Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(px(18), px(10)), border_radius: BorderRadius::all(px(10)), ..default() },
        BackgroundColor(Color::srgb(0.86, 0.22, 0.24)),
        children![bold_in("FICHA DE ENTRENADOR", 22.0, Color::WHITE), bold_in(format!("Nº ID {:05}", progress.id), 20.0, Color::WHITE)],
    ));
    let body = commands.spawn((ChildOf(w), Node { column_gap: px(22), ..default() })).id();
    let info = commands.spawn((ChildOf(body), Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, ..default() })).id();
    info_line(commands, info, "Nombre", progress.name.clone());
    info_line(commands, info, "Dinero", format!("{} ₽", progress.money));
    info_line(commands, info, "Pokédex", format!("{} capturados · {} vistos", progress.caught.count(), progress.seen.count()));
    info_line(commands, info, "Tiempo de juego", progress.time_text());
    info_line(commands, info, "Pasos", progress.steps.to_string());
    info_line(commands, info, "Pokémon en el PC", progress.boxed.len().to_string());
    if progress.champion {
        commands.spawn((ChildOf(info), Node { margin: UiRect::top(px(10)), ..default() }, children![pill("CAMPEÓN DE LA LIGA POKÉMON", Color::srgb(0.85, 0.65, 0.1))]));
    }
    let image = portraits.map(|p| p.player());
    portrait(commands, body, image, 230.0, Color::srgb(0.86, 0.22, 0.24));

    commands.spawn((ChildOf(w), bold("Medallas", 20.0), Node { margin: UiRect::top(px(6)), ..default() }));
    let badges = commands.spawn((ChildOf(w), Node { column_gap: px(10), justify_content: JustifyContent::SpaceBetween, ..default() })).id();
    for (i, b) in BADGES.iter().enumerate() {
        let earned = progress.has_badge(i as u8);
        let cell = commands
            .spawn((ChildOf(badges), Node { width: px(88), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(4), ..default() }))
            .id();
        commands.spawn((ChildOf(cell), ui::badge(b.color, earned, 46.0)));
        commands.spawn((ChildOf(cell), text(if earned { b.name.trim_start_matches("Medalla ") } else { "???" }, 13.0)));
        commands.spawn((ChildOf(cell), muted(b.leader, 11.0)));
    }
    commands.spawn((ChildOf(w), muted("X: volver", 14.0)));
}

fn draw_save(commands: &mut Commands, screen: &SaveScreen, progress: &Progress, party: &Party, place: &str) {
    let w = window(commands, 560.0, false);
    header(commands, w, "Guardar partida");
    info_line(commands, w, "Jugador", progress.name.clone());
    info_line(commands, w, "Lugar", place.to_string());
    info_line(commands, w, "Medallas", format!("{} / 8", progress.badge_count()));
    info_line(commands, w, "Pokédex", format!("{} capturados", progress.caught.count()));
    info_line(commands, w, "Tiempo de juego", progress.time_text());
    let team = commands.spawn((ChildOf(w), Node { column_gap: px(6), margin: UiRect::vertical(px(6)), ..default() })).id();
    for _ in &party.0 {
        commands.spawn((ChildOf(team), pokeball(20.0)));
    }
    match screen {
        SaveScreen::Asking(yes) => {
            commands.spawn((ChildOf(w), bold("¿Quieres guardar la partida?", 20.0)));
            let choice = commands.spawn((ChildOf(w), Node { column_gap: px(12), ..default() })).id();
            for (label, on) in [("Sí", *yes), ("No", !*yes)] {
                let r = row(commands, choice, on);
                commands.spawn((ChildOf(r), bold(label, 20.0)));
            }
        }
        SaveScreen::Writing => {
            commands.spawn((ChildOf(w), bold("Guardando... no apagues el juego.", 20.0)));
        }
        SaveScreen::Done(msg) => {
            commands.spawn((ChildOf(w), bold_in(msg.clone(), 20.0, Color::srgb(0.15, 0.55, 0.25))));
            commands.spawn((ChildOf(w), muted("Z: cerrar", 14.0)));
        }
    }
}

fn draw_shop(
    mut commands: Commands,
    shop: Option<Res<Shop>>,
    bag: Res<Bag>,
    progress: Res<Progress>,
    roots: Query<Entity, With<MenuRoot>>,
    menu: Option<Res<GameMenu>>,
) {
    let Some(shop) = shop else { return };
    if menu.is_some() || (!roots.is_empty() && !(shop.is_changed() || bag.is_changed() || progress.is_changed())) {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let w = window(&mut commands, 860.0, false);
    header(&mut commands, w, "Tienda Pokémon");
    commands.spawn((
        ChildOf(w),
        Node { justify_content: JustifyContent::SpaceBetween, ..default() },
        children![muted("Lo que vendemos depende de tus medallas.", 15.0), bold(format!("Tu dinero: {} ₽", progress.money), 18.0)],
    ));
    if shop.stage == ShopStage::Choose {
        for (i, label) in ["Comprar", "Vender", "Salir"].iter().enumerate() {
            let r = row(&mut commands, w, i == shop.cursor);
            commands.spawn((ChildOf(r), bold(*label, 22.0)));
        }
        commands.spawn((ChildOf(w), muted("Z: elegir   ·   X: salir", 14.0)));
        return;
    }
    let buying = shop.stage == ShopStage::Buy;
    let list = if buying { shop_stock(progress.badge_count()) } else { sellable(&bag) };
    let body = commands.spawn((ChildOf(w), Node { column_gap: px(18), min_height: px(340), ..default() })).id();
    let col = commands.spawn((ChildOf(body), Node { width: px(440), flex_direction: FlexDirection::Column, row_gap: px(4), ..default() })).id();
    if list.is_empty() {
        commands.spawn((ChildOf(col), muted("No tienes nada que vender.", 17.0)));
    }
    let start = shop.cursor.saturating_sub(BAG_ROWS - 1);
    for (i, &item) in list.iter().enumerate().skip(start).take(BAG_ROWS) {
        let it = &ITEMS[item];
        let r = row(&mut commands, col, i == shop.cursor);
        commands.spawn((ChildOf(r), ui::item_icon(it, 24.0)));
        commands.spawn((ChildOf(r), text(it.name, 19.0), Node { flex_grow: 1.0, ..default() }));
        let price = if buying { it.price } else { it.price / 2 };
        let extra = if buying { String::new() } else { format!("(tienes {})  ", bag.count(item)) };
        commands.spawn((ChildOf(r), muted(extra, 14.0)));
        commands.spawn((ChildOf(r), bold(format!("{price} ₽"), 18.0)));
    }
    let side = commands.spawn((ChildOf(body), Node { flex_grow: 1.0, flex_direction: FlexDirection::Column, row_gap: px(10), ..default() })).id();
    if let Some(&item) = list.get(shop.cursor) {
        let it = &ITEMS[item];
        let price = if buying { it.price } else { it.price / 2 };
        commands.spawn((ChildOf(side), Node { align_items: AlignItems::Center, column_gap: px(10), ..default() }, children![ui::item_icon(it, 44.0), bold(it.name, 24.0)]));
        commands.spawn((ChildOf(side), text(it.desc, 17.0)));
        commands.spawn((
            ChildOf(side),
            Node { align_items: AlignItems::Center, column_gap: px(10), margin: UiRect::top(px(8)), ..default() },
            children![muted("Cantidad", 17.0), bold(format!("‹ {} ›", shop.qty), 22.0), muted("Total", 17.0), bold(format!("{} ₽", price * shop.qty as u32), 22.0)],
        ));
        if bag.count(item) > 0 && buying {
            commands.spawn((ChildOf(side), muted(format!("En la mochila: {}", bag.count(item)), 15.0)));
        }
    }
    if let Some(note) = &shop.note {
        commands.spawn((ChildOf(side), bold_in(note.clone(), 17.0, ACCENT)));
    }
    let verb = if buying { "comprar" } else { "vender" };
    commands.spawn((ChildOf(w), muted(format!("Arriba/abajo: objeto   ·   Izquierda/derecha: cantidad   ·   Z: {verb}   ·   X: volver"), 14.0)));
}
