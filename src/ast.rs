use crate::vm::VirtualMachine;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum EvalValue {
    Number(f64),

    Boolean(bool),
    String(String),
    Nil,

    NativeFunction(fn(Vec<EvalValue>) -> Result<EvalValue, String>),
    DeclaredFunction {
        arguments: Vec<String>,
        body: Vec<Statement>,
    },

    Void, // For internal use, the return value of a statement that doesn't return anything
}
impl EvalValue {
    fn is_true(&self) -> bool {
        !matches!(self, EvalValue::Nil | EvalValue::Boolean(false))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    NumberLiteral(f64),
    BooleanLiteral(bool),
    StringLiteral(String),
    NilLiteral,
    IdentifierExpression(String),
    BinaryExpression(Box<Expression>, String, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
    RepeatUntilLoop {
        code_block: Vec<Statement>,
        loop_condition: Box<Expression>,
    },
    IfStatement {
        basic_condition: Box<Expression>,
        code_block: Vec<Statement>,
        elseif_statements: Vec<(Box<Expression>, Vec<Statement>)>,
        else_block: Option<Vec<Statement>>,
    },
    ExpressionStatement(Box<Expression>),
    FunctionDeclaration {
        function_name: String,
        function_arguments: Vec<String>,
        function_body: Vec<Statement>,
    },
    ReturnStatement(Box<Expression>),
}

impl Expression {
    fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
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
                    Some(EvalValue::DeclaredFunction { arguments, body }) => {
                        _g.enter_scope();

                        if arguments.len() != args.len() {
                            return Err(format!(
                                "Expected {} arguments, got {}",
                                arguments.len(),
                                args.len()
                            ));
                        }

                        for (arg_name, arg_value) in arguments.iter().zip(args) {
                            _g.declare_variable(arg_name.clone(), arg_value);
                        }
                        for statement in body {
                            let return_value = statement.execute(_g)?;

                            if return_value != EvalValue::Void {
                                _g.exit_scope();
                                return Ok(return_value);
                            }
                        }
                        _g.exit_scope();
                        Ok(EvalValue::Nil)
                    }
                    _ => Err(format!("Function '{}' not found", function_name)),
                }
            }
        }
    }
}

impl Statement {
    pub fn execute(&self, _g: &mut VirtualMachine) -> Result<EvalValue, String> {
        match self {
            Statement::LocalVariableDeclaration(variable_name, expr) => {
                let value = expr.execute(_g)?;
                _g.declare_variable(variable_name.clone(), value);
                Ok(EvalValue::Void)
            }
            Statement::AssigmentStatement(variable_name, expr) => {
                let value = expr.execute(_g)?;
                _g.change_or_create_value(variable_name.clone(), value);
                Ok(EvalValue::Void)
            }
            Statement::WhileLoop {
                loop_condition,
                code_block,
            } => {
                _g.enter_scope();

                while loop_condition.execute(_g)?.is_true() {
                    for statement in code_block {
                        let return_value = statement.execute(_g)?;

                        if return_value != EvalValue::Void {
                            _g.exit_scope();
                            return Ok(return_value);
                        }
                    }
                }

                _g.exit_scope();
                Ok(EvalValue::Void)
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
                        let return_value = statement.execute(_g)?;

                        if return_value != EvalValue::Void {
                            _g.exit_scope();
                            return Ok(return_value);
                        }
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
                Ok(EvalValue::Void)
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
                        let return_value = statement.execute(_g)?;

                        if return_value != EvalValue::Void {
                            _g.exit_scope();
                            return Ok(return_value);
                        }
                    }
                } else {
                    for (condition, block) in elseif_statements {
                        if condition.execute(_g)?.is_true() {
                            for statement in block {
                                let return_value = statement.execute(_g)?;

                                if return_value != EvalValue::Void {
                                    _g.exit_scope();
                                    return Ok(return_value);
                                }
                            }
                            return Ok(EvalValue::Void);
                        }
                    }
                    if let Some(block) = else_block {
                        for statement in block {
                            let return_value = statement.execute(_g)?;

                            if return_value != EvalValue::Void {
                                _g.exit_scope();
                                return Ok(return_value);
                            }
                        }
                    }
                }

                _g.exit_scope();
                Ok(EvalValue::Void)
            }
            Statement::ExpressionStatement(expr) => {
                expr.execute(_g)?;
                Ok(EvalValue::Void)
            }
            Statement::FunctionDeclaration {
                function_name,
                function_arguments,
                function_body,
            } => {
                _g.declare_variable(
                    function_name.clone(),
                    EvalValue::DeclaredFunction {
                        arguments: function_arguments.clone(),
                        body: function_body.clone(),
                    },
                );
                Ok(EvalValue::Void)
            }
            Statement::ReturnStatement(expression) => Ok(expression.execute(_g)?),
            Statement::RepeatUntilLoop {
                code_block,
                loop_condition,
            } => {
                _g.enter_scope();

                loop {
                    for statement in code_block {
                        let return_value = statement.execute(_g)?;

                        if return_value != EvalValue::Void {
                            _g.exit_scope();
                            return Ok(return_value);
                        }
                    }

                    if loop_condition.execute(_g)?.is_true() {
                        break;
                    }
                }

                _g.exit_scope();
                Ok(EvalValue::Void)
            }
        }
    }
}
