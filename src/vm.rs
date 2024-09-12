use crate::ast::EvalValue;
use crate::ast::Statement;
use std::collections::HashMap;

type ValueMap = HashMap<String, EvalValue>;

#[derive(Debug)]
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
        let mut target_scope = self
            .scopes_stack
            .iter_mut()
            .rev()
            .find(|v| v.contains_key(&name));

        if target_scope.is_none() {
            target_scope = self.scopes_stack.first_mut()
        }
        
        target_scope.unwrap().insert(name, value);
    }

    pub fn execute(&mut self, ast: &Vec<Statement>) -> Result<(), String> {
        for stat in ast {
            stat.execute(self)?;
        }

        Ok(())
    }
}
