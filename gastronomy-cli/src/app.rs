use std::fmt::Display;
use std::iter;
use std::path::PathBuf;
use std::rc::Rc;
use std::{collections::BTreeMap, io};

use crate::utils;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use gastronomy::execution_trace::{ExBudget, RawFrame};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use uplc::ast::NamedDeBruijn;
use uplc::machine::indexed_term::IndexedTerm;
use uplc::machine::value::Value;
use uplc::machine::Context;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Term,
    Context,
    Env,
}
impl Default for Focus {
    fn default() -> Self {
        Self::Term
    }
}
impl Display for Focus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Term => "Term",
            Self::Context => "Context",
            Self::Env => "Env",
        };
        f.write_str(str)
    }
}

#[derive(Default)]
pub struct App<'a> {
    pub file_name: PathBuf,
    pub index: Option<usize>,
    pub cursor: usize,
    pub frames: Vec<RawFrame<'a>>,
    pub source_files: BTreeMap<String, String>,
    pub source_token_indices: Vec<usize>,
    pub view_source: bool,
    pub exit: bool,
    pub focus: Focus,
    pub term_scroll: u16,
    pub context_scroll: u16,
    pub env_scroll: u16,
    pub return_scroll: u16,
}

impl App<'_> {
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
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if key_event.modifiers.contains(event::KeyModifiers::SHIFT) {
                            if self.view_source {
                                self.cursor = self
                                    .source_token_indices
                                    .iter()
                                    .last()
                                    .copied()
                                    .unwrap_or(self.frames.len() - 1);
                            } else {
                                self.cursor = self.frames.len() - 1;
                            }
                        }
                    }
                    KeyCode::Char('N') | KeyCode::Char('n') | KeyCode::Right => {
                        if self.view_source {
                            self.cursor = self
                                .source_token_indices
                                .iter()
                                .find(|i| **i > self.cursor)
                                .copied()
                                .unwrap_or(self.frames.len() - 1);
                        } else {
                            let stride = if key_event.modifiers.contains(event::KeyModifiers::SHIFT)
                            {
                                50
                            } else {
                                1
                            };
                            let next = self.cursor + stride;
                            self.cursor = if next < self.frames.len() {
                                next
                            } else {
                                self.frames.len() - 1
                            };
                        };
                    }
                    KeyCode::Char('P') | KeyCode::Char('p') | KeyCode::Left => {
                        if self.view_source {
                            self.cursor = self
                                .source_token_indices
                                .iter()
                                .rev()
                                .find(|i| **i < self.cursor)
                                .copied()
                                .unwrap_or(0);
                        } else {
                            let stride = if key_event.modifiers.contains(event::KeyModifiers::SHIFT)
                            {
                                50
                            } else {
                                1
                            };
                            self.cursor = self.cursor.saturating_sub(stride);
                        }
                    }
                    KeyCode::Char('C') | KeyCode::Char('c') => {
                        let curr_frame = &self.frames[self.cursor];
                        let text = match self.focus {
                            Focus::Term => {
                                let term_text = curr_frame.term.to_string();
                                if self.view_source {
                                    curr_frame
                                        .location
                                        .map(|loc| {
                                            let (file, _, _) = parse_location(loc);
                                            self.source_files
                                                .get(file)
                                                .map(|c| c.as_str())
                                                .unwrap_or("File not found")
                                                .to_string()
                                        })
                                        .unwrap_or(term_text)
                                } else {
                                    term_text
                                }
                            }
                            Focus::Context => utils::context_to_string(curr_frame.context.clone()),
                            Focus::Env => utils::env_to_string(curr_frame.env),
                        };
                        if let Err(e) = terminal_clipboard::set_string(text) {
                            eprintln!("Could not copy to clipboard: {e}");
                        };
                    }
                    KeyCode::Char('v') => {
                        self.view_source = !self.view_source;
                    }
                    KeyCode::Tab => match self.focus {
                        Focus::Term => self.focus = Focus::Context,
                        Focus::Context => self.focus = Focus::Env,
                        Focus::Env => self.focus = Focus::Term,
                    },
                    KeyCode::Up => match self.focus {
                        Focus::Term => self.term_scroll = self.term_scroll.saturating_sub(1),
                        Focus::Context => {
                            self.context_scroll = self.context_scroll.saturating_sub(1)
                        }
                        Focus::Env => self.env_scroll = self.env_scroll.saturating_sub(1),
                    },
                    KeyCode::Down => match self.focus {
                        Focus::Term => self.term_scroll = self.term_scroll.saturating_add(1),
                        Focus::Context => {
                            self.context_scroll = self.context_scroll.saturating_add(1)
                        }
                        Focus::Env => self.env_scroll = self.env_scroll.saturating_add(1),
                    },
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let curr_frame = &self.frames[self.cursor];
        let label = curr_frame.label;
        let context = curr_frame.context;
        let env = curr_frame.env;
        let term = curr_frame.term;
        let location = curr_frame.location;
        let ret_value = curr_frame.ret_value;

        let layout = render_block_region(
            self.file_name.clone(),
            self.index,
            location,
            area,
            self.focus,
            buf,
        );

        let gauge_region = layout[0];
        let command_region = layout[1];
        let main_region = layout[2];

        render_gauge_region(self.cursor, &self.frames, gauge_region, buf);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_region);
        let term_region = layout[0];
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[1]);
        let context_region = layout[0];
        let env_region = layout[1];

        render_command_region(label, self.cursor, &self.frames, command_region, buf);
        render_term_region(
            self.focus,
            term,
            location,
            &self.source_files,
            self.term_scroll,
            self.view_source,
            term_region,
            buf,
        );
        render_context_region(
            self.focus,
            context,
            context_region,
            self.context_scroll,
            buf,
        );
        render_env_region(env, self.focus, self.env_scroll, env_region, buf);
        render_clear_popup_region(area, ret_value, buf);
    }
}

