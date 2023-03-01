use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum EvalValue {
    Number(f64),
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
            value: Box::new(NumberExpression { value: 0.0 }),
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
    statements: Vec<Box<dyn Statement>>,
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
pub struct NumberExpression {
    value: f64,
}

impl Expression for NumberExpression {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        Ok(EvalValue::Number(self.value))
    }
}

impl NumberExpression {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl std::fmt::Debug for dyn Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct IdentifierExpression {
    name: String,
}

impl Expression for IdentifierExpression {
    fn execute(&self, g: &mut GlobalMap) -> Result<EvalValue, String> {
        g.get(&self.name)
            .ok_or(format!("Unknown identifier: '{}'", self.name))
            .map(|v| v.clone())
    }
}

impl IdentifierExpression {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug)]
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
        let mut MAP: GlobalMap = GlobalMap::new();
        let expr = NumberExpression { value: 5.0 };
        assert_eq!(expr.execute(&mut MAP).unwrap(), EvalValue::Number(5.0));
    }

    #[test]
    fn test_binary_addition_on_2_numbers() {
        let mut MAP: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "+".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut MAP).unwrap(), EvalValue::Number(10.0));
    }

    #[test]
    fn test_binary_subtraction_on_2_numbers() {
        let mut MAP: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "-".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut MAP).unwrap(), EvalValue::Number(0.0));
    }

    #[test]
    fn test_binary_multiplication_on_2_numbers() {
        let mut MAP: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "*".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut MAP).unwrap(), EvalValue::Number(25.0));
    }

    #[test]
    fn test_binary_division_on_2_numbers() {
        let mut MAP: GlobalMap = GlobalMap::new();
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "/".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut MAP).unwrap(), EvalValue::Number(1.0));
    }
}
