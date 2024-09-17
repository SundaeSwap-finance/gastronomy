use std::collections::BTreeMap;
use std::io::{self};
use std::path::PathBuf;
use std::rc::Rc;

use crate::utils;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use uplc::ast::NamedDeBruijn;
use uplc::machine::cost_model::ExBudget;
use uplc::machine::indexed_term::IndexedTerm;
use uplc::machine::value::Value;
use uplc::machine::{Context, MachineState};

#[derive(Default)]
pub struct App {
    pub file_name: PathBuf,
    pub cursor: usize,
    pub states: Vec<(MachineState, ExBudget)>,
    pub source_map: BTreeMap<u64, String>,
    pub exit: bool,
    pub focus: String,
    pub term_scroll: u16,
    pub context_scroll: u16,
    pub env_scroll: u16,
    pub return_scroll: u16,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut utils::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.exit = true;
                    }
                    KeyCode::Char('N') | KeyCode::Char('n') | KeyCode::Right => {
                        let stride = if key_event.modifiers.contains(event::KeyModifiers::SHIFT) {
                            50
                        } else {
                            1
                        };
                        let next = self.cursor + stride;
                        self.cursor = if next < self.states.len() {
                            next
                        } else {
                            self.states.len() - 1
                        };
                    }
                    KeyCode::Char('P') | KeyCode::Char('p') | KeyCode::Left => {
                        let stride = if key_event.modifiers.contains(event::KeyModifiers::SHIFT) {
                            50
                        } else {
                            1
                        };
                        self.cursor = self.cursor.saturating_sub(stride);
                    }
                    KeyCode::Tab => match self.focus.as_str() {
                        "Term" => self.focus = "Context".into(),
                        "Context" => self.focus = "Env".into(),
                        "Env" => self.focus = "Term".into(),
                        _ => {}
                    },
                    KeyCode::Up => match self.focus.as_str() {
                        "Term" => self.term_scroll = self.term_scroll.saturating_sub(1),
                        "Context" => self.context_scroll = self.context_scroll.saturating_sub(1),
                        "Env" => self.env_scroll = self.env_scroll.saturating_sub(1),
                        _ => {}
                    },
                    KeyCode::Down => match self.focus.as_str() {
                        "Term" => self.term_scroll = self.term_scroll.saturating_add(1),
                        "Context" => self.context_scroll = self.context_scroll.saturating_add(1),
                        "Env" => self.env_scroll = self.env_scroll.saturating_add(1),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = render_block_region(self.file_name.clone(), area, buf);

        let gauge_region = layout[0];
        let command_region = layout[1];
        let main_region = layout[2];

        render_gauge_region(self.cursor, &self.states, gauge_region, buf);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_region);
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(3)])
            .split(layout[0]);
        let term_region = left_layout[0];
        let source_region = left_layout[1];
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[1]);
        let context_region = right_layout[0];
        let env_region = right_layout[1];

        let curr_state = &self.states[self.cursor];
        let (label, context, env, term, ret_value) = match &curr_state.0 {
            MachineState::Return(context, value) => {
                let mut prev_idx = self.cursor - 1;
                while prev_idx > 0 {
                    if let MachineState::Compute(_, _, _) = &self.states[prev_idx].0 {
                        break;
                    }
                    prev_idx -= 1;
                }
                let last_state = &self.states[prev_idx];
                if let MachineState::Compute(_, env, term) = &last_state.0 {
                    ("Return", context, env, term, Some(value))
                } else {
                    return;
                }
            }
            MachineState::Compute(context, env, term) => ("Compute", context, env, term, None),
            MachineState::Done(term) => {
                if self.cursor == 0 {
                    return;
                }
                let mut prev_idx = self.cursor - 1;
                while prev_idx > 0 {
                    if let MachineState::Compute(_, _, _) = &self.states[prev_idx].0 {
                        break;
                    }
                    prev_idx -= 1;
                }
                let last_state = &self.states[prev_idx];
                if let MachineState::Compute(context, env, _) = &last_state.0 {
                    ("Done", context, env, term, None)
                } else {
                    return;
                }
            }
        };

        render_command_region(label, self.cursor, &self.states, command_region, buf);
        render_term_region(self.focus.clone(), term, self.term_scroll, term_region, buf);
        render_source_region(term.index(), &self.source_map, source_region, buf);
        render_context_region(
            self.focus.clone(),
            context,
            context_region,
            self.context_scroll,
            buf,
        );
        render_env_region(env, self.focus.clone(), self.env_scroll, env_region, buf);
        render_clear_popup_region(area, ret_value, buf);
    }
}

