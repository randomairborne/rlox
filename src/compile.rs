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
    scope_depth: usize,
    locals: Vec<Local>,
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
            scope_depth: 0,
            locals: Vec::new(),
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
        if self.match_t(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }
    fn statement(&mut self) {
        if self.match_t(TokenKind::Print) {
            self.print_statement();
        } else if self.match_t(TokenKind::If) {
            self.if_statement();
        } else if self.match_t(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }
    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_t(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit(Op::Nil);
        }
        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );

        self.define_variable(global);
    }
    fn parse_variable(&mut self, error: &'static str) -> usize {
        self.consume(TokenKind::Identifier, error);

        self.declare_variable();
        if self.scope_depth > 0 {
            if let Some(v) = self.locals.last_mut() {
                v.init = true;
            }
            return 0;
        };

        let previous = self.previous.clone();
        self.identifier_constant(&previous)
    }
    fn if_statement(&mut self) {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let jump_loc = self.emit_jump(Op::JumpIfFalse(usize::MAX));
        self.statement();

        self.chunk.code[jump_loc] = Op::JumpIfFalse(jump_loc);
    }
    fn identifier_constant(&mut self, token: &Token) -> usize {
        let const_data = token.src.clone().into();
        self.current_chunk().add_const(Value::Str(const_data))
    }
    fn define_variable(&mut self, global: usize) {
        if self.scope_depth > 0 {
            return;
        }
        self.emit(Op::DefineGlobal(global));
    }
    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let name = self.previous.clone();
        for local in self.locals.clone().iter().rev() {
            if local.depth < self.scope_depth {
                break;
            }

            if name.src == local.name.src {
                self.error(format!(
                    "Already a variable with the name {} in this scope.",
                    name.src
                ));
            }
        }

        self.add_local(name.clone());
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
    fn variable(&mut self, can_assign: bool) {
        let previous = self.previous.clone();
        self.named_variable(&previous, can_assign);
    }
    fn named_variable(&mut self, name: &Token, can_assign: bool) {
        let get_op;
        let set_op;
        if let Some(idx) = self.resolve_local(&name) {
            get_op = Op::GetLocal(idx);
            set_op = Op::SetLocal(idx);
        } else {
            let idx = self.identifier_constant(name);
            get_op = Op::GetGlobal(idx);
            set_op = Op::SetGlobal(idx);
        }
        if can_assign && self.match_t(TokenKind::Equal) {
            self.expression();
            self.emit(set_op);
        } else {
            self.emit(get_op);
        }
    }
    fn resolve_local(&mut self, name: &Token) -> Option<usize> {
        let mut index = self.locals.len();
        for local in self.locals.iter().rev() {
            index -= 1;
            if local.name.src == name.src {
                if !local.init {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(index);
            }
        }

        return None;
    }
    fn number(&mut self, _can_assign: bool) {
        let num: f64 = self
            .previous
            .src
            .parse()
            .expect("Manually validated float unparsable");
        self.emit_const(Value::Number(num));
    }
    fn unary(&mut self, _can_assign: bool) {
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
    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }
    fn binary(&mut self, _can_assign: bool) {
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
    fn literal(&mut self, _can_assign: bool) {
        match self.previous.kind {
            TokenKind::Nil => self.emit(Op::Nil),
            TokenKind::False => self.emit(Op::False),
            TokenKind::True => self.emit(Op::True),
            _ => unreachable!(),
        }
    }
    fn string(&mut self, _can_assign: bool) {
        let last_idx = self.previous.src.len() - 2;
        self.emit_const(Value::Str(self.previous.src[1..=last_idx].into()));
    }
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let Some(prefix_rule) = self.previous.kind.rule().prefix else {
            self.error("Expect expression.");
            return;
        };
        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule(self, can_assign);
        while precedence <= self.current.kind.rule().precedence {
            self.advance();
            let Some(infix_rule) = self.previous.kind.rule().infix else {
                panic!("no infix rule when one was expected (ICE)");
            };
            infix_rule(self, can_assign);
        }
        if can_assign && self.match_t(TokenKind::Equal) {
            self.error("Invalid assignment target");
        };
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
    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.kind != TokenKind::Eof {
            if self.previous.kind == TokenKind::Semicolon {
                return;
            };
            match self.current.kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => {
                    return;
                }

                _ => {}
            }

            self.advance();
        }
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
    fn emit_const(&mut self, value: Value) -> usize {
        let const_idx = self.current_chunk().add_const(value);
        self.emit(Op::Const(const_idx));
        const_idx
    }
    fn emit_return(&mut self) {
        let previous_line = self.previous.line;
        self.current_chunk().add_op(Op::Return, previous_line);
    }
    fn emit_jump(&mut self, instruction: Op) -> usize {
        self.emit(instruction);
        compile_error!("Chapter 22, jumps");
        self.current_chunk().
    }
    fn add_local(&mut self, name: Token) {
        let local = Local {
            name,
            depth: self.scope_depth,
            init: false,
        };
        self.locals.push(local)
    }
    fn block(&mut self) {
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            self.declaration();
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after block.");
    }
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }
    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        let scope_depth = self.scope_depth;
        while self
            .locals
            .last()
            .is_some_and(|local| local.depth > scope_depth)
        {
            self.emit(Op::Pop);
            self.locals.pop();
        }
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
        true
    }
    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
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

type ParseFn = fn(&mut Compiler, can_assign: bool);

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
            TokenKind::Identifier => ParseRule::new(Some(C::variable), None, Prec::None),
            TokenKind::RightParen
            | TokenKind::LeftBrace
            | TokenKind::RightBrace
            | TokenKind::Comma
            | TokenKind::Dot
            | TokenKind::Semicolon
            | TokenKind::Equal
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

#[derive(Clone, Debug)]
pub struct Local {
    name: Token,
    depth: usize,
    init: bool,
}
