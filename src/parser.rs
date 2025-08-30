// Recursive Descent Parser (Top-Down)
use crate::tokenizer::{Token, Statement, Atom, Term, Database, Rule};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.peek() {
            Some(tok) if tok == expected => {
                self.consume();
                Ok(())
            }
            other => Err(format!("Expected {:?}, got {:?}", expected, other)),
        }
    }

    pub fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek() {
            Some(Token::QueryOperator) => {
                self.consume(); // consume "?-"
                let mut body = vec![self.parse_atom()?];
                while let Some(Token::Comma) = self.peek() {
                    self.consume();
                    body.push(self.parse_atom()?);
                }
                self.expect(&Token::Period)?;
                Ok(Statement::Query { body })
            }
            _ => {
                let head = self.parse_atom()?;
                match self.peek() {
                    Some(Token::Period) => {
                        self.consume();
                        Ok(Statement::Fact(head))
                    }
                    Some(Token::RuleArrow) => {
                        self.consume();
                        let mut body = vec![self.parse_atom()?];
                        while let Some(Token::Comma) = self.peek() {
                            self.consume();
                            body.push(self.parse_atom()?);
                        }
                        self.expect(&Token::Period)?;
                        Ok(Statement::Rule { head, body })
                    }
                    other => Err(format!("Expected '.' or ':-' after atom, got {:?}", other)),
                }
            }
        }
    }

    fn parse_atom(&mut self) -> Result<Atom, String> {
        if let Some(Token::Identifier(name)) = self.peek() {
            let name = name.clone();
            self.consume();
            let args = if let Some(Token::LParen) = self.peek() {
                self.consume();
                let mut args = vec![self.parse_term()?];
                while let Some(Token::Comma) = self.peek() {
                    self.consume();
                    args.push(self.parse_term()?);
                }
                self.expect(&Token::RParen)?;
                args
            } else {
                vec![]
            };
            Ok(Atom { name, args })
        } else {
            Err(format!("Expected identifier for atom, got {:?}", self.peek()))
        }
    }

    fn parse_term(&mut self) -> Result<Term, String> {
        match self.peek() {
            Some(Token::Identifier(_)) => {
                let atom = self.parse_atom()?;
                if atom.args.is_empty() {
                    Ok(Term::Constant(atom.name))
                } else {
                    Ok(Term::Compound {
                        name: atom.name,
                        args: atom.args,
                    })
                }
            }
            Some(Token::Variable(name)) => {
                let name = name.clone();
                self.consume();
                Ok(Term::Variable(name))
            }
            other => Err(format!("Expected term, got {:?}", other)),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();

        while self.peek().is_some() {
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => return Err(e),
            }
        }

        Ok(stmts)
    }
}


fn parse_tokens(tokens: Vec<Token>) -> Result<Statement, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}

pub fn build_database(stmts: Vec<Statement>) -> Database {
    let mut facts = Vec::new();
    let mut rules = Vec::new();

    for stmt in stmts {
        match stmt {
            Statement::Fact(atom) => facts.push(atom),
            Statement::Rule { head, body } => rules.push(Rule { head, body }),
            Statement::Query { .. } => {
                // ignore here; queries will be parsed separately from console
            }
        }
    }

    Database { facts, rules }
}

