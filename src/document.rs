use std::io;

#[derive(Debug)]
pub struct Document {
    pub lines: Vec<String>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }
}

impl Document {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let lines = content
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(Self { 
            lines: if lines.is_empty() {
                vec![String::new()]
            } else {
                lines
            },
         })
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        let content = self.lines.join("\n");
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn insert_char(&mut self, x: usize, y: usize, c: char,) {
        self.lines[y].insert(x, c);
    }

    pub fn split_line(&mut self, x: usize, y: usize,) {
        let new_line = self.lines[y].split_off(x);

        self.lines.insert(
            y + 1,
            new_line,
        );
    }

    pub fn remove_char(&mut self, x: usize, y: usize,) {
        self.lines[y].remove(x);
    }

    pub fn backspace(&mut self, x: usize, y: usize,) -> (usize, usize) {

        if x > 0 {
            self.lines[y].remove(x - 1);

            return (x - 1, y);
        }

        if y > 0 {
            let current_line = self.lines.remove(y);

            let previous_length = self.lines[y - 1].len();

            self.lines[y - 1].push_str(&current_line);

            return (previous_length, y - 1);
        }

        (x, y)
    }
}