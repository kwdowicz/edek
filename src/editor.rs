use crate::{Content, Line, Terminal};
pub use log::{info, LevelFilter};
use std::env;
use std::io::stdout;
use termion::color;
use termion::{event::Key, raw::IntoRawMode};

const STATUS_FG_COLOR: color::Rgb = color::Rgb(32, 32, 32);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(202, 165, 216);
const CMD_PROMPT: &str = ">";

#[derive(Debug)]
pub enum Mode {
    Insert,
    Command,
}

#[derive(Default, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct Editor {
    quitting: bool,
    terminal: Terminal,
    position: Position,
    content: Content,
    offset: Position,
    status_message: String,
    mode: Mode,
    command_buf: String,
}

impl Editor {
    pub fn create() -> Self {
        let args: Vec<String> = env::args().collect();

        let content = if args.len() > 1 {
            Content::open(&args[1]).unwrap_or_default()
        } else {
            Content::new()
        };

        Self {
            quitting: false,
            terminal: Terminal::create().expect("Failed to initialize terminal"),
            position: Position::default(),
            content,
            offset: Position::default(),
            status_message: "".to_string(),
            mode: Mode::Insert,
            command_buf: "".to_string(),
        }
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        let _stdout = stdout().into_raw_mode()?;

        loop {
            if let Err(error) = self.refresh() {
                panic!("{}", error);
            }
            if let Err(error) = self.keypress() {
                panic!("{}", error);
            }
            if self.quitting {
                break;
            }
        }
        Ok(())
    }

    fn keypress(&mut self) -> Result<(), std::io::Error> {
        let key = Terminal::read_key()?;
        info!("Pressed: {:?}", key);
        match (key, &self.mode) {
            (Key::Ctrl('q'), Mode::Insert) => self.quitting = true,
            (Key::Delete, Mode::Insert) => self.content.delete(&self.position),
            (Key::Backspace, Mode::Insert) => {
                if self.position.x > 0 || self.position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.content.delete(&self.position);
                }
            }
            (Key::Char(c), Mode::Command) => {
                info!("Command mode input: {}", c);
                self.enter_command(c);
            }
            (Key::Esc, Mode::Command) => {
                self.enter_mode(Mode::Insert);
            }
            (Key::Esc, Mode::Insert) => {
                self.enter_mode(Mode::Command);
            }
            (Key::Char(c), Mode::Insert) => {
                self.content.insert(&self.position, c);
                if c == '\n' {
                    self.move_cursor(Key::Down);
                    self.move_cursor_repeat(Key::Left, self.position.x);
                } else {
                    self.move_cursor(Key::Right);
                }
            }
            (Key::Ctrl('s'), Mode::Insert) => {
                self.save_file();
            }
            (Key::Down | Key::Up | Key::Left | Key::Right, Mode::Insert) => self.move_cursor(key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn save_file(&mut self) {
        info!("Saving file: {}", self.content.filename());
        if self.content.save().is_err() {
            info!("Saving file: {} -> Error", self.content.filename());
            if self.content.filename().is_empty() {
                self.status_message = "Not saved! Need to specify the file name!".to_string();
            } else {
                self.status_message = "Not saved! Error saving the file!".to_string();
            }
        } else {
            self.status_message = format!("File saved as: {}", &self.content.filename());
            info!("Saving file: {} -> Saved", self.content.filename());
        }
    }

    fn enter_command(&mut self, c: char) {
        if c == '\n' {
            info!("Executing command: {}", &self.command_buf);
            let command: Vec<&str> = self.command_buf.as_str().split(' ').collect();
            info!("comand: {:#?}", command);
            match command[0] {
                "s" | "save" => {
                    if command.get(1).is_some() {
                        self.content.set_file_name(command.get(1).unwrap());
                    }
                    self.save_file();
                }
                "q" | "quit" => {
                    self.quitting = true;
                }
                _ => {
                    self.status_message = format!("Invalid command: {}", &self.command_buf);
                }
            }
            self.command_buf = "".to_string();
            self.enter_mode(Mode::Insert);
        } else {
            self.command_buf.push(c);
            self.status_message = format!("{} {}", CMD_PROMPT, self.command_buf);
        }
    }

    fn enter_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => {
                self.command_buf = "".to_string();
                self.mode = Mode::Insert;
                Terminal::cursor_show();
            }
            Mode::Command => {
                self.mode = Mode::Command;
                self.status_message = format!("{}", CMD_PROMPT);
                Terminal::cursor_hide();
            }
        }
    }

    fn move_cursor_repeat(&mut self, key: Key, repeat: usize) {
        for _ in 0..repeat {
            self.move_cursor(key);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.position;
        let height = self.content.lines_count();
        let mut width = if let Some(line) = self.content.line(y) {
            line.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height.saturating_sub(1) {
                    y = y.saturating_add(1)
                }
            }
            Key::Right => {
                if x < width {
                    x = x.saturating_add(1)
                }
            }
            Key::Left => x = x.saturating_sub(1),
            _ => (),
        }
        width = if let Some(line) = self.content.line(y) {
            line.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }
        self.position = Position { x, y }
    }

    fn refresh(&self) -> Result<(), std::io::Error> {
        //Terminal::cursor_hide(); // Why this is here?
        Terminal::cursor_position(&Position { x: 0, y: 0 });
        if self.quitting {
            Terminal::clear_screen();
            println!("Bye!\r");
        } else {
            self.draw_lines();
            Terminal::cursor_position(&Position {
                x: self.position.x.saturating_sub(self.offset.x),
                y: self.position.y.saturating_sub(self.offset.y),
            });
        }
        //Terminal::cursor_show(); // Why this is here?
        Terminal::flush()
    }

    fn draw_line(&self, line: &Line) {
        let start = self.offset.x;
        let end = self.width() + self.offset.x;
        let line = line.render(start, end);
        println!("{}\r", line);
    }

    fn draw_lines(&self) {
        for row in 0..self.height().saturating_sub(1) {
            Terminal::clear_current_line();
            if let Some(line) = self.content.line(row as usize + self.offset.y) {
                self.draw_line(line);
            } else {
                println!("~{}\r", row);
            }
        }
        self.draw_status_bar();
    }

    fn return_mode(mode: &Mode) -> String {
        match mode {
            Mode::Insert => "INSERT".to_string(),
            Mode::Command => "COMMAND".to_string(),
        }
    }

    fn draw_status_bar(&self) {
        let mut status_1;
        status_1 = format!(
            "[{}] [{}]",
            self.content.filename(),
            Editor::return_mode(&self.mode)
        );
        status_1 = format!(
            "{}{}",
            status_1,
            " ".repeat(self.width().saturating_sub(status_1.len()))
        );
        status_1.truncate(self.width());
        let status_2 = format!("[{}]", self.status_message);
        Terminal::clear_current_line();
        Terminal::set_fg_color(STATUS_FG_COLOR);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        println!("{}\r", status_1); // status bar 1
        Terminal::clear_current_line();
        print!("{}", status_2);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.position;
        if y < self.offset.y {
            self.offset.y = y;
        } else if y >= self.offset.y.saturating_add(self.height() - 1) {
            self.offset.y = y.saturating_sub(self.height() - 2);
        }
        if x < self.offset.x {
            self.offset.x = x;
        } else if x >= self.offset.x.saturating_add(self.width()) {
            self.offset.x = x.saturating_sub(self.width()).saturating_add(1);
        }
    }

    fn width(&self) -> usize {
        self.terminal.size().width as usize
    }

    fn height(&self) -> usize {
        self.terminal.size().height as usize
    }
}
