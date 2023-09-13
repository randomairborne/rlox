use crate::{
    chunk::{Chunk, Op},
    scan::{Scanner, Token, TokenKind},
    value::Value,
};

pub struct Compiler {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    chunk: Chunk,
}

impl Compiler {
    pub fn compile(source: String) -> Result<Chunk, ()> {
        let scanner = Scanner::init(source);
        let chunk = Chunk::init();
        let mut compiler = Compiler {
            scanner,
            chunk,
            current: Default::default(),
            previous: Default::default(),
            had_error: false,
            panic_mode: false,
        };
        compiler.advance();
        while !compiler.match_t(TokenKind::Eof) {
            compiler.declaration();
        }
        compiler.end();
        if compiler.had_error {
            Err(())
        } else {
            Ok(compiler.chunk)
        }
    }
    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }
    fn declaration(&mut self) {
        self.statement();
    }
    fn statement(&mut self) {
        if self.match_t(TokenKind::Print) {
            self.print_statement();
        }
    }
    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit(Op::Print);
    }
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit(Op::Pop);
    }
    fn number(&mut self) {
        let num: f64 = self
            .previous
            .src
            .parse()
            .expect("Manually validated float unparsable");
        self.emit_const(Value::Number(num));
    }
    fn unary(&mut self) {
        let operator_kind = self.previous.kind;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match operator_kind {
            TokenKind::Minus => self.emit(Op::Negate),
            TokenKind::Bang => self.emit(Op::Not),
            _ => unreachable!(),
        }
    }
    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }
    fn binary(&mut self) {
        let operator_kind = self.previous.kind;
        let rule: ParseRule = operator_kind.into();
        self.parse_precedence(rule.precedence.next());

        match operator_kind {
            TokenKind::Plus => self.emit(Op::Add),
            TokenKind::Minus => self.emit(Op::Subtract),
            TokenKind::Star => self.emit(Op::Multiply),
            TokenKind::Slash => self.emit(Op::Divide),
            TokenKind::BangEqual => self.emit2(Op::Equal, Op::Not),
            TokenKind::EqualEqual => self.emit(Op::Equal),
            TokenKind::Greater => self.emit(Op::Greater),
            TokenKind::GreaterEqual => self.emit2(Op::Less, Op::Not),
            TokenKind::Less => self.emit(Op::Less),
            TokenKind::LessEqual => self.emit2(Op::Greater, Op::Not),
            _ => unreachable!(),
        }
    }
    fn literal(&mut self) {
        match self.previous.kind {
            TokenKind::Nil => self.emit(Op::Nil),
            TokenKind::False => self.emit(Op::False),
            TokenKind::True => self.emit(Op::True),
            _ => unreachable!(),
        }
    }
    fn string(&mut self) {
        let last_idx = self.previous.src.len() - 2;
        self.emit_const(Value::Str(self.previous.src[1..=last_idx].into()));
    }
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let Some(prefix_rule) = self.previous.kind.rule().prefix else {
            self.error("Expect expression.");
            return;
        };
        prefix_rule(self);
        while precedence <= self.current.kind.rule().precedence {
            self.advance();
            let Some(infix_rule) = self.previous.kind.rule().infix else {
                self.error("no infix rule when one was expected (ICE)");
                return;
            };
            infix_rule(self);
        }
    }
    fn scan_token(&mut self) -> Token {
        self.scanner.scan_token()
    }
    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scan_token();
            if self.current.kind != TokenKind::Error {
                break;
            };
            let message = self.current.src.clone();
            self.error_at_current(message);
        }
    }
    fn error_at_current(&mut self, message: impl std::fmt::Display) {
        self.error_at(self.current.clone(), message);
    }
    fn error(&mut self, message: impl std::fmt::Display) {
        self.error_at(self.previous.clone(), message);
    }
    fn error_at(&mut self, token: Token, message: impl std::fmt::Display) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        if token.kind == TokenKind::Eof {
            eprint!(" at end");
        } else if token.kind != TokenKind::Error {
            eprint!(" at '{}'", token.src);
        }

        eprintln!(": {}\n", message);
        self.had_error = true;
    }
    fn consume(&mut self, kind: TokenKind, message: impl std::fmt::Display) {
        if self.current.kind == kind {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }
    fn emit(&mut self, instruction: Op) {
        let previous_line = self.previous.line;
        self.current_chunk().add_op(instruction, previous_line);
    }
    fn emit2(&mut self, i1: Op, i2: Op) {
        self.emit(i1);
        self.emit(i2);
    }
    fn emit_const(&mut self, value: Value) {
        let previous_line = self.previous.line;
        self.current_chunk().add_const(value, previous_line);
    }
    fn emit_return(&mut self) {
        let previous_line = self.previous.line;
        self.current_chunk().add_op(Op::Return, previous_line);
    }
    fn end(&mut self) {
        self.emit_return();
        #[cfg(debug_assertions)]
        if !self.had_error {
            eprintln!("{}", self.current_chunk().disassemble("code").unwrap())
        }
    }
    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunk
    }
    fn match_t(&mut self, kind: TokenKind) -> bool {
        if !self.check(kind) {
            return false;
        };
        self.advance();
        return true;
    }
    fn check(&self, kind: TokenKind) -> bool {
        return self.current.kind == kind;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
    pub fn last(self) -> Self {
        match self {
            Precedence::None => Precedence::None,
            Precedence::Assignment => Precedence::None,
            Precedence::Or => Precedence::Assignment,
            Precedence::And => Precedence::Or,
            Precedence::Equality => Precedence::And,
            Precedence::Comparison => Precedence::Equality,
            Precedence::Term => Precedence::Comparison,
            Precedence::Factor => Precedence::Term,
            Precedence::Unary => Precedence::Factor,
            Precedence::Call => Precedence::Unary,
            Precedence::Primary => Precedence::Call,
        }
    }
}

