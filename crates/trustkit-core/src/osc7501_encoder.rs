// OSC 7501 Terminal Telemetry Encoder.
// Constitutional Principle I: Zero external dependencies (std only).
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::errors::DomainViolation;

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn to_base64(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if i + 1 < input.len() {
            out.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < input.len() {
            out.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Osc7501Payload {
    pub state: String,
    pub id: String,
    pub kind: String,
    pub progress: Option<u8>,
    pub app: String,
    pub title: String,
    pub msg: String,
}

fn contains_emoji(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '\u{1F600}'..='\u{1F64F}' |
            '\u{1F300}'..='\u{1F5FF}' |
            '\u{1F680}'..='\u{1F6FF}' |
            '\u{1F700}'..='\u{1F77F}' |
            '\u{1F780}'..='\u{1F7FF}' |
            '\u{1F800}'..='\u{1F8FF}' |
            '\u{1F900}'..='\u{1F9FF}' |
            '\u{1FA00}'..='\u{1FA6F}' |
            '\u{1FA70}'..='\u{1FAFF}' |
            '\u{2600}'..='\u{26FF}'   |
            '\u{2700}'..='\u{27BF}'
        )
    })
}

pub fn encode_osc7501(payload: &Osc7501Payload) -> Result<String, DomainViolation> {
    if contains_emoji(&payload.title) || contains_emoji(&payload.msg) {
        return Err(DomainViolation::AstIntegrityOrInvariantViolation {
            invariant_id: "RN-001".to_string(),
            intercepted_command: "encode_osc7501".to_string(),
            reason: "OSC 7501 telemetry cannot contain emojis or icons".to_string(),
            remediation: "Remove all unicode emoji symbols from title and message".to_string(),
        });
    }

    let mut fields = Vec::with_capacity(7);
    fields.push(format!("state={}", payload.state));

    if !payload.id.is_empty() {
        fields.push(format!("id={}", payload.id));
    }
    if !payload.kind.is_empty() {
        fields.push(format!("kind={}", payload.kind));
    }
    if let Some(p) = payload.progress {
        fields.push(format!("progress={}", p.min(100)));
    }
    if !payload.app.is_empty() {
        fields.push(format!("app={}", payload.app));
    }
    if !payload.title.is_empty() {
        fields.push(format!("title={}", to_base64(payload.title.as_bytes())));
    }
    if !payload.msg.is_empty() {
        fields.push(format!("msg={}", to_base64(payload.msg.as_bytes())));
    }

    Ok(format!("\x1b]7501;{}\x1b\\", fields.join(":")))
}
