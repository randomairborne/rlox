use crate::chunk::Chunk;
use std::fmt::Write;

impl crate::chunk::Chunk {
    pub fn disassemble(&self, name: &str) -> Result<String, std::fmt::Error> {
        let mut f = String::with_capacity(1024 * 64);
        writeln!(f, "=== {name} ===")?;
        let mut last_line = 0;
        for (index, op) in self.code.iter().enumerate() {
            let line = {
                let current_line = self.lines[index];
                let text = if current_line != last_line {
                    format!("{current_line:0>4}")
                } else {
                    "  | ".to_owned()
                };
                last_line = current_line;
                text
            };
            write!(f, "{index:0>4} {line} {}", op.disassemble(self).unwrap())?;
            if index != self.code.len() - 1 {
                f.push('\n');
            }
        }
        Ok(f)
    }
}

impl crate::chunk::Op {
    pub fn disassemble(&self, chunk: &Chunk) -> Result<String, std::fmt::Error> {
        let mut f = String::with_capacity(1024);
        match self {
            Self::Return => write!(f, "Op::Return")?,
            Self::Negate => write!(f, "Op::Negate")?,
            Self::Add => write!(f, "Op::Add")?,
            Self::Subtract => write!(f, "Op::Subtract")?,
            Self::Multiply => write!(f, "Op::Multiply")?,
            Self::Divide => write!(f, "Op::Divide")?,
            Self::Const(idx) => write!(f, "Op::Const {idx} {:?}", chunk.constants[*idx])?,
            Self::Nil => write!(f, "Op::Nil")?,
            Self::True => write!(f, "Op::True")?,
            Self::False => write!(f, "Op::False")?,
            Self::Not => write!(f, "Op::Not")?,
            Self::Equal => write!(f, "Op::Equal")?,
            Self::Greater => write!(f, "Op::Greater")?,
            Self::Less => write!(f, "Op::Less")?,
        }
        Ok(f)
    }
}
