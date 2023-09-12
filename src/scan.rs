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
            while self.peek().is_ascii_alphanumeric() {
                self.advance();
            }
            let ident = self.identifier();
            return self.token(ident);
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
            '\0' => TokenKind::Eof,
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
            src: self.src[self.start..self.current].iter().collect(),
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
        if self.is_at_end() {
            return '\0';
        }
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
    fn identifier(&mut self) -> TokenKind {
        match self.src[self.start] {
            'a' => return self.check_keyword(1, 2, "nd", TokenKind::And),
            'c' => return self.check_keyword(1, 4, "lass", TokenKind::Class),
            'e' => return self.check_keyword(1, 3, "lse", TokenKind::Else),
            'i' => return self.check_keyword(1, 1, "f", TokenKind::If),
            'n' => return self.check_keyword(1, 2, "il", TokenKind::Nil),
            'o' => return self.check_keyword(1, 1, "r", TokenKind::Or),
            'p' => return self.check_keyword(1, 4, "rint", TokenKind::Print),
            'r' => return self.check_keyword(1, 5, "eturn", TokenKind::Return),
            's' => return self.check_keyword(1, 4, "uper", TokenKind::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.src[self.start + 1] {
                        'h' => return self.check_keyword(2, 2, "is", TokenKind::This),
                        'r' => return self.check_keyword(2, 2, "ue", TokenKind::True),
                        _ => {}
                    }
                }
            }
            'v' => return self.check_keyword(1, 2, "ar", TokenKind::Var),
            'w' => return self.check_keyword(1, 4, "hile", TokenKind::While),
            'f' => {
                if self.current - self.start > 1 {
                    match self.src[self.start + 1] {
                        'a' => return self.check_keyword(2, 3, "lse", TokenKind::False),
                        'o' => return self.check_keyword(2, 1, "r", TokenKind::For),
                        'u' => return self.check_keyword(2, 1, "n", TokenKind::Fun),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        TokenKind::Identifier
    }
    fn check_keyword(
        &self,
        start: usize,
        length: usize,
        rest: &'static str,
        kind: TokenKind,
    ) -> TokenKind {
        let token_start = self.start + start;
        if self.current - self.start == start + length
            && self.src[token_start..token_start + length]
                .iter()
                .collect::<String>()
                == rest
        {
            return kind;
        }

        return TokenKind::Identifier;
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub src: String,
    pub line: usize,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            kind: TokenKind::Error,
            src: "".to_string(),
            line: 0,
        }
    }
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
