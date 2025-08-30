pub mod parser;
pub mod tokenizer;
pub mod unificator;
pub mod solver;

use std::{fs::File, io::{Read, Write}, panic, path::PathBuf, time::{Duration, Instant}};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use std::io;
use crate::parser::{build_database, Parser};
use crate::solver::{extract_query_results, get_query_vars};
use crate::tokenizer::{tokenize, Statement};

#[derive(PartialEq)]
enum Focus {
    Editor,
    Console,
}

struct App {
    editor: Vec<String>,
    console_input: String,
    output: Vec<String>,
    editor_scroll: u16,
    console_scroll: u16,
    output_scroll: u16,
    focus: Focus,
    cursor_x: usize,
    cursor_y: usize,
    console_cursor_x: usize,
    editor_width: u16,
    console_width: u16,
    top_height: u16,
    output_height: u16,
}

impl App {
    fn new() -> Self {
        Self {
            editor: vec![String::new()],
            console_input: String::new(),
            output: vec!["Welcome! Type --help in Console.".to_string()],
            editor_scroll: 0,
            console_scroll: 0,
            output_scroll: 0,
            focus: Focus::Editor,
            cursor_x: 0,
            cursor_y: 0,
            console_cursor_x: 0,
            editor_width: 50,
            console_width: 50,
            top_height: 70,
            output_height: 30,
        }
    }

