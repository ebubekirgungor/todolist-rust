use std::{error::Error, io};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json;
use winreg::enums::*;
use winreg::RegKey;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, SetSize, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
enum InputMode {
    Normal,
    Editing,
}

#[derive(Serialize, Deserialize)]
struct Todo {
    id: usize,
    text: String,
    done: bool,
}

struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: HashMap<usize, Todo>,
    count: usize,
}
/*pub struct Todo {
    id: i32,
    text: String,
    done: bool,
}*/

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: HashMap::new(),
            cursor_position: 0,
            count: 0,
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
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
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

    
    
    fn submit_message(&mut self) -> Result<(), Box<dyn std::error::Error>>  {
        //let todo = get_todos();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let todokey = hkcu.open_subkey("SOFTWARE\\todolist")?;
        let todos: String = todokey.get_value("todos")?;
        let todo: Vec<Todo> = serde_json::from_str(&todos)?;
        for todo in todo.iter() {
            let id: usize = todo.id;
            let text: String = todo.text;
            let done: bool = todo.done;
            let todo = Todo {
                id: id,
                text: text,
                done: done,
            };
            self.messages.insert(id, todo);
        }

        let todo = Todo {
            id: self.count,
            text: String::from(self.input.clone()),
            done: false,
        };
        self.messages.insert(self.count, todo);
        self.input.clear();
        self.reset_cursor();
        self.count = self.count + 1;
        Ok(())
    }
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, SetSize(60, 40))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
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
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
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
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    /*let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit, ".into(),
                "e".bold(),
                " to start editing.".bold(),
            ],
            Style::default().add_modifier(Modifier::BOLD),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default().add_modifier(Modifier::BOLD),
        ),
    };
    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);*/

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input").add_modifier(Modifier::BOLD).padding(Padding::new(1, 0, 0, 0)).title_alignment(Alignment::Center));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            f.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                chunks[1].x + app.cursor_position as u16 + 2,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    //let todos_block = Block::default().borders(Borders::ALL).add_modifier(Modifier::BOLD);


    /*app.messages.iter().map(|(i, t)| {
        //let (i, m): (&usize, &String) = todo;
        f.render_widget(Paragraph::new(t.text.to_string()).block(Block::default().borders(Borders::ALL).add_modifier(Modifier::BOLD)), chunks[2+i]);
    }).collect();*/


    for todo in app.messages.iter() {
        let (i, t): (&usize, &Todo) = todo;
        f.render_widget(Paragraph::new(if t.done { "[./] ".to_owned() + &t.text.to_string()} else {"[  ] ".to_owned() + &t.text.to_string()} ).block(Block::default().borders(Borders::ALL).add_modifier(Modifier::BOLD).padding(Padding::new(1, 0, 0, 0))), chunks[2+i]);
    }

    /*let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|(i, m)| {
            let content = Line::from(Span::raw(format!("{i}: {m}")));
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).add_modifier(Modifier::BOLD));
    f.render_widget(messages, chunks[2]);*/
}