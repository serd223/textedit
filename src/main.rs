use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    style::Stylize,
    symbols::border::{self},
    text::{Line, Span, ToSpan},
    widgets::{Block, List, ListState},
    DefaultTerminal,
};

/// Chars To String
fn cts(vec: &Vec<char>) -> String {
    let mut res = String::new();
    for u in vec.iter() {
        res.push(*u);
    }
    res
}

fn main() -> Result<(), std::io::Error> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Usage: {} <file-path>", args[0]);
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);

    if path.exists() && !path.is_file() {
        println!("{} exists and is not a file.", &args[1]);
        return Ok(());
    }

    let terminal = ratatui::init();

    let res = run(terminal, path);

    ratatui::restore();
    res
}

fn run(mut terminal: DefaultTerminal, file_path: PathBuf) -> Result<(), std::io::Error> {
    let mut text_edit = TextEdit::default();
    let source = {
        if file_path.exists() {
            std::fs::read_to_string(&file_path)?
        } else {
            String::new()
        }
    };
    let mut lines = source
        .lines()
        .map(|l| l.chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>();
    let mut selected_line = ListState::default();
    selected_line.select_first();
    if let Some(i) = selected_line.selected() {
        if i < lines.len() {
            text_edit.buf = lines[i].clone();
            // text_edit.buf = lines[i].clone();
        }
    }
    loop {
        terminal.draw(|f| {
            let file_str = if let Some(s) = file_path.to_str() {
                s.to_string()
            } else {
                "[ERROR]: Couldn't parse file path".to_string()
            };
            let view = Block::bordered()
                .border_set(border::DOUBLE)
                .title(file_str.to_span().blue())
                .title_bottom(
                    Line::from(vec![
                        "Save&Quit: ".to_span(),
                        "<ESC>".to_span().blue(),
                        " | Quit: ".to_span(),
                        "<C-q>".to_span().blue(),
                    ])
                    .centered(),
                );

            let input_str = cts(&text_edit.buf);
            let formatted_lines = lines.iter().map(|v| cts(v)).collect::<Vec<String>>();
            f.render_stateful_widget(
                formatted_lines
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let num_text = (i + 1).to_string();
                        let header = Span::raw(format!("{:>8}| ", {
                            if num_text.len() > 8 {
                                &num_text[..8]
                            } else {
                                &num_text
                            }
                        },))
                        .dim();
                        if let Some(is) = selected_line.selected() {
                            if is == i || (i == lines.len() - 1 && is == lines.len()) {
                                let j = text_edit.offset as usize;
                                if j < text_edit.buf.len() {
                                    let span1 = Span::raw(cts(&text_edit.buf[..j].to_vec()));
                                    let span2 = Span::raw(cts(&text_edit.buf[j..j + 1].to_vec()))
                                        .reversed();
                                    let line = {
                                        if j + 1 < text_edit.buf.len() {
                                            let span3 =
                                                Span::raw(cts(&text_edit.buf[j + 1..].to_vec()));
                                            Line::from(vec![header, span1, span2, span3])
                                        } else {
                                            Line::from(vec![header, span1, span2])
                                        }
                                    };

                                    line
                                } else if j == text_edit.buf.len() {
                                    let span1 = Span::raw(cts(&text_edit.buf[..].to_vec()));
                                    let span2 = Span::raw(" ").reversed();
                                    Line::from(vec![header, span1, span2])
                                } else {
                                    Line::from(vec![header, input_str.to_span()])
                                }
                            } else {
                                Line::from(vec![header, s.to_span()])
                            }
                        } else {
                            Line::from(vec![header, s.to_span()])
                        }
                    })
                    .collect::<List>()
                    .block(view),
                f.area(),
                &mut selected_line,
            );
        })?;

        // Use event::poll if this shouldn't block
        let event = event::read()?;
        if let Event::Key(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Backspace => {
                        if text_edit.offset > 0 {
                            text_edit.remove();
                        } else {
                            if let Some(current) = selected_line.selected() {
                                if current > 0 && current < lines.len() {
                                    let prev_len = lines[current - 1].len();
                                    for c in text_edit.buf.iter() {
                                        lines[current - 1].push(*c);
                                    }
                                    // lines[current - 1].push_str(&text_edit.buf);
                                    lines.remove(current);
                                    selected_line.scroll_up_by(1);
                                    text_edit.buf = lines[current - 1].clone();
                                    text_edit.offset = prev_len as u16;
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if lines.len() == 0 {
                            lines.push(Vec::new());
                            selected_line.select_first();
                            text_edit.buf = Vec::new();
                            text_edit.offset = 0;
                        } else {
                            if let Some(current) = selected_line.selected() {
                                if current < lines.len() {
                                    if (text_edit.offset as usize) < text_edit.buf.len() {
                                        lines.insert(
                                            current + 1,
                                            text_edit.buf.split_off(text_edit.offset as usize),
                                        );
                                        lines[current] = text_edit.buf.clone();
                                        text_edit.buf = lines[current + 1].clone();
                                    } else {
                                        lines.insert(current + 1, Vec::new());
                                        lines[current] = text_edit.buf.clone();
                                        text_edit.buf = Vec::new();
                                    }
                                    text_edit.offset = 0;
                                    selected_line.scroll_down_by(1);
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            if c == 'q' {
                                break;
                            }
                        } else if key_event.modifiers == KeyModifiers::SHIFT {
                            text_edit.insert(c.to_ascii_uppercase());
                        } else {
                            text_edit.insert(c);
                        }
                    }
                    KeyCode::Tab => {
                        // TODO and NOTE: ratatui doesn't seem to be able to render regular tabs ('\t')
                        // either make the tab width configurable or fix this somehow
                        text_edit.insert(' ');
                        text_edit.insert(' ');
                        text_edit.insert(' ');
                        text_edit.insert(' ');
                    }
                    KeyCode::Home => text_edit.offset = 0,
                    KeyCode::End => text_edit.offset = text_edit.buf.len() as u16,
                    KeyCode::Up => {
                        let prev_i = selected_line.selected();
                        selected_line.scroll_up_by(1);
                        let should_update = {
                            if let Some(prev_i) = prev_i {
                                if let Some(i) = selected_line.selected() {
                                    i != prev_i
                                } else {
                                    false
                                }
                            } else {
                                selected_line.selected().is_some()
                            }
                        };

                        if should_update {
                            if let Some(prev_i) = prev_i {
                                lines[prev_i] = text_edit.buf.clone();
                            }
                            let i = selected_line.selected().unwrap();
                            if i < lines.len() {
                                text_edit.buf = lines[i].clone();
                                text_edit.offset = text_edit.offset.min(text_edit.buf.len() as u16);
                            }
                        }
                    }
                    KeyCode::Down => {
                        let prev_i = selected_line.selected();
                        selected_line.scroll_down_by(1);
                        let should_update = {
                            if let Some(prev_i) = prev_i {
                                if let Some(i) = selected_line.selected() {
                                    i != prev_i
                                } else {
                                    false
                                }
                            } else {
                                selected_line.selected().is_some()
                            }
                        };

                        if should_update {
                            if let Some(prev_i) = prev_i {
                                lines[prev_i] = text_edit.buf.clone();
                            }
                            let i = selected_line.selected().unwrap();
                            if i < lines.len() {
                                text_edit.buf = lines[i].clone();
                                text_edit.offset = text_edit.offset.min(text_edit.buf.len() as u16);
                            }
                        }
                    }
                    KeyCode::Right => text_edit.right(),
                    KeyCode::Left => text_edit.left(),
                    KeyCode::Esc => {
                        let mut dest = String::new();
                        if let Some(i) = selected_line.selected() {
                            if i < lines.len() {
                                lines[i] = text_edit.buf.clone();
                            }
                        }

                        for l in lines {
                            dest.push_str(&cts(&l));
                            dest.push('\n');
                        }

                        std::fs::write(file_path, dest)?;

                        break;
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

#[derive(Default, Debug)]
struct TextEdit {
    buf: Vec<char>,
    offset: u16,
}

impl TextEdit {
    pub fn right(&mut self) {
        if (self.offset as usize) < self.buf.len() {
            self.offset += 1;
        }
    }
    pub fn left(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }

    pub fn insert(&mut self, c: char) {
        if self.offset as usize <= self.buf.len() {
            self.buf.insert(self.offset as usize, c);
            self.offset += 1;
        }
    }

    pub fn remove(&mut self) {
        if self.offset > 0 && (self.offset as usize) <= self.buf.len() {
            self.buf.remove(self.offset as usize - 1);

            self.offset -= 1;
        }
    }
}
