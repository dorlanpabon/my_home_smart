use std::collections::{BTreeSet, HashMap};

use serde_json::Value;

use crate::models::{
    app::{Device, DeviceChannel, DeviceLocalMetadata, LocalMetadata, RawDeviceData},
    tuya::{TuyaFunction, TuyaStatus},
};

pub fn infer_device_channels(
    device_id: &str,
    functions: &[TuyaFunction],
    status: &[TuyaStatus],
    capabilities: &[TuyaFunction],
    metadata: &LocalMetadata,
) -> Vec<DeviceChannel> {
    let status_map = status_map(status);
    let function_codes = codes_set(functions);
    let capability_codes = codes_set(capabilities);

    let multi_gang_codes = ["switch_1", "switch_2", "switch_3", "switch_4"]
        .into_iter()
        .filter(|code| {
            function_codes.contains(*code)
                || capability_codes.contains(*code)
                || status_map.contains_key(*code)
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    let selected_codes = if !multi_gang_codes.is_empty() {
        multi_gang_codes
    } else if function_codes.contains("switch")
        || capability_codes.contains("switch")
        || status_map.contains_key("switch")
    {
        vec!["switch".into()]
    } else if function_codes.contains("switch_led")
        || capability_codes.contains("switch_led")
        || status_map.contains_key("switch_led")
    {
        vec!["switch_led".into()]
    } else {
        fallback_switch_codes(functions, status, capabilities)
    };

    selected_codes
        .into_iter()
        .enumerate()
        .map(|(index, code)| {
            let alias = metadata
                .channel_alias_for(device_id, &code)
                .map(str::to_string);
            let current_state = status_map.get(&code).and_then(parse_boolean);
            let controllable =
                function_codes.contains(code.as_str()) || capability_codes.contains(code.as_str());

            DeviceChannel {
                display_name: alias
                    .clone()
                    .unwrap_or_else(|| default_channel_name(&code, index + 1)),
                code,
                index: index + 1,
                current_state,
                controllable,
                alias,
            }
        })
        .collect()
}

pub fn normalize_device(
    summary: Value,
    details: Value,
    functions: Vec<TuyaFunction>,
    status: Vec<TuyaStatus>,
    capabilities: Vec<TuyaFunction>,
    specifications: Value,
    metadata: &LocalMetadata,
) -> Device {
    let id = first_non_empty_string(&[
        summary.get("id"),
        summary.get("device_id"),
        details.get("id"),
        details.get("device_id"),
    ])
    .unwrap_or_else(|| "unknown-device".into());

    let channels = infer_device_channels(&id, &functions, &status, &capabilities, metadata);
    let alias = metadata.device_alias_for(&id).map(str::to_string);
    let cloud_name = first_non_empty_string(&[summary.get("name"), details.get("name")])
        .unwrap_or_else(|| "Unnamed device".into());
    let category = first_non_empty_string(&[summary.get("category"), details.get("category")]);
    let inferred_type = infer_device_type(category.as_deref(), channels.len());

    Device {
        id: id.clone(),
        name: alias.clone().unwrap_or(cloud_name),
        online: first_boolean(&[summary.get("online"), details.get("online")]).unwrap_or(false),
        category,
        product_id: first_non_empty_string(&[summary.get("product_id"), details.get("product_id")]),
        inferred_type,
        gang_count: channels.len(),
        channels,
        raw: RawDeviceData {
            summary,
            details,
            functions,
            status,
            capabilities,
            specifications,
        },
        metadata: Some(DeviceLocalMetadata { alias }),
    }
}

fn infer_device_type(category: Option<&str>, gang_count: usize) -> String {
    if gang_count > 0 {
        return if gang_count == 1 {
            "Single-channel light switch".into()
        } else {
            format!("{gang_count}-gang light switch")
        };
    }

    match category.unwrap_or_default() {
        "kg" | "cz" | "cjkg" | "tdq" => "Light switch".into(),
        "" => "Unknown device".into(),
        other => format!("{other} device"),
    }
}

fn fallback_switch_codes(
    functions: &[TuyaFunction],
    status: &[TuyaStatus],
    capabilities: &[TuyaFunction],
) -> Vec<String> {
    let mut codes = BTreeSet::new();

    for code in functions
        .iter()
        .filter(|entry| looks_like_switch_code(&entry.code))
        .map(|entry| entry.code.clone())
    {
        codes.insert(code);
    }

    for code in capabilities
        .iter()
        .filter(|entry| looks_like_switch_code(&entry.code))
        .map(|entry| entry.code.clone())
    {
        codes.insert(code);
    }

    for code in status
        .iter()
        .filter(|entry| looks_like_switch_code(&entry.code))
        .map(|entry| entry.code.clone())
    {
        codes.insert(code);
    }

    codes.into_iter().collect()
}

fn looks_like_switch_code(code: &str) -> bool {
    let lower = code.to_ascii_lowercase();
    lower == "switch"
        || lower == "switch_led"
        || lower.starts_with("switch_")
        || lower.contains("switch")
}

fn default_channel_name(code: &str, index: usize) -> String {
    match code {
        "switch" => "Main channel".into(),
        "switch_led" => "Channel 1".into(),
        _ if code.starts_with("switch_") => format!("Channel {index}"),
        _ => prettify_code(code),
    }
}

fn prettify_code(code: &str) -> String {
    code.replace('_', " ")
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_boolean(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(inner) => Some(*inner),
        Value::String(inner) if inner.eq_ignore_ascii_case("true") => Some(true),
        Value::String(inner) if inner.eq_ignore_ascii_case("false") => Some(false),
        _ => None,
    }
}

fn status_map(status: &[TuyaStatus]) -> HashMap<String, Value> {
    status
        .iter()
        .map(|entry| (entry.code.clone(), entry.value.clone()))
        .collect()
}

fn codes_set(entries: &[TuyaFunction]) -> BTreeSet<&str> {
    entries.iter().map(|entry| entry.code.as_str()).collect()
}

fn first_non_empty_string(values: &[Option<&Value>]) -> Option<String> {
    values.iter().find_map(|value| {
        value
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn first_boolean(values: &[Option<&Value>]) -> Option<bool> {
    values
        .iter()
        .find_map(|value| value.and_then(Value::as_bool))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::models::{
        app::LocalMetadata,
        tuya::{TuyaFunction, TuyaStatus},
    };

    use super::infer_device_channels;

    #[test]
    fn detects_four_gangs() {
        let functions = vec![
            TuyaFunction {
                code: "switch_1".into(),
                ..Default::default()
            },
            TuyaFunction {
                code: "switch_2".into(),
                ..Default::default()
            },
            TuyaFunction {
                code: "switch_3".into(),
                ..Default::default()
            },
            TuyaFunction {
                code: "switch_4".into(),
                ..Default::default()
            },
        ];
        let status = vec![
            TuyaStatus {
                code: "switch_1".into(),
                value: json!(true),
            },
            TuyaStatus {
                code: "switch_2".into(),
                value: json!(false),
            },
            TuyaStatus {
                code: "switch_3".into(),
                value: json!(true),
            },
            TuyaStatus {
                code: "switch_4".into(),
                value: json!(false),
            },
        ];

        let channels = infer_device_channels(
            "device-1",
            &functions,
            &status,
            &[],
            &LocalMetadata::default(),
        );

        assert_eq!(channels.len(), 4);
        assert!(channels[0].controllable);
    }

    #[test]
    fn uses_switch_led_as_single_channel_fallback() {
        let status = vec![TuyaStatus {
            code: "switch_led".into(),
            value: json!(true),
        }];
        let channels =
            infer_device_channels("device-1", &[], &status, &[], &LocalMetadata::default());
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].code, "switch_led");
    }
}
