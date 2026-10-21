use std::process::Command;
use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffItem {
    Unchanged(String),
    Removed(String),
    Added(String),
}

/// Computes a line-by-line diff between original lines and modified lines using LCS.
pub fn compute_line_diff(old_lines: &[String], new_lines: &[String]) -> Vec<DiffItem> {
    let n = old_lines.len();
    let m = new_lines.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    for i in 0..n {
        for j in 0..m {
            if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut diff = Vec::new();
    let mut i = n;
    let mut j = m;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            diff.push(DiffItem::Unchanged(old_lines[i - 1].clone()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            diff.push(DiffItem::Added(new_lines[j - 1].clone()));
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i][j - 1] < dp[i - 1][j]) {
            diff.push(DiffItem::Removed(old_lines[i - 1].clone()));
            i -= 1;
        }
    }

    diff.reverse();
    diff
}

/// Strips markdown fences (e.g. ```rust ... ```) if returned by the LLM
pub fn strip_markdown_code_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    let mut lines: Vec<&str> = trimmed.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    if lines[0].starts_with("```") {
        lines.remove(0);
    }

    if let Some(last) = lines.last() {
        if last.trim().starts_with("```") {
            lines.pop();
        }
    }

    lines.join("\n")
}

/// Resolves the API key from config or environment variables
pub fn resolve_api_key(config_key: &str) -> Option<String> {
    let trimmed = config_key.trim();
    if !trimmed.is_empty() {
        if trimmed.starts_with("$(") && trimmed.ends_with(')') {
            let var_name = &trimmed[2..trimmed.len() - 1];
            if let Ok(val) = std::env::var(var_name) {
                if !val.trim().is_empty() {
                    return Some(val.trim().to_string());
                }
            }
        } else if trimmed.starts_with('$') {
            let var_name = &trimmed[1..];
            if let Ok(val) = std::env::var(var_name) {
                if !val.trim().is_empty() {
                    return Some(val.trim().to_string());
                }
            }
        } else {
            return Some(trimmed.to_string());
        }
    }

    // Fallbacks to standard environment variables
    if let Ok(val) = std::env::var("GEMINI_API_KEY") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }
    if let Ok(val) = std::env::var("OPENAI_API_KEY") {
        if !val.trim().is_empty() {
            return Some(val.trim().to_string());
        }
    }

    None
}

