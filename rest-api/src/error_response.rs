use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<HashMap<String, String>>,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
            data: None,
        }
    }

    pub fn with_data(mut self, key: &str, val: &str) -> Self {
        if let Some(m) = self.data.as_mut() {
            m.insert(key.to_owned(), val.to_owned());
            return self;
        }

        let mut data = HashMap::new();
        data.insert(key.to_owned(), val.to_owned());
        Self {
            code: self.code,
            message: self.message,
            data: Some(data),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self, key: &str) -> Option<&str> {
        self.data
            .as_ref()
            .and_then(|d| d.get(key))
            .map(|d| d.as_str())
    }
}
