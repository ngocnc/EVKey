//! Gemini text actions (add diacritics, fix spelling, rewrite, translate) on the selected text.
//! REST `generateContent`, key sent in the `x-goog-api-key` header (not in the URL):
//! https://ai.google.dev/gemini-api/docs/generate-content/get-started.md.txt
//! Model id from https://ai.google.dev/gemini-api/docs/models ("For any new projects, use 3.5 Flash-Lite or 3.8 Flash").
//! Request shape (`systemInstruction`, `contents`, `generationConfig`) and response shape (`candidates[0].content.parts[].text`,
//! `error.message` on failure) follow the same page's REST examples.

use std::time::Duration;

use serde_json::{json, Value};

use crate::i18n::tr;
use crate::secret;

const API: &str = "https://generativelanguage.googleapis.com/v1beta/models";
/// Fast, low-cost model for short text transforms. Change here only.
const MODEL: &str = "gemini-3.5-flash-lite";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tone {
    Formal,
    Shorter,
    Friendly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AiAction {
    /// Unaccented Vietnamese -> Vietnamese with tone and vowel marks.
    AddDiacritics,
    FixSpelling,
    Rewrite(Tone),
    /// Vietnamese -> English, anything else -> Vietnamese.
    Translate,
}

impl AiAction {
    /// The id the web UI sends to `ai_run`.
    fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "add_diacritics" => Self::AddDiacritics,
            "fix_spelling" => Self::FixSpelling,
            "rewrite_formal" => Self::Rewrite(Tone::Formal),
            "rewrite_shorter" => Self::Rewrite(Tone::Shorter),
            "rewrite_friendly" => Self::Rewrite(Tone::Friendly),
            "translate" => Self::Translate,
            _ => return None,
        })
    }

    fn system_prompt(self) -> &'static str {
        match self {
            Self::AddDiacritics => "The user's text is Vietnamese written without diacritics. Add the Vietnamese tone and vowel marks. Do not change, add or remove any word, its order, its case or its punctuation. Return ONLY the resulting text: no quotes, no labels, no explanations. Keep the line breaks.",
            Self::FixSpelling => "Fix spelling, typing and grammar mistakes in the user's text. Keep its language, meaning and line breaks. Return ONLY the corrected text: no quotes, no labels, no explanations. Keep the line breaks.",
            Self::Rewrite(Tone::Formal) => "Rewrite the user's text in a formal, polite tone. Keep its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks.",
            Self::Rewrite(Tone::Shorter) => "Rewrite the user's text to be shorter and more concise while keeping its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks.",
            Self::Rewrite(Tone::Friendly) => "Rewrite the user's text in a friendly, warm, casual tone. Keep its language and meaning. Return ONLY the rewritten text: no quotes, no labels, no explanations. Keep the line breaks.",
            Self::Translate => "If the user's text is mainly Vietnamese, translate it into English; otherwise translate it into Vietnamese. Return ONLY the translation: no quotes, no labels, no explanations. Keep the line breaks.",
        }
    }
}

fn request_body(action: AiAction, text: &str) -> Value {
    json!({
        "systemInstruction": { "parts": [{ "text": action.system_prompt() }] },
        "contents": [{ "role": "user", "parts": [{ "text": text }] }],
        "generationConfig": { "temperature": 0.2 }
    })
}

/// The generated text, or a user-facing error when Gemini blocked or returned nothing.
fn parse_response(json: &Value) -> Result<String, String> {
    if json["promptFeedback"]["blockReason"].is_string() {
        return Err(tr("Gemini từ chối xử lý văn bản này", "Gemini refused to process this text").into());
    }
    let text: String = json["candidates"][0]["content"]["parts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| p["thought"].as_bool() != Some(true))
        .filter_map(|p| p["text"].as_str())
        .collect();
    let text = text.trim();
    if text.is_empty() {
        return Err(tr("Gemini không trả về nội dung", "Gemini returned no content").into());
    }
    Ok(text.to_string())
}

fn http_error(status: u16, body: &Value) -> String {
    match status {
        400 | 401 | 403 => tr("API key Gemini không hợp lệ hoặc không có quyền", "The Gemini API key is invalid or not allowed").into(),
        429 => tr("Đã hết quota Gemini, thử lại sau", "Gemini quota exceeded, try again later").into(),
        _ => {
            let detail = body["error"]["message"].as_str().map_or_else(|| format!("HTTP {status}"), str::to_string);
            format!("{}: {detail}", tr("Lỗi Gemini", "Gemini error"))
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("EVKey/", env!("CARGO_PKG_VERSION")))
        .build()
}

fn call(req: ureq::Request, body: Option<Value>) -> Result<Value, String> {
    let res = match body {
        Some(body) => req.send_json(body),
        None => req.call(),
    };
    match res {
        Ok(resp) => resp.into_json::<Value>().map_err(|_| tr("Phản hồi Gemini không hợp lệ", "Invalid Gemini response").into()),
        Err(ureq::Error::Status(code, resp)) => Err(http_error(code, &resp.into_json::<Value>().unwrap_or(Value::Null))),
        Err(ureq::Error::Transport(_)) => Err(tr("Không kết nối được Gemini, kiểm tra mạng", "Could not reach Gemini, check your network").into()),
    }
}

/// Runs one action on `text` with the saved key. Blocking: call from `spawn_blocking`.
pub fn run(action: &AiAction, text: &str) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err(tr("Chưa có văn bản để xử lý", "No text to process").into());
    }
    let key = secret::gemini_key().ok_or_else(|| tr("Chưa lưu API key Gemini", "No Gemini API key saved"))?;
    let req = agent().post(&format!("{API}/{MODEL}:generateContent")).set("x-goog-api-key", &key);
    parse_response(&call(req, Some(request_body(*action, text)))?)
}

