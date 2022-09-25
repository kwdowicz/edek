use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug)]
pub struct Line {
    string: String,
    len: usize,
}

impl Line {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn update_len(&mut self) {
        self.len = self.string[..].graphemes(true).count();
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len {
            self.string.push(c);
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let reminder: String = self.string[..].graphemes(true).skip(at).collect();
            result.push(c);
            result.push_str(&reminder);
            self.string = result;
        }
        self.update_len();
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        } else {
            let mut result: String = self.string[..].graphemes(true).take(at).collect();
            let remainder: String = self.string[..].graphemes(true).skip(at + 1).collect();
            result.push_str(&remainder);
            self.string = result;
        }
        self.update_len();
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.update_len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let beginning: String = self.string[..].graphemes(true).take(at).collect();
        let remainder: String = self.string[..].graphemes(true).skip(at).collect();
        self.string = beginning;
        self.update_len();
        Self::from(&remainder[..])
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(end, self.string.len());
        let start = std::cmp::min(start, end);
        let mut result = String::new();
        for grapheme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            // result.push_str(grapheme);
            if grapheme == "\t" {
                result.push_str(" ");
            } else {
                result.push_str(grapheme);
            }
        }
        result
    }

    pub fn text(&self) -> &String {
        &self.string
    }
}

impl From<&str> for Line {
    fn from(slice: &str) -> Self {
        let mut line = Self {
            string: String::from(slice),
            len: 0,
        };
        line.update_len();
        line
    }
}

#[cfg(test)]
mod line_test {
    use super::*;

    #[test]
    fn default_line() {
        let line = Line::default();
        assert_eq!(line.string, "");
        assert_eq!(line.len, 0);
    }

    #[test]
    fn update_insert_delete_len_test() {
        let mut line = Line::default();
        line.insert(0, 'a');
        assert_eq!(line.len, 1);
        line.insert(1, 'b');
        assert_eq!(line.len, 2);
        line.insert(0, 'c');
        assert_eq!(line.len, 3);
        line.delete(0);
        assert_eq!(line.len, 2);
    }

    #[test]
    fn append_test() {
        let mut line1 = Line::default();
        let mut line2 = Line::default();
        line1.insert(0, 'a');
        line2.insert(0, 'b');
        line1.append(&line2);
        assert_eq!(line1.len, 2);
        assert_eq!(line1.string, "ab");
    }

    #[test]
    fn split_test() {
        let mut line = Line::default();
        line.insert(0, 'l');
        line.insert(0, 'i');
        line.insert(0, 'm');
        line.insert(0, 'a');
        line.insert(0, 'K');
        let line2 = line.split(3);
        assert_eq!(line.len, 3);
        assert_eq!(line2.len, 2);
    }
}
