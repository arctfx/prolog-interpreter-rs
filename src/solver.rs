use crate::tokenizer::{Statement, Atom, Term};
use crate::unificator::{Substitution, unify_atoms, unify_terms};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResolutionNode {
    pub goal: Option<Atom>,         // None for the root
    pub subs: Substitution,         // Current substitution at this node
    pub children: Vec<ResolutionNode>,
}

pub fn resolve_query(query: &[Atom], db: &Vec<Statement>) -> ResolutionNode {
    fn resolve(
        goal: &[Atom],
        db: &Vec<Statement>,
        subs: &mut Substitution,
        counter: &mut usize,
        current_goal: Option<Atom>,
    ) -> ResolutionNode {
        if goal.is_empty() {
            return ResolutionNode {
                goal: current_goal,
                subs: subs.clone(),
                children: vec![],
            };
        }

        let first = &goal[0];
        let rest = &goal[1..];

        let mut children_nodes = vec![];

        for stmt in db {
            match stmt {
                Statement::Fact(fact) => {
                    let mut new_subs = subs.clone();
                    if let Some(s) = unify_atoms(first, fact) {
                        for (k, v) in s {
                            new_subs.insert(k, v);
                        }
                        let child = resolve(rest, db, &mut new_subs, counter, Some(fact.clone()));
                        children_nodes.push(child);
                    }
                }
                Statement::Rule { head, body } => {
                    // Freshen rule consistently
                    let (fresh_head, fresh_body) = fresh_rule(head, body, counter);

                    let mut new_subs = subs.clone();
                    if let Some(s) = unify_atoms(first, &fresh_head) {
                        for (k, v) in s {
                            new_subs.insert(k, v);
                        }

                        let mut new_goals = fresh_body;
                        new_goals.extend_from_slice(rest);
                        let child = resolve(&new_goals, db, &mut new_subs, counter, Some(fresh_head));
                        children_nodes.push(child);
                    }
                }
                _ => {}
            }
        }

        ResolutionNode {
            goal: current_goal,
            subs: subs.clone(),
            children: children_nodes,
        }
    }

    let mut counter = 0;
    resolve(query, db, &mut Substitution::new(), &mut counter, None)
}

pub fn fresh_rule(head: &Atom, body: &[Atom], counter: &mut usize) -> (Atom, Vec<Atom>) {
    let mut var_map: HashMap<String, String> = HashMap::new();

    fn freshen_term(term: &Term, counter: &mut usize, var_map: &mut HashMap<String, String>) -> Term {
        match term {
            Term::Variable(v) => {
                let name = var_map.entry(v.clone()).or_insert_with(|| {
                    *counter += 1;
                    format!("{}_{}", v, counter)
                });
                Term::Variable(name.clone())
            }
            Term::Constant(c) => Term::Constant(c.clone()),
            Term::Compound { name, args } => Term::Compound {
                name: name.clone(),
                args: args.iter().map(|t| freshen_term(t, counter, var_map)).collect(),
            },
        }
    }

    let fresh_head = Atom {
        name: head.name.clone(),
        args: head.args.iter().map(|t| freshen_term(t, counter, &mut var_map)).collect(),
    };

    let fresh_body: Vec<Atom> = body
        .iter()
        .map(|atom| Atom {
            name: atom.name.clone(),
            args: atom.args.iter().map(|t| freshen_term(t, counter, &mut var_map)).collect(),
        })
        .collect();

    (fresh_head, fresh_body)
}


// Backwards substitution
pub fn extract_query_results(tree: &ResolutionNode, query_vars: &[String]) -> Vec<Substitution> {
    fn merge_subs(parent: &Substitution, child: &Substitution) -> Substitution { // helper
        let mut merged = parent.clone();
        for (k, v) in child {
            merged.insert(k.clone(), v.clone());
        }
        merged
    }
    fn apply_subs(term: &Term, subs: &Substitution) -> Term { // helper
        match term {
            Term::Variable(v) => {
                if let Some(t) = subs.get(v) {
                    apply_subs(t, subs)
                } else {
                    term.clone()
                }
            }
            Term::Constant(_) => term.clone(),
            Term::Compound { name, args } => Term::Compound {
                name: name.clone(),
                args: args.iter().map(|t| apply_subs(t, subs)).collect(),
            },
        }
    }

    if tree.children.is_empty() {
        // Leaf node: apply substitution to query variables
        let mut filtered = Substitution::new();
        for var in query_vars {
            let val = apply_subs(&Term::Variable(var.clone()), &tree.subs);
            filtered.insert(var.clone(), val);
        }
        return vec![filtered];
    }

    let mut results = vec![];
    for child in &tree.children {
        let merged = merge_subs(&tree.subs, &child.subs);
        let mut child_node = ResolutionNode {
            goal: child.goal.clone(),
            subs: merged,
            children: child.children.clone(),
        };
        let mut child_results = extract_query_results(&child_node, query_vars);
        results.append(&mut child_results);
    }
    results
}

// Helpers
pub fn get_query_vars(query: &[Atom]) -> Vec<String> {
    let mut vars_set = HashSet::new();
    for atom in query {
        for term in &atom.args {
            get_term_vars(term, &mut vars_set);
        }
    }
    let mut vars: Vec<String> = vars_set.into_iter().collect();
    vars.sort(); // optional, to have deterministic order
    vars
}

