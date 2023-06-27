use std::io::Write;

use crate::hashtable::HashTable;
use crate::prelude::*;
use crate::value::LoxObject;
use crate::{
    prelude::Chunk,
    scanner::{Scanner, Token, TokenType, TokenType::*},
};

const U8_COUNT: usize = u8::MAX as usize + 1;

pub struct Local<'a> {
    name: Token<'a>,
    depth: Option<usize>, // None if unitialized
}

pub struct Compiler<'a, 'b, WE: Write> {
    scanner: Scanner<'a>,
    current: Option<Token<'a>>,
    previous: Option<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
    chunk: Chunk<'a>,
    strings: HashTable<'a>,

    locals: smallvec::SmallVec<[Local<'a>; U8_COUNT]>,
    scope_depth: usize,

    stderr: &'b mut WE,
}
macro_rules! emit_jump {
    ($compiler:ident, $enum_variant:ident) => {{
        let a = OpCode::$enum_variant { offset: 0 };
        $compiler.emit_op(a);
        $compiler.chunk.code.len()
    }};
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

/// Takes (&mut Compiler, can_assign: bool)
type ParseFn<'a, 'b, WE: Write> = for<'c> fn(&'c mut Compiler<'a, 'b, WE>, bool) -> ();

struct ParseRule<'a, 'b, WE: Write> {
    prefix: Option<ParseFn<'a, 'b, WE>>,
    infix: Option<ParseFn<'a, 'b, WE>>,
    precedence: Precedence,
}

impl<'a, 'b, WE: Write> Compiler<'a, 'b, WE> {
    pub fn new(source: &'a str, strings: HashTable<'a>, stderr: &'b mut WE) -> Self {
        Compiler {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
            chunk: Chunk::new(),
            strings,

            locals: smallvec::SmallVec::new(),
            scope_depth: 0,

            stderr,
        }
    }

