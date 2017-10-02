use list::List;

#[derive(Debug, PartialEq)]
pub struct Variable {
    data: List,
    bindings: List
}