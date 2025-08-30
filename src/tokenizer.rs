#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Term {
    Constant(String),
    Variable(String),
    Compound { name: String, args: Vec<Term> },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Atom {
    pub name: String,
    pub args: Vec<Term>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Statement { // Clause
    Fact(Atom),
    Rule { head: Atom, body: Vec<Atom> },
    Query { body: Vec<Atom> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Variable(String),
    LParen,
    RParen,
    Comma,
    Period,
    RuleArrow,
    QueryOperator,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub head: Atom,
    pub body: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub facts: Vec<Atom>,
    pub rules: Vec<Rule>,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_lowercase() {
            let mut s = c.to_string();
            i += 1;
            while i < chars.len() && chars[i].is_alphanumeric() {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::Identifier(s));
        } else if c.is_uppercase() {
            let mut s = c.to_string();
            i += 1;
            while i < chars.len() && chars[i].is_alphanumeric() {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::Variable(s));
        } else {
            match c {
                '(' => { tokens.push(Token::LParen); i += 1; },
                ')' => { tokens.push(Token::RParen); i += 1; },
                ',' => { tokens.push(Token::Comma); i += 1; },
                '.' => { tokens.push(Token::Period); i += 1; },
                ':' => {
                    if i + 1 < chars.len() && chars[i+1] == '-' {
                        tokens.push(Token::RuleArrow);
                        i += 2;
                    } else { panic!("Unexpected ':'"); }
                }
                '?' => {
                    if i + 1 < chars.len() && chars[i + 1] == '-' {
                        tokens.push(Token::QueryOperator);
                        i += 2;
                    } else {
                        panic!("Unexpected '?'");
                    }
                }
                _ => panic!("Unknown char: {}", c)
            }
        }
    }

    tokens
}


// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("parent(X, Y).");
        assert_eq!(tokens, vec![
            Token::Identifier("parent".to_string()),
            Token::LParen,
            Token::Variable("X".to_string()),
            Token::Comma,
            Token::Variable("Y".to_string()),
            Token::RParen,
            Token::Period
        ]);
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize("abc");
        assert_eq!(tokens, vec![Token::Identifier("abc".to_string())]);
    }

    #[test]
    fn test_tokenize_variable() {
        let tokens = tokenize("X");
        assert_eq!(tokens, vec![Token::Variable("X".to_string())]);
    }

    #[test]
    fn test_tokenize_symbols() {
        let tokens = tokenize("(),.");
        assert_eq!(tokens, vec![
            Token::LParen, Token::RParen, Token::Comma, Token::Period
        ]);
    }

    #[test]
    fn test_tokenize_rule_arrow() {
        let tokens = tokenize(":-");
        assert_eq!(tokens, vec![Token::RuleArrow]);
    }

    #[test]
    fn test_tokenize_query_operator() {
        let tokens = tokenize("?-");
        assert_eq!(tokens, vec![Token::QueryOperator]);
    }

    #[test]
    #[should_panic(expected = "Unexpected '?'")]
    fn test_tokenize_unexpected_question() {
        // Single '?' without '-' is invalid
        let _tokens = tokenize("?.");
    }

    #[test]
    #[should_panic(expected = "Unknown char: #")]
    fn test_tokenize_unknown_char() {
        let _tokens = tokenize("parent(X, y#)");
    }

    #[test]
    #[should_panic(expected = "Unexpected ':'")]
    fn test_tokenize_unexpected_colon() {
        // Single ':' without '-' is invalid
        let _tokens = tokenize(":.");
    }

}

