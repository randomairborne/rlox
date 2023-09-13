use crate::{
    chunk::{Chunk, Op},
    compile::Compiler,
    value::Value,
};

const STACK_PREALLOC: usize = 1024;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl Vm {
    pub fn init() -> Self {
        Self {
            chunk: Chunk::init(),
            ip: 0,
            stack: Vec::with_capacity(STACK_PREALLOC),
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let Ok(chunk) = Compiler::compile(source) else {
            return InterpretResult::CompileError;
        };

        self.chunk = chunk;
        self.ip = 0;
        if let Err(err) = self.run() {
            self.runtime_error(err);
            InterpretResult::RuntimeError
        } else {
            InterpretResult::Ok
        }
    }
    pub fn run(&mut self) -> Result<(), &'static str> {
        loop {
            let instruction = self.chunk.code[self.ip];
            #[cfg(debug_assertions)]
            {
                instruction.disassemble(&self.chunk).unwrap();
                for entry in &self.stack {
                    print!("[ {entry} ]");
                }
                println!();
            }
            self.ip += 1;
            match instruction {
                Op::Const(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    self.push(constant)
                }
                Op::Add => self.add()?,
                Op::Subtract => crate::binary_op!(self, Value::Number, -),
                Op::Multiply => crate::binary_op!(self, Value::Number, *),
                Op::Divide => crate::binary_op!(self, Value::Number, /),
                Op::Greater => crate::binary_op!(self, Value::Bool, >),
                Op::Less => crate::binary_op!(self, Value::Bool, <),
                Op::Negate => {
                    if let Value::Number(val) = self.pop() {
                        self.push(Value::Number(-val));
                    } else {
                        return Err("Operand to negate (-) must be a number.");
                    }
                }
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Not => {
                    let data = self.pop().is_falsey();
                    self.push(Value::Bool(data));
                }
                Op::Equal => {
                    let a = self.pop();
                    let b = self.pop();
                    self.push(Value::Bool(a == b))
                }
                Op::Return => return Ok(()),
            }
        }
    }
    fn add(&mut self) -> Result<(), &'static str> {
        if self.peek(0).is_str() && self.peek(1).is_str() {
            let Value::Str(b) = self.pop() else {
                return Err("data guarded as str was not a str");
            };
            let Value::Str(a) = self.pop() else {
                return Err("data guarded as str was not a str");
            };
            self.push(Value::Str(format!("{}{}", a.as_ref(), b.as_ref()).into()));
        } else if self.peek(0).is_number() && self.peek(1).is_number() {
            let Value::Number(b) = self.pop() else {
                return Err("data guarded as number was not a number");
            };
            let Value::Number(a) = self.pop() else {
                return Err("data guarded as number was not a number");
            };
            self.push(Value::Number(a + b));
        } else {
            return Err("Operands to + must be two numbers or two strings.");
        }
        Ok(())
    }
    fn runtime_error(&mut self, data: impl std::fmt::Display) {
        let line = self
            .chunk
            .lines
            .get(self.ip)
            .expect("self.ip out of line bounds");
        println!("{data}");
        println!("[line {line}] in script");
        self.reset_stack();
    }
    fn reset_stack(&mut self) {
        self.stack.clear();
    }
    fn push(&mut self, data: Value) {
        self.stack.push(data);
    }
    fn pop(&mut self) -> Value {
        let stack_top = self.stack.len() - 1;
        self.stack.remove(stack_top)
    }
    fn peek(&self, distance: usize) -> &Value {
        let stack_top = self.stack.len() - 1;
        &self.stack[stack_top - distance]
    }
}

pub enum InterpretResult {
    CompileError,
    RuntimeError,
    Ok,
}

#[macro_export]
macro_rules! binary_op {
    ($vm:ident, $out:expr, $op:tt) => {
        {
            use $crate::value::Value;
            if !matches!($vm.peek(0), Value::Number(_)) || !matches!($vm.peek(1), Value::Number(_)) {
                return Err("Operands must be numbers.");
            }
            let Value::Number(b) = $vm.pop() else {
                panic!("previously checked value of $vm.pop was invalid (not number, b)");
            };
            let Value::Number(a) = $vm.pop() else {
                panic!("previously checked value of $vm.pop was invalid (not number, a)");
            };
            $vm.push($out(a $op b));
    }
    }
}