fn render_block_region(
    file_name: PathBuf,
    index: Option<usize>,
    location: Option<&String>,
    area: Rect,
    focus: Focus,
    buf: &mut Buffer,
) -> Rc<[Rect]> {
    let title = Line::from(vec![
        " Gastronomy Debugger (".bold(),
        file_name.to_str().unwrap().bold(),
        index.map(|i| format!(" #{i}")).unwrap_or_default().bold(),
        ")".bold(),
    ]);
    let mut instructions = if location.is_some() {
        vec![" View Source ".into(), "<V>".blue().bold()]
    } else {
        vec![]
    };
    instructions.extend([
        format!(" Copy {focus} ").into(),
        "<C>".blue().bold(),
        " Next ".into(),
        "<N>".blue().bold(),
        " Previous ".into(),
        "<P>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    let instructions = Line::from(instructions);

    let block = Block::default()
        .title(title.centered())
        .title_bottom(instructions.centered())
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
    frames: &[RawFrame<'_>],
    gauge_region: Rect,
    buf: &mut Buffer,
) {
    Gauge::default()
        .gauge_style(Style::default().fg(Color::Green))
        .label(format!("Step {}/{}", cursor, frames.len() - 1))
        .ratio(cursor as f64 / frames.len() as f64)
        .render(gauge_region, buf);
}

fn get_next<'a>(cursor: usize, frames: &[RawFrame<'a>]) -> &'a str {
    if cursor < frames.len() - 1 {
        frames[cursor + 1].label
    } else {
        "None"
    }
}

