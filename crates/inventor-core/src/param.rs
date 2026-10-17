use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Millimeter,
    Meter,
    Inch,
    Degree,
    Radian,
    Unitless,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub expression: String,
    pub value: f64,
    pub unit: Unit,
    pub comment: String,
}

impl Parameter {
    pub fn new(name: impl Into<String>, value: f64, unit: Unit) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            expression: format!("{:.4}", value),
            value,
            unit,
            comment: String::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ParameterTable {
    params: HashMap<String, Parameter>,
}

impl ParameterTable {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn set(&mut self, param: Parameter) {
        self.params.insert(param.name.clone(), param);
    }

    pub fn get(&self, name: &str) -> Option<&Parameter> {
        self.params.get(name)
    }

    pub fn get_value(&self, name: &str) -> Option<f64> {
        self.params.get(name).map(|p| p.value)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.params.contains_key(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Parameter> {
        self.params.remove(name)
    }

    pub fn list(&self) -> Vec<&Parameter> {
        let mut list: Vec<&Parameter> = self.params.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}