type ParseFn = fn(&mut Compiler);

pub struct ParseRule {
    prefix: Option<ParseFn>,
    infix: Option<ParseFn>,
    precedence: Precedence,
}

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
    pub const EMPTY: Self = Self {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    };
    pub fn from_token(token_kind: TokenKind) -> Self {
        token_kind.into()
    }
}

impl From<TokenKind> for ParseRule {
    fn from(val: TokenKind) -> Self {
        use Compiler as C;
        use ParseRule as P;
        use Precedence as Prec;
        match val {
            TokenKind::LeftParen => P::new(Some(C::grouping), None, Prec::None),
            TokenKind::Minus => P::new(Some(C::unary), Some(C::binary), Prec::Term),
            TokenKind::Plus => P::new(None, Some(C::binary), Prec::Term),
            TokenKind::Slash => P::new(None, Some(C::binary), Prec::Factor),
            TokenKind::Star => P::new(None, Some(C::binary), Prec::Factor),
            TokenKind::Number => P::new(Some(C::number), None, Prec::None),
            TokenKind::True => P::new(Some(C::literal), None, Prec::None),
            TokenKind::False => P::new(Some(C::literal), None, Prec::None),
            TokenKind::Nil => P::new(Some(C::literal), None, Prec::None),
            TokenKind::Bang => P::new(Some(C::unary), None, Prec::None),
            TokenKind::BangEqual => P::new(None, Some(C::binary), Prec::Equality),
            TokenKind::EqualEqual => P::new(None, Some(C::binary), Prec::Equality),
            TokenKind::Greater => ParseRule::new(None, Some(C::binary), Prec::Comparison),
            TokenKind::GreaterEqual => ParseRule::new(None, Some(C::binary), Prec::Comparison),
            TokenKind::Less => ParseRule::new(None, Some(C::binary), Prec::Comparison),
            TokenKind::LessEqual => ParseRule::new(None, Some(C::binary), Prec::Comparison),
            TokenKind::String => ParseRule::new(Some(C::string), None, Prec::None),
            TokenKind::RightParen
            | TokenKind::LeftBrace
            | TokenKind::RightBrace
            | TokenKind::Comma
            | TokenKind::Dot
            | TokenKind::Semicolon
            | TokenKind::Equal
            | TokenKind::Identifier
            | TokenKind::And
            | TokenKind::Class
            | TokenKind::Else
            | TokenKind::For
            | TokenKind::Fun
            | TokenKind::If
            | TokenKind::Or
            | TokenKind::Print
            | TokenKind::Return
            | TokenKind::Super
            | TokenKind::This
            | TokenKind::Var
            | TokenKind::While
            | TokenKind::Error
            | TokenKind::Eof => P::EMPTY,
        }
    }
}
