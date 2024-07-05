use std::collections::HashMap;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum EvalValue {
    Number(f64),
    Boolean(bool),
    String(String),
    NativeFunction(fn(Vec<EvalValue>) -> Result<EvalValue, String>),
    Nil,
}

impl EvalValue {
    pub fn is_true(&self) -> bool {
        !matches!(self, EvalValue::Nil | EvalValue::Boolean(false))
    }
}

pub trait Expression: std::fmt::Debug {
    fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String>;
}

type ValueMap = HashMap<String, EvalValue>;

pub struct VirtualMachine {
    scopes_stack: Vec<ValueMap>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        let mut virtual_machine = VirtualMachine {
            scopes_stack: vec![ValueMap::new()],
        };

        virtual_machine.declare_variable(
            String::from("print"),
            EvalValue::NativeFunction(|args| {
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
            }),
        );

        virtual_machine
    }

    pub fn enter_scope(&mut self) {
        self.scopes_stack.push(ValueMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes_stack.pop();
    }

    pub fn declare_variable(&mut self, name: String, value: EvalValue) {
        self.scopes_stack
            .last_mut()
            .expect("No scope found")
            .insert(name, value);
    }

    pub fn lookup_variable(&self, name: &str) -> Option<EvalValue> {
        for scope in self.scopes_stack.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn change_or_create_value(&mut self, name: String, value: EvalValue) {
        let mut target_scope = self.scopes_stack.iter_mut().rev().find(|v| v.contains_key(&name));
        if target_scope.is_none() {
            target_scope = self.scopes_stack.first_mut()
        }
        target_scope.unwrap().insert(name, value); 
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
        g.declare_variable(self.name.clone(), value);
        Ok(())
    }
}

impl LocalVariableDeclaration {
    pub fn new(name: String, value: Box<dyn Expression>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug)]
pub struct AssigmentStatement {
    identifier: String,
    expression: Box<dyn Expression>
}

impl Statement for AssigmentStatement {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<(), String> {
        let expression_result = self.expression.execute(_g)?;
        _g.change_or_create_value(self.identifier.clone(), expression_result);

        Ok(())
    }
}

impl AssigmentStatement {
    pub fn new(identifier: String, expression: Box<dyn Expression>) -> Self {
        Self {
            identifier,
            expression
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<dyn Statement>>,
}

impl Statement for Block {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        g.enter_scope();
        for statement in &self.statements {
            statement.execute(g)?;
        }
        g.exit_scope();
        Ok(())
    }
}

impl Block {
    pub fn new(statements: Vec<Box<dyn Statement>>) -> Self {
        Self { statements }
    }
}

#[derive(Debug)]
pub struct WhileLoop {
    looping_condition: Box<dyn Expression>,
    code_block: Block,
}

impl Statement for WhileLoop {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        while self.looping_condition.execute(g)?.is_true() {
            self.code_block.execute(g)?;
        }
        Ok(())
    }
}

impl WhileLoop {
    pub fn new(looping_condition: Box<dyn Expression>, code_block: Block) -> Self {
        Self {
            looping_condition,
            code_block,
        }
    }
}

#[derive(Debug)]
pub struct ForLoop {
    iterator_identifier: String,
    starting_value: Box<dyn Expression>,
    ending_value: Box<dyn Expression>,
    step_value: Box<dyn Expression>,
    code_block: Block,
}

impl ForLoop {
    pub fn new(
        iterator_identifier: String,
        starting_value: Box<dyn Expression>,
        ending_value: Box<dyn Expression>,
        step_value: Box<dyn Expression>,
        code_block: Block,
    ) -> Self {
        Self {
            iterator_identifier,
            starting_value,
            ending_value,
            step_value,
            code_block,
        }
    }
}

impl Statement for ForLoop {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        g.enter_scope();        
        
        let starting_value = self.starting_value.execute(g)?;
        g.declare_variable(self.iterator_identifier.clone(), starting_value);

        let step_value = match self.step_value.execute(g)? {
            EvalValue::Number(n) => n,
            _ => return Err("Invalid step value".to_string())
        };

        while g.lookup_variable(&self.iterator_identifier).unwrap() <= self.ending_value.execute(g)? {
            self.code_block.execute(g)?;

            let current_value = match g.lookup_variable(&self.iterator_identifier) {
                Some(EvalValue::Number(n)) => n,
                _ => return Err(format!("Invalid {} value", self.iterator_identifier))
            } + step_value;

            g.change_or_create_value(self.iterator_identifier.clone(), EvalValue::Number(current_value));
        }

        g.exit_scope();
        Ok(())
    }
}

#[derive(Debug)]
pub struct IfStatement {
    basic_condition: Box<dyn Expression>,
    code_block: Block,
    elseif_statements: Vec<(Box<dyn Expression>, Block)>,
    else_block: Option<Block>,
}

impl Statement for IfStatement {
    fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
        g.enter_scope();

        if self.basic_condition.execute(g)?.is_true() {
            self.code_block.execute(g)?;
        } else {
            for (condition, block) in &self.elseif_statements {
                if condition.execute(g)?.is_true() {
                    block.execute(g)?;
                    return Ok(());
                }
            }
            if let Some(block) = &self.else_block {
                block.execute(g)?;
            }
        }

        g.exit_scope();
        Ok(())
    }
}

impl IfStatement {
    pub fn new(
        basic_condition: Box<dyn Expression>,
        code_block: Block,
        elseif_statements: Vec<(Box<dyn Expression>, Block)>,
        else_block: Option<Block>,
    ) -> Self {
        Self {
            basic_condition,
            code_block,
            elseif_statements,
            else_block,
        }
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
        Ok(g.lookup_variable(&self.name).unwrap_or(EvalValue::Nil))
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
        match g.lookup_variable(&self.name) {
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
