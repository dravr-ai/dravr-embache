// ABOUTME: Converts tool definitions between generic format and copilot-sdk Tool format.
// ABOUTME: Bridges function declarations to copilot_sdk::Tool for native tool calling.
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use copilot_sdk::Tool;
use serde_json::Value;

/// Convert function declarations to copilot-sdk `Tool` definitions.
///
/// Each declaration is a tuple of `(name, description, optional_parameters)`.
/// The SDK expects one `Tool` per function with name, description, and JSON Schema parameters.
#[must_use]
pub fn convert_function_declarations(
    declarations: &[(String, String, Option<Value>)],
) -> Vec<Tool> {
    declarations
        .iter()
        .map(|(name, description, parameters)| {
            let mut tool = Tool::new(name).description(description);
            if let Some(params) = parameters {
                tool = tool.schema(params.clone());
            }
            tool
        })
        .collect()
}

/// Extract function declarations from a Gemini-style `Tool` JSON value.
///
/// The Gemini `Tool` struct has `function_declarations: Vec<FunctionDeclaration>` where each
/// `FunctionDeclaration` has `name`, `description`, and optional `parameters`.
/// This function extracts them into a flat list suitable for `convert_function_declarations`.
pub fn extract_declarations_from_tool_value(
    tool_value: &Value,
) -> Vec<(String, String, Option<Value>)> {
    let declarations = tool_value
        .get("function_declarations")
        .or_else(|| tool_value.get("functionDeclarations"))
        .and_then(Value::as_array);

    let Some(declarations) = declarations else {
        return Vec::new();
    };

    declarations
        .iter()
        .filter_map(|decl| {
            let name = decl.get("name")?.as_str()?.to_owned();
            let description = decl
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let parameters = decl.get("parameters").cloned();
            Some((name, description, parameters))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_function_declarations() {
        let declarations = vec![
            (
                "get_weather".to_owned(),
                "Get weather for a city".to_owned(),
                Some(json!({
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    },
                    "required": ["city"]
                })),
            ),
            ("get_time".to_owned(), "Get current time".to_owned(), None),
        ];

        let tools = convert_function_declarations(&declarations);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "get_weather");
        assert_eq!(tools[1].name, "get_time");
    }

    #[test]
    fn test_extract_declarations_from_tool_value() {
        let tool_value = json!({
            "function_declarations": [
                {
                    "name": "get_activities",
                    "description": "Get user activities",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "limit": { "type": "integer" }
                        }
                    }
                },
                {
                    "name": "get_athlete",
                    "description": "Get athlete profile"
                }
            ]
        });

        let declarations = extract_declarations_from_tool_value(&tool_value);
        assert_eq!(declarations.len(), 2);
        assert_eq!(declarations[0].0, "get_activities");
        assert_eq!(declarations[1].0, "get_athlete");
        assert!(declarations[0].2.is_some());
        assert!(declarations[1].2.is_none());
    }

    #[test]
    fn test_extract_declarations_empty() {
        let tool_value = json!({});
        let declarations = extract_declarations_from_tool_value(&tool_value);
        assert!(declarations.is_empty());
    }
}
