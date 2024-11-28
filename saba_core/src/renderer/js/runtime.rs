use core::borrow::Borrow;
use core::ops::Add;
use core::ops::Sub;

use alloc::rc::Rc;

use crate::renderer::js::ast::Node;
use crate::renderer::js::ast::Program;

#[derive(Debug, Clone)]
pub struct JsRuntime {}

impl JsRuntime {
    pub fn new() -> Self {
        Self {}
    }
}

impl JsRuntime {
    pub fn execute(&mut self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()));
        }
    }

    fn eval(&mut self, node: &Option<Rc<Node>>) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None,
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => return self.eval(&expr),
            Node::AdditiveExpression {
                operator,
                left,
                right,
            } => {
                let left_value = match self.eval(&left) {
                    Some(value) => value,
                    None => return None,
                };
                let right_value = match self.eval(&right) {
                    Some(value) => value,
                    None => return None,
                };

                if operator == &'+' {
                    Some(left_value + right_value)
                } else if operator == &'-' {
                    Some(left_value - right_value)
                } else {
                    None
                }
            }
            Node::AssgnmentExpression {
                operator,
                left,
                right,
            } => todo!(),
            Node::MemberExpression { object, property } => todo!(),
            Node::NumericLiteral(value) => Some(RuntimeValue::Number(*value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number(u64),
}

impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> RuntimeValue {
        let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs);
        return RuntimeValue::Number(left_num + right_num);
    }
}

impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> Self::Output {
        let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs);
        return RuntimeValue::Number(left_num - right_num);
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::renderer::js::{ast::JsParser, token::JsLexer};

    use super::*;

    #[test]
    fn test_num() {
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(42))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()));
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(3))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()));
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_sub_nums() {
        let input = "2 -1 ".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()));
            assert_eq!(expected[i], result);
            i += 1;
        }
    }
}