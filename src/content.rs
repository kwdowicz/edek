use crate::Line;
use crate::Position;
use log::info;
use std::fs;
use std::io::{Error, ErrorKind, Write};

#[derive(Default, Debug)]
pub struct Content {
    lines: Vec<Line>,
    filename: String,
}

impl Content {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in contents.lines() {
            lines.push(Line::from(line))
        }
        Ok(Self {
            lines,
            filename: String::from(filename),
        })
    }

    pub fn new() -> Self {
        Self {
            lines: vec![],
            filename: "".to_string(),
        }
    }

    pub fn line(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn lines_count(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.lines_count() {
            return;
        }
        if at.y == self.lines_count() {
            self.lines.push(Line::default());
            return;
        }
        let new_line = self.lines.get_mut(at.y).unwrap().split(at.x);
        self.lines.insert(at.y + 1, new_line);
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        info!(
            "Inserting at: {:?}[{:?}]:{:?}[{:?}] => {:?}",
            at.x,
            0,
            at.y,
            self.lines_count(),
            c
        );
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.lines_count() {
            let mut line = Line::default();
            line.insert(0, c);
            self.lines.push(line);
        } else {
            let line = self.lines.get_mut(at.y).unwrap();
            line.insert(at.x, c);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.lines_count();
        if at.y >= len {
            return;
        }
        if at.x == self.lines.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_line = self.lines.remove(at.y + 1);
            let line = self.lines.get_mut(at.y).unwrap();
            line.append(&next_line);
        } else {
            let row = self.lines.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        if self.filename.is_empty() {
            Err(Error::new(ErrorKind::Other, "No filename provided"))
        } else {
            let mut file = fs::File::create(&self.filename)?;
            for line in &self.lines {
                file.write_all(line.text().as_bytes())?;
                file.write_all(b"\n")?;
            }
            Ok(())
        }
    }

    pub fn set_file_name(&mut self, name: &str) {
        self.filename = name.to_string();
    }
}
