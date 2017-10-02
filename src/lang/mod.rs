mod ast;
mod compiler;
mod grammar;
mod grammar2;
mod rule;

#[cfg(test)]
mod tests {
    use lexer;
    use super::*;

    #[test]
    fn grammar_params_test() {
        assert!(grammar::parse_params(tokenized("blah : foo")).is_ok());
        assert!(grammar::parse_params(tokenized("blah :")).is_ok());
        assert!(grammar::parse_params(tokenized("blah")).is_ok());
    }

    #[test]
    fn grammar_eflags_test() {
        assert!(grammar::parse_eflags(tokenized("quietly")).is_ok());
        assert!(grammar::parse_eflags(tokenized("9blah")).is_err());
    }

    #[test]
    fn grammar2_rule_test() {
        assert!(grammar2::parse_rule(tokenized("rule Foo { }")).is_ok());
        let r = "rule Foo { local Bar = Baz ; local Bar2 = Baz2 ; }";
        let result = grammar2::parse_rule(tokenized(r));
        assert!(result.is_ok());
    }

    #[test]
    fn grammar2_statement_test() {
        assert!(grammar2::parse_statement(tokenized("local Foo = bar ;")).is_ok());
        assert!(grammar2::parse_statement(tokenized("local Foo ;")).is_ok());
        assert!(grammar2::parse_statement(tokenized("Foo = bar ;")).is_ok());
        assert!(grammar2::parse_statement(tokenized("Foo ;")).is_ok());
    }

    fn tokenized(s: &str) -> lexer::Tokenizer {
        let mut t = lexer::Tokenizer::new();
        let bytes: Vec<u8> = s.bytes().collect();
        t.include_data(bytes.as_slice());
        t
    }
}


