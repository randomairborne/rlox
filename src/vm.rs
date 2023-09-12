use crate::{
    chunk::{Chunk, Op},
    compile::Compiler,
    value::Value,
};

const STACK_MAX: usize = 1024;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    sp: usize,
}

impl Vm {
    pub fn init() -> Self {
        Self {
            chunk: Chunk::init(),
            ip: 0,
            stack: [Value::default(); STACK_MAX],
            sp: 0,
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let chunk = Compiler::compile(source);
        InterpretResult::Ok
    }
    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.chunk.code[self.ip];
            #[cfg(debug_assertions)]
            {
                instruction.disassemble(&self.chunk).unwrap();
                let mut i = 0;
                while i < self.sp {
                    print!("[ {} ]", self.stack[i]);
                    i += 1;
                }
                println!();
            }
            self.ip += 1;
            match instruction {
                Op::Const(idx) => {
                    let constant = self.chunk.constants[idx];
                    self.push(constant)
                }
                Op::Add => self.add(),
                Op::Subtract => self.subtract(),
                Op::Multiply => self.multiply(),
                Op::Divide => self.divide(),
                Op::Negate => {
                    let neg = -self.pop();
                    self.push(neg);
                }
                Op::Return => {
                    println!("{}", self.pop());
                    return InterpretResult::Ok;
                }
            }
        }
    }
    fn push(&mut self, data: Value) {
        self.stack[self.sp] = data;
        self.sp += 1;
    }
    fn pop(&mut self) -> Value {
        self.sp -= 1;
        self.stack[self.sp]
    }
    fn add(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a + b);
    }
    fn subtract(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a - b);
    }
    fn multiply(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a * b);
    }
    fn divide(&mut self) {
        let a = self.pop();
        let b = self.pop();
        self.push(a / b);
    }
}

pub enum InterpretResult {
    CompileError,
    RuntimeError,
    Ok,
}
