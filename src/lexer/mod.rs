use std::io;
use std::path;
use std::str;

use self::helpers::*;
use self::LexerErrorCode::*;

mod helpers {
    pub fn is_quote(b: u8) -> bool {
        let c = char::from(b);
        c == '\'' || c == '"'
    }

    pub fn is_whitespace(b: u8) -> bool { char::from(b).is_whitespace() }

    pub fn is_not_ws(b: u8) -> bool { return !is_whitespace(b) }

    pub fn is_ident_start(b: u8) -> bool {
        match char::from(b) {
            '_' | '$' => true,
            'a'...'z' => true,
            'A'...'Z' => true,
            '0'...'9' => true,
            '/' | '\\' => true,
            '.' | '-' => true,
            '*' => true,
            _ => false
        }
    }

    pub fn is_ident_continue(b: u8) -> bool {
        match char::from(b) {
            '(' | ')' => true,
            '+' | ',' => true,
            '=' => true,
            _ if is_ident_start(b) => true,
            _ => false
        }
    }
}


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Token {
    Bang,
    BangEquals,
    Amper,
    AmperAmper,
    LeftParen,
    RightParen,
    PlusEquals,
    Colon,
    SemiColon,
    LeftAngle,
    LeftAngleEquals,
    Equals,
    RightAngle,
    RightAngleEquals,
    QuestionEquals,
    LeftBracket,
    RightBracket,
    Bar,
    BarBar,
    LeftBrace,
    RightBrace,
    Actions,
    ActionString(String),
    Bind,
    Break,
    Case,
    Continue,
    Default,
    Else,
    Existing,
    For,
    Ident(String),
    If,
    Ignore,
    In,
    Include,
    Local,
    Maxline,
    On,
    Piecemeal,
    Quietly,
    Return,
    Rule,
    StringLiteral(String),
    Switch,
    Together,
    Updated,
    While,
}

const KEYWORDS: &'static [(&'static str, Token)] = &[
    ("actions", Token::Actions),
    ("bind", Token::Bind),
    ("break", Token::Break),
    ("case", Token::Case),
    ("continue", Token::Continue),
    ("default", Token::Default),
    ("else", Token::Else),
    ("existing", Token::Existing),
    ("for", Token::For),
    ("if", Token::If),
    ("ignore", Token::Ignore),
    ("in", Token::In),
    ("include", Token::Include),
    ("local", Token::Local),
    ("maxline", Token::Maxline),
    ("on", Token::On),
    ("piecemeal", Token::Piecemeal),
    ("quietly", Token::Quietly),
    ("return", Token::Return),
    ("rule", Token::Rule),
    ("switch", Token::Switch),
    ("together", Token::Together),
    ("updated", Token::Updated),
    ("while", Token::While),
];

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexerErrorCode {
    UnexpectedToken,
    UnrecognizedToken,
    UnterminatedCodeBlock,
    UnterminatedStringLiteral
}

#[derive(Debug)]
pub struct LexerError {
    pub code: LexerErrorCode,
    pub location: usize,
}

impl LexerError {
    pub fn create<T>(c: LexerErrorCode, l: usize) -> Result<T, LexerError> {
        Err(LexerError { code: c, location: l })
    }
}

struct DynamicByteBuffer {
    v: Vec<u8>,
    idx: usize,
}

impl DynamicByteBuffer {
    pub fn new() -> DynamicByteBuffer {
        DynamicByteBuffer {
            v: Vec::new(),
            idx: 0
        }
    }

    pub fn as_slice(&self) -> &[u8] { self.v.as_slice() }

    pub fn as_str(&self) -> Result<&str, str::Utf8Error> { str::from_utf8(self.v.as_slice()) }

    pub fn insert(&mut self, buf: &[u8]) {
        use std::io::Write;
        let new_sz = self.v.len() + buf.len();
        let mut v = vec![0; new_sz];
        {
            let mut b = v.as_mut_slice();
            if self.idx > 0 {
                let res = b.write(&self.v[0..self.idx]);
                assert_eq!(res.unwrap(), self.idx);
            }
            let res = b.write(buf);
            assert_eq!(res.unwrap(), buf.len());
            if self.v.len() > self.idx {
                let res = b.write(&self.v[self.idx..]);
                assert_eq!(res.is_ok(), true);
            }
        }
        self.v = v;
    }

