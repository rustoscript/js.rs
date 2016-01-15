use french_press::js_types::js_type::JsVar;
use std::collections::HashMap;

#[derive(Debug)]
pub struct StateManager {
    vars: HashMap<String, JsVar>,
}

impl StateManager {
    pub fn new() -> StateManager {
        StateManager { vars: HashMap::new() }
    }

    pub fn insert(&mut self, string: String, val: JsVar) -> Option<JsVar> {
        self.vars.insert(string, val)
    }

    pub fn get(&self, string: &String) -> Option<&JsVar> {
        self.vars.get(string)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.vars.len()
    }
}
