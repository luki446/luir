
pub enum EvalValue {
    Number(f64)
}

trait Expression {
    fn execute(&self) -> Result<EvalValue, String>;
}

pub struct NumberExpression {
    value: f64
}

impl Expression for NumberExpression {
    fn execute(&self) -> Result<EvalValue, String> {
        Ok(EvalValue::Number(self.value))
    }
}

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: String,
    right: Box<dyn Expression>
}

impl Expression for BinaryExpression {
    fn execute(&self) -> Result<EvalValue, String> {
        let lhs = self.left.execute()?;
        let rhs = self.right.execute()?;

        match (lhs, rhs) {
            (EvalValue::Number(left), EvalValue::Number(right)) => {
                match self.operator.as_str() {
                    "+" => Ok(EvalValue::Number(left + right)),
                    "-" => Ok(EvalValue::Number(left - right)),
                    "*" => Ok(EvalValue::Number(left * right)),
                    "/" => Ok(EvalValue::Number(left / right)),
                    _ => Err(format!("Unknown operator: '{}'", self.operator))
                }
            }
            _ => Err("Invalid operands".to_string()),
        }
    }
}

