use std::collections::BTreeSet;

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

        validate_unique_method_names("all_methods", &self.all_methods)?;
        let all_methods = self
            .all_methods
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut advanced_methods = BTreeSet::new();
        for method in &self.advanced_methods {
            method.validate()?;
            if !all_methods.contains(method.method.as_str()) {
                return Err(format!(
                    "advanced method `{}` is missing from `all_methods`",
                    method.method
                ));
            }
            if !advanced_methods.insert(method.method.as_str()) {
                return Err(format!("duplicate advanced method `{}`", method.method));
            }
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

impl MethodSpec {
    fn validate(&self) -> Result<(), String> {
        validate_method_name("advanced method", &self.method)?;
        if self.fn_name.trim().is_empty() {
            return Err(format!("method `{}` is missing `fn_name`", self.method));
        }

        let mut params = BTreeSet::new();
        for param in &self.params {
            param.validate(&self.method)?;
            if !params.insert(param.name.as_str()) {
                return Err(format!(
                    "method `{}` contains duplicate parameter `{}`",
                    self.method, param.name
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ParamSpec {
    pub(crate) name: String,
    pub(crate) field_name: String,
    pub(crate) required: bool,
    pub(crate) type_raw: String,
    pub(crate) type_rust: String,
}

impl ParamSpec {
    fn validate(&self, method: &str) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err(format!(
                "method `{method}` contains a parameter without `name`"
            ));
        }
        if self.field_name.trim().is_empty() {
            return Err(format!(
                "method `{method}` parameter `{}` is missing `field_name`",
                self.name
            ));
        }
        if self.type_raw.trim().is_empty() {
            return Err(format!(
                "method `{method}` parameter `{}` is missing `type_raw`",
                self.name
            ));
        }
        if self.type_rust.trim().is_empty() {
            return Err(format!(
                "method `{method}` parameter `{}` is missing `type_rust`",
                self.name
            ));
        }
        Ok(())
    }
}

fn validate_unique_method_names(label: &str, methods: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for method in methods {
        validate_method_name(label, method)?;
        if !seen.insert(method.as_str()) {
            return Err(format!("duplicate method `{method}` in `{label}`"));
        }
    }
    Ok(())
}

fn validate_method_name(label: &str, method: &str) -> Result<(), String> {
    let mut chars = method.chars();
    let Some(first) = chars.next() else {
        return Err(format!("{label} contains an empty method name"));
    };
    if !first.is_ascii_lowercase() || !method.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(format!("{label} `{method}` is not a Bot API method name"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> BotApiSpec {
        BotApiSpec {
            version: "Bot API 9.6".to_owned(),
            generated_from: "https://core.telegram.org/bots/api".to_owned(),
            all_methods: vec!["getMe".to_owned()],
            advanced_methods: vec![MethodSpec {
                fn_name: "get_me".to_owned(),
                method: "getMe".to_owned(),
                return_desc: "a User object".to_owned(),
                params: vec![ParamSpec {
                    name: "user_id".to_owned(),
                    field_name: "user_id".to_owned(),
                    required: true,
                    type_raw: "Integer".to_owned(),
                    type_rust: "UserId".to_owned(),
                }],
            }],
        }
    }

    #[test]
    fn validates_consistent_spec() {
        assert!(valid_spec().validate().is_ok());
    }

    #[test]
    fn rejects_duplicate_all_methods() {
        let mut spec = valid_spec();
        spec.all_methods.push("getMe".to_owned());
        assert!(spec.validate().is_err());
    }

    #[test]
    fn rejects_advanced_method_missing_from_all_methods() {
        let mut spec = valid_spec();
        spec.all_methods.clear();
        spec.all_methods.push("sendMessage".to_owned());
        assert!(spec.validate().is_err());
    }
}