fn render_command_region(
    label: &str,
    cursor: usize,
    frames: &[RawFrame<'_>],
    command_region: Rect,
    buf: &mut Buffer,
) {
    let next = get_next(cursor, frames);

    let ExBudget {
        steps,
        mem,
        steps_diff,
        mem_diff,
    } = frames[cursor].budget;

    Line::from(vec![
        "Current: ".into(),
        label.fg(Color::Blue).add_modifier(Modifier::BOLD),
    ])
    .left_aligned()
    .render(command_region, buf);
    Line::from(vec![
        "Budget: ".into(),
        format!("{} steps ", steps)
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        if steps_diff > 0 {
            format!("(+{}) ", steps_diff).fg(Color::Green)
        } else {
            "".into()
        },
        format!("{} mem ", mem)
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
        if mem_diff > 0 {
            format!("(+{}) ", mem_diff).fg(Color::Green)
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

#[allow(clippy::too_many_arguments)]
fn render_term_region(
    focus: Focus,
    term: &IndexedTerm<NamedDeBruijn>,
    location: Option<&String>,
    source_files: &BTreeMap<String, String>,
    mut term_scroll: u16,
    view_source: bool,
    term_region: Rect,
    buf: &mut Buffer,
) {
    let title = if view_source { " Source " } else { " Term " };
    let term_block = Block::default()
        .title(title.fg(if focus == Focus::Term {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .border_set(border::PLAIN);

    if let Some(location) = location {
        // holding onto this so the lifespan of the Line<'_>s outlive the conditional where we made them
        #[allow(unused_assignments)]
        let mut term_text = String::new();

        let term_lines = if view_source {
            let (file, line, column) = parse_location(location);

            let old_term_text = source_files
                .get(file)
                .map(|c| c.as_str())
                .unwrap_or("File not found");
            // to highlight lines properly, each line needs to take up the full width of its region
            term_text = pad_lines_with_spaces(old_term_text, term_region.width as usize);
            highlight_text(&term_text, line, column)
        } else {
            term_text = term.to_string();
            split_text(&term_text)
        };

        let max_term_scroll = term_lines.len() as u16 - 1;
        if term_scroll > max_term_scroll {
            term_scroll = max_term_scroll;
        }

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100), Constraint::Min(3)])
            .split(term_region);

        let term_region = layout[0];
        let source_region = layout[1];

        let term_block = term_block.borders(Borders::TOP | Borders::LEFT);
        Paragraph::new(term_lines)
            .block(term_block)
            .scroll((term_scroll, 0))
            .render(term_region, buf);
        render_source_region(location, source_region, buf);
    } else {
        let term_text = term.to_string();
        let max_term_scroll = term_text.lines().count() as u16 - 1;
        if term_scroll > max_term_scroll {
            term_scroll = max_term_scroll;
        }

        Paragraph::new(term_text)
            .block(term_block)
            .scroll((term_scroll, 0))
            .render(term_region, buf);
    }
}

fn render_source_region(location: &str, source_region: Rect, buf: &mut Buffer) {
    let source_block = Block::default()
        .title(" Source Location ".fg(Color::Reset))
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .border_set(border::PLAIN);

    Paragraph::new(location)
        .block(source_block)
        .render(source_region, buf);
}

fn render_context_region(
    focus: Focus,
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
        .title(" Context ".fg(if focus == Focus::Context {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_set(top_right_border_set);

    let context_text = utils::context_to_string(context.clone());
    let max_context_scroll = context_text.lines().count() as u16 - 1;
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
    focus: Focus,
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
        .title(" Env ".fg(if focus == Focus::Env {
            Color::Blue
        } else {
            Color::Reset
        }))
        .borders(Borders::ALL)
        .border_set(collapsed_top_and_left_border_set);

    let env_text = utils::env_to_string(env);
    let line_count = env_text.lines().count() as u16;
    let max_env_scroll = if line_count == 0 {
        line_count
    } else {
        line_count - 1
    };
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

fn split_text(text: &str) -> Vec<Line<'_>> {
    text.lines().map(|i| i.into()).collect()
}

fn pad_lines_with_spaces(text: &str, line_width: usize) -> String {
    let mut result: Vec<String> = vec![];
    for line in text.lines() {
        let full_lines = line.len() / line_width;
        let remaining_chars = line.len() % line_width;
        let total_lines = full_lines + if remaining_chars > 0 { 1 } else { 0 };
        let total_chars = total_lines * line_width;
        result.push(
            line.chars()
                .chain(iter::repeat(' '))
                .take(total_chars)
                .collect(),
        );
    }
    result.join("\n")
}

fn highlight_text(text: &str, line: usize, column: usize) -> Vec<Line<'_>> {
    text.split('\n')
        .enumerate()
        .map(|(line_number, line_text)| {
            if line_number + 1 != line {
                line_text.into()
            } else {
                let (before, at_after) = line_text.split_at(column - 1);
                let (at, after) = at_after.split_at(1);

                vec![
                    before.bg(Color::DarkGray),
                    at.bg(Color::Gray).underlined(),
                    after.bg(Color::DarkGray),
                ]
                .into()
            }
        })
        .collect()
}

fn parse_location(location: &str) -> (&str, usize, usize) {
    let mut pieces = location.split(":");
    let file = pieces.next().unwrap();
    let line = pieces.next().unwrap().parse().unwrap();
    let column = pieces.next().unwrap().parse().unwrap();
    (file, line, column)
}
