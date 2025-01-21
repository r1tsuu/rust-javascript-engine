use std::{cell::RefCell, rc::Rc};

use crate::{
    error::EngineError,
    execution_scope::ExecutionScope,
    javascript_object::{JavascriptObjectKind, JavascriptObjectRef},
    memory::{Memory, MemoryRef},
    parser::{Expression, Parser},
    tokenizer::{TokenKind, Tokenizer},
};

pub struct ExecutionEngine {
    scopes: Vec<ExecutionScope>,
    memory: MemoryRef,
    execution_tick: u64,
}

const UNDEFINED_NAME: &str = "undefined";
const TRUE_NAME: &str = "true";
const FALSE_NAME: &str = "false";

impl ExecutionEngine {
    fn new() -> Self {
        let mut engine = ExecutionEngine {
            scopes: vec![],
            memory: Rc::new(RefCell::new(Memory::new())),
            execution_tick: 0,
        };

        engine.initialize_global_scope();

        engine
    }

    pub fn execute_source<T: ToString>(source: T) -> Result<JavascriptObjectRef, EngineError> {
        let mut tokens = vec![];

        for token in Tokenizer::from_source(source.to_string()).to_iter() {
            match token {
                Ok(token) => tokens.push(token),
                Err(err) => return Err(err),
            };
        }

        match Parser::new(tokens).parse_program() {
            Ok(program) => Self::new().execute_expression(program),
            Err(err) => return Err(err),
        }
    }

    fn initialize_global_scope(&mut self) {
        let mut global_scope = ExecutionScope::new(None, self.memory.clone());

        global_scope
            .define(
                UNDEFINED_NAME.to_string(),
                self.memory.borrow_mut().allocate_undefined(),
            )
            .unwrap();

        global_scope
            .define(
                TRUE_NAME.to_string(),
                self.memory.borrow_mut().allocate_boolean(true),
            )
            .unwrap();

        global_scope
            .define(
                FALSE_NAME.to_string(),
                self.memory.borrow_mut().allocate_boolean(false),
            )
            .unwrap();

        self.scopes.push(global_scope);
    }

    fn get_global_scope(&self) -> &ExecutionScope {
        self.scopes.get(0).unwrap()
    }

    fn get_undefined(&self) -> JavascriptObjectRef {
        self.get_global_scope()
            .get(UNDEFINED_NAME.to_string())
            .unwrap()
    }

    fn get_boolean(&self, value: bool) -> JavascriptObjectRef {
        self.get_global_scope()
            .get((if value { TRUE_NAME } else { FALSE_NAME }).to_string())
            .unwrap()
    }

    fn get_current_scope(&mut self) -> &mut ExecutionScope {
        self.scopes.last_mut().unwrap()
    }

    fn execute_expression(
        &mut self,
        expression: Expression,
    ) -> Result<JavascriptObjectRef, EngineError> {
        let result = match expression {
            Expression::Program { expressions } => {
                for (index, expr) in expressions.iter().enumerate() {
                    match self.execute_expression(expr.clone()) {
                        Err(err) => return Err(err),
                        Ok(value) => {
                            if index == expressions.len() - 1 {
                                return Ok(value);
                            }
                        }
                    }
                }

                return Ok(self.get_undefined());
            }
            Expression::LetVariableDeclaration { name, initializer } => {
                match self.execute_expression(*initializer.clone()) {
                    Ok(object) => self.get_current_scope().define(name, object),
                    Err(err) => Err(err),
                }
            }
            Expression::NumberLiteral { value } => {
                Ok(self.memory.borrow_mut().allocate_number(value))
            }
            Expression::Parenthesized { expression } => {
                self.execute_expression(*expression.clone())
            }
            Expression::Identifier { name } => match self.get_current_scope().get(name.clone()) {
                Some(value) => Ok(value),
                None => Err(EngineError::execution_engine_error(format!(
                    "No variable {} found in the scope",
                    name
                ))),
            },
            Expression::BinaryOp { left, op, right } => {
                if op.is_equals() {
                    match *left.clone() {
                        Expression::Identifier { name } => {
                            match self.get_current_scope().get(name.clone()) {
                                Some(var) => var,
                                None => {
                                    return Err(EngineError::execution_engine_error(format!(
                                        "No variable {} found in the scope",
                                        name
                                    )))
                                }
                            }
                        }
                        _ => {
                            return Err(EngineError::execution_engine_error(format!(
                                "Expected identifier in assigment, got: {:#?}",
                                left
                            )))
                        }
                    };

                    let value = match self.execute_expression(*right) {
                        Ok(res) => res,
                        Err(err) => return Err(err),
                    };

                    self.get_current_scope()
                        .assign(left.unwrap_name(), value.clone());

                    return Ok(value);
                }

                let left_result = self.execute_expression(Parser::reorder_expression(*left))?;
                let right_result = self.execute_expression(Parser::reorder_expression(*right))?;

                match op.kind {
                    TokenKind::EqualsEquals => Ok(self
                        .get_boolean(left_result.borrow().is_equal_to_non_strict(&right_result))),
                    TokenKind::Plus => Ok(self.memory.borrow_mut().allocate_number(
                        left_result.borrow().cast_to_number()
                            + right_result.borrow().cast_to_number(),
                    )),
                    TokenKind::Minus => Ok(self.memory.borrow_mut().allocate_number(
                        left_result.borrow().cast_to_number()
                            - right_result.borrow().cast_to_number(),
                    )),
                    TokenKind::Multiply => Ok(self.memory.borrow_mut().allocate_number(
                        left_result.borrow().cast_to_number()
                            * right_result.borrow().cast_to_number(),
                    )),
                    TokenKind::Divide => Ok(self.memory.borrow_mut().allocate_number(
                        left_result.borrow().cast_to_number()
                            / right_result.borrow().cast_to_number(),
                    )),
                    _ => Err(EngineError::execution_engine_error(format!(
                        "Failed to execute binary expression with operator: {:#?}",
                        op
                    ))),
                }
            }
            Expression::StringLiteral { value } => {
                Ok(self.memory.borrow_mut().allocate_string(value))
            }
        };

        self.execution_tick += 1;

        if self.execution_tick % 10 == 0 {
            self.collect_garbage();
        }

        result
    }

    fn collect_garbage(&mut self) {
        for scope in self.scopes.iter() {
            self.memory
                .borrow_mut()
                .deallocate_except_ids(&scope.get_variable_ids());
        }
    }
}
