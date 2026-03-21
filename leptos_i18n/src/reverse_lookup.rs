/// Build a [`HashMap`](std::collections::HashMap) mapping default-locale string values to target-locale string values.
///
/// Walks two JSON trees (parsed from locale files) in parallel and collects
/// all leaf string pairs where the values differ.
///
/// # Errors
///
/// Returns a [`serde_json::Error`] if either JSON string is malformed.
pub fn translation_map_builder(
    default_json: &str,
    target_json: &str,
) -> Result<std::collections::HashMap<String, String>, serde_json::Error> {
    use std::collections::HashMap;

    fn collect_pairs(
        default: &serde_json::Value,
        target: &serde_json::Value,
        map: &mut HashMap<String, String>,
    ) {
        match (default, target) {
            (serde_json::Value::Object(d), serde_json::Value::Object(t)) => {
                for (key, d_val) in d {
                    if let Some(t_val) = t.get(key) {
                        collect_pairs(d_val, t_val, map);
                    }
                }
            }
            (serde_json::Value::String(d), serde_json::Value::String(t)) => {
                if d != t {
                    map.insert(d.clone(), t.clone());
                }
            }
            _ => {}
        }
    }

    let default: serde_json::Value = serde_json::from_str(default_json)?;
    let target: serde_json::Value = serde_json::from_str(target_json)?;
    let mut map = HashMap::new();
    collect_pairs(&default, &target, &mut map);
    Ok(map)
}
