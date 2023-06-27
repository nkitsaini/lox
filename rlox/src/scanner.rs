#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single Character
    /// (
    LeftParen,
    /// )
    RightParen,
    /// {
    LeftBrace,
    /// }
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Semantic
    Error,
    Eof,
}

#[derive(Clone, Copy, Debug)]
pub struct Token<'a> {
    pub ty: TokenType,
    pub string: &'a str,
    pub line: usize,
}

pub struct Scanner<'a> {
    src: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            src: source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        use TokenType::*;

        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(Eof);
        }

        let c = self.advance();

        match c {
            '(' => return self.make_token(LeftParen),
            ')' => return self.make_token(RightParen),
            '{' => return self.make_token(LeftBrace),
            '}' => return self.make_token(RightBrace),
            ';' => return self.make_token(Semicolon),
            ',' => return self.make_token(Comma),
            '.' => return self.make_token(Dot),
            '-' => return self.make_token(Minus),
            '+' => return self.make_token(Plus),
            '/' => return self.make_token(Slash),
            '*' => return self.make_token(Star),

            '!' => self.conditional_token('=', BangEqual, Bang),
            '=' => self.conditional_token('=', EqualEqual, Equal),
            '<' => self.conditional_token('=', LessEqual, Less),
            '>' => self.conditional_token('=', GreaterEqual, Greater),

            '"' => self.string(),

            c if c.is_numeric() => self.number(),
            c if Self::is_alpha(c) => self.identifier(),

            _ => self.error_token("Unepxected charater"),
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;

        // TODO(perf): utf-8 indexing will slow things down.
        // If yes, make the content ascii-only.
        // or store the location with string.
        return self.src.chars().nth(self.current - 1).unwrap();
    }

    fn match_char(&mut self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.src.chars().nth(self.current).unwrap() != ch {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        self.src.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.src.chars().nth(self.current + 1).unwrap_or('\0')
    }
    fn is_at_end(&self) -> bool {
        return self.current >= self.src.len();
    }

    fn make_token(&self, ty: TokenType) -> Token<'a> {
        return Token {
            ty,

            // TODO(perf): utf-8 indexing will slow things down.
            // If yes, make the content ascii-only.
            // or store the location with string.
            string: &self.src[self.start..self.current],
            line: self.line,
        };
    }

    #[inline]
    fn conditional_token(&mut self, ch: char, if_yes: TokenType, if_no: TokenType) -> Token<'a> {
        if self.match_char(ch) {
            self.make_token(if_yes)
        } else {
            self.make_token(if_no)
        }
    }

    fn error_token(&self, msg: &'static str) -> Token<'a> {
        return Token {
            ty: TokenType::Error,
            string: msg,
            line: self.line,
        };
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // Rest of the line is a comment
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    }
                }
                _ => return,
            }
        }
    }

    fn string(&mut self) -> Token<'a> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return self.error_token("Unterminated String");
        }
        // Consume closing quote
        self.advance();
        return self.make_token(TokenType::String);
    }

    fn number(&mut self) -> Token<'a> {
        while self.peek().is_numeric() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_numeric() {
            self.advance();
            while self.peek().is_numeric() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        while Self::is_alpha(self.peek()) || self.peek().is_numeric() {
            self.advance();
        }
        return self.make_token(self.identifier_type());
    }

    fn is_alpha(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_'
    }

    fn check_keyword(&self, val: &str, ty: TokenType) -> TokenType {
        if &self.src[self.start + 1..self.current] == val {
            ty
        } else {
            TokenType::Identifier
        }
    }

    fn identifier_type(&self) -> TokenType {
        use TokenType::*;
        match self.src.chars().nth(self.start).unwrap() {
            'a' => return self.check_keyword("nd", And),
            'c' => return self.check_keyword("lass", Class),
            'e' => return self.check_keyword("lse", Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.src.chars().nth(self.start + 1).unwrap() {
                        // TODO(perf): unnecessary twice-check
                        'a' => return self.check_keyword("alse", False),
                        'o' => return self.check_keyword("or", For),
                        'u' => return self.check_keyword("un", Fun),
                        _ => (),
                    }
                }
            }
            'i' => return self.check_keyword("f", If),
            'n' => return self.check_keyword("il", Nil),
            'o' => return self.check_keyword("r", Or),
            'p' => return self.check_keyword("rint", Print),
            'r' => return self.check_keyword("eturn", Return),
            's' => return self.check_keyword("uper", Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.src.chars().nth(self.start + 1).unwrap() {
                        // TODO(perf): unnecessary twice-check
                        'h' => return self.check_keyword("his", This),
                        'r' => return self.check_keyword("rue", True),
                        _ => (),
                    }
                }
            }
            'v' => return self.check_keyword("ar", Var),
            'w' => return self.check_keyword("hile", While),
            _ => (),
        };
        return TokenType::Identifier;
    }
}
