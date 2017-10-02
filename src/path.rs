use std::fmt;
use std::path;

#[derive(Debug, PartialEq)]
pub struct Path {
    path: Box<path::Path>,
    grist: Option<String>,
    member: Option<String>
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();    
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn works() {}
}