// Engine Lang Lexer Implementation
// Deterministic LL(1) Tokenization Engine

use crate::ast::SourceLocation;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Module,
    Use,
    Type,
    Where,
    Entity,
    Struct,
    Enum,
    Trait,
    Impl,
    For,
    Action,
    Fn,
    Pub,
    Datasource,
    Register,
    Boundary,
    Route,
    Let,
    Mut,
    Atomic,
    Return,
    If,
    Else,
    Match,
    Contract,
    Invariant,
    True,
    False,
    NoneVal,

    // Identifiers & Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),

    // Delimiters
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    LBrace,      // {
    RBrace,      // }
    Comma,       // ,
    Semicolon,   // ;
    Colon,       // :
    DoubleColon, // ::
    Dot,         // .
    SafeDot,     // ?.
    At,          // @

    // Custom & Compound Operators
    CosineMetric, // <=>
    VecMul,       // @*
    VecAdd,       // @+
    VecSub,       // @-
    Pipe,         // |>
    Backpipe,     // <|
    AsyncPipe,    // ~>
    CausalEdge,   // ->
    FatArrow,     // =>
    AtomicTry,    // ?!
    Try,          // ?

    // Comparison Operators
    Equal,        // ==
    NotEqual,     // !=
    LessEqual,    // <=
    GreaterEqual, // >=
    Less,         // <
    Greater,      // >

    // Shift Operators
    Shl, // <<
    Shr, // >>

    // Assignment & Compound Assignment
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=

    // Arithmetic & Bitwise Operators
    Exp,       // **
    Plus,      // +
    Minus,     // -
    Star,      // *
    Slash,     // /
    Percent,   // %
    Ampersand, // &
    PipeChar,  // |
    Caret,     // ^
    Tilde,     // ~
    Bang,      // !
    LogAnd,    // &&
    LogOr,     // ||

    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: SourceLocation,
}

