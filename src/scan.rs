pub struct Scanner {
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub src: Vec<char>,
}

impl Scanner {
    pub fn init(src: String) -> Self {
        Self {
            start: 0,
            current: 0,
            line: 1,
            src: src.chars().collect(),
        }
    }
    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;
        if self.is_at_end() {
            return self.token(TokenKind::Eof);
        }
        let c = self.advance();
        if c.is_ascii_alphabetic() || c == '_' {
            compile_error!(
                "CURRENTLY HERE, SEE https://craftinginterpreters.com/scanning-on-demand.html"
            );
        }
        if c.is_ascii_digit() {
            return self.number();
        }
        let tk = match c {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '-' => TokenKind::Minus,
            '+' => TokenKind::Plus,
            '/' => TokenKind::Slash,
            '*' => TokenKind::Star,
            '!' => {
                if self.match_c('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '=' => {
                if self.match_c('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '<' => {
                if self.match_c('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.match_c('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '"' => return self.string(),
            _ => return self.error_token("Unexpected character."),
        };
        self.token(tk)
    }
    fn token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            src: self.src[self.current..self.start].iter().collect(),
            line: self.line,
        }
    }
    fn error_token(&self, msg: impl Into<String>) -> Token {
        Token {
            kind: TokenKind::Error,
            src: msg.into(),
            line: self.line,
        }
    }
    fn is_at_end(&self) -> bool {
        self.current == self.src.len()
    }
    fn advance(&mut self) -> char {
        let out = self.src[self.current];
        self.current += 1;
        out
    }
    fn match_c(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.src[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }
    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }
    fn peek(&self) -> char {
        self.src[self.current]
    }
    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.src[self.current + 1]
    }
    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            };
            self.advance();
        }
        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        };
        // The closing quote.
        self.advance();
        self.token(TokenKind::String)
    }
    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the ".".
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.token(TokenKind::Number)
    }
}

pub struct Token {
    pub kind: TokenKind,
    pub src: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}