fn render_block_region(file_name: PathBuf, area: Rect, buf: &mut Buffer) -> Rc<[Rect]> {
    let title = Title::from(vec![
        " Gastronomy Debugger (".bold(),
        file_name.to_str().unwrap().bold(),
        ")".bold(),
    ]);
    let instructions = Title::from(Line::from(vec![
        " Next ".into(),
        "<N>".blue().bold(),
        " Previous ".into(),
        "<P>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]));

    let block = Block::default()
        .title(title.alignment(Alignment::Center))
        .title(
            instructions
                .alignment(Alignment::Center)
                .position(block::Position::Bottom),
        )
        .borders(Borders::ALL)
        .border_set(border::THICK);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Percentage(100),
        ])
        .split(block.inner(area));

    block.render(area, buf);
    layout
}

fn render_gauge_region(
    cursor: usize,
    states: &[(MachineState, ExBudget)],
    gauge_region: Rect,
    buf: &mut Buffer,
) {
    Gauge::default()
        .gauge_style(Style::default().fg(Color::Green))
        .label(format!("Step {}/{}", cursor, states.len() - 1))
        .ratio(cursor as f64 / states.len() as f64)
        .render(gauge_region, buf);
}

fn get_next(cursor: usize, states: &[(MachineState, ExBudget)]) -> &str {
    if cursor < states.len() - 1 {
        match &states[cursor + 1].0 {
            MachineState::Compute(_, _, _) => "Compute",
            MachineState::Return(_, _) => "Return",
            MachineState::Done(_) => "Done",
        }
    } else {
        "None"
    }
}

const MAX_CPU: i64 = 10000000000;
const MAX_MEM: i64 = 14000000;

fn render_command_region(
    label: &str,
    cursor: usize,
    states: &[(MachineState, ExBudget)],
    command_region: Rect,
    buf: &mut Buffer,
) {
    let next = get_next(cursor, states);

    let (cpu, mem) = {
        let curr_budget = states[cursor].1;
        (MAX_CPU - curr_budget.cpu, MAX_MEM - curr_budget.mem)
    };
    let (prev_cpu, prev_mem) = if cursor > 0 {
        let prev_budget = states[cursor - 1].1;
        (MAX_CPU - prev_budget.cpu, MAX_MEM - prev_budget.mem)
    } else {
        (0, 0)
    };

    Line::from(vec![
        "Current: ".into(),
        label.fg(Color::Blue).add_modifier(Modifier::BOLD),
    ])
    .left_aligned()
    .render(command_region, buf);
    Line::from(vec![
        "Budget: ".into(),
        format!("{} steps ", cpu)
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        if prev_cpu < cpu {
            format!("(+{}) ", cpu - prev_cpu).fg(Color::Green)
        } else {
            "".into()
        },
        format!("{} mem ", mem)
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        if prev_mem < mem {
            format!("(+{}) ", mem - prev_mem).fg(Color::Green)
        } else {
            "".into()
        },
    ])
    .centered()
    .render(command_region, buf);
    Line::from(vec![
        "Next: ".into(),
        next.fg(Color::Blue).add_modifier(Modifier::ITALIC),
    ])
    .right_aligned()
    .render(command_region, buf);
}

fn render_term_region(
    focus: String,
    term: &IndexedTerm<NamedDeBruijn>,
    mut term_scroll: u16,
    term_region: Rect,
    buf: &mut Buffer,
) {
    let term_block = Block::default()
        .title(" Term ".fg(if focus == "Term" {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::TOP | Borders::LEFT)
        .border_set(border::PLAIN);

    let term_text = term.to_string();
    let max_term_scroll = term_text.split('\n').count() as u16 - 1;
    if term_scroll > max_term_scroll {
        term_scroll = max_term_scroll;
    }

    Paragraph::new(term_text)
        .block(term_block)
        .scroll((term_scroll, 0))
        .render(term_region, buf);
}

fn render_source_region(
    index: Option<u64>,
    source_map: &BTreeMap<u64, String>,
    source_region: Rect,
    buf: &mut Buffer,
) {
    let source_block = Block::default()
        .title(" Source ".fg(Color::Reset))
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .border_set(border::PLAIN);

    let location = index
        .and_then(|i| source_map.get(&i))
        .cloned()
        .unwrap_or_default();

    Paragraph::new(location)
        .block(source_block)
        .render(source_region, buf);
}

fn render_context_region(
    focus: String,
    context: &Context,
    context_region: Rect,
    mut context_scroll: u16,
    buf: &mut Buffer,
) {
    let top_right_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        ..symbols::border::PLAIN
    };
    let context_block = Block::default()
        .title(" Context ".fg(if focus == "Context" {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_set(top_right_border_set);

    let context_text = utils::context_to_string(context.clone());
    let max_context_scroll = context_text.split('\n').count() as u16 - 1;
    if context_scroll > max_context_scroll {
        context_scroll = max_context_scroll;
    }

    Paragraph::new(context_text)
        .block(context_block)
        .scroll((context_scroll, 0))
        .render(context_region, buf);
}

fn render_env_region(
    env: &Rc<Vec<uplc::machine::value::Value>>,
    focus: String,
    mut env_scroll: u16,
    env_region: Rect,
    buf: &mut Buffer,
) {
    let collapsed_top_and_left_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };
    let env_block = Block::default()
        .title(" Env ".fg(if focus == "Env" {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::ALL)
        .border_set(collapsed_top_and_left_border_set);

    let env_text = utils::env_to_string(env);
    let max_env_scroll = env_text.split('\n').count() as u16 - 1;
    if env_scroll > max_env_scroll {
        env_scroll = max_env_scroll;
    }

    Paragraph::new(env_text)
        .block(env_block)
        .scroll((env_scroll, 0))
        .render(env_region, buf);
}

fn render_clear_popup_region(area: Rect, ret_value: Option<&Value>, buf: &mut Buffer) {
    if let Some(value) = ret_value {
        let ret_block = Block::default()
            .title(" Return Value ")
            .borders(Borders::ALL)
            .border_set(border::PLAIN);

        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };
        Clear.render(popup_area, buf);
        Paragraph::new(uplc::machine::discharge::value_as_term(value.clone()).to_string())
            .block(ret_block)
            .render(popup_area, buf);
    }
}