pub fn get_term_vars(term: &Term, vars: &mut HashSet<String>) {
    match term {
        Term::Variable(v) => {
            vars.insert(v.clone());
        }
        Term::Constant(_) => {}
        Term::Compound { args, .. } => {
            for t in args {
                get_term_vars(t, vars);
            }
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::unificator::print_substitution;
    use super::*;

    fn print_tree(node: &ResolutionNode, depth: usize) { // helper
        let indent = "  ".repeat(depth);
        println!("{}Goal: {:?}, Sub: {:?}", indent, node.goal, node.subs);
        for child in &node.children {
            print_tree(child, depth + 1);
        }
    }

    #[test]
    fn test_fresh_rule_simple() {
        let head = Atom {
            name: "p".to_string(),
            args: vec![Term::Variable("X".to_string())],
        };
        let body = vec![
            Atom {
                name: "q".to_string(),
                args: vec![Term::Variable("X".to_string())],
            }
        ];
        let mut counter = 0;
        let (fresh_head, fresh_body) = fresh_rule(&head, &body, &mut counter);

        assert_eq!(fresh_head.args[0], Term::Variable("X_1".to_string()));
        assert_eq!(fresh_body[0].args[0], Term::Variable("X_1".to_string()));
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_fresh_rule_multiple_vars() {
        let head = Atom {
            name: "p".to_string(),
            args: vec![Term::Variable("X".to_string()), Term::Variable("Y".to_string())],
        };
        let body = vec![
            Atom {
                name: "q".to_string(),
                args: vec![Term::Variable("X".to_string()), Term::Variable("Y".to_string())],
            }
        ];
        let mut counter = 0;
        let (fresh_head, fresh_body) = fresh_rule(&head, &body, &mut counter);

        assert_eq!(fresh_head.args[0], Term::Variable("X_1".to_string()));
        assert_eq!(fresh_head.args[1], Term::Variable("Y_2".to_string()));
        assert_eq!(fresh_body[0].args[0], Term::Variable("X_1".to_string()));
        assert_eq!(fresh_body[0].args[1], Term::Variable("Y_2".to_string()));
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_extract_query_results_leaf_node() {
        let query_vars = vec!["X".to_string()];
        let leaf = ResolutionNode {
            goal: Option::from(Atom {
                name: "p".to_string(),
                args: vec![Term::Variable("X".to_string())],
            }),
            subs: {
                let mut s = Substitution::new();
                s.insert("X".to_string(), Term::Constant("a".to_string()));
                s
            },
            children: vec![],
        };
        let results = extract_query_results(&leaf, &query_vars);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X").unwrap(), &Term::Constant("a".to_string()));
    }

    #[test]
    fn test_extract_query_results_with_child() {
        let query_vars = vec!["X".to_string()];
        let child = ResolutionNode {
            goal: Option::from(Atom {
                name: "q".to_string(),
                args: vec![Term::Variable("X".to_string())],
            }),
            subs: {
                let mut s = Substitution::new();
                s.insert("X".to_string(), Term::Constant("b".to_string()));
                s
            },
            children: vec![],
        };
        let parent = ResolutionNode {
            goal: Option::from(Atom {
                name: "p".to_string(),
                args: vec![Term::Variable("X".to_string())],
            }),
            subs: Substitution::new(),
            children: vec![child],
        };
        let results = extract_query_results(&parent, &query_vars);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get("X").unwrap(), &Term::Constant("b".to_string()));
    }

    #[test]
    fn test_get_query_vars() {
        let query = vec![
            Atom {
                name: "p".to_string(),
                args: vec![Term::Variable("X".to_string()), Term::Constant("a".to_string())],
            },
            Atom {
                name: "q".to_string(),
                args: vec![Term::Variable("Y".to_string())],
            },
        ];
        let vars = get_query_vars(&query);
        assert_eq!(vars, vec!["X".to_string(), "Y".to_string()]);
    }

    #[test]
    fn test_get_term_vars_nested() {
        let term = Term::Compound {
            name: "f".to_string(),
            args: vec![
                Term::Variable("X".to_string()),
                Term::Compound {
                    name: "g".to_string(),
                    args: vec![Term::Variable("Y".to_string())],
                },
            ],
        };
        let mut vars_set = std::collections::HashSet::new();
        get_term_vars(&term, &mut vars_set);
        let mut vars: Vec<String> = vars_set.into_iter().collect();
        vars.sort();
        assert_eq!(vars, vec!["X".to_string(), "Y".to_string()]);
    }

    #[test]
    fn test_resolution_tree() {
        let db = vec![
            Statement::Fact(Atom {
                name: "parent".to_string(),
                args: vec![Term::Constant("john".to_string()), Term::Constant("mary".to_string())],
            }),
            Statement::Fact(Atom {
                name: "parent".to_string(),
                args: vec![Term::Constant("mary".to_string()), Term::Constant("pesho".to_string())],
            }),
            Statement::Rule {
                head: Atom {
                    name: "grandparent".to_string(),
                    args: vec![Term::Variable("X".to_string()), Term::Variable("Y".to_string())],
                },
                body: vec![
                    Atom {
                        name: "parent".to_string(),
                        args: vec![Term::Variable("X".to_string()), Term::Variable("Z".to_string())],
                    },
                    Atom {
                        name: "parent".to_string(),
                        args: vec![Term::Variable("Z".to_string()), Term::Variable("Y".to_string())],
                    },
                ],
            },
        ];

        // ?- grandparent(john, Y).
        let query = vec![
            Atom {
                name: "grandparent".to_string(),
                args: vec![Term::Constant("john".to_string()), Term::Variable("Y".to_string())],
            }
        ];

        let tree = resolve_query(&query, &db);

        print_tree(&tree, 0);

        let query_vars = get_query_vars(&*query); //vec!["Y".to_string()];
        let results = extract_query_results(&tree, &query_vars);

        for s in results {
            println!("{}", print_substitution(&s));
            //println!("{:?}", s);
        }
    }
}