/// Checks the key with a metadata lookup (no generation, no text quota spent).
pub fn test_key(key: &str) -> Result<(), String> {
    let req = agent().get(&format!("{API}/{MODEL}")).set("x-goog-api-key", key);
    call(req, None).map(|_| ())
}

#[tauri::command]
pub async fn gemini_key_status() -> bool {
    secret::gemini_key().is_some()
}

#[tauri::command]
pub async fn gemini_key_set(key: String) -> Result<(), String> {
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(tr("API key trống", "The API key is empty").into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        test_key(&key)?;
        secret::set_gemini_key(&key)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn gemini_key_delete() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(secret::delete_gemini_key).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ai_run(action: String, text: String) -> Result<String, String> {
    let action = AiAction::from_id(&action).ok_or_else(|| tr("Thao tác AI không tồn tại", "Unknown AI action").to_string())?;
    tauri::async_runtime::spawn_blocking(move || run(&action, &text)).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_carries_system_prompt_and_text() {
        let body = request_body(AiAction::AddDiacritics, "tieng viet");
        assert!(body["systemInstruction"]["parts"][0]["text"].as_str().unwrap().contains("tone and vowel marks"));
        assert_eq!(body["contents"][0]["parts"][0]["text"], "tieng viet");
        assert_eq!(body["contents"][0]["role"], "user");
    }

    #[test]
    fn every_action_has_its_own_prompt() {
        let prompts = [
            AiAction::AddDiacritics,
            AiAction::FixSpelling,
            AiAction::Rewrite(Tone::Formal),
            AiAction::Rewrite(Tone::Shorter),
            AiAction::Rewrite(Tone::Friendly),
            AiAction::Translate,
        ]
        .map(|a| a.system_prompt());
        for (i, p) in prompts.iter().enumerate() {
            assert!(p.contains("ONLY"), "prompt {i} must ask for only the result");
            assert!(prompts[..i].iter().all(|q| q != p), "prompt {i} duplicates an earlier one");
        }
    }

    #[test]
    fn action_ids_map_to_actions() {
        assert_eq!(AiAction::from_id("add_diacritics"), Some(AiAction::AddDiacritics));
        assert_eq!(AiAction::from_id("rewrite_shorter"), Some(AiAction::Rewrite(Tone::Shorter)));
        assert_eq!(AiAction::from_id("translate"), Some(AiAction::Translate));
        assert_eq!(AiAction::from_id("nope"), None);
    }

    #[test]
    fn parses_text_and_skips_thought_parts() {
        let json = json!({"candidates": [{"content": {"parts": [
            {"text": "thinking", "thought": true},
            {"text": "  Xin chào\nbạn  "}
        ]}}]});
        assert_eq!(parse_response(&json).as_deref(), Ok("Xin chào\nbạn"));
    }

    #[test]
    fn empty_or_blocked_responses_are_errors() {
        assert!(parse_response(&json!({"candidates": []})).is_err());
        assert!(parse_response(&json!({"promptFeedback": {"blockReason": "SAFETY"}})).is_err());
    }

    #[test]
    fn http_errors_map_to_user_messages() {
        let quota = json!({"error": {"code": 429, "message": "quota"}});
        assert_eq!(http_error(429, &quota), tr("Đã hết quota Gemini, thử lại sau", "Gemini quota exceeded, try again later"));
        let bad_key = json!({"error": {"code": 400, "message": "API key not valid"}});
        assert!(http_error(400, &bad_key).contains("API key"));
        let other = json!({"error": {"code": 500, "message": "boom"}});
        assert!(http_error(500, &other).ends_with("boom"));
    }
}
