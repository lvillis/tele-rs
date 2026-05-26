use serde_json::Value;

pub(crate) fn tagged_kind(value: &Value) -> Option<&str> {
    tagged_field(value, "type")
}

pub(crate) fn tagged_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .as_object()
        .and_then(|object| object.get(field))
        .and_then(Value::as_str)
}

pub(crate) fn strip_type(value: Value) -> Value {
    strip_tag(value, "type")
}

pub(crate) fn strip_tag(mut value: Value, field: &str) -> Value {
    if let Value::Object(object) = &mut value {
        object.remove(field);
    }
    value
}
