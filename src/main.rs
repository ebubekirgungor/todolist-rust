use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetSize,
        SetTitle,
    },
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;
use std::{error::Error, io};
use winreg::enums::*;
use winreg::RegKey;
enum InputMode {
    Normal,
    Editing,
    Updating,
}

#[derive(Serialize, Deserialize)]
struct Todo {
    id: usize,
    text: String,
    done: bool,
}

struct Editing {
    edit: bool,
}

struct App {
    input: String,
    cursor_position: usize,
    input_mode: InputMode,
    count: usize,
    todos: Vec<Todo>,
    editing: Vec<Editing>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            cursor_position: 0,
            count: 0,
            todos: Vec::new(),
            editing: Vec::new(),
        }
    }
}

impl App {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) {
        let todo = Todo {
            id: self.count,
            text: String::from(self.input.clone()),
            done: false,
        };
        let edit = Editing { edit: false };
        self.editing.push(edit);
        self.todos.insert(self.count, todo);
        self.input.clear();
        self.reset_cursor();
        self.count += 1;
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        SetSize(60, 40),
        SetTitle("Todolist")
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = Path::new("SOFTWARE").join("todolist");
    let (key, _disp) = hkcu.create_subkey_with_flags(&path, KEY_ALL_ACCESS)?;
    let todos: String;

    match key.get_value::<String, _>("todos") {
        Ok(_) => todos = key.get_value("todos")?,
        Err(_) => {
            key.set_value("todos", &"[]")?;
            todos = key.get_value("todos")?
        }
    }

    let todo: Vec<Todo> = serde_json::from_str(&todos)?;
    app.todos = todo;
    app.count = app.todos.len();
    let mut i = 0;
    while i < app.count {
        let edit = Editing { edit: false };
        app.editing.push(edit);
        i += 1;
    }
    loop {
        let json = serde_json::to_string(&app.todos)?;
        key.set_value("todos", &json)?;
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => app.submit_message(),
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    _ => {}
                },
                InputMode::Updating if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        app.input_mode = InputMode::Normal;
                        let mut i = 0;
                        while i < app.todos.len() {
                            if app.editing[i].edit {
                                app.editing[i].edit = false;
                            }
                            i += 1;
                        }
                    }
                    KeyCode::Char(to_insert) => {
                        let mut i = 0;
                        while i < app.todos.len() {
                            if app.editing[i].edit {
                                let len = app.todos[i].text.len();
                                app.todos[i].text.insert((len) as usize, to_insert);
                            }
                            i += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        let mut i = 0;
                        while i < app.todos.len() {
                            if app.editing[i].edit {
                                app.todos[i].text.pop();
                            }
                            i += 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else if let Event::Mouse(mouse_event) = event::read()? {
            match mouse_event {
                MouseEvent {
                    kind,
                    column,
                    row,
                    modifiers: _,
                } => {
                    if kind == MouseEventKind::Up(MouseButton::Left)
                        || kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        if column >= 1 && row == 3 || row == 2 || row == 1 {
                            app.input_mode = InputMode::Editing;
                        } else {
                            app.input_mode = InputMode::Normal;
                        }
                        let mut i = 0;
                        let mut rw = 0;
                        while i < app.todos.len() {
                            if i == 0 {
                                if (column == 3 || column == 4) && row == i as u16 + 5 {
                                    app.todos[i].done = !app.todos[i].done;
                                } else if column == 56 && row == i as u16 + 5 {
                                    app.todos.remove(i);
                                    app.count -= 1;
                                    let mut j = 0;
                                    while j < app.todos.len() {
                                        app.todos[j].id = j;
                                        j += 1;
                                    }
                                } else if column > 5 && column < 56 && row == i as u16 + 5 {
                                    app.editing[i].edit = true;
                                    app.input_mode = InputMode::Updating;
                                } else {
                                    app.editing[i].edit = false;
                                }
                            } else {
                                if (column == 3 || column == 4) && row == 5 + rw {
                                    app.todos[i].done = !app.todos[i].done;
                                } else if column == 56 && row == 5 + rw {
                                    app.todos.remove(i);
                                    app.count -= 1;
                                    let mut j = 0;
                                    while j < app.todos.len() {
                                        app.todos[j].id = j;
                                        j += 1;
                                    }
                                } else if column > 5 && column < 56 && row == 5 + rw {
                                    app.editing[i].edit = true;
                                    app.input_mode = InputMode::Updating;
                                } else {
                                    app.editing[i].edit = false;
                                }
                            }
                            i += 1;
                            rw += 3;
                        }
                        let json = serde_json::to_string(&app.todos)?;
                        key.set_value("todos", &json)?;
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let mut constraints = Vec::new();
    constraints.push(Constraint::Length(1));
    for _ in 0..17 {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_slice().as_ref())
        .split(f.size());

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Updating => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Add Todo ")
                .add_modifier(Modifier::BOLD)
                .padding(Padding::new(1, 0, 0, 0))
                .title_alignment(Alignment::Center),
        );
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Updating => {}
        InputMode::Editing => f.set_cursor(
            chunks[1].x + app.cursor_position as u16 + 2,
            chunks[1].y + 1,
        ),
    }

    for todo in app.todos.iter() {
        let t: &Todo = todo;
        let mut i = 0;
        let mut space = String::new();
        while i < 48 - t.text.len() {
            space += " ";
            i += 1;
        }

        f.render_widget(
            Paragraph::new(if t.done {
                "[./] ".to_owned() + &t.text.to_string() + &space + "[x] "
            } else {
                "[  ] ".to_owned() + &t.text.to_string() + &space + "[x] "
            })
            .style(if app.editing[t.id].edit {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .add_modifier(Modifier::BOLD)
                    .padding(Padding::new(1, 0, 0, 0)),
            ),
            chunks[2 + t.id],
        );

        if app.editing[t.id].edit {
            f.set_cursor(
                chunks[2 + t.id].x + t.text.len() as u16 + 7,
                chunks[2 + t.id].y + 1,
            );
        }
    }
}
