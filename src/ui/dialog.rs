//! Cuadro de diálogo: mensajes, quién habla, preguntas de tipos y qué pasa al terminar
//! (empezar un combate, abrir la tienda...).

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::AppState;
use crate::battle::PendingBattle;
use crate::pokemon::Party;
use crate::game::items::{Bag, ITEMS};
use crate::ui::menu::Shop;
use crate::world::npcs::{Quiz, Reward, Role, npc};
use crate::game::progress::Progress;
use crate::ui;

/// Separa quién habla del texto (un carácter de control: nunca sale en pantalla).
const SPEAKER: char = '\u{1}';

/// Una línea dicha por alguien: sale con su nombre encima del cuadro.
pub fn speak(name: &str, text: &str) -> String {
    format!("{name}{SPEAKER}{text}")
}

/// Qué pasa cuando se cierra el diálogo.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum After {
    /// Combate contra ese entrenador (`npcs::npc_id`).
    Battle(usize),
    Shop,
}

#[derive(Resource)]
pub struct Dialog {
    lines: VecDeque<String>,
    /// Persona cuya pregunta se hace al terminar las líneas.
    quiz: Option<usize>,
    asking: bool,
    cursor: usize,
    picked: Vec<bool>,
    after: Option<After>,
}

impl Dialog {
    pub fn message(lines: Vec<String>) -> Self {
        Self { lines: lines.into(), quiz: None, asking: false, cursor: 0, picked: vec![], after: None }
    }

    pub fn then(mut self, after: After) -> Self {
        self.after = Some(after);
        self
    }

    /// Hablar con alguien que te hace una pregunta (o que ya te la hizo).
    pub fn quiz(id: usize, progress: &Progress) -> Self {
        let person = npc(id);
        let quiz = quiz_of(id);
        if progress.done.get(id) {
            return Self::message(vec![speak(person.name, quiz.beaten)]);
        }
        let lines = quiz.intro.iter().map(|l| speak(person.name, l)).collect();
        Self { quiz: Some(id), picked: vec![false; quiz.options.len()], ..Self::message(lines) }
    }
}

fn quiz_of(id: usize) -> &'static Quiz {
    match &npc(id).role {
        Role::Quiz(q) => q,
        _ => panic!("{} no hace preguntas", npc(id).name),
    }
}

#[derive(Component)]
struct DialogBox;

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (dialog_input.run_if(resource_exists::<Dialog>), sync_dialog_ui)
                .chain()
                .in_set(DialogSet)
                .run_if(in_state(AppState::Overworld)),
        );
    }
}

/// Para que el overworld pueda ejecutarse después y no reciba la misma tecla.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogSet;

pub const CONFIRM_KEYS: [KeyCode; 3] = [KeyCode::Enter, KeyCode::Space, KeyCode::KeyZ];

#[allow(clippy::too_many_arguments)]
fn dialog_input(
    mut commands: Commands,
    mut input: ResMut<ButtonInput<KeyCode>>,
    mut dialog: ResMut<Dialog>,
    mut party: ResMut<Party>,
    mut bag: ResMut<Bag>,
    mut progress: ResMut<Progress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if dialog.asking {
        let id = dialog.quiz.unwrap();
        let (name, quiz) = (npc(id).name, quiz_of(id));
        let n = quiz.options.len();

        if input.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::ArrowRight, KeyCode::KeyD]) {
            dialog.cursor = (dialog.cursor + 1) % n;
        }
        if input.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::ArrowLeft, KeyCode::KeyA]) {
            dialog.cursor = (dialog.cursor + n - 1) % n;
        }
        if input.any_just_pressed([KeyCode::Space, KeyCode::KeyZ]) {
            let i = dialog.cursor;
            if quiz.correct.len() == 1 {
                dialog.picked.iter_mut().for_each(|p| *p = false);
            }
            let count = dialog.picked.iter().filter(|p| **p).count();
            if dialog.picked[i] || count < quiz.correct.len() {
                dialog.picked[i] = !dialog.picked[i];
            }
        }
        if input.just_pressed(KeyCode::Enter) {
            let picked: Vec<usize> = (0..n).filter(|&i| dialog.picked[i]).collect();
            if picked.len() == quiz.correct.len() {
                dialog.asking = false;
                dialog.quiz = None;
                if picked == quiz.correct {
                    progress.done.set(id);
                    dialog.lines.push_back(speak(name, "¡Correcto! Toma esto."));
                    match quiz.reward {
                        Reward::Item(item, qty) => {
                            bag.add(item, qty);
                            dialog.lines.push_back(format!("¡Has recibido {} × {}!", ITEMS[item].name, qty));
                        }
                        Reward::Xp(xp) => {
                            let lead = party.first_alive().unwrap_or(0);
                            dialog.lines.extend(party.0[lead].gain_xp(xp));
                        }
                    }
                } else {
                    dialog.lines.push_back(speak(name, "¡Uy, no! Vuelve cuando lo sepas."));
                }
            }
        }
    } else if input.any_just_pressed(CONFIRM_KEYS) {
        dialog.lines.pop_front();
        if dialog.lines.is_empty() {
            if dialog.quiz.is_some() {
                dialog.asking = true;
            } else {
                commands.remove_resource::<Dialog>();
                match dialog.after {
                    Some(After::Battle(id)) => {
                        commands.insert_resource(PendingBattle::Trainer(id));
                        next_state.set(AppState::Battle);
                    }
                    Some(After::Shop) => commands.insert_resource(Shop::default()),
                    None => {}
                }
            }
        }
    }

    // Que nadie más procese estas teclas este frame (si no, cerrar un diálogo lo vuelve a abrir)
    for key in CONFIRM_KEYS {
        input.clear_just_pressed(key);
    }
}

