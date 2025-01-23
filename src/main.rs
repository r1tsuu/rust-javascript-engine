use std::{
    backtrace::Backtrace,
    cell::RefCell,
    collections::HashMap,
    process::ExitCode,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use execution_engine::ExpressionEvaluator;
use javascript_object::JavascriptObject;
use parser::Parser;
use tokenizer::Tokenizer;
mod error;
mod execution_engine;
mod execution_scope;
mod javascript_object;
mod memory;
mod parser;
mod tests;
mod tokenizer;

fn main() -> ExitCode {
    let source = String::from(
        "
let x = {x: {b: {c: \"1\", f: 2}}}
            ",
    ); // 3
    let mut tokens = vec![];

    for token in Tokenizer::from_source(source.to_string()).to_iter() {
        match token {
            Ok(token) => tokens.push(token),
            Err(err) => {
                err.print();
                return ExitCode::FAILURE;
            }
        };
    }

    let program = Parser::new(tokens).parse_program();
    println!("{program:#?}");

    ExitCode::SUCCESS

    //     let now = SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .unwrap()
    //         .as_micros();

    //     let source = String::from(
    //         "
    // function one() {
    //         return 1;
    // }

    // function apply(f) {
    //         return f();
    // }

    // apply(one);
    //         ",
    //     ); // 3

    //     match ExpressionEvaluator::evaluate_source(source) {
    //         Ok(value) => {
    //             println!(
    //                 "Executed with value: {value:?}, time: {}",
    //                 SystemTime::now()
    //                     .duration_since(UNIX_EPOCH)
    //                     .unwrap()
    //                     .as_micros()
    //                     - now
    //             );
    //             ExitCode::SUCCESS
    //         }
    //         Err(err) => {
    //             err.print();
    //             ExitCode::FAILURE
    //         }
    //     }
}
