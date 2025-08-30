use crate::tokenizer::{Term, Atom};

use std::collections::HashMap;
pub(crate) type Substitution = HashMap<String, Term>;

pub fn print_substitution(subs: &Substitution) -> String {
    let pairs: Vec<String> = subs
        .iter()
        .map(|(var, term)| format!("{}: {}", var, format_term(term)))
        .collect();
    format!("{{{}}}", pairs.join(", "))
}

fn format_term(term: &Term) -> String { // helper
    match term {
        Term::Variable(v) => v.clone(),
        Term::Constant(c) => c.clone(),
        Term::Compound { name, args } => {
            let args_str: Vec<String> = args.iter().map(|t| format_term(t)).collect();
            format!("{}({})", name, args_str.join(", "))
        }
    }
}

pub fn unify_terms(t1: &Term, t2: &Term, subs: &mut Substitution) -> bool {
    match (t1, t2) {
        (Term::Variable(v), t) | (t, Term::Variable(v)) => {
            if let Some(bound) = subs.get(v).cloned() {
                unify_terms(&bound, t, subs)
            } else if occurs_check(v, t, subs) {
                false
            } else {
                subs.insert(v.clone(), (*t).clone());
                true
            }
        }

        (Term::Constant(c1), Term::Constant(c2)) => c1 == c2,

        (Term::Compound { name: n1, args: a1 },
            Term::Compound { name: n2, args: a2 },) => {
            if n1 != n2 || a1.len() != a2.len() {
                return false;
            }
            a1.iter().zip(a2.iter()).all(|(x, y)| unify_terms(x, y, subs))
        }

        _ => false,
    }
}

fn occurs_check(var: &str, term: &Term, subs: &Substitution) -> bool {
    match term {
        Term::Variable(v) => {
            if v == var { true }
            else if let Some(t) = subs.get(v) { occurs_check(var, t, subs) }
            else { false }
        }
        Term::Compound { args, .. } => args.iter().any(|t| occurs_check(var, t, subs)),
        _ => false,
    }
}

pub fn unify_atoms(a1: &Atom, a2: &Atom) -> Option<Substitution> {
    if a1.name != a2.name || a1.args.len() != a2.args.len() {
        return None;
    }
    let mut subs = Substitution::new();
    for (t1, t2) in a1.args.iter().zip(a2.args.iter()) {
        if !unify_terms(t1, t2, &mut subs) {
            return None;
        }
    }
    Some(subs)
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_constants_equal() {
        let mut subs = Substitution::new();
        let t1 = Term::Constant("john".to_string());
        let t2 = Term::Constant("john".to_string());
        assert!(unify_terms(&t1, &t2, &mut subs));
        //println!("{:?}", subs);
        assert!(subs.is_empty());
    }

    #[test]
    fn test_unify_constants_not_equal() {
        let mut subs = Substitution::new();
        let t1 = Term::Constant("john".to_string());
        let t2 = Term::Constant("mary".to_string());
        assert!(!unify_terms(&t1, &t2, &mut subs));
        println!("Test unify constants not equal substitution: {:?}", subs); // should be empty
    }

    #[test]
    fn test_unify_variable() {
        let mut subs = Substitution::new();
        let t1 = Term::Variable("X".to_string());
        let t2 = Term::Constant("mary".to_string());
        assert!(unify_terms(&t1, &t2, &mut subs));
        println!("Test unify variable substitution: {:?}", subs);
        assert_eq!(subs.get("X").unwrap(), &Term::Constant("mary".to_string()));
    }

    #[test]
    fn test_unify_compound() {
        let mut subs = Substitution::new();
        let t1 = Term::Compound {
            name: "parent".to_string(),
            args: vec![Term::Variable("X".to_string()), Term::Constant("mary".to_string())],
        };
        let t2 = Term::Compound {
            name: "parent".to_string(),
            args: vec![Term::Constant("john".to_string()), Term::Constant("mary".to_string())],
        };
        assert!(unify_terms(&t1, &t2, &mut subs));
        println!("Test unify compound substitution: {:?}", subs);
        assert_eq!(subs.get("X").unwrap(), &Term::Constant("john".to_string()));
    }

    #[test]
    fn test_occurs_check() {
        let mut subs = Substitution::new();
        let t1 = Term::Variable("X".to_string());
        let t2 = Term::Compound {
            name: "f".to_string(),
            args: vec![Term::Variable("X".to_string())],
        };
        assert!(occurs_check("X", &t2, &subs));
        println!("Test occurs check substitution: {:?}", subs); // should be empty
    }

    #[test]
    fn test_unify_atoms_success() {
        let a1 = Atom {
            name: "parent".to_string(),
            args: vec![Term::Variable("X".to_string()), Term::Variable("Y".to_string())],
        };
        let a2 = Atom {
            name: "parent".to_string(),
            args: vec![Term::Constant("john".to_string()), Term::Constant("mary".to_string())],
        };
        let subs = unify_atoms(&a1, &a2).unwrap();
        assert_eq!(subs.get("X").unwrap(), &Term::Constant("john".to_string()));
        assert_eq!(subs.get("Y").unwrap(), &Term::Constant("mary".to_string()));
    }

    #[test]
    fn test_unify_atoms_fail() {
        let a1 = Atom {
            name: "parent".to_string(),
            args: vec![Term::Variable("X".to_string()), Term::Constant("bob".to_string())],
        };
        let a2 = Atom {
            name: "parent".to_string(),
            args: vec![Term::Constant("john".to_string()), Term::Constant("mary".to_string())],
        };
        assert!(unify_atoms(&a1, &a2).is_none());
    }

}



