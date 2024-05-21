use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum EvalValue {
    Number(f64),
    Boolean(bool),
    String(String),
    NativeFunction(fn(Vec<EvalValue>) -> Result<EvalValue, String>),
    Nil,
}

pub trait Expression: std::fmt::Debug {
    fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String>;
}

type GlobalMap = HashMap<String, EvalValue>;

pub struct VirtualMachine {
    pub global_map: GlobalMap,
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut virtual_machine = VirtualMachine {
            global_map: HashMap::new(),
        };

        virtual_machine.global_map.insert(String::from("print"), EvalValue::NativeFunction(|args| {
            for arg in args {
                match arg {
                    EvalValue::Number(n) => print!("{}\t", n),
                    EvalValue::Boolean(b) => print!("{}\t", b),
                    EvalValue::String(s) => print!("{}\t", s),
                    EvalValue::Nil => print!("nil\t"),
                    _ => return Err("Invalid argument".to_string()),
                }
            }
            println!();
            Ok(EvalValue::Nil)
        }));

        virtual_machine
    }
}

pub trait Statement: std::fmt::Debug {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<(), String>;
}

#[derive(Debug)]
pub struct LocalVariableDeclaration {
    name: String,
    value: Box<dyn Expression>,
}

impl Statement for LocalVariableDeclaration {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        let value = self.value.execute(g)?;
        g.global_map.insert(self.name.clone(), value);
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

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Statement for Block {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
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
    fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
        Ok(EvalValue::Number(self.value))
    }
}

impl NumberLiteral {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct BooleanLiteral {
    value: bool,
}

impl Expression for BooleanLiteral {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
        Ok(EvalValue::Boolean(self.value))
    }
}

impl BooleanLiteral {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct NilLiteral {}

impl Expression for NilLiteral {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
        Ok(EvalValue::Nil)
    }
}

#[derive(Debug)]
pub struct StringLiteral {
    value: String,
}

impl Expression for StringLiteral {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
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
    fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
        Ok(g.global_map
            .get(&self.name)
            .cloned()
            .unwrap_or(EvalValue::Nil))
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
    fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
        let lhs = self.left.execute(g)?;
        let rhs = self.right.execute(g)?;

        match (lhs.clone(), rhs.clone()) {
            (EvalValue::Number(left), EvalValue::Number(right)) => match self.operator.as_str() {
                "+" => Ok(EvalValue::Number(left + right)),
                "-" => Ok(EvalValue::Number(left - right)),
                "*" => Ok(EvalValue::Number(left * right)),
                "/" => Ok(EvalValue::Number(left / right)),

                "<" => Ok(EvalValue::Boolean(left < right)),
                ">" => Ok(EvalValue::Boolean(left > right)),

                "<=" => Ok(EvalValue::Boolean(left <= right)),
                ">=" => Ok(EvalValue::Boolean(left >= right)),

                "==" => Ok(EvalValue::Boolean(left == right)),
                "~=" => Ok(EvalValue::Boolean(left != right)),

                _ => Err(format!("Unknown operator: '{}'", self.operator)),
            },
            _ => Err(format!(
                "Invalid expression {:?} {} {:?}",
                lhs, &self.operator, rhs
            )),
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

#[derive(Debug)]
pub struct ExpressionStatement {
    expression: Box<dyn Expression>,
}

impl Statement for ExpressionStatement {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        self.expression.execute(g)?;
        Ok(())
    }
}

impl ExpressionStatement {
    pub fn new(expression: Box<dyn Expression>) -> Self {
        Self { expression }
    }
}

#[derive(Debug)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<Box<dyn Expression>>,
}

impl FunctionCall {
    pub fn new(name: String, arguments: Vec<Box<dyn Expression>>) -> Self {
        Self { name, arguments }
    }
}

impl Expression for FunctionCall {
    fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
        let mut args: Vec<EvalValue> = Vec::new();
        for arg in &self.arguments {
            args.push(arg.execute(g)?);
        }
        match g.global_map.get(&self.name) {
            Some(EvalValue::NativeFunction(f)) => f(args),
            _ => Err(format!("Function '{}' not found", self.name)),
        }
    }
}

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[test]
    fn test_number_expression() {
        let mut vm: VirtualMachine = VirtualMachine::new();
        let expr = NumberLiteral { value: 5.0 };
        assert_eq!(expr.execute(&mut vm).unwrap(), EvalValue::Number(5.0));
    }

    #[test]
    fn test_binary_addition_on_2_numbers() {
        let mut vm: VirtualMachine = VirtualMachine::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "+".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut vm).unwrap(), EvalValue::Number(10.0));
    }

    #[test]
    fn test_binary_subtraction_on_2_numbers() {
        let mut vm: VirtualMachine = VirtualMachine::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "-".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut vm).unwrap(), EvalValue::Number(0.0));
    }

    #[test]
    fn test_binary_multiplication_on_2_numbers() {
        let mut vm: VirtualMachine = VirtualMachine::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "*".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut vm).unwrap(), EvalValue::Number(25.0));
    }

    #[test]
    fn test_binary_division_on_2_numbers() {
        let mut vm: VirtualMachine = VirtualMachine::new();
        let expr = BinaryExpression {
            left: Box::new(NumberLiteral { value: 5.0 }),
            operator: "/".to_string(),
            right: Box::new(NumberLiteral { value: 5.0 }),
        };
        assert_eq!(expr.execute(&mut vm).unwrap(), EvalValue::Number(1.0));
    }
}
