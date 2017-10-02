#[derive(Clone, Debug, PartialEq)]
pub struct List(Vec<String>);

impl List {
    pub fn new() -> List {
        List(Vec::new())
    }

    pub fn append<T: ToString>(&mut self, s: T) {
        self.0.push(s.to_string());
    }

    pub fn expand(&self) -> List {
        unimplemented!();
    }

    pub fn get(&self, idx: usize) -> Option<&str> {
        self.0.get(idx).map(|s| s.as_str())
    }

    pub fn length(&self) -> usize { self.0.len() }

    pub fn print(&self) {
        unimplemented!();
    }

    pub fn print_quoted(&self) {
        unimplemented!();
    }

    pub fn sublist(&self, start: usize, count: usize) -> List {
        let mut l = Self::new();
        l.0.extend_from_slice(&self.0[start..(start + count)]);
        l
    }
}

#[cfg(test)]
mod tests {
    use super::List;

    #[test]
    fn new_list_is_empty() {
        let l = List::new();
        assert!(l.length() == 0);
    }

    #[test]
    fn appending_to_a_list_updates_length() {
        let mut l = List::new();
        l.append("Hello");
        assert!(l.length() == 1);
    }

    #[test]
    fn appending_and_retrieving_a_string() {
        let mut l = List::new();
        l.append("Hello");
        assert!(l.get(0).unwrap() == "Hello");
    }
}