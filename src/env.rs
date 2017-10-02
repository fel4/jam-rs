use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use variable::Variable;

#[derive(Debug, PartialEq)]
pub struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    vars: HashMap<String, Variable>
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            parent: None,
            vars: HashMap::new()
        }
    }

    pub fn new_with_parent(env: &'a Environment) -> Environment<'a> {
        Environment {
            parent: Some(env),
            vars: HashMap::new()
        }
    }

    pub fn parent(&'a self) -> Option<&'a Environment<'a>> {
        self.parent
    }
}

impl<'a, T> Index<T> for Environment<'a> where T: ToString {
    type Output = Variable;

    fn index(&self, idx: T) -> &Self::Output {
        unimplemented!();
    }
}

impl<'a, T> IndexMut<T> for Environment<'a> where T: ToString {
    fn index_mut(&mut self, idx: T) -> &mut Self::Output {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_root_environment() {
        let e = Environment::new();
        assert_eq!(e.parent(), None);
    }

    #[test]
    fn test_new_child_environment() {
        let p = Environment::new();
        let c = Environment::new_with_parent(&p);
        assert!(c.parent().is_some());
        assert_eq!(p, *c.parent().unwrap());
    }
}