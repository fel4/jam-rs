use jambase;
use super::*;

#[test]
fn test_dynamic_byte_buffer() {
    use std::str;
    let mut dbb = DynamicByteBuffer::new();
    assert_eq!(dbb.index(), 0);
    assert_eq!(dbb.len(), 0);
    assert_eq!(dbb.as_str().unwrap(), "");
    dbb.insert(b"hello");
    assert_eq!(dbb.as_str().unwrap(), "hello");
    let len = dbb.len();
    dbb.set_index(len);
    assert_eq!(dbb.index(), len);
    dbb.insert(b"hello");
    assert_eq!(dbb.as_str().unwrap(), "hellohello");
    dbb.set_index(5);
    dbb.insert(b"world");
    assert_eq!(dbb.as_str().unwrap(), "helloworldhello");
    dbb.swap(5, b"goodbye");
    assert_eq!(dbb.as_str().unwrap(), "hellogoodbyehello");
    assert_eq!(str::from_utf8(dbb.left()).unwrap(), "goodbyehello");
    dbb.set_index(0);
    assert_eq!(str::from_utf8(dbb.left()).unwrap(), "hellogoodbyehello");
    dbb.set_index(17);
    assert!(dbb.next().is_none());
    assert_eq!(str::from_utf8(dbb.left()).unwrap(), "");
    assert_eq!(str::from_utf8(dbb.slice(5, 12)).unwrap(), "goodbye");
    assert_eq!(dbb.slice_string(5, 12), "goodbye");
}

#[test]
fn test_tokenizer_simple() {
    let mut t = Tokenizer::new();
    t.include_data(b"bah");
    let tok = t.next();
    assert_token(tok, Some(Ok((0, Token::Ident("bah".to_string()), 3))));
    assert_token(t.next(), None);
}

#[test]
fn test_tokenizer_actionstring() {
    let mut t = Tokenizer::new();
    t.mode = TokenizerMode::Action;
    t.include_data(b"{ cmd.exe {}{}}");
    assert_eq!(t.buffer.len(), 15);
    assert_token(t.next(), Some(Ok((0, Token::LeftBracket, 1))));
    assert_token(t.next(), Some(Ok((1, Token::ActionString(" cmd.exe {}{}".to_string()), 14))));
    assert_token(t.next(), Some(Ok((14, Token::RightBracket, 15))));
}

#[test]
fn test_tokenizer_comment() {
    let mut t = Tokenizer::new();
    t.include_data(b"foo\n#a comment\nbar");
    assert_token(t.next(), Some(Ok((0, Token::Ident("foo".to_string()), 3))));
    assert_token(t.next(), Some(Ok((15, Token::Ident("bar".to_string()), 18))));
}

#[test]
fn test_tokenizer_statement() {
    let mut t = Tokenizer::new();
    t.include_data(b"local val = foo ;");
    assert_token(t.next(), Some(Ok((0, Token::Local, 5))));
    assert_token(t.next(), Some(Ok((6, Token::Ident("val".to_string()), 9))));
    assert_token(t.next(), Some(Ok((10, Token::Equals, 11))));
    assert_token(t.next(), Some(Ok((12, Token::Ident("foo".to_string()), 15))));
    assert_token(t.next(), Some(Ok((16, Token::SemiColon, 17))));
    assert_token(t.next(), None);
}

#[test]
fn test_tokenizer_string_literal() {
    let mut t = Tokenizer::new();
    t.include_data(b"local val = \"boom boom pow\" ;");
    assert_token(t.next(), Some(Ok((0, Token::Local, 5))));
    assert_token(t.next(), Some(Ok((6, Token::Ident("val".to_string()), 9))));
    assert_token(t.next(), Some(Ok((10, Token::Equals, 11))));
    assert_token(t.next(), Some(Ok((12, Token::StringLiteral("boom boom pow".to_string()), 27))));
    assert_token(t.next(), Some(Ok((28, Token::SemiColon, 29))));
    assert_token(t.next(), None);
}

#[test]
fn test_tokenizer_ident_with_expansion() {
    let mut t = Tokenizer::new();
    t.include_data(b"local val = ba$(BLAH) ;");
    assert_token(t.next(), Some(Ok((0, Token::Local, 5))));
    assert_token(t.next(), Some(Ok((6, Token::Ident("val".to_string()), 9))));
    assert_token(t.next(), Some(Ok((10, Token::Equals, 11))));
    assert_token(t.next(), Some(Ok((12, Token::Ident("ba$(BLAH)".to_string()), 21))));
    assert_token(t.next(), Some(Ok((22, Token::SemiColon, 23))));
    assert_token(t.next(), None);
}

#[test]
fn test_tokenizer_ident_with_expansion_start() {
    let mut t = Tokenizer::new();
    t.include_data(b"local val = $(BLAH)foo ;");
    assert_token(t.next(), Some(Ok((0, Token::Local, 5))));
    assert_token(t.next(), Some(Ok((6, Token::Ident("val".to_string()), 9))));
    assert_token(t.next(), Some(Ok((10, Token::Equals, 11))));
    assert_token(t.next(), Some(Ok((12, Token::Ident("$(BLAH)foo".to_string()), 22))));
    assert_token(t.next(), Some(Ok((23, Token::SemiColon, 24))));
    assert_token(t.next(), None);
}

#[test]
#[ignore]
fn test_tokenizer_jambase() {
    let mut t = Tokenizer::new();
    t.include_data(jambase::data());
    let tok = t.lookahead;
    assert_eq!(tok, Some((0, b'#')));
    assert!(t.advance().is_some());
    let mut t2 = t.next();
    while t2.is_some() {
        let t3 = t2.unwrap();
        assert!(t3.is_ok(), "t3 {:?} == {:?}", char::from(t.lookahead.unwrap_or((0, 0)).1), t3);
        println!("{:?}", t3);
        t2 = t.next();
    }
}

#[test]
fn test_tokenizer_rule() {
    let mut t = Tokenizer::new();
    t.include_data(b"rule Foo { }");
    assert_token(t.next(), Some(Ok((0, Token::Rule, 4))));
    assert_token(t.next(), Some(Ok((5, Token::Ident("Foo".to_string()), 8))));
    assert_token(t.next(), Some(Ok((9, Token::LeftBracket, 10))));
    assert_token(t.next(), Some(Ok((11, Token::RightBracket, 12))));
    assert_token(t.next(), None);
}

fn assert_token(t: Option<Spanned<Token, usize, LexerError>>, t2: Option<Spanned<Token, usize, LexerError>>) {
    assert_eq!(t.is_some(), t2.is_some(), "expected: {:?}, got: {:?}", t2, t);
    if t.is_some() {
        let t = t.unwrap();
        let t2 = t2.unwrap();
        assert_eq!(t.is_ok(), t2.is_ok());
        if t.is_ok() {
            let t = t.unwrap();
            let t2 = t2.unwrap();
            assert_eq!(t.0, t2.0, "expected: {:?}, got: {:?}", t2.0, t.0);
            assert_eq!(t.1, t2.1, "expected: {:?}, got: {:?}", t2.1, t.1);
            assert_eq!(t.2, t2.2, "expected: {:?}, got: {:?}", t2.2, t.2);
            
        }
    }
}