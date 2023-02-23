
pub enum EvalValue {
    Number(f64)
}

trait Expression {
    fn execute(&self) -> EvalValue;
}

pub struct NumberExpression {
    value: f64
}

impl Expression for NumberExpression {
    fn execute(&self) -> EvalValue {
        EvalValue::Number(self.value)
    }
}

pub struct BinaryExpression {
    left: Box<dyn Expression>,
    operator: String,
    right: Box<dyn Expression>
}

impl Expression for BinaryExpression {
    fn execute(&self) -> EvalValue {
        let left = self.left.execute();
        let right = self.right.execute();

        match self.operator.as_str() {
            "+" => {
                match (left, right) {
                    (EvalValue::Number(left), EvalValue::Number(right)) => EvalValue::Number(left + right),
                    _ => panic!("Invalid types for addition")
                }
            },
            "-" => {
                match (left, right) {
                    (EvalValue::Number(left), EvalValue::Number(right)) => EvalValue::Number(left - right),
                    _ => panic!("Invalid types for subtraction")
                }
            },
            "*" => {
                match (left, right) {
                    (EvalValue::Number(left), EvalValue::Number(right)) => EvalValue::Number(left * right),
                    _ => panic!("Invalid types for multiplication")
                }
            },
            "/" => {
                match (left, right) {
                    (EvalValue::Number(left), EvalValue::Number(right)) => EvalValue::Number(left / right),
                    _ => panic!("Invalid types for division")
                }
            },
            _ => panic!("Invalid operator")
        }
    }
}

