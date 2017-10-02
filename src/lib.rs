extern crate jam_depgraph;
#[macro_use]
extern crate nom;

mod action;
mod calculator1;

mod env;
mod jambase;
mod lang;
mod lexer;
mod list;
mod path;
mod rule;
mod target;
mod variable;

#[test]
fn it_works() {
    let data = jambase::data();
    println!("size of Jambase file: {}", data.len());
}

#[test]
fn calc1() {
    assert!(calculator1::parse_Term("22").is_ok());
    assert!(calculator1::parse_Term("(22)").is_ok());
    assert!(calculator1::parse_Term("((((22))))").is_ok());
    assert!(calculator1::parse_Term("((22)").is_err());
}