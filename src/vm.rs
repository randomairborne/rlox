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

const DEFAULT_STACK: [Value; STACK_MAX] = [Value::Nil; STACK_MAX];

impl Vm {
    pub fn init() -> Self {
        Self {
            chunk: Chunk::init(),
            ip: 0,
            stack: DEFAULT_STACK,
            sp: 0,
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let (compile_result, chunk) = Compiler::compile(source);
        let InterpretResult::Ok = compile_result else {
            return InterpretResult::CompileError;
        };

        self.chunk = chunk;
        self.ip = 0;
        self.run()
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
                Op::Add => crate::binary_op!(self, Value::Number, +),
                Op::Subtract => crate::binary_op!(self, Value::Number, -),
                Op::Multiply => crate::binary_op!(self, Value::Number, *),
                Op::Divide => crate::binary_op!(self, Value::Number, /),
                Op::Greater => crate::binary_op!(self, Value::Bool, >),
                Op::Less => crate::binary_op!(self, Value::Bool, <),
                Op::Negate => {
                    if let Value::Number(val) = self.pop() {
                        self.push(Value::Number(-val));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
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
                Op::Return => {
                    return InterpretResult::Ok;
                }
            }
        }
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
        self.stack = DEFAULT_STACK;
        self.sp = 0;
    }
    fn push(&mut self, data: Value) {
        self.stack[self.sp] = data;
        self.sp += 1;
    }
    fn pop(&mut self) -> Value {
        self.sp -= 1;
        self.stack[self.sp]
    }
    fn peek(&self, distance: usize) -> Value {
        self.stack[self.sp - (distance + 1)]
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
                $vm.runtime_error("Operands must be numbers.");
                return InterpretResult::RuntimeError;
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