    fn evaluate_query(&self, query_str: &str) -> Vec<String> {
        let db_text = self.editor.join("\n");
        let tokens = tokenize(&db_text);
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse_program();

        let query = {
            let tokens = tokenize(query_str);
            let mut parser = Parser::new(tokens);
            match parser.parse_statement() {
                Statement::Query { body } => body,
                _ => return vec!["Expected a query!".to_string()],
            }
        };

        let tree = solver::resolve_query(&query, &stmts);
        let query_vars = get_query_vars(&query);
        let results = extract_query_results(&tree, &query_vars);

        if results.is_empty() {
            vec!["No solutions.".to_string()]
        } else {
            results
                .into_iter()
                .map(|subs| {
                    subs.into_iter()
                        .map(|(var, term)| format!("{} = {:?}", var, term))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .collect()
        }
    }


    fn insert_char(&mut self, c: char) {
        if self.focus == Focus::Editor {
            let line = &mut self.editor[self.cursor_y];
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
        } else {
            self.console_input.insert(self.console_cursor_x, c);
            self.console_cursor_x += 1;
        }
    }

    fn backspace(&mut self) {
        if self.focus == Focus::Editor {
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
                self.editor[self.cursor_y].remove(self.cursor_x);
            } else if self.cursor_y > 0 {
                let prev_len = self.editor[self.cursor_y - 1].len();
                let line = self.editor.remove(self.cursor_y);
                self.cursor_y -= 1;
                self.cursor_x = prev_len;
                self.editor[self.cursor_y].push_str(&line);
            }
        } else if self.console_cursor_x > 0 {
            self.console_cursor_x -= 1;
            self.console_input.remove(self.console_cursor_x);
        }
    }
}

fn main() -> Result<(), io::Error> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    let mut last_key: Option<KeyCode> = None;
    let mut last_time = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(app.top_height.into()),
                    Constraint::Percentage(app.output_height.into()),
                ])
                .split(f.size());

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(app.editor_width.into()),
                    Constraint::Percentage(app.console_width.into()),
                ])
                .split(chunks[0]);

            // Editor
            let editor_style = if app.focus == Focus::Editor {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let editor_text = app.editor.join("\n");
            let editor_widget = Paragraph::new(editor_text)
                .block(Block::default().title("Editor").borders(Borders::ALL).style(editor_style))
                .scroll((app.editor_scroll, 0))
                .wrap(Wrap { trim: false });
            f.render_widget(editor_widget, top_chunks[0]);

            // Console
            let console_style = if app.focus == Focus::Console {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let console_widget = Paragraph::new(app.console_input.as_str())
                .block(Block::default().title("Console").borders(Borders::ALL).style(console_style))
                .style(Style::default().fg(Color::Rgb(0, 100, 0)))
                .scroll((app.console_scroll, 0))
                .wrap(Wrap { trim: false });
            f.render_widget(console_widget, top_chunks[1]);

            // Output
            let output_widget = Paragraph::new(app.output.join("\n"))
                .block(Block::default()
                        .title("Output")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),)
                .style(Style::default().fg(Color::Yellow))
                .scroll((app.output_scroll, 0))
                .wrap(Wrap { trim: false });
            f.render_widget(output_widget, chunks[1]);

            // Set cursor position only (visibility controlled outside)
            if app.focus == Focus::Editor {
                let x = app.cursor_x as u16 + 1;
                let y = (app.cursor_y as u16)
                    .saturating_sub(app.editor_scroll)
                    .min(top_chunks[0].height.saturating_sub(2))
                    + 1;
                f.set_cursor(x, y);
            }
        })?;

        // Control cursor visibility outside draw
        if app.focus == Focus::Editor {
            terminal.show_cursor().ok();
        } else {
            terminal.hide_cursor().ok();
        }

        // Handle Input
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            let now = Instant::now();
            if last_key == Some(code)
                && now.duration_since(last_time) < Duration::from_millis(80)
            {
                continue;
            }
            last_key = Some(code);
            last_time = now;

            match code {
                KeyCode::F(1) => break,
                KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                    //app.output.push("File saved!".to_string());
                }
                KeyCode::Left if modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::Editor;
                }
                KeyCode::Right if modifiers.contains(KeyModifiers::CONTROL) => {
                    app.focus = Focus::Console;
                }
                KeyCode::Up => match app.focus {
                    Focus::Editor => {
                        if app.cursor_y > 0 { app.cursor_y -= 1; }
                        if app.editor_scroll > 0 { app.editor_scroll -= 1; }
                    }
                    Focus::Console => {
                        //if app.console_scroll > 0 { app.console_scroll -= 1; }
                        if app.output_scroll > 0 {
                            app.output_scroll -= 1;
                        }
                    }
                },
                KeyCode::Down => match app.focus {
                    Focus::Editor => {
                        app.cursor_y += 1;
                        if app.cursor_y >= app.editor.len() {
                            app.editor.push(String::new());
                        }
                        app.editor_scroll += 1;
                    }
                    Focus::Console => {
                        //app.console_scroll += 1;
                        app.output_scroll += 1;
                    }
                },
                KeyCode::Char('+') if modifiers.contains(KeyModifiers::CONTROL) => { // no working
                    if app.editor_width < 80 {
                        app.editor_width += 5;
                        app.console_width = app.console_width.saturating_sub(5);
                    }
                }
                KeyCode::Char('-') if modifiers.contains(KeyModifiers::CONTROL) => { // not working
                    if app.console_width < 80 {
                        app.console_width += 5;
                        app.editor_width = app.editor_width.saturating_sub(5);
                    }
                }
                KeyCode::Char('[') => {
                    if app.top_height > 20 {
                        app.top_height -= 5;
                        app.output_height += 5;
                    }
                }
                KeyCode::Char(']') => {
                    if app.output_height > 20 {
                        app.output_height -= 5;
                        app.top_height += 5;
                    }
                }
                KeyCode::Char(c) => app.insert_char(c),
                KeyCode::Backspace => app.backspace(),
                KeyCode::Enter => {
                    if app.focus == Focus::Editor {
                        let line = app.editor[app.cursor_y].split_off(app.cursor_x);
                        app.cursor_x = 0;
                        app.cursor_y += 1;
                        app.editor.insert(app.cursor_y, line);
                    } else {
                        let cmd = app.console_input.trim().to_string();
                        if !cmd.is_empty() {
                            match cmd.as_str() {
                                "--help" => app.output.push(
                                    //--load <filename>       Load database in editor\n\
                                    //Ctrl+S       Save\n\
                                    "Key bindings:\n\
F1            Quit\n\
Ctrl+←/→     Switch focus\n\
↑/↓          Scroll active pane\n\
+ / -        Resize Editor vs Console\n\
[ / ]        Resize Top vs Output\n\
Enter        Newline (Editor) / Run (Console)\n\
--help       Show this help text"
                                        .to_string(),
                                ),
                                _ => { //app.output.push(format!("> {}", cmd)),
                                    let result = std::panic::catch_unwind(|| {
                                        app.evaluate_query(&cmd)
                                    });

                                    match result {
                                        Ok(output_vec) => {
                                            app.output.push(format!("> {}", cmd));
                                            app.output.extend(output_vec);
                                        }
                                        Err(_) => {
                                            app.output.push("Error: not a query or a command!".to_string());
                                        }
                                    }

                                    app.console_input.clear();
                                    app.console_cursor_x = 0;
                                }
                            }
                            app.console_input.clear();
                            app.console_cursor_x = 0;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    Ok(())
}