    pub fn left(&self) -> &[u8] { &self.v[self.idx..] }

    pub fn len(&self) -> usize { self.v.len() }

    pub fn index(&self) -> usize { self.idx }

    pub fn remove(&mut self, len: usize) -> Vec<u8> {
        let sz = if self.idx + len > self.v.len() { self.v.len() - self.idx } else { len };
        let mut v = Vec::with_capacity(sz);
        for x in 0..sz {
            v.push(self.v.remove(self.idx))
        }
        v
    }

    pub fn set_index(&mut self, new_idx: usize) {
        self.idx = if self.v.len() == 0 { 0 }
            else if new_idx >= self.v.len() { self.v.len() }
            else { new_idx }
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        let end = if end > self.len() { self.len() } else { end };
        &self.v[start..end]
    }

    pub fn slice_string(&self, start:usize, end: usize) -> String {
        str::from_utf8(self.slice(start, end)).unwrap().to_string()
    }

    pub fn swap(&mut self, out_bytes: usize, in_buf: &[u8]) {
        self.remove(out_bytes);
        self.insert(in_buf);
    }
}

impl Iterator for DynamicByteBuffer {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let byte = {
            let left = self.left();
            if left.len() > 0 { Some(left[0]) } else { None }
        };
        match byte {
            Some(b) => {
                let idx = { self.index() };
                self.set_index(idx + 1);
                Some((idx, b))
            },
            None => None
        }
    }
}

pub struct DataReader {
    buffer: Vec<u8>,
    ptr: usize,
}

impl DataReader {
    pub fn new(data: &[u8]) -> DataReader {
        DataReader { buffer: data.to_owned(), ptr: 0 }
    }

    fn end_index(&self, count: usize) -> usize {
        let req = self.ptr + count;
        let max = self.buffer.len() - 1;
        if req < max { req } else { max }
    }
}

impl io::Read for DataReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let new_ptr = self.end_index(buf.len());
        let mut i = 0;
        for byte in self.buffer[self.ptr..new_ptr].iter() {
            buf[i] = *byte;
            i += 1;
        }
        let sz = new_ptr - self.ptr;
        self.ptr = new_ptr;
        Ok(sz)
    }
}

impl io::Seek for DataReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(sz) => self.ptr = sz as usize,
            io::SeekFrom::Current(sz) => self.ptr = self.end_index(sz as usize),
            io::SeekFrom::End(sz) => self.ptr = self.end_index(sz as usize),
        }
        Ok(self.ptr as u64)
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum TokenizerMode {
    Action,
    Normal
}

