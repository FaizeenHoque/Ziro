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

    pub fn insert_char(&mut self, x: usize, y: usize, c: char) {
        self.lines[y].insert(x, c);
    }

    pub fn insert_str(&mut self, x: usize, y: usize, text: &str) -> (usize, usize) {
        let normalized = text.replace("\r\n", "\n");
        let parts: Vec<&str> = normalized.split('\n').collect();

        if parts.len() == 1 {
            self.lines[y].insert_str(x, parts[0]);
            return (x + parts[0].len(), y);
        }

        let line = &self.lines[y];
        let before = line[..x].to_string();
        let after = line[x..].to_string();

        self.lines[y] = format!("{}{}", before, parts[0]);

        let mut insert_at = y + 1;
        for part in &parts[1..parts.len() - 1] {
            self.lines.insert(insert_at, part.to_string());
            insert_at += 1;
        }

        let last_part = parts[parts.len() - 1];
        let final_line = format!("{}{}", last_part, after);
        self.lines.insert(insert_at, final_line);

        (last_part.len(), insert_at)
    }

    pub fn split_line(&mut self, x: usize, y: usize) {
        let new_line = self.lines[y].split_off(x);

        self.lines.insert(y + 1, new_line);
    }

    pub fn remove_char(&mut self, x: usize, y: usize) {
        self.lines[y].remove(x);
    }

    pub fn backspace(&mut self, x: usize, y: usize) -> (usize, usize) {
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

    pub fn extract_range(&self, start: (usize, usize), end: (usize, usize)) -> String {
        let (sx, sy) = start;
        let (ex, ey) = end;

        if sy == ey {
            return self.lines[sy][sx..ex].to_string();
        }

        let mut result = String::new();
        result.push_str(&self.lines[sy][sx..]);
        result.push('\n');

        for line in &self.lines[sy + 1..ey] {
            result.push_str(line);
            result.push('\n');
        }

        result.push_str(&self.lines[ey][..ex]);
        result
    }

    pub fn delete_range(&mut self, start: (usize, usize), end: (usize, usize)) -> (usize, usize) {
        let (sx, sy) = start;
        let (ex, ey) = end;

        if sy == ey {
            self.lines[sy].replace_range(sx..ex, "");
            return (sx, sy);
        }

        let tail = self.lines[ey][ex..].to_string();
        let head = self.lines[sy][..sx].to_string();

        // Remove the fully-enclosed middle lines and the end line,
        // then splice head+tail into the start line.
        self.lines.drain(sy + 1..=ey);
        self.lines[sy] = format!("{}{}", head, tail);

        (sx, sy)
    }
}
