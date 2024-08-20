use crate::vm::VirtualMachine;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum EvalValue {
    Number(f64),

    Boolean(bool),
    String(String),
    Nil,

    NativeFunction(fn(Vec<EvalValue>) -> Result<EvalValue, String>),
}
impl EvalValue {
    fn is_true(&self) -> bool {
        matches!(self, EvalValue::Nil | EvalValue::Boolean(true))
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    NumberLiteral(f64),
    BooleanLiteral(bool),
    StringLiteral(String),
    NilLiteral,
    IdentifierExpression(String),
    BinaryExpression(Box<Expression>, String, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
}

impl Expression {
    pub fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
        match &self {
            Expression::NumberLiteral(number) => Ok(EvalValue::Number(*number)),
            Expression::BooleanLiteral(boolean_value) => Ok(EvalValue::Boolean(*boolean_value)),
            Expression::StringLiteral(string_value) => Ok(EvalValue::String(string_value.clone())),
            Expression::NilLiteral => Ok(EvalValue::Nil),
            Expression::IdentifierExpression(ident) => {
                Ok(_g.lookup_variable(ident).unwrap_or(EvalValue::Nil))
            }
            Expression::BinaryExpression(lhs, operator, rhs) => {
                let lhs = lhs.execute(_g)?;
                let rhs = rhs.execute(_g)?;

                match (lhs.clone(), rhs.clone()) {
                    (EvalValue::Number(left), EvalValue::Number(right)) => {
                        match operator.as_str() {
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

                            _ => Err(format!("Unknown operator for numbers: '{}'", operator)),
                        }
                    }
                    (EvalValue::String(left), EvalValue::String(right)) => {
                        match operator.as_str() {
                            ".." => Ok(EvalValue::String(left.clone() + &right.clone())),
                            "==" => Ok(EvalValue::Boolean(left == right)),
                            "~=" => Ok(EvalValue::Boolean(left != right)),
                            _ => Err(format!("Invalid operator for strings: '{}'", operator)),
                        }
                    }
                    (EvalValue::Boolean(left), EvalValue::Boolean(right)) => {
                        match operator.as_str() {
                            "==" => Ok(EvalValue::Boolean(left == right)),
                            "~=" => Ok(EvalValue::Boolean(left != right)),
                            _ => Err(format!("Invalid operator for booleans: '{}'", operator)),
                        }
                    }
                    _ => Err(format!(
                        "Invalid expression {:?} {} {:?}",
                        lhs, operator, rhs
                    )),
                }
            }
            Expression::FunctionCall(function_name, function_arguments) => {
                let mut args: Vec<EvalValue> = Vec::new();
                for arg in function_arguments {
                    args.push(arg.execute(_g)?);
                }
                match _g.lookup_variable(function_name) {
                    Some(EvalValue::NativeFunction(f)) => f(args),
                    _ => Err(format!("Function '{}' not found", function_name)),
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    LocalVariableDeclaration(String, Box<Expression>),
    AssigmentStatement(String, Box<Expression>),

    WhileLoop {
        loop_condition: Box<Expression>,
        code_block: Vec<Statement>,
    },
    ForLoop {
        iterator_identifier: String,
        starting_value: Box<Expression>,
        ending_value: Box<Expression>,
        step_value: Box<Expression>,
        code_block: Vec<Statement>,
    },
    IfStatement {
        basic_condition: Box<Expression>,
        code_block: Vec<Statement>,
        elseif_statements: Vec<(Box<Expression>, Vec<Statement>)>,
        else_block: Option<Vec<Statement>>,
    },
    ExpressionStatement(Box<Expression>),
}

impl Statement {
    pub fn execute(&self, _g: &mut VirtualMachine) -> Result<(), String> {
        match self {
            Statement::LocalVariableDeclaration(variable_name, expr) => {
                let value = expr.execute(_g)?;
                _g.declare_variable(variable_name.clone(), value);
                Ok(())
            }
            Statement::AssigmentStatement(variable_name, expr) => {
                let value = expr.execute(_g)?;
                _g.change_or_create_value(variable_name.clone(), value);
                Ok(())
            }
            Statement::WhileLoop {
                loop_condition,
                code_block,
            } => {
                _g.enter_scope();

                while loop_condition.execute(_g)?.is_true() {
                    for statement in code_block {
                        statement.execute(_g)?;
                    }
                }

                _g.exit_scope();
                Ok(())
            }
            Statement::ForLoop {
                iterator_identifier,
                starting_value,
                ending_value,
                step_value,
                code_block,
            } => {
                _g.enter_scope();

                let starting_value = starting_value.execute(_g)?;
                _g.declare_variable(iterator_identifier.clone(), starting_value);

                let step_value = match step_value.execute(_g)? {
                    EvalValue::Number(n) => n,
                    _ => return Err("Invalid step value".to_string()),
                };

                while _g.lookup_variable(iterator_identifier).unwrap()
                    <= ending_value.execute(_g)?
                {
                    for statement in code_block {
                        statement.execute(_g)?;
                    }

                    let current_value = match _g.lookup_variable(iterator_identifier) {
                        Some(EvalValue::Number(n)) => n,
                        _ => return Err(format!("Invalid {} value", iterator_identifier)),
                    } + step_value;

                    _g.change_or_create_value(
                        iterator_identifier.clone(),
                        EvalValue::Number(current_value),
                    );
                }

                _g.exit_scope();
                Ok(())
            }
            Statement::IfStatement {
                basic_condition,
                code_block,
                elseif_statements,
                else_block,
            } => {
                _g.enter_scope();

                if basic_condition.execute(_g)?.is_true() {
                    for statement in code_block {
                        statement.execute(_g)?;
                    }
                } else {
                    for (condition, block) in elseif_statements {
                        if condition.execute(_g)?.is_true() {
                            for statement in block {
                                statement.execute(_g)?;
                            }
                            return Ok(());
                        }
                    }
                    if let Some(block) = else_block {
                        for statement in block {
                            statement.execute(_g)?;
                        }
                    }
                }

                _g.exit_scope();
                Ok(())
            }
            Statement::ExpressionStatement(expr) => {
                expr.execute(_g)?;
                Ok(())
            }
        }
    }
}

// pub trait Statement: std::fmt::Debug {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<(), String>;
// }

// #[derive(Debug)]
// pub struct LocalVariableDeclaration {
//     name: String,
//     value: Box<dyn Expression>,
// }

// impl Statement for LocalVariableDeclaration {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         let value = self.value.execute(g)?;
//         g.declare_variable(self.name.clone(), value);
//         Ok(())
//     }
// }

// impl LocalVariableDeclaration {
//     pub fn new(name: String, value: Box<dyn Expression>) -> Self {
//         Self { name, value }
//     }
// }

// #[derive(Debug)]
// pub struct AssigmentStatement {
//     identifier: String,
//     expression: Box<dyn Expression>
// }

// impl Statement for AssigmentStatement {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<(), String> {
//         let expression_result = self.expression.execute(_g)?;
//         _g.change_or_create_value(self.identifier.clone(), expression_result);

//         Ok(())
//     }
// }

// impl AssigmentStatement {
//     pub fn new(identifier: String, expression: Box<dyn Expression>) -> Self {
//         Self {
//             identifier,
//             expression
//         }
//     }
// }

// #[derive(Debug)]
// pub struct Block {
//     pub statements: Vec<Box<dyn Statement>>,
// }

// impl Statement for Block {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         g.enter_scope();
//         for statement in &self.statements {
//             statement.execute(g)?;
//         }
//         g.exit_scope();
//         Ok(())
//     }
// }

// impl Block {
//     pub fn new(statements: Vec<Box<dyn Statement>>) -> Self {
//         Self { statements }
//     }
// }

// #[derive(Debug)]
// pub struct WhileLoop {
//     looping_condition: Box<dyn Expression>,
//     code_block: Block,
// }

// impl Statement for WhileLoop {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         while self.looping_condition.execute(g)?.is_true() {
//             self.code_block.execute(g)?;
//         }
//         Ok(())
//     }
// }

// impl WhileLoop {
//     pub fn new(looping_condition: Box<dyn Expression>, code_block: Block) -> Self {
//         Self {
//             looping_condition,
//             code_block,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ForLoop {
//     iterator_identifier: String,
//     starting_value: Box<dyn Expression>,
//     ending_value: Box<dyn Expression>,
//     step_value: Box<dyn Expression>,
//     code_block: Block,
// }

// impl ForLoop {
//     pub fn new(
//         iterator_identifier: String,
//         starting_value: Box<dyn Expression>,
//         ending_value: Box<dyn Expression>,
//         step_value: Box<dyn Expression>,
//         code_block: Block,
//     ) -> Self {
//         Self {
//             iterator_identifier,
//             starting_value,
//             ending_value,
//             step_value,
//             code_block,
//         }
//     }
// }

// impl Statement for ForLoop {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         g.enter_scope();

//         let starting_value = self.starting_value.execute(g)?;
//         g.declare_variable(self.iterator_identifier.clone(), starting_value);

//         let step_value = match self.step_value.execute(g)? {
//             EvalValue::Number(n) => n,
//             _ => return Err("Invalid step value".to_string())
//         };

//         while g.lookup_variable(&self.iterator_identifier).unwrap() <= self.ending_value.execute(g)? {
//             self.code_block.execute(g)?;

//             let current_value = match g.lookup_variable(&self.iterator_identifier) {
//                 Some(EvalValue::Number(n)) => n,
//                 _ => return Err(format!("Invalid {} value", self.iterator_identifier))
//             } + step_value;

//             g.change_or_create_value(self.iterator_identifier.clone(), EvalValue::Number(current_value));
//         }

//         g.exit_scope();
//         Ok(())
//     }
// }

// #[derive(Debug)]
// pub struct IfStatement {
//     basic_condition: Box<dyn Expression>,
//     code_block: Block,
//     elseif_statements: Vec<(Box<dyn Expression>, Block)>,
//     else_block: Option<Block>,
// }

// impl Statement for IfStatement {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         g.enter_scope();

//         if self.basic_condition.execute(g)?.is_true() {
//             self.code_block.execute(g)?;
//         } else {
//             for (condition, block) in &self.elseif_statements {
//                 if condition.execute(g)?.is_true() {
//                     block.execute(g)?;
//                     return Ok(());
//                 }
//             }
//             if let Some(block) = &self.else_block {
//                 block.execute(g)?;
//             }
//         }

//         g.exit_scope();
//         Ok(())
//     }
// }

// impl IfStatement {
//     pub fn new(
//         basic_condition: Box<dyn Expression>,
//         code_block: Block,
//         elseif_statements: Vec<(Box<dyn Expression>, Block)>,
//         else_block: Option<Block>,
//     ) -> Self {
//         Self {
//             basic_condition,
//             code_block,
//             elseif_statements,
//             else_block,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct NumberLiteral {
//     value: f64,
// }

// impl Expression for NumberLiteral {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         Ok(EvalValue::Number(self.value))
//     }
// }

// impl NumberLiteral {
//     pub fn new(value: f64) -> Self {
//         Self { value }
//     }
// }

// #[derive(Debug)]
// pub struct BooleanLiteral {
//     value: bool,
// }

// impl Expression for BooleanLiteral {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         Ok(EvalValue::Boolean(self.value))
//     }
// }

// impl BooleanLiteral {
//     pub fn new(value: bool) -> Self {
//         Self { value }
//     }
// }

// #[derive(Debug)]
// pub struct NilLiteral {}

// impl Expression for NilLiteral {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         Ok(EvalValue::Nil)
//     }
// }

// #[derive(Debug)]
// pub struct StringLiteral {
//     value: String,
// }

// impl Expression for StringLiteral {
//     fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         Ok(EvalValue::String(self.value.clone()))
//     }
// }

// impl StringLiteral {
//     pub fn new(value: String) -> Self {
//         Self { value }
//     }
// }

// #[derive(Debug)]
// pub struct IdentifierExpression {
//     name: String,
// }

// impl Expression for IdentifierExpression {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         Ok(g.lookup_variable(&self.name).unwrap_or(EvalValue::Nil))
//     }
// }

// impl IdentifierExpression {
//     pub fn new(name: String) -> Self {
//         Self { name }
//     }
// }

// #[derive(Debug)]
// pub struct BinaryExpression {
//     left: Box<dyn Expression>,
//     operator: String,
//     right: Box<dyn Expression>,
// }

// impl Expression for BinaryExpression {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         let lhs = self.left.execute(g)?;
//         let rhs = self.right.execute(g)?;

//         match (lhs.clone(), rhs.clone()) {
//             (EvalValue::Number(left), EvalValue::Number(right)) => match self.operator.as_str() {
//                 "+" => Ok(EvalValue::Number(left + right)),
//                 "-" => Ok(EvalValue::Number(left - right)),
//                 "*" => Ok(EvalValue::Number(left * right)),
//                 "/" => Ok(EvalValue::Number(left / right)),

//                 "<" => Ok(EvalValue::Boolean(left < right)),
//                 ">" => Ok(EvalValue::Boolean(left > right)),

//                 "<=" => Ok(EvalValue::Boolean(left <= right)),
//                 ">=" => Ok(EvalValue::Boolean(left >= right)),

//                 "==" => Ok(EvalValue::Boolean(left == right)),
//                 "~=" => Ok(EvalValue::Boolean(left != right)),

//                 _ => Err(format!("Unknown operator: '{}'", self.operator)),
//             },
//             _ => Err(format!(
//                 "Invalid expression {:?} {} {:?}",
//                 lhs, &self.operator, rhs
//             )),
//         }
//     }
// }

// impl BinaryExpression {
//     pub fn from(left: Box<dyn Expression>, right: Box<dyn Expression>, operator: String) -> Self {
//         Self {
//             left,
//             operator,
//             right,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ExpressionStatement {
//     expression: Box<dyn Expression>,
// }

// impl Statement for ExpressionStatement {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<(), String> {
//         self.expression.execute(g)?;
//         Ok(())
//     }
// }

// impl ExpressionStatement {
//     pub fn new(expression: Box<dyn Expression>) -> Self {
//         Self { expression }
//     }
// }

// #[derive(Debug)]
// pub struct FunctionCall {
//     name: String,
//     arguments: Vec<Box<dyn Expression>>,
// }

// impl FunctionCall {
//     pub fn new(name: String, arguments: Vec<Box<dyn Expression>>) -> Self {
//         Self { name, arguments }
//     }
// }

// impl Expression for FunctionCall {
//     fn execute(&self, g: &mut VirtualMachine) -> Result<EvalValue, String> {
//         let mut args: Vec<EvalValue> = Vec::new();
//         for arg in &self.arguments {
//             args.push(arg.execute(g)?);
//         }
//         match g.lookup_variable(&self.name) {
//             Some(EvalValue::NativeFunction(f)) => f(args),
//             _ => Err(format!("Function '{}' not found", self.name)),
//         }
//     }
// }
