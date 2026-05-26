use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde_json::Value;

pub(crate) fn is_reserved_key(key: &str, reserved: &[&str]) -> bool {
    reserved.contains(&key)
}

pub(crate) fn field_len(extra: &BTreeMap<String, Value>, reserved: &[&str]) -> usize {
    extra
        .keys()
        .filter(|key| !is_reserved_key(key, reserved))
        .count()
}

pub(crate) fn serialize_fields<M>(
    object: &mut M,
    extra: &BTreeMap<String, Value>,
    reserved: &[&str],
) -> std::result::Result<(), M::Error>
where
    M: SerializeMap,
{
    for (key, value) in extra {
        if !is_reserved_key(key, reserved) {
            object.serialize_entry(key, value)?;
        }
    }

    Ok(())
}
