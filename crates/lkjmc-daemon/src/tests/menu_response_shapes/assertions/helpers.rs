use serde_json::Value;

pub(super) fn non_empty_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    let values = array_field(value, key)?;
    if values.is_empty() {
        return Err(format!("{key} must contain a seeded row"));
    }
    Ok(values)
}

pub(super) fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    field(value, key)?
        .as_array()
        .ok_or_else(|| format!("missing array key: {key}"))
}

pub(super) fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    as_object(field(value, key)?, key)
}

pub(super) fn as_object<'a>(value: &'a Value, label: &str) -> Result<&'a Value, String> {
    value
        .as_object()
        .map(|_| value)
        .ok_or_else(|| format!("missing object: {label}"))
}

pub(super) fn travel_row(row: &Value, name_key: &str) -> Result<(), String> {
    string(row, name_key)?;
    string(row, "serverId")?;
    let location = object(row, "location")?;
    string(location, "world")?;
    number(location, "x")?;
    number(location, "y")?;
    number(location, "z")
}

pub(super) fn string(value: &Value, key: &str) -> Result<(), String> {
    field(value, key)?
        .as_str()
        .map(|_| ())
        .ok_or_else(|| format!("missing string key: {key}"))
}

pub(super) fn bool_field(value: &Value, key: &str) -> Result<(), String> {
    field(value, key)?
        .as_bool()
        .map(|_| ())
        .ok_or_else(|| format!("missing boolean key: {key}"))
}

pub(super) fn true_bool(value: &Value, key: &str) -> Result<(), String> {
    match field(value, key)?.as_bool() {
        Some(true) => Ok(()),
        Some(false) => Err(format!("{key} must be true for seeded row")),
        None => Err(format!("missing boolean key: {key}")),
    }
}

pub(super) fn integer(value: &Value, key: &str) -> Result<(), String> {
    field(value, key)?
        .as_i64()
        .map(|_| ())
        .ok_or_else(|| format!("missing integer key: {key}"))
}

pub(super) fn integer_or_null(value: &Value, key: &str) -> Result<(), String> {
    let value = field(value, key)?;
    if value.is_null() || value.as_i64().is_some() {
        return Ok(());
    }
    Err(format!("missing integer-or-null key: {key}"))
}

pub(super) fn number(value: &Value, key: &str) -> Result<(), String> {
    if field(value, key)?.is_number() {
        return Ok(());
    }
    Err(format!("missing number key: {key}"))
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value.get(key).ok_or_else(|| format!("missing key: {key}"))
}
