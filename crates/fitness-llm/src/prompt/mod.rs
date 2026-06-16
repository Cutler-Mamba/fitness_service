pub mod chat;
pub mod form_check;
pub mod nutrition_advice;
pub mod plan_generation;

pub struct PromptTemplate {
    pub system: String,
    pub json_schema: Option<serde_json::Value>,
}

impl PromptTemplate {
    pub fn new(system: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            json_schema: None,
        }
    }

    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.json_schema = Some(schema);
        self
    }
}
