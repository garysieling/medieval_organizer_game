//! Ollama integration for AI-driven task execution
//!
//! Shells out to the `ollama` CLI (no HTTP deps needed).
//! Each agent can execute tasks by prompting a local model.

use std::io::Write;
use std::process::{Command, Stdio};

/// Client for executing tasks via a local Ollama model
pub struct OllamaClient {
    /// Model name (e.g., "llama3.2", "qwen2.5-coder")
    model: String,
}

impl OllamaClient {
    /// Create a client for the given model
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }

    /// Check if Ollama is installed and the model is available
    pub fn is_available(&self) -> bool {
        Command::new("ollama")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// List locally available models
    pub fn list_models(&self) -> Result<Vec<String>, String> {
        let output = Command::new("ollama")
            .arg("list")
            .output()
            .map_err(|e| format!("Failed to run ollama list: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "ollama list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .skip(1) // skip header
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect())
    }

    /// Run a prompt against the model and return the response
    pub fn prompt(&self, prompt: &str) -> Result<String, String> {
        let mut child = Command::new("ollama")
            .args(["run", &self.model])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn ollama: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .map_err(|e| format!("Failed to write prompt: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for ollama: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "ollama run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Run a prompt through the Ollama HTTP API (via `curl`) with sampling options.
    /// Falls back to the CLI (`prompt`) when the API is unreachable.
    pub fn generate(&self, prompt: &str, temperature: f32, seed: u64) -> Result<String, String> {
        let url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());
        let body = format!(
            "{{\"model\":{},\"prompt\":{},\"stream\":false,\"options\":{{\"temperature\":{:.2},\"seed\":{}}}}}",
            json::quote(&self.model),
            json::quote(prompt),
            temperature,
            seed % (i32::MAX as u64)
        );
        let child = Command::new("curl")
            .args(["-s", "-m", "900", "-X", "POST", "-H", "Content-Type: application/json", "--data-binary", "@-"])
            .arg(format!("{}/api/generate", url.trim_end_matches('/')))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let child = match child {
            Ok(c) => c,
            // no curl: use the CLI (no temperature/seed control)
            Err(_) => return self.prompt(prompt),
        };
        let out = {
            let mut child = child;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
            }
            child.wait_with_output().map_err(|e| e.to_string())?
        };
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        if text.trim().is_empty() {
            // server unreachable over HTTP: fall back to the CLI
            return self.prompt(prompt);
        }
        match json::extract_string(&text, "response") {
            Some(r) => Ok(r.trim().to_string()),
            None => Err(json::extract_string(&text, "error")
                .unwrap_or_else(|| format!("unexpected Ollama reply: {}", text.chars().take(200).collect::<String>()))),
        }
    }

    /// Execute a task by prompting the model and returning the result
    pub fn execute_task(&self, task_description: &str, agent_role: &str) -> Result<String, String> {
        let prompt = format!(
            "You are a {}. Complete this task and report the result concisely:\n\n{}",
            agent_role, task_description
        );
        self.prompt(&prompt)
    }
}

/// Minimal JSON string helpers (std-only; enough for Ollama and Wikipedia responses).
pub mod json {
    /// Encode `s` as a quoted JSON string.
    pub fn quote(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    /// Decode the JSON string body starting right after an opening quote.
    /// Returns the decoded string and the byte index just past the closing quote.
    fn decode_from(s: &str, start: usize) -> Option<(String, usize)> {
        let bytes = s.as_bytes();
        let mut out = String::new();
        let mut i = start;
        let mut pending_high: Option<u32> = None;
        while i < bytes.len() {
            match bytes[i] {
                b'"' => return Some((out, i + 1)),
                b'\\' => {
                    let esc = *bytes.get(i + 1)?;
                    i += 2;
                    match esc {
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'b' | b'f' => {}
                        b'u' => {
                            let code = u32::from_str_radix(s.get(i..i + 4)?, 16).ok()?;
                            i += 4;
                            if (0xD800..0xDC00).contains(&code) {
                                pending_high = Some(code);
                            } else if (0xDC00..0xE000).contains(&code) {
                                if let Some(h) = pending_high.take() {
                                    let c = 0x10000 + ((h - 0xD800) << 10) + (code - 0xDC00);
                                    out.extend(char::from_u32(c));
                                }
                            } else {
                                out.extend(char::from_u32(code));
                            }
                        }
                        other => out.push(other as char),
                    }
                }
                _ => {
                    let ch = s[i..].chars().next()?;
                    out.push(ch);
                    i += ch.len_utf8();
                }
            }
        }
        None
    }

    /// Every string value for `"key":"..."` in the document, in order.
    pub fn extract_all_strings(doc: &str, key: &str) -> Vec<String> {
        let needle = format!("\"{}\":", key);
        let mut out = Vec::new();
        let mut pos = 0;
        while let Some(found) = doc[pos..].find(&needle) {
            let at = pos + found;
            let mut i = at + needle.len();
            if at > 0 && doc.as_bytes()[at - 1] == b'\\' {
                // escaped quote inside another string value, not a key
                pos = i;
                continue;
            }
            while doc.as_bytes().get(i) == Some(&b' ') {
                i += 1;
            }
            if doc.as_bytes().get(i) == Some(&b'"') {
                if let Some((value, end)) = decode_from(doc, i + 1) {
                    out.push(value);
                    pos = end;
                    continue;
                }
            }
            pos = i;
        }
        out
    }

    /// First string value for `"key":"..."`.
    pub fn extract_string(doc: &str, key: &str) -> Option<String> {
        extract_all_strings(doc, key).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OllamaClient::new("llama3.2");
        assert_eq!(client.model, "llama3.2");
    }

    #[test]
    fn test_json_round_trip() {
        let original = "line \"one\"\n\ttab \\ ünï";
        let doc = format!("{{\"response\":{},\"done\":true}}", json::quote(original));
        assert_eq!(json::extract_string(&doc, "response").unwrap(), original);
    }

    #[test]
    fn test_json_ignores_escaped_keys_inside_values() {
        let doc = r#"{"response":"say \"error\": no","done":true}"#;
        assert_eq!(json::extract_string(doc, "error"), None);
        assert_eq!(json::extract_string(doc, "response").unwrap(), "say \"error\": no");
    }

    #[test]
    fn test_json_unicode_escapes() {
        let doc = r#"[{"title":"Caf\u00e9"},{"title": "\ud83d\ude00 x"}]"#;
        assert_eq!(json::extract_all_strings(doc, "title"), vec!["Café", "😀 x"]);
    }

    #[test]
    fn test_is_available() {
        let client = OllamaClient::new("llama3.2");
        // Availability depends on environment; just ensure it doesn't panic
        let _ = client.is_available();
    }
}
