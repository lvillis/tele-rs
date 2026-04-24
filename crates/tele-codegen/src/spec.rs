use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BotApiSpec {
    pub(crate) version: String,
    pub(crate) generated_from: String,
    #[serde(default)]
    pub(crate) all_methods: Vec<String>,
    #[serde(default)]
    pub(crate) advanced_methods: Vec<MethodSpec>,
}

impl BotApiSpec {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("spec is missing `version`".to_owned());
        }
        if self.generated_from.trim().is_empty() {
            return Err("spec is missing `generated_from`".to_owned());
        }
        if self.all_methods.is_empty() {
            return Err("spec is missing `all_methods`".to_owned());
        }
        if self.advanced_methods.is_empty() {
            return Err("spec is missing `advanced_methods`".to_owned());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MethodSpec {
    pub(crate) fn_name: String,
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) return_desc: String,
    #[serde(default)]
    pub(crate) params: Vec<ParamSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ParamSpec {
    pub(crate) name: String,
    pub(crate) field_name: String,
    pub(crate) required: bool,
    pub(crate) type_raw: String,
    pub(crate) type_rust: String,
}