/// "Nombre\u{1}texto" -> (Some(nombre), texto) si quien habla es alguien.
fn split_speaker(line: &str) -> (Option<&str>, &str) {
    match line.split_once(SPEAKER) {
        Some((name, text)) => (Some(name), text),
        None => (None, line),
    }
}

/// Rehace el cuadro de diálogo cada vez que cambia (es pequeño, sale más simple que actualizarlo).
fn sync_dialog_ui(mut commands: Commands, dialog: Option<Res<Dialog>>, ui_root: Query<Entity, With<DialogBox>>) {
    let Some(dialog) = dialog else {
        for e in &ui_root {
            commands.entity(e).despawn();
        }
        return;
    };
    if !ui_root.is_empty() && !dialog.is_changed() {
        return;
    }
    for e in &ui_root {
        commands.entity(e).despawn();
    }

    let quiz = dialog.quiz.filter(|_| dialog.asking).map(|id| (npc(id).name, quiz_of(id)));
    let (speaker, body) = match quiz {
        Some((name, q)) => (Some(name), q.question),
        None => split_speaker(dialog.lines.front().map(String::as_str).unwrap_or_default()),
    };

    let root = commands
        .spawn((
            DialogBox,
            DespawnOnExit(AppState::Overworld),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                right: px(24),
                bottom: px(20),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            GlobalZIndex(5),
        ))
        .id();
    if let Some(name) = speaker {
        commands.spawn((
            ChildOf(root),
            ui::bold_in(name.to_string(), 18.0, Color::WHITE),
            Node {
                padding: UiRect::axes(px(16), px(5)),
                margin: UiRect::left(px(18)),
                border_radius: BorderRadius::top(px(10)),
                ..default()
            },
            BackgroundColor(ui::FRAME),
        ));
    }
    let panel = commands
        .spawn((
            ChildOf(root),
            ui::panel(Node {
                width: percent(100),
                min_height: px(120),
                padding: UiRect::axes(px(24), px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            }),
        ))
        .id();

    let Some((_, quiz)) = quiz else {
        commands.spawn((
            ChildOf(panel),
            Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::FlexEnd, column_gap: px(16), ..default() },
            children![
                (ui::text(body.to_string(), 26.0), Node { flex_grow: 1.0, flex_basis: px(0), ..default() }),
                ui::bold_in("▼", 18.0, ui::ACCENT),
            ],
        ));
        return;
    };

    commands.spawn((ChildOf(panel), ui::bold(body, 24.0)));
    let picked = dialog.picked.iter().filter(|p| **p).count();
    commands.spawn((
        ChildOf(panel),
        ui::muted(
            format!(
                "Elige {} ({} de {})   ·   Z o Espacio: marcar   ·   Enter: responder",
                quiz.correct.len(),
                picked,
                quiz.correct.len()
            ),
            16.0,
        ),
    ));
    let grid = commands
        .spawn((
            ChildOf(panel),
            Node {
                width: percent(100),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::SpaceBetween,
                row_gap: px(10),
                margin: UiRect::top(px(6)),
                ..default()
            },
        ))
        .id();
    for (i, option) in quiz.options.iter().enumerate() {
        let selected = dialog.cursor == i;
        let checked = dialog.picked[i];
        let row = commands
            .spawn((
                ChildOf(grid),
                Node {
                    width: percent(49),
                    align_items: AlignItems::Center,
                    column_gap: px(12),
                    padding: UiRect::axes(px(14), px(9)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(10)),
                    ..default()
                },
                BackgroundColor(if selected { Color::srgb(0.9, 0.94, 1.0) } else { Color::WHITE }),
                BorderColor::all(if selected { ui::ACCENT } else { ui::TRACK }),
            ))
            .id();
        let checkbox = commands
            .spawn((
                ChildOf(row),
                Node {
                    width: px(22),
                    height: px(22),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(if checked { ui::ACCENT } else { Color::WHITE }),
                BorderColor::all(if checked || selected { ui::ACCENT } else { ui::MUTED }),
            ))
            .id();
        if checked {
            commands.spawn((
                ChildOf(checkbox),
                Node { width: px(8), height: px(8), border_radius: BorderRadius::all(px(2)), ..default() },
                BackgroundColor(Color::WHITE),
            ));
        }
        commands.spawn((ChildOf(row), ui::text(*option, 22.0)));
    }
}
