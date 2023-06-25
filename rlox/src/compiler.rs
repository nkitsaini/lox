use crate::hashtable::HashTable;
use crate::prelude::*;
use crate::value::LoxObject;
use crate::{
    prelude::Chunk,
    scanner::{Scanner, Token, TokenType, TokenType::*},
};

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
    previous: Option<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
    chunk: Chunk<'a>,
    strings: HashTable<'a>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    PlaceHolderHighest,
}

impl Precedence {
    fn get_next(&self) -> Precedence {
        use Precedence::*;
        match self {
            None => Assignment,
            Assignment => Or,
            Or => And,
            And => Equality,
            Equality => Comparison,
            Comparison => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => PlaceHolderHighest,
            _ => unreachable!(),
        }
    }
}

// type ParseFn = Fn(&mut Compiler) -> ();

type ParseFn<'a> = for<'c> fn(&'c mut Compiler<'a>) -> ();

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, strings: HashTable<'a>) -> Self {
        Compiler {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            chunk: Chunk::new(),
            strings,
        }
    }

    pub fn compile(source: &'a str, strings: HashTable<'a>) -> Option<(Chunk<'a>, HashTable<'a>)> {
        let mut compiler = Self::new(source, strings);
        compiler.advance();
        compiler.expression();
        compiler.consume(TokenType::Eof, "Expect End of expression.");
        compiler.end_compiler();
        if compiler.had_error {
            return None;
        } else {
            return Some((compiler.chunk, compiler.strings));
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value: f64 = self.previous.as_ref().unwrap().string.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self) {
        let prv = self.previous.unwrap().string;

        // remove the quotes
        let str = self.allocate_string(prv[1..prv.len() - 1].to_string());
        self.emit_constant(Value::Object(str));
    }

    fn allocate_string(&mut self, val: std::string::String) -> Rc<LoxObject<'a>> {
        let lox_str = LoxObject::new_string(val);
        let entry = self.strings.find_string(&lox_str).clone();
        if let Some(x) = entry {
            return x;
        }
        let str = Rc::new(lox_str);
        self.strings.set(str.clone(), Value::Nil);
        return str;
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator = self.previous.as_ref().unwrap().ty;
        self.parse_precedence(Precedence::Unary);
        match operator {
            TokenType::Minus => self.emit_op(OpCode::Negate),
            TokenType::Bang => self.emit_op(OpCode::Not),
            _ => return,
        }
    }
    fn binary(&mut self) {
        let operator = self.previous.as_ref().unwrap().ty;
        let rule = Self::get_rule(operator);
        self.parse_precedence(rule.precedence.get_next());
        match operator {
            BangEqual => self.emit_ops(OpCode::Equal, OpCode::Not),
            EqualEqual => self.emit_op(OpCode::Equal),
            Greater => self.emit_op(OpCode::Greater),
            GreaterEqual => self.emit_ops(OpCode::Less, OpCode::Not),
            Less => self.emit_op(OpCode::Less),
            LessEqual => self.emit_ops(OpCode::Greater, OpCode::Not),

            TokenType::Plus => self.emit_op(OpCode::Add),
            TokenType::Minus => self.emit_op(OpCode::Subtract),
            TokenType::Star => self.emit_op(OpCode::Multiply),
            TokenType::Slash => self.emit_op(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        match self.previous.unwrap().ty {
            False => self.emit_op(OpCode::False),
            Nil => self.emit_op(OpCode::Nil),
            True => self.emit_op(OpCode::True),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let rule = Self::get_rule(self.previous.as_ref().unwrap().ty);
        let prefix_fn = match rule.prefix {
            None => {
                self.error("Expect expression.");
                return;
            }
            Some(x) => x,
        };
        prefix_fn(self);

        while precedence <= Self::get_rule(self.current.as_ref().unwrap().ty).precedence {
            self.advance();
            let infix_rule = Self::get_rule(self.previous.as_ref().unwrap().ty).infix;
            infix_rule.unwrap()(self);
        }
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            let token = self.scanner.scan_token();
            self.current = Some(token);
            if self.current.unwrap().ty != TokenType::Error {
                break;
            };

            let value = self.current.unwrap().string.to_string();
            self.error_at_current(&value);
        }
    }

    fn consume(&mut self, ty: TokenType, msg: &str) {
        if self.current.unwrap().ty == ty {
            self.advance();
            return;
        }
        self.error_at_current(msg);
    }

    fn end_compiler(&mut self) {
        self.emit_op(OpCode::Return);
        #[cfg(feature = "trace_execution")]
        {
            if !self.had_error {
                disassemble_chunk(&self.chunk, "code");
            }
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.chunk.write(op, self.previous.unwrap().line);
    }

    fn emit_ops(&mut self, op1: OpCode, op2: OpCode) {
        self.emit_op(op1);
        self.emit_op(op2);
    }

    fn emit_constant(&mut self, value: Value<'a>) {
        if self.chunk.constants.len() == u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return;
        }
        self.chunk
            .write_constant(value, self.previous.unwrap().line);
    }

    fn error_at_current(&mut self, msg: &str) {
        self.error_at(self.current.unwrap(), msg);
    }

    fn error(&mut self, msg: &str) {
        self.error_at(self.previous.unwrap(), msg);
    }

    fn error_at(&mut self, token: Token, msg: &str) {
        // If in panic mode there might be multiple errors
        // We do not report all the cascading errors until we find a new token
        // that we know is start of a new statement
        if self.panic_mode {
            return;
        }
        eprint!("[line {}] Error", token.line);

        match token.ty {
            TokenType::Eof => {
                eprint!(" at end");
            }
            TokenType::Error => {}
            _ => {
                eprint!(" at '{}'", token.string);
            }
        }
        eprintln!(": {}", msg);
        self.had_error = true;
    }

    fn get_rule(ty: TokenType) -> ParseRule<'a> {
        match ty {
            TokenType::LeftParen => {
                ParseRule::new(Some(Compiler::grouping), None, Precedence::None)
            }
            TokenType::RightParen => (None, None, Precedence::None).into(),
            TokenType::LeftBrace => (None, None, Precedence::None).into(),
            TokenType::RightBrace => (None, None, Precedence::None).into(),
            TokenType::Comma => (None, None, Precedence::None).into(),
            TokenType::Dot => (None, None, Precedence::None).into(),
            TokenType::Minus => ParseRule::new(
                Some(Compiler::unary),
                Some(Compiler::binary),
                Precedence::Term,
            ),
            TokenType::Plus => ParseRule::new(None, Some(Compiler::binary), Precedence::Term),
            TokenType::Semicolon => ParseRule::new(None, None, Precedence::None),
            TokenType::Slash => ParseRule::new(None, Some(Compiler::binary), Precedence::Factor),
            TokenType::Star => ParseRule::new(None, Some(Compiler::binary), Precedence::Factor),
            TokenType::Bang => ParseRule::new(Some(Compiler::unary), None, Precedence::None),
            TokenType::BangEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Equality)
            }
            TokenType::Equal => ParseRule::new(None, None, Precedence::None),
            TokenType::EqualEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Equality)
            }
            TokenType::Greater => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenType::GreaterEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenType::Less => ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison),
            TokenType::LessEqual => {
                ParseRule::new(None, Some(Compiler::binary), Precedence::Comparison)
            }
            TokenType::Identifier => ParseRule::new(None, None, Precedence::None),
            TokenType::String => ParseRule::new(Some(Compiler::string), None, Precedence::None),
            TokenType::Number => ParseRule::new(Some(Compiler::number), None, Precedence::None),
            TokenType::And => ParseRule::new(None, None, Precedence::None),
            TokenType::Class => ParseRule::new(None, None, Precedence::None),
            TokenType::Else => ParseRule::new(None, None, Precedence::None),
            TokenType::False => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
            TokenType::For => ParseRule::new(None, None, Precedence::None),
            TokenType::Fun => ParseRule::new(None, None, Precedence::None),
            TokenType::If => ParseRule::new(None, None, Precedence::None),
            TokenType::Nil => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
            TokenType::Or => ParseRule::new(None, None, Precedence::None),
            TokenType::Print => ParseRule::new(None, None, Precedence::None),
            TokenType::Return => ParseRule::new(None, None, Precedence::None),
            TokenType::Super => ParseRule::new(None, None, Precedence::None),
            TokenType::This => ParseRule::new(None, None, Precedence::None),
            TokenType::True => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
            TokenType::Var => ParseRule::new(None, None, Precedence::None),
            TokenType::While => ParseRule::new(None, None, Precedence::None),
            TokenType::Error => ParseRule::new(None, None, Precedence::None),
            TokenType::Eof => ParseRule::new(None, None, Precedence::None),
        }
    }
}

// All of this madness is to implement the pratt-parser in similar fashion as the book.
impl<'a>
    From<(
        Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
        Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
        Precedence,
    )> for ParseRule<'a>
{
    fn from(
        value: (
            Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
            Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
            Precedence,
        ),
    ) -> Self {
        Self {
            prefix: value.0,
            infix: value.1,
            precedence: value.2,
        }
    }
}

impl<'a> ParseRule<'a> {
    fn new(
        suffix: Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
        infix: Option<for<'c> fn(&'c mut Compiler<'a>) -> ()>,
        precedence: Precedence,
    ) -> Self {
        return (suffix, infix, precedence).into();
    }
}
