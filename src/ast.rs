use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum EvalValue {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

pub trait Expression {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String>;
}

pub type GlobalMap = HashMap<String, EvalValue>;

pub trait Statement {
    fn execute(&self, _g: &mut GlobalMap) -> Result<(), String>;
}

pub struct LocalVariableDeclaration {
    name: String,
    value: Box<dyn Expression>,
}

impl Statement for LocalVariableDeclaration {
    fn execute(&self, g: &mut GlobalMap) -> Result<(), String> {
        let value = self.value.execute(g)?;
        g.insert(self.name.clone(), value);
        Ok(())
    }
}

impl LocalVariableDeclaration {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: Box::new(NumberLiteral { value: 0.0 }),
        }
    }

    pub fn set_identifier(&mut self, identifier: String) {
        self.name = identifier;
    }

    pub fn set_expression(&mut self, expression: Box<dyn Expression>) {
        self.value = expression;
    }
}

pub struct Block {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Statement for Block {
    fn execute(&self, g: &mut GlobalMap) -> Result<(), String> {
        for statement in &self.statements {
            statement.execute(g)?;
        }
        Ok(())
    }
}

impl Block {
    pub fn new(statements: Vec<Box<dyn Statement>>) -> Self {
        Self { statements }
    }
}

#[derive(Debug)]
pub struct NumberLiteral {
    value: f64,
}

impl Expression for NumberLiteral {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        Ok(EvalValue::Number(self.value))
    }
}

impl NumberLiteral {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

pub struct BooleanLiteral {
    value: bool,
}

impl Expression for BooleanLiteral {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        Ok(EvalValue::Boolean(self.value))
    }
}

impl BooleanLiteral {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

pub struct NilLiteral {}

impl Expression for NilLiteral {
    fn execute(&self, _g: &mut GlobalMap) -> Result<EvalValue, String> {
        Ok(EvalValue::Nil)
    }
}

pub struct StringLiteral {
    value: String,
}

impl Expression for StringLiteral {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        Ok(EvalValue::String(self.value.clone()))
    }
}

impl StringLiteral {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct IdentifierExpression {
    name: String,
}

impl Expression for IdentifierExpression {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        g.get(&self.name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: '{}'", self.name))
    }
}

impl IdentifierExpression {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: String,
    right: Box<dyn Expression>,
}

impl Expression for BinaryExpression {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        let lhs = self.left.execute(g)?;
        let rhs = self.right.execute(g)?;

        match (lhs, rhs) {
            (EvalValue::Number(left), EvalValue::Number(right)) => match self.operator.as_str() {
                "+" => Ok(EvalValue::Number(left + right)),
                "-" => Ok(EvalValue::Number(left - right)),
                "*" => Ok(EvalValue::Number(left * right)),
                "/" => Ok(EvalValue::Number(left / right)),
                _ => Err(format!("Unknown operator: '{}'", self.operator)),
            },
            _ => Err("Invalid operands".to_string()),
        }
    }
}

impl BinaryExpression {
    pub fn from(left: Box<dyn Expression>, right: Box<dyn Expression>, operator: String) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[test]
    fn test_number_expression() {
        let mut map: GlobalMap = GlobalMap::new();
        let expr = NumberLiteral { value: 5.0 };
        assert_eq!(expr.execute(&mut map).unwrap(), EvalValue::Number(5.0));
    }

    #[test]
    fn test_binary_addition_on_2_numbers() {
        let mut map: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "+".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut map).unwrap(), EvalValue::Number(10.0));
    }

    #[test]
    fn test_binary_subtraction_on_2_numbers() {
        let mut map: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "-".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut map).unwrap(), EvalValue::Number(0.0));
    }

    #[test]
    fn test_binary_multiplication_on_2_numbers() {
        let mut map: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "*".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut map).unwrap(), EvalValue::Number(25.0));
    }

    #[test]
    fn test_binary_division_on_2_numbers() {
        let mut map: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "/".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut map).unwrap(), EvalValue::Number(1.0));
    }
}
