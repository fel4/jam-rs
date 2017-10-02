use lexer::Token;
use super::ast;

pub fn saw_rule(name: String) {
    println!("saw rule name: {}", name);
}

pub fn saw_local_assignment(name: String, value: String) {
    println!("saw local assignment: {} => {}", name, value);
}

pub fn saw_local_declaration(name: String) {
    println!("saw local declaration: {}", name);
}

pub fn saw_assignment(ao: ast::AssignmentExpr) {
    match ao.value {
        Some(val) => {
            if ao.local {
                println!("saw local assignment: {} => {}", ao.name, val);
            } else {
                println!("saw global assignment: {} => {}", ao.name, val);
            }
        },
        None => {
            if ao.local {
                println!("saw local declaration: {}", ao.name);
            } else {
                println!("saw global declaration: {}", ao.name);
            }
        }
    }
}
