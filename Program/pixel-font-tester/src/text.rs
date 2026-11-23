// The text in the tester's box: lines of ASCII, and the caret between their
// characters. Each character is a byte. Only what the font can draw goes in:
// ASCII's printable characters and tab, which has the box.

// A place between characters: before the character at `column` on `line`, or
// at the end of the line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    lines: Vec<Vec<u8>>,
    caret: Position,
    // The column that Up and Down keep to, from where the caret was before
    // they moved it to a shorter line.
    goal_column: Option<usize>,
}

impl Default for Text {
    fn default() -> Self {
        Text::new()
    }
}

// Whether a character can go in the text: ASCII's printable characters, from
// space to tilde, and tab.
pub fn is_text(character: char) -> bool {
    character == '\t' || (' '..='~').contains(&character)
}

impl Text {
    // One empty line.
    pub fn new() -> Text {
        Text {
            lines: vec![Vec::new()],
            caret: Position::default(),
            goal_column: None,
        }
    }

    // The text, with \n between lines, as if it were typed: what can't go
    // in, like letters past ASCII, is left out. The caret is at its end.
    pub fn typed(text: &str) -> Text {
        let mut result = Text::new();
        for character in text.chars() {
            if character == '\n' {
                result.new_line();
            } else if is_text(character) {
                result.insert(character as u8);
            }
        }
        result
    }

    pub fn lines(&self) -> &[Vec<u8>] {
        &self.lines
    }

    pub fn caret(&self) -> Position {
        self.caret
    }

    // The lines with \n between them.
    pub fn to_text(&self) -> String {
        let lines: Vec<String> = self.lines.iter().map(|line| String::from_utf8_lossy(line).into_owned()).collect();
        lines.join("\n")
    }

    fn line(&self) -> &Vec<u8> {
        &self.lines[self.caret.line]
    }

    // Puts the character before the caret. Characters that can't go in the
    // text are ignored.
    pub fn insert(&mut self, character: u8) {
        if !is_text(char::from(character)) {
            return;
        }
        let Position { line, column } = self.caret;
        self.lines[line].insert(column, character);
        self.caret.column += 1;
        self.goal_column = None;
    }

    // Breaks the line at the caret, which goes to the start of the new one.
    pub fn new_line(&mut self) {
        let Position { line, column } = self.caret;
        let rest = self.lines[line].split_off(column);
        self.lines.insert(line + 1, rest);
        self.caret = Position { line: line + 1, column: 0 };
        self.goal_column = None;
    }

    // Deletes the character before the caret, or at the start of a line,
    // joins it to the line before.
    pub fn backspace(&mut self) {
        let Position { line, column } = self.caret;
        if column > 0 {
            self.lines[line].remove(column - 1);
            self.caret.column -= 1;
        } else if line > 0 {
            let rest = self.lines.remove(line);
            let previous = &mut self.lines[line - 1];
            self.caret = Position { line: line - 1, column: previous.len() };
            previous.extend(rest);
        }
        self.goal_column = None;
    }

    // Deletes the character after the caret, or at the end of a line, joins
    // the next line to it.
    pub fn delete(&mut self) {
        let Position { line, column } = self.caret;
        if column < self.lines[line].len() {
            self.lines[line].remove(column);
        } else if line + 1 < self.lines.len() {
            let next = self.lines.remove(line + 1);
            self.lines[line].extend(next);
        }
        self.goal_column = None;
    }

    // Moves the caret a character left, to the end of the line before from
    // the start of one.
    pub fn left(&mut self) {
        if self.caret.column > 0 {
            self.caret.column -= 1;
        } else if self.caret.line > 0 {
            self.caret.line -= 1;
            self.caret.column = self.line().len();
        }
        self.goal_column = None;
    }

    // Moves the caret a character right, to the start of the next line from
    // the end of one.
    pub fn right(&mut self) {
        if self.caret.column < self.line().len() {
            self.caret.column += 1;
        } else if self.caret.line + 1 < self.lines.len() {
            self.caret = Position { line: self.caret.line + 1, column: 0 };
        }
        self.goal_column = None;
    }

    // Moves the caret a line up, or to the start of the first line.
    pub fn up(&mut self) {
        if self.caret.line == 0 {
            self.caret.column = 0;
            self.goal_column = None;
        } else {
            self.vertical(self.caret.line - 1);
        }
    }

    // Moves the caret a line down, or to the end of the last line.
    pub fn down(&mut self) {
        if self.caret.line + 1 == self.lines.len() {
            self.caret.column = self.line().len();
            self.goal_column = None;
        } else {
            self.vertical(self.caret.line + 1);
        }
    }

    // To the line, in the column that the caret was in before it moved up
    // or down, or at the line's end if it's shorter.
    fn vertical(&mut self, line: usize) {
        let goal = *self.goal_column.get_or_insert(self.caret.column);
        self.caret = Position {
            line,
            column: goal.min(self.lines[line].len()),
        };
    }

    // To the start of the line.
    pub fn home(&mut self) {
        self.caret.column = 0;
        self.goal_column = None;
    }

    // To the end of the line.
    pub fn end(&mut self) {
        self.caret.column = self.line().len();
        self.goal_column = None;
    }

    // To the start of the text.
    pub fn start(&mut self) {
        self.caret = Position::default();
        self.goal_column = None;
    }

    // To the end of the text.
    pub fn finish(&mut self) {
        let line = self.lines.len() - 1;
        self.caret = Position {
            line,
            column: self.lines[line].len(),
        };
        self.goal_column = None;
    }
}
