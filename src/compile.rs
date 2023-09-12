use crate::scan::{Scanner, TokenKind};

pub struct Compiler {}

impl Compiler {
    pub fn compile(source: String) {
        let mut scanner = Scanner::init(source);
        let mut line = 0;
        loop {
            let token = scanner.scan_token();
            if token.line != line {
                print!("{:4>0} ", token.line);
                line = token.line;
            } else {
                print!("   | ");
            }
            println!(
                "{:?} '{}'",
                token.kind,
                scanner.src[scanner.start..scanner.current]
                    .iter()
                    .collect::<String>()
            );

            if token.kind == TokenKind::Eof {
                break;
            }
        }
    }
}
