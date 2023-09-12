use crate::{
    chunk::{Chunk, Op},
    scan::{Scanner, Token, TokenKind},
    value::Value,
    vm::InterpretResult,
};

pub struct Compiler<'a> {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
    chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    pub fn compile(source: String, chunk: &mut Chunk) -> InterpretResult {
        let mut scanner = Scanner::init(source);
        let mut line = 0;
        let mut compiler = Compiler {
            scanner,
            chunk,
            current: Default::default(),
            previous: Default::default(),
            had_error: false,
            panic_mode: false,
        };
        compiler.advance();
        compiler.expression();
        compiler.consume(TokenKind::Eof, "Expected end of expression");
        if compiler.had_error {
            InterpretResult::CompileError
        } else {
            InterpretResult::Ok
        }
    }
    fn expression(&mut self) {}
    compile_error!("https://craftinginterpreters.com/compiling-expressions.html#unary-negation");
    fn number(&mut self) {
        let num: f64 = self
            .previous
            .src
            .parse()
            .expect("Manually validated float unparsable");
        self.emit_const(num);
    }
    fn unary(&mut self) {
        let operator_kind = self.previous.kind;

        // Compile the operand.
        self.expression();

        // Emit the operator instruction.
        match operator_kind {
            TokenKind::Minus => self.emit(Op::Negate),
            _ => unreachable!(),
        }
    }
    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
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
    fn emit_const(&mut self, value: Value) {
        let previous_line = self.previous.line;
        self.current_chunk().add_const(value, previous_line);
    }
    fn emit_return(&mut self) {
        let previous_line = self.previous.line;
        self.current_chunk().add_op(Op::Return, previous_line);
    }
    fn end(mut self) {
        self.emit_return();
    }
    fn current_chunk(&mut self) -> &mut Chunk {
        self.chunk
    }
}