/*pub fn parse_query(tokens: Vec<Token>) -> Vec<Atom> {
    let mut parser = Parser::new(tokens);
    match parser.parse_statement() {
        Statement::Query { body } => body,
        _ => panic!("Expected query"),
    }
}*/
pub fn parse_query(tokens: Vec<Token>) -> Result<Vec<Atom>, String> {
    let mut parser = Parser::new(tokens);
    match parser.parse_statement() {
        Ok(Statement::Query { body }) => Ok(body),
        _ => Err("Expected query".to_string()),
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{tokenize};
    use crate::tokenizer::Term::{Compound, Constant, Variable};

    #[test]
    fn test_parse() {
        let input = "?- ancestor(father(john), X), parent(X, mary).";
        let tokens = tokenize(input);
        match tokens {
            Ok(T) => {
                let mut parser = Parser::new(T);
                let stmt = parser.parse_statement();

                match stmt {
                    Ok(stmt) => {
                        //println!("{:#?}", stmt);
                        let st = Statement::Query {
                            body: vec![
                                Atom {
                                    name: "ancestor".to_string(),
                                    args: vec![
                                        Compound {
                                            name: String::from("father"),
                                            args: vec![
                                                Constant(String::from("john")),
                                            ]
                                        },
                                        Variable(String::from("X")),
                                    ]
                                },
                                Atom {
                                    name: "parent".to_string(),
                                    args: vec![
                                        Variable(String::from("X")),
                                        Constant(String::from("mary")),
                                    ]
                                },
                            ]
                        };
                        //println!("{:#?}", st);

                        assert_eq!(stmt, st)
                    }
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                }
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        }


    }

    #[test]
    fn test_parse_fact() {
        // parent(john, mary).
        let tokens = vec![
            Token::Identifier("parent".to_string()),
            Token::LParen,
            Token::Identifier("john".to_string()),
            Token::Comma,
            Token::Identifier("mary".to_string()),
            Token::RParen,
            Token::Period
        ];
        let stmt = parse_tokens(tokens);
        match stmt {
            Ok(stmt) => {
                //println!("{:#?}", stmt);
                assert_eq!(
                    stmt,
                    Statement::Fact(Atom {
                        name: "parent".to_string(),
                        args: vec![
                            Constant("john".to_string()),
                            Constant("mary".to_string())
                        ]
                    })
                );
            },
            Err(err) => {}
        }
    }

    #[test]
    fn test_parse_rule() {
        // grandparent(X, Y) :- parent(X, Z), parent(Z, Y).
        let tokens = vec![
            Token::Identifier("grandparent".to_string()),
            Token::LParen,
            Token::Variable("X".to_string()),
            Token::Comma,
            Token::Variable("Y".to_string()),
            Token::RParen,
            Token::RuleArrow,
            Token::Identifier("parent".to_string()),
            Token::LParen,
            Token::Variable("X".to_string()),
            Token::Comma,
            Token::Variable("Z".to_string()),
            Token::RParen,
            Token::Comma,
            Token::Identifier("parent".to_string()),
            Token::LParen,
            Token::Variable("Z".to_string()),
            Token::Comma,
            Token::Variable("Y".to_string()),
            Token::RParen,
            Token::Period
        ];
        let stmt = parse_tokens(tokens);
        assert!(matches!(stmt, Ok(Statement::Rule { .. })));
    }

    #[test]
    fn test_parse_query() {
        // ?- parent(X, mary).
        let tokens = vec![
            Token::QueryOperator,
            Token::Identifier("parent".to_string()),
            Token::LParen,
            Token::Variable("X".to_string()),
            Token::Comma,
            Token::Identifier("mary".to_string()),
            Token::RParen,
            Token::Period
        ];
        let stmt = parse_tokens(tokens);
        assert!(matches!(stmt, Ok(Statement::Query { .. })));
    }

    #[test]
    fn test_parse_nested_term() {
        // ?- ancestor(father(john), X).
        let tokens = vec![
            Token::QueryOperator,
            Token::Identifier("ancestor".to_string()),
            Token::LParen,
            Token::Identifier("father".to_string()),
            Token::LParen,
            Token::Identifier("john".to_string()),
            Token::RParen,
            Token::Comma,
            Token::Variable("X".to_string()),
            Token::RParen,
            Token::Period
        ];
        let stmt = parse_tokens(tokens);
        if let Ok(Statement::Query { body }) = stmt {
            if let Term::Compound { name, args } = &body[0].args[0] {
                assert_eq!(name, "father");
            } else {
                panic!("Nested term parsing failed");
            }
        } else {
            panic!("Query parsing failed");
        }
    }

}