/// Call AI assistant to modify lines
pub fn call_ai_assistant(
    prompt: &str,
    target_lines: &[String],
    file_context: &str,
    config: &Config,
) -> Result<Vec<String>, String> {
    let api_key = resolve_api_key(&config.ai_api_key).ok_or_else(|| {
        "No API key found. Set ApiKey in .nimorc or export GEMINI_API_KEY / OPENAI_API_KEY".to_string()
    })?;

    let is_gemini = if !config.ai_provider.is_empty() {
        config.ai_provider.to_lowercase().contains("gemini")
    } else if config.ai_model.to_lowercase().starts_with("gemini") {
        true
    } else if api_key.starts_with("AIza") {
        true
    } else {
        false
    };

    let model = if !config.ai_model.is_empty() {
        config.ai_model.clone()
    } else if is_gemini {
        "gemini-2.5-flash".to_string()
    } else {
        "gpt-4o-mini".to_string()
    };

    let target_code = target_lines.join("\n");
    let full_user_prompt = format!(
        "[Instruction]:\n{}\n\n[File Context]:\n{}\n\n[Code lines to modify]:\n{}",
        prompt, file_context, target_code
    );

    let system_instructions = "You are an expert code assistant embedded in a fast terminal text editor.\nModify the provided code lines according to the user's instruction.\nCRITICAL RULES:\n1. Return ONLY the replacement code lines for the specified range.\n2. Do NOT wrap output in markdown code fences (no ``` or ```rust).\n3. Do NOT include any explanations, greetings, or notes.\n4. Maintain correct indentation.";

    if is_gemini {
        let endpoint = if !config.ai_endpoint.is_empty() {
            config.ai_endpoint.clone()
        } else {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, api_key
            )
        };

        let payload = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        { "text": full_user_prompt }
                    ]
                }
            ],
            "systemInstruction": {
                "parts": [
                    { "text": system_instructions }
                ]
            }
        });

        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let output = Command::new("curl")
            .arg("-s")
            .arg("-S")
            .arg("--max-time")
            .arg("45")
            .arg("-X")
            .arg("POST")
            .arg(&endpoint)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&payload_str)
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("curl failed: {}", err_msg));
        }

        let resp_str = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from Gemini: {} (response: {})", e, resp_str))?;

        if let Some(err) = resp_json.get("error") {
            let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("API error");
            return Err(format!("Gemini API error: {}", msg));
        }

        let reply_text = resp_json
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("content"))
            .and_then(|cnt| cnt.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p0| p0.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| format!("Unexpected response format from Gemini: {}", resp_str))?;

        let cleaned = strip_markdown_code_fences(reply_text);
        Ok(cleaned.lines().map(|s| s.to_string()).collect())
    } else {
        // OpenAI-compatible endpoint
        let endpoint = if !config.ai_endpoint.is_empty() {
            config.ai_endpoint.clone()
        } else {
            "https://api.openai.com/v1/chat/completions".to_string()
        };

        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_instructions
                },
                {
                    "role": "user",
                    "content": full_user_prompt
                }
            ]
        });

        let payload_str = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        let output = Command::new("curl")
            .arg("-s")
            .arg("-S")
            .arg("--max-time")
            .arg("45")
            .arg("-X")
            .arg("POST")
            .arg(&endpoint)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", api_key))
            .arg("-d")
            .arg(&payload_str)
            .output()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("curl failed: {}", err_msg));
        }

        let resp_str = String::from_utf8_lossy(&output.stdout);
        let resp_json: serde_json::Value = serde_json::from_str(&resp_str)
            .map_err(|e| format!("Invalid JSON from OpenAI endpoint: {} (response: {})", e, resp_str))?;

        if let Some(err) = resp_json.get("error") {
            let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("API error");
            return Err(format!("API error: {}", msg));
        }

        let reply_text = resp_json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c0| c0.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|cnt| cnt.as_str())
            .ok_or_else(|| format!("Unexpected response format from OpenAI API: {}", resp_str))?;

        let cleaned = strip_markdown_code_fences(reply_text);
        Ok(cleaned.lines().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_line_diff() {
        let old_lines = vec!["fn main() {".to_string(), "    println!(\"hello\");".to_string(), "}".to_string()];
        let new_lines = vec!["fn main() {".to_string(), "    println!(\"world\");".to_string(), "}".to_string()];

        let diff = compute_line_diff(&old_lines, &new_lines);
        assert_eq!(
            diff,
            vec![
                DiffItem::Unchanged("fn main() {".to_string()),
                DiffItem::Removed("    println!(\"hello\");".to_string()),
                DiffItem::Added("    println!(\"world\");".to_string()),
                DiffItem::Unchanged("}".to_string()),
            ]
        );
    }

    #[test]
    fn test_strip_markdown_code_fences() {
        let text = "```rust\nlet x = 10;\nlet y = 20;\n```";
        assert_eq!(strip_markdown_code_fences(text), "let x = 10;\nlet y = 20;");

        let text_no_fence = "let a = 1;\nlet b = 2;";
        assert_eq!(strip_markdown_code_fences(text_no_fence), "let a = 1;\nlet b = 2;");
    }

    #[test]
    fn test_resolve_api_key_env() {
        unsafe { std::env::set_var("TEST_NIMO_KEY", "secret_123") };
        assert_eq!(resolve_api_key("$(TEST_NIMO_KEY)").unwrap(), "secret_123");
        assert_eq!(resolve_api_key("$TEST_NIMO_KEY").unwrap(), "secret_123");
        assert_eq!(resolve_api_key("literal_key").unwrap(), "literal_key");
    }
}
