use std::{
    backtrace::Backtrace,
    cell::RefCell,
    collections::HashMap,
    process::ExitCode,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use execution_engine::ExpressionEvaluator;
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
    //     let source = String::from(
    //         "
    // x(1,2)
    //         ",
    //     ); // 3
    //     let mut tokens = vec![];

    //     let mut z = 1;

    //     let mut a = || z = 3;

    //     a();

    //     for token in Tokenizer::from_source(source.to_string()).to_iter() {
    //         match token {
    //             Ok(token) => tokens.push(token),
    //             Err(err) => {
    //                 err.print();
    //                 return ExitCode::FAILURE;
    //             }
    //         };
    //     }

    //     let program = Parser::new(tokens).parse_program();
    //     println!("{program:#?}");

    //     ExitCode::SUCCESS

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();

    let source = String::from(
        "
let c = 1;
        function a(x) {
    return c+1;}; a(3);",
    ); // 3

    match ExpressionEvaluator::evaluate_source(source) {
        Ok(value) => {
            println!(
                "Executed with value: {value:?}, time: {}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    - now
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            err.print();
            ExitCode::FAILURE
        }
    }
}
