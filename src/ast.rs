#[derive(Debug, PartialEq)]
pub enum EvalValue {
    Number(f64),
}

trait Expression {
    fn execute(&self) -> Result<EvalValue, String>;
}

pub struct NumberExpression {
    value: f64,
}

impl Expression for NumberExpression {
    fn execute(&self) -> Result<EvalValue, String> {
        Ok(EvalValue::Number(self.value))
    }
}

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: String,
    right: Box<dyn Expression>,
}

impl Expression for BinaryExpression {
    fn execute(&self) -> Result<EvalValue, String> {
        let lhs = self.left.execute()?;
        let rhs = self.right.execute()?;

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

#[cfg(test)]
mod ast_tests {
    use super::*;

    #[test]
    fn test_number_expression() {
        let expr = NumberExpression { value: 5.0 };
        assert_eq!(expr.execute().unwrap(), EvalValue::Number(5.0));
    }

    #[test]
    fn test_binary_addition_on_2_numbers() {
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "+".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute().unwrap(), EvalValue::Number(10.0));
    }

    #[test]
    fn test_binary_subtraction_on_2_numbers() {
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "-".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute().unwrap(), EvalValue::Number(0.0));
    }

    #[test]
    fn test_binary_multiplication_on_2_numbers() {
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "*".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute().unwrap(), EvalValue::Number(25.0));
    }

    #[test]
    fn test_binary_division_on_2_numbers() {
        let expr = BinaryExpression {
            left: Box::new(NumberExpression { value: 5.0 }),
            operator: "/".to_string(),
            right: Box::new(NumberExpression { value: 5.0 }),
        };
        assert_eq!(expr.execute().unwrap(), EvalValue::Number(1.0));
    }
}