impl Token {
    pub fn new(kind: TokenKind, loc: SourceLocation) -> Self {
        Self { kind, loc }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            input,
            chars,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos].1)
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos + 1 < self.chars.len() {
            Some(self.chars[self.pos + 1].1)
        } else {
            None
        }
    }

    fn peek_char_at(&self, offset: usize) -> Option<char> {
        if self.pos + offset < self.chars.len() {
            Some(self.chars[self.pos + offset].1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current_char() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_char() == Some('/') {
                // Line comment
                while let Some(c) = self.current_char() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else if ch == '/' && self.peek_char() == Some('*') {
                // Block comment
                self.advance(); // consume /
                self.advance(); // consume *
                while let Some(c) = self.current_char() {
                    if c == '*' && self.peek_char() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();

        let loc = SourceLocation::new(self.line, self.col);

        let ch = match self.current_char() {
            Some(c) => c,
            None => return Ok(Token::new(TokenKind::Eof, loc)),
        };

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(c) = self.current_char() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = match ident.as_str() {
                "module" => TokenKind::Module,
                "use" => TokenKind::Use,
                "type" => TokenKind::Type,
                "where" => TokenKind::Where,
                "entity" => TokenKind::Entity,
                "struct" => TokenKind::Struct,
                "enum" => TokenKind::Enum,
                "trait" => TokenKind::Trait,
                "impl" => TokenKind::Impl,
                "for" => TokenKind::For,
                "action" => TokenKind::Action,
                "fn" => TokenKind::Fn,
                "pub" => TokenKind::Pub,
                "datasource" => TokenKind::Datasource,
                "register" => TokenKind::Register,
                "boundary" => TokenKind::Boundary,
                "route" => TokenKind::Route,
                "let" => TokenKind::Let,
                "mut" => TokenKind::Mut,
                "atomic" => TokenKind::Atomic,
                "return" => TokenKind::Return,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "match" => TokenKind::Match,
                "contract" => TokenKind::Contract,
                "invariant" => TokenKind::Invariant,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "None" => TokenKind::NoneVal,
                _ => TokenKind::Identifier(ident),
            };

            return Ok(Token::new(kind, loc));
        }

        // Numeric Literals (Hexadecimal or Decimal)
        if ch == '0' && (self.peek_char() == Some('x') || self.peek_char() == Some('X')) {
            self.advance(); // consume '0'
            self.advance(); // consume 'x' / 'X'
            let mut hex_str = String::new();
            while let Some(c) = self.current_char() {
                if c.is_ascii_hexdigit() {
                    hex_str.push(c);
                    self.advance();
                } else if c == '_' {
                    self.advance(); // skip underscore in literals
                } else {
                    break;
                }
            }
            if hex_str.is_empty() {
                return Err(format!("Empty hexadecimal literal at {}", loc));
            }
            let val = i64::from_str_radix(&hex_str, 16).map_err(|e| {
                format!(
                    "Invalid hexadecimal literal '0x{}' at {}: {}",
                    hex_str, loc, e
                )
            })?;
            return Ok(Token::new(TokenKind::Integer(val), loc));
        }

        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            let mut is_float = false;

            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else if c == '_' {
                    self.advance(); // skip underscore separator
                } else if c == '.' && self.peek_char().is_some_and(|next| next.is_ascii_digit()) {
                    is_float = true;
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            if is_float {
                let val: f64 = num_str
                    .parse()
                    .map_err(|e| format!("Invalid float '{}' at {}: {}", num_str, loc, e))?;
                return Ok(Token::new(TokenKind::Float(val), loc));
            } else {
                let val: i64 = num_str
                    .parse()
                    .map_err(|e| format!("Invalid integer '{}' at {}: {}", num_str, loc, e))?;
                return Ok(Token::new(TokenKind::Integer(val), loc));
            }
        }

        // String Literals
        if ch == '"' {
            self.advance(); // consume opening quote
            let mut s = String::new();
            while let Some(c) = self.current_char() {
                if c == '"' {
                    self.advance(); // consume closing quote
                    return Ok(Token::new(TokenKind::StringLiteral(s), loc));
                } else if c == '\\' {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(esc) => s.push(esc),
                        None => return Err(format!("Unterminated string escape at {}", loc)),
                    }
                } else {
                    s.push(c);
                    self.advance();
                }
            }
            return Err(format!("Unterminated string literal at {}", loc));
        }

        // Complex multi-character operators
        // <=> (CosineMetric)
        if ch == '<' && self.peek_char() == Some('=') && self.peek_char_at(2) == Some('>') {
            self.advance();
            self.advance();
            self.advance();
            return Ok(Token::new(TokenKind::CosineMetric, loc));
        }

        // @*, @+, @-
        if ch == '@' {
            match self.peek_char() {
                Some('*') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::VecMul, loc));
                }
                Some('+') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::VecAdd, loc));
                }
                Some('-') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::VecSub, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::At, loc));
                }
            }
        }

        // ?! (AtomicTry), ?., ?
        if ch == '?' {
            match self.peek_char() {
                Some('!') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::AtomicTry, loc));
                }
                Some('.') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::SafeDot, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Try, loc));
                }
            }
        }

        // ->, -=, --
        if ch == '-' {
            match self.peek_char() {
                Some('>') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::CausalEdge, loc));
                }
                Some('=') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::SubAssign, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Minus, loc));
                }
            }
        }

        // ~> (AsyncPipe)
        if ch == '~' {
            if self.peek_char() == Some('>') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::AsyncPipe, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Tilde, loc));
            }
        }

        // |>, ||, |
        if ch == '|' {
            match self.peek_char() {
                Some('>') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Pipe, loc));
                }
                Some('|') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::LogOr, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::PipeChar, loc));
                }
            }
        }

        // <|, <=, <<, <
        if ch == '<' {
            match self.peek_char() {
                Some('|') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Backpipe, loc));
                }
                Some('=') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::LessEqual, loc));
                }
                Some('<') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Shl, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Less, loc));
                }
            }
        }

        // =>, ==, =
        if ch == '=' {
            match self.peek_char() {
                Some('>') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::FatArrow, loc));
                }
                Some('=') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Equal, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Assign, loc));
                }
            }
        }

        // !=, !
        if ch == '!' {
            if self.peek_char() == Some('=') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::NotEqual, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Bang, loc));
            }
        }

        // >=, >>, >
        if ch == '>' {
            match self.peek_char() {
                Some('=') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::GreaterEqual, loc));
                }
                Some('>') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Shr, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Greater, loc));
                }
            }
        }

        // +=, +
        if ch == '+' {
            if self.peek_char() == Some('=') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::AddAssign, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Plus, loc));
            }
        }

        // **, *=, *
        if ch == '*' {
            match self.peek_char() {
                Some('*') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::Exp, loc));
                }
                Some('=') => {
                    self.advance();
                    self.advance();
                    return Ok(Token::new(TokenKind::MulAssign, loc));
                }
                _ => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Star, loc));
                }
            }
        }

        // /=, /
        if ch == '/' {
            if self.peek_char() == Some('=') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::DivAssign, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Slash, loc));
            }
        }

        // &&, &
        if ch == '&' {
            if self.peek_char() == Some('&') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::LogAnd, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Ampersand, loc));
            }
        }

        // ::, :
        if ch == ':' {
            if self.peek_char() == Some(':') {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::DoubleColon, loc));
            } else {
                self.advance();
                return Ok(Token::new(TokenKind::Colon, loc));
            }
        }

        // Single character tokens
        let token_kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            _ => return Err(format!("Unexpected character '{}' at {}", ch, loc)),
        };

        self.advance();
        Ok(Token::new(token_kind, loc))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
