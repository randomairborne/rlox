use crate::{
    chunk::{Chunk, Op},
    compile::Compiler,
    value::Value,
};
use ahash::AHashMap;
use std::rc::Rc;

const STACK_PREALLOC: usize = 1024;
const GLOBAL_PREALLOC: usize = 1024;

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: AHashMap<Rc<str>, Value>,
}

impl Vm {
    pub fn init() -> Self {
        Self {
            chunk: Chunk::init(),
            ip: 0,
            stack: Vec::with_capacity(STACK_PREALLOC),
            globals: AHashMap::with_capacity(GLOBAL_PREALLOC),
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
    pub fn run(&mut self) -> Result<(), String> {
        loop {
            let instruction = self.chunk.code[self.ip];
            #[cfg(debug_assertions)]
            {
                instruction.disassemble(&self.chunk).unwrap();
                for entry in &self.stack {
                    print!("[ {entry:?} ]");
                }
                println!();
            }
            self.ip += 1;
            match instruction {
                Op::Const(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    self.push(constant)
                }
                Op::DefineGlobal(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    let Value::Str(name) = constant else {
                        panic!("ICE: tried to access {idx} in constant table (value {constant})- expected string, was not string");
                    };
                    let new_val = self.pop();
                    self.globals.insert(name, new_val);
                }
                Op::GetGlobal(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    let Value::Str(name) = constant else {
                        panic!("ICE: tried to access {idx} in constant table (value {constant})- expected string, was not string");
                    };
                    let Some(value) = self.globals.get(name.as_ref()) else {
                        return Err(format!("Undefined variable {name}"));
                    };
                    self.push(value.clone());
                }
                Op::SetGlobal(idx) => {
                    let constant = self.chunk.constants[idx].clone();
                    let Value::Str(name) = constant else {
                        panic!("ICE: tried to access {idx} in constant table (value {constant})- expected string, was not string");
                    };
                    let top = self.peek(0).clone();
                    if let Some(value) = self.globals.get_mut(name.as_ref()) {
                        *value = top;
                    } else {
                        return Err(format!("Undefined variable {name}"));
                    }
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
                        return Err("Operand to negate (-) must be a number.".to_string());
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
                Op::Print => println!("{}", self.pop()),
                Op::Pop => {
                    self.pop();
                }
                Op::Return => return Ok(()),
            }
        }
    }
    fn add(&mut self) -> Result<(), String> {
        if self.peek(0).is_str() && self.peek(1).is_str() {
            let maybe_b = self.pop();
            let maybe_a = self.pop();
            let Value::Str(b) = maybe_b else {
                panic!("data ({maybe_b:?}) guarded as str was not a str");
            };
            let Value::Str(a) = maybe_a else {
                panic!("data ({maybe_a:?}) guarded as str was not a str");
            };
            self.push(Value::Str(format!("{}{}", a.as_ref(), b.as_ref()).into()));
        } else if self.peek(0).is_number() && self.peek(1).is_number() {
            let maybe_b = self.pop();
            let maybe_a = self.pop();
            let Value::Number(b) = maybe_b else {
                panic!("data ({maybe_b:?}) guarded as number was not a number");
            };
            let Value::Number(a) = maybe_a else {
                panic!("data ({maybe_a:?}) guarded as number was not a number");
            };
            self.push(Value::Number(a + b));
        } else {
            return Err("Operands to + must be two numbers or two strings.".to_string());
        }
        Ok(())
    }
    fn runtime_error(&mut self, data: impl std::fmt::Display) {
        let line = self
            .chunk
            .lines
            .get(self.ip)
            .expect("self.ip out of line bounds");
        eprintln!("{data}");
        eprintln!("[line {line}] in script");
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
                return Err("Operands must be numbers.".to_owned());
            }
            let maybe_b = $vm.pop();
            let maybe_a = $vm.pop();
            let Value::Number(b) = maybe_b else {
                panic!("previously checked value of $vm.pop was invalid (not number, {maybe_b:?})");
            };
            let Value::Number(a) = maybe_a else {
                panic!("previously checked value of $vm.pop was invalid (not number, {maybe_a:?})");
            };
            $vm.push($out(a $op b));
    }
    }
}