    pub fn compile(
        source: &'a str,
        strings: HashTable<'a>,
        stderr: &'b mut WE,
    ) -> Option<(Chunk<'a>, HashTable<'a>)> {
        let mut compiler = Self::new(source, strings, stderr);
        compiler.advance();
        while !compiler.match_(Eof) {
            compiler.declaration();
        }
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

    fn block(&mut self) {
        while !self.check(RightBrace) && !self.check(Eof) {
            self.declaration();
        }
        self.consume(RightBrace, "Expect '}' after block.");
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(Semicolon, "Expect ';' after value.");
        self.emit_op(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(Semicolon, "Expect ';' after value.");

        // Expression emits value, but a expression statement should not
        self.emit_op(OpCode::Pop);
    }

    fn if_statement(&mut self) {
        // if (abc ) {...}
        self.consume(LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(RightParen, "Expect ')' after condition.");

        let then_jump = emit_jump!(self, JumpIfFalse);

        self.statement();

        let else_jump = emit_jump!(self, Jump);
        self.patch_jump(then_jump);

        if self.match_(Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        // while (condition) {statement}
        let loop_start = self.chunk.code.len();

        self.consume(LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(RightParen, "Expect ')' after condition.");

        let condition_jump = emit_jump!(self, JumpIfFalse);
        self.emit_op(OpCode::Pop);

        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(condition_jump);
        self.emit_op(OpCode::Pop);
    }

    fn for_statement(&mut self) {
        self.begin_scope();

        // ------------------ 1. Initialization
        self.consume(LeftParen, "Expect '(' after 'for'.");
        // we
        if self.match_(Semicolon) {
            // No initialization
        } else if self.match_(Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let expr_loc = self.chunk.code.len();
        let mut end_jump = None;

        // ------------------ 2. Condition
        if !self.match_(Semicolon) {
            self.expression();
            self.consume(Semicolon, "Expect ';' after condition.");

            end_jump = Some(emit_jump!(self, JumpIfFalse));
            self.emit_op(OpCode::Pop);
        }

        // skip-increment
        let loop_body_jump = emit_jump!(self, Jump);
        let increment_loc = self.chunk.code.len();

        // ------------------ 3. Increment
        if !self.match_(RightParen) {
            self.expression();
            self.emit_op(OpCode::Pop);
            self.consume(RightParen, "Expect ')' after increment clause.");
        }

        // Go back to condition
        self.emit_loop(expr_loc);

        self.patch_jump(loop_body_jump);

        self.statement();

        self.emit_loop(increment_loc);

        if let Some(x) = end_jump {
            self.patch_jump(x);
            self.emit_op(OpCode::Pop);
        }

        self.end_scope();
    }

    fn define_variable(&mut self, location: u8) {
        // local variable is referenced by index in stack instead of name
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_op(OpCode::DefineGlobal { location });
    }

    fn and_(&mut self, _can_assing: bool) {
        let end_jump = emit_jump!(self, JumpIfFalse);
        self.emit_op(OpCode::Pop);
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or_(&mut self, _can_assing: bool) {
        // 1. true or X => true
        // 2. false or X => X

        let else_jump = emit_jump!(self, JumpIfFalse);
        let end_jump = emit_jump!(self, Jump);

        self.patch_jump(else_jump);
        self.emit_op(OpCode::Pop); // Case 2
        self.parse_precedence(Precedence::Or);

        self.patch_jump(end_jump);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_(Equal) {
            self.expression();
        } else {
            self.emit_op(OpCode::Nil);
        }
        self.consume(Semicolon, "Expect `;` after variable declaration.");
        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_(Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        // A new statement marks beginning of a new life
        // Sins of past are forgotten
        //
        // If there is an error then we find the next point where we should
        // start scanning.
        // For example, We need this if the error was `;` not found that previous statement is kinda
        // still active.
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.unwrap().ty != Eof {
            if self.previous.unwrap().ty == Semicolon {
                return;
            }
            match self.current.unwrap().ty {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    fn statement(&mut self) {
        if self.match_(Print) {
            self.print_statement();
        } else if self.match_(If) {
            self.if_statement();
        } else if self.match_(While) {
            self.while_statement();
        } else if self.match_(For) {
            self.for_statement();
        } else if self.match_(LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn number(&mut self, _can_assing: bool) {
        let value: f64 = self.previous.as_ref().unwrap().string.parse().unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, _can_assing: bool) {
        let prv = self.previous.unwrap().string;

        // remove the quotes
        let str = self.allocate_string(prv[1..prv.len() - 1].to_string());
        self.emit_constant(Value::Object(str));
    }

    fn named_variable(&mut self, can_assign: bool, token: Token<'a>) {
        let get_op;
        let set_op;
        let arg = self.resolve_local(token);
        match arg {
            None => {
                let arg = self.identifier_constant(token);
                get_op = OpCode::GetGlobal { location: arg };
                set_op = OpCode::SetGlobal { location: arg };
            }
            Some(x) => {
                get_op = OpCode::GetLocal { stack_idx: x };
                set_op = OpCode::SetLocal { stack_idx: x };
            }
        }

        if can_assign && self.match_(Equal) {
            self.expression();
            self.emit_op(set_op);
        } else {
            self.emit_op(get_op);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign, self.previous.unwrap());
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

    fn grouping(&mut self, _can_assing: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assing: bool) {
        let operator = self.previous.as_ref().unwrap().ty;
        self.parse_precedence(Precedence::Unary);
        match operator {
            TokenType::Minus => self.emit_op(OpCode::Negate),
            TokenType::Bang => self.emit_op(OpCode::Not),
            _ => return,
        }
    }
    fn binary(&mut self, _can_assing: bool) {
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

    fn literal(&mut self, _can_assing: bool) {
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
        let can_assign = precedence <= Precedence::Assignment;
        prefix_fn(self, can_assign);

        while precedence <= Self::get_rule(self.current.as_ref().unwrap().ty).precedence {
            self.advance();
            let infix_rule = Self::get_rule(self.previous.as_ref().unwrap().ty).infix;
            infix_rule.unwrap()(self, can_assign);
        }

        if can_assign && self.match_(Equal) {
            self.error("Invalid Assignment Target.");
        }
    }

    fn identifier_constant(&mut self, token: Token<'a>) -> u8 {
        let string = self.allocate_string(token.string.to_string());
        self.make_constant(Value::Object(string))
    }

    fn identifiers_equal(&self, a: Token<'a>, b: Token<'a>) -> bool {
        return a.string == b.string;
    }

    fn resolve_local(&mut self, token: Token<'a>) -> Option<u8> {
        for (i, value) in self.locals.iter().enumerate().rev() {
            if self.identifiers_equal(value.name, token) {
                if value.depth.is_none() {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(i as u8);
            }
        }
        None
    }

    fn add_local(&mut self, token: Token<'a>) {
        if self.locals.len() == U8_COUNT {
            self.error("Too many local variables in function.");
            return;
        }
        self.locals.push(Local {
            name: token,
            depth: None,
        });
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let val = *self.previous.as_ref().unwrap();
        for i in (0..self.locals.len()).rev() {
            let local = &self.locals[i];
            if local.depth.is_some() && local.depth.unwrap() < self.scope_depth {
                break;
            }
            if self.identifiers_equal(val, local.name) {
                self.error("Already a variable with this name in this scope.");
            }
        }
        self.add_local(val);
    }

    fn parse_variable(&mut self, msg: &str) -> u8 {
        self.consume(Identifier, msg);
        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        };
        self.identifier_constant(self.previous.unwrap())
    }

    fn mark_initialized(&mut self) {
        self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
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

    fn check(&self, ty: TokenType) -> bool {
        self.current.unwrap().ty == ty
    }

    fn match_(&mut self, ty: TokenType) -> bool {
        if !self.check(ty) {
            false
        } else {
            self.advance();
            true
        }
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

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while !self.locals.is_empty()
            && self.locals.last().unwrap().depth.is_some()
            && self.locals.last().unwrap().depth.unwrap() > self.scope_depth
        {
            self.emit_op(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn emit_op(&mut self, op: OpCode) {
        self.chunk.write(op, self.previous.unwrap().line);
    }

    fn emit_ops(&mut self, op1: OpCode, op2: OpCode) {
        self.emit_op(op1);
        self.emit_op(op2);
    }

    fn emit_loop(&mut self, chunk_loc: usize) {
        let offset = self.chunk.code.len() - chunk_loc;
        if offset > u16::MAX as usize {
            self.error("Loop body too large.");
        }

        self.emit_op(OpCode::Loop {
            offset: offset as u16 + 1, // +1 to include this current loop opcode
        });
    }

    fn patch_jump(&mut self, opcode_loc: usize) {
        let jump = self.chunk.code.len() - opcode_loc;

        if jump > u16::MAX as usize {
            self.error("Too much code to jump over.");
        }

        match self.chunk.code.get_mut(opcode_loc - 1).unwrap() {
            (OpCode::JumpIfFalse { offset: target }, _) => *target = jump as u16,
            (OpCode::Jump { offset: target }, _) => *target = jump as u16,
            _ => unreachable!(),
        }
    }

    fn make_constant(&mut self, value: Value<'a>) -> u8 {
        if self.chunk.constants.len() == u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            // TOOD: rustic way
            return 0;
        }
        self.chunk.add_constant(value)
    }

    fn emit_constant(&mut self, value: Value<'a>) {
        let location = self.make_constant(value);
        self.emit_op(OpCode::Constant { location });
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

        write!(self.stderr, "[line {}] Error", token.line).unwrap();

        match token.ty {
            TokenType::Eof => {
                write!(self.stderr, " at end").unwrap();
            }
            TokenType::Error => {}
            _ => {
                write!(self.stderr, " at '{}'", token.string).unwrap();
            }
        }
        writeln!(self.stderr, ": {}", msg).unwrap();
        self.had_error = true;
    }

    fn get_rule(ty: TokenType) -> ParseRule<'a, 'b, WE> {
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
            TokenType::Identifier => {
                ParseRule::new(Some(Compiler::variable), None, Precedence::None)
            }
            TokenType::String => ParseRule::new(Some(Compiler::string), None, Precedence::None),
            TokenType::Number => ParseRule::new(Some(Compiler::number), None, Precedence::None),
            TokenType::And => ParseRule::new(None, Some(Compiler::and_), Precedence::And),
            TokenType::Class => ParseRule::new(None, None, Precedence::None),
            TokenType::Else => ParseRule::new(None, None, Precedence::None),
            TokenType::False => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
            TokenType::For => ParseRule::new(None, None, Precedence::None),
            TokenType::Fun => ParseRule::new(None, None, Precedence::None),
            TokenType::If => ParseRule::new(None, None, Precedence::None),
            TokenType::Nil => ParseRule::new(Some(Compiler::literal), None, Precedence::None),
            TokenType::Or => ParseRule::new(None, Some(Compiler::or_), Precedence::Or),
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
impl<'a, 'b, WE: Write>
    From<(
        Option<ParseFn<'a, 'b, WE>>,
        Option<ParseFn<'a, 'b, WE>>,
        Precedence,
    )> for ParseRule<'a, 'b, WE>
{
    fn from(
        value: (
            Option<ParseFn<'a, 'b, WE>>,
            Option<ParseFn<'a, 'b, WE>>,
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

impl<'a, 'b, WE: Write> ParseRule<'a, 'b, WE> {
    fn new(
        suffix: Option<ParseFn<'a, 'b, WE>>,
        infix: Option<ParseFn<'a, 'b, WE>>,
        precedence: Precedence,
    ) -> Self {
        return (suffix, infix, precedence).into();
    }
}
