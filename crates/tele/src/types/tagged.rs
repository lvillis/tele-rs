use serde::Serialize;
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

pub(crate) fn serialize_tagged<S, T>(
    serializer: S,
    kind: &str,
    value: &T,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize,
{
    serialize_tagged_field(serializer, "type", kind, value)
}

pub(crate) fn serialize_tagged_field<S, T>(
    serializer: S,
    field: &str,
    kind: &str,
    value: &T,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Serialize,
{
    let mut value = serde_json::to_value(value).map_err(serde::ser::Error::custom)?;
    let Value::Object(object) = &mut value else {
        return Err(serde::ser::Error::custom(
            "tagged Telegram object must serialize as an object",
        ));
    };
    object.insert(field.to_owned(), Value::String(kind.to_owned()));
    value.serialize(serializer)
}