pub struct Tokenizer {
    buffer: DynamicByteBuffer,
    lookahead: Option<(usize, u8)>,
    mode: TokenizerMode
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        let mut t = Tokenizer {
            buffer: DynamicByteBuffer::new(),
            lookahead: None,
            mode: TokenizerMode::Normal,
        };
        t.advance();
        t
    }

    pub fn advance(&mut self) -> Option<(usize, u8)> {
        self.lookahead = self.buffer.next();
        self.lookahead
    }

    fn action(&mut self, idx0: usize, start: u8, end: u8) -> Spanned<Token, usize, LexerError> {
        let mut balance = 1;
        loop {
            if let Some((idx, b)) = self.advance() {
                if b == b'\"' || b == b'\'' {
                    self.advance();
                    try!(self.string_literal(idx, b));
                    continue;
                } else if b == start {
                    balance += 1;
                } else if b == end {
                    balance -= 1;
                    if balance == 0 {
                        debug_assert!(balance == 0);
                        self.lookahead = Some((idx, b));
                        self.mode = TokenizerMode::Normal;
                        return Ok((idx0, Token::ActionString(self.buffer.slice_string(idx0, idx)), idx));
                    }
                }
            } else if balance > 0 {
                return LexerError::create(UnterminatedCodeBlock, idx0);
            } else {
                debug_assert!(balance == 0);
                self.mode = TokenizerMode::Normal;
                return Ok((idx0, Token::ActionString(self.buffer.slice_string(idx0, self.buffer.len())), self.buffer.len()));
            }
        }
    }

    fn action_scanner(&mut self) -> Option<Spanned<Token, usize, LexerError>> {
        loop {
            return match self.lookahead {
                Some((idx0, b'{')) => {
                    self.advance();
                    Some(Ok((idx0, Token::LeftBracket, idx0+1)))
                },
                Some((idx0, b'}')) => {
                    self.advance();
                    Some(Ok((idx0, Token::RightBracket, idx0+1)))
                },
                Some((idx0, _)) => {
                    Some(self.action(idx0, b'{', b'}'))
                },
                None => {
                    Some(LexerError::create(UnrecognizedToken, self.buffer.index()))
                }
            }
        }
    }

    fn ident(&mut self, idx0: usize) -> Spanned<Token, usize, LexerError> {
        let (start, word, end) = self.word(idx0);
        let token = KEYWORDS.iter()
                .filter(|&&(w, _)| w == word)
                .map(|&(_, ref t)| t.clone())
                .next()
                .unwrap_or_else(|| {
                    Token::Ident(word.to_string())
                });
        if token == Token::Actions { self.mode == TokenizerMode::Action; }
        Ok((start, token.clone(), end))
    }

    pub fn include_data(&mut self, buffer: &[u8]) {
        self.buffer.insert(buffer);
        if self.lookahead == None { self.advance(); }
    }

    pub fn include_file<P: AsRef<path::Path>>(&mut self, path: P) -> io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        let mut f = File::open(path)?;
        let mut b = Vec::new();
        f.read_to_end(&mut b);
        Ok(self.include_data(b.as_slice()))
    }

    pub fn index(&self) -> usize { self.buffer.index() }

    fn normal_scanner(&mut self) -> Option<Spanned<Token, usize, LexerError>> {
        loop {
            return match self.lookahead {
                Some((idx0, b'&')) => {
                    match self.advance() {
                        Some((idx1, b'&')) => {
                            self.advance();
                            Some(Ok((idx0, Token::AmperAmper, idx1+1)))
                        },
                        _ => Some(Ok((idx0, Token::Amper, idx0+1)))
                    }
                },
                Some((idx0, b'|')) => {
                    match self.advance() {
                        Some((idx1, b'|')) => {
                            self.advance();
                            Some(Ok((idx0, Token::BarBar, idx1+1)))
                        },
                        _ => Some(Ok((idx0, Token::Bar, idx0+1)))
                    }
                },
                Some((idx0, b'!')) => {
                    match self.advance() {
                        Some((idx1, b'=')) => {
                            self.advance();
                            Some(Ok((idx0, Token::BangEquals, idx1+1)))
                        },
                        _ => Some(Ok((idx0, Token::Bang, idx0+1)))
                    }
                },
                Some((idx0, b'<')) => {
                    match self.advance() {
                        Some((idx1, b'=')) => {
                            self.advance();
                            Some(Ok((idx0, Token::LeftAngleEquals, idx1+1)))
                        },
                        _ => Some(Ok((idx0, Token::LeftAngle, idx0+1)))
                    }
                },
                Some((idx0, b'>')) => {
                    match self.advance() {
                        Some((idx1, b'=')) => {
                            self.advance();
                            Some(Ok((idx0, Token::RightAngleEquals, idx1+1)))
                        },
                        _ => Some(Ok((idx0, Token::RightAngle, idx0+1)))
                    }
                },
                Some((idx0, b'?')) => {
                    match self.advance() {
                        Some((idx1, b'=')) => {
                            self.advance();
                            Some(Ok((idx0, Token::QuestionEquals, idx1+1)))
                        },
                        _ => Some(LexerError::create(UnexpectedToken, idx0))
                    }
                },
                Some((idx0, b'+')) => {
                    match self.advance() {
                        Some((idx1, b'=')) => {
                            self.advance();
                            Some(Ok((idx0, Token::PlusEquals, idx1+1)))
                        },
                        _ => Some(LexerError::create(UnexpectedToken, idx0))
                    }
                },
                Some((idx0, b'(')) => {
                    self.advance();
                    Some(Ok((idx0, Token::LeftParen, idx0+1)))
                },
                Some((idx0, b')')) => {
                    self.advance();
                    Some(Ok((idx0, Token::RightParen, idx0+1)))
                },
                Some((idx0, b'{')) => {
                    self.advance();
                    Some(Ok((idx0, Token::LeftBracket, idx0+1)))    
                },
                Some((idx0, b'}')) => {
                    self.advance();
                    Some(Ok((idx0, Token::RightBracket, idx0+1)))
                },
                Some((idx0, b'[')) => {
                    self.advance();
                    Some(Ok((idx0, Token::LeftBrace, idx0+1)))
                },
                Some((idx0, b']')) => {
                    self.advance();
                    Some(Ok((idx0, Token::RightBrace, idx0+1)))
                },
                Some((idx0, b';')) => {
                    self.advance();
                    Some(Ok((idx0, Token::SemiColon, idx0+1)))
                },
                Some((idx0, b':')) => {
                    self.advance();
                    Some(Ok((idx0, Token::Colon, idx0+1)))
                },
                Some((idx0, b'=')) => {
                    self.advance();
                    Some(Ok((idx0, Token::Equals, idx0+1)))
                },
                Some((idx, b)) if is_quote(b) => {
                    self.advance();
                    Some(self.string_literal(idx, b))
                },
                Some((_, b'#')) => {
                    self.take_until_and_consume(|b| b == b'\n');
                    continue;
                },
                Some((_, b)) if is_whitespace(b) => {
                    self.advance();
                    continue;
                },
                Some((idx0, b)) if is_ident_start(b) => {
                    Some(self.ident(idx0))
                },
                Some((idx0, _)) => {
                    Some(LexerError::create(UnrecognizedToken, idx0))
                },
                None => None
            }
        }
    }

    pub fn seek(&mut self, new_idx: usize) { self.buffer.set_index(new_idx); }

    fn string_literal(&mut self, idx0: usize, delim: u8) -> Spanned<Token, usize, LexerError> {
        let mut escape = false;
        let term = |b: u8| {
            if escape {
                escape = false;
                false
            } else if b == b'\\' {
                escape = true;
                false
            } else if b == delim {
                true
            } else {
                false
            }
        };
        match self.take_until(term) {
            Some(idx1) => {
                self.advance(); // consume the closing delimiter.
                let s = self.buffer.slice_string(idx0+1, idx1);
                Ok((idx0, Token::StringLiteral(s), idx1+1))
            },
            None => {
                LexerError::create(UnterminatedStringLiteral, idx0)
            }
        }
    }

    fn take_while<F>(&mut self, mut keep_going: F) -> Option<usize>
        where F: FnMut(u8) -> bool
    {
        self.take_until(|b| !keep_going(b))
    }

    fn take_until<F>(&mut self, mut terminate: F) -> Option<usize>
        where F: FnMut(u8) -> bool
    {
        loop {
            match self.lookahead {
                None => { return None; },
                Some((idx0, b)) => {
                    if terminate(b) { return Some(idx0); } else { self.advance(); }
                }
            }
        }
    }

    fn take_until_and_consume<F>(&mut self, mut terminate: F) -> Option<usize>
        where F: FnMut(u8) -> bool
    {
        self.take_until(terminate).and_then(|_| {
            self.advance().map(|p| {p.0})
        })
    }

    fn word(&mut self, idx0: usize) -> (usize, String, usize) {
        match self.take_while(is_not_ws) {
            Some(idx1) => (idx0, self.buffer.slice_string(idx0, idx1), idx1),
            None => (idx0, self.buffer.slice_string(idx0, self.buffer.len()), self.buffer.len())
        }
    }
}

impl Iterator for Tokenizer {
    type Item = Spanned<Token, usize, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mode == TokenizerMode::Action {
            return self.action_scanner();
        }
        return self.normal_scanner()
    }
}

#[cfg(test)]
mod tests;