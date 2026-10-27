use serde::{Deserialize, Serialize};
use crate::source::location::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
    pub preview: String,
}

impl SearchMatch {
    pub fn id(&self) -> String {
        format!("{}:{}:{}", self.path, self.line, self.column)
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(&self.path, self.byte_offset)
    }

    /// Decodes ripgrep JSON output line into zero or more search match records.
    pub fn decode_rg_json_line(line: &str) -> Vec<SearchMatch> {
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            return Vec::new();
        };

        if val.get("type").and_then(|t| t.as_str()) != Some("match") {
            return Vec::new();
        }

        let Some(data) = val.get("data") else {
            return Vec::new();
        };

        let path = if let Some(p) = data.get("path").and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
            p.to_string()
        } else if let Some(bytes_b64) = data.get("path").and_then(|p| p.get("bytes")).and_then(|b| b.as_str()) {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(bytes_b64)
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
                .unwrap_or_default()
        } else {
            return Vec::new();
        };

        let clean_path = if path.starts_with("./") {
            path[2..].to_string()
        } else {
            path
        };

        let preview = if let Some(l) = data.get("lines").and_then(|l| l.get("text")).and_then(|t| t.as_str()) {
            l.trim_end_matches(['\r', '\n']).to_string()
        } else {
            String::new()
        };

        let line_number = data.get("line_number").and_then(|n| n.as_u64()).unwrap_or(1) as usize;
        let abs_offset = data.get("absolute_offset").and_then(|n| n.as_u64()).unwrap_or(0) as usize;

        let mut matches = Vec::new();
        if let Some(submatches) = data.get("submatches").and_then(|s| s.as_array()) {
            for sub in submatches {
                let start = sub.get("start").and_then(|s| s.as_u64()).unwrap_or(0) as usize;
                matches.push(SearchMatch {
                    path: clean_path.clone(),
                    line: line_number,
                    column: start + 1,
                    byte_offset: abs_offset + start,
                    preview: preview.clone(),
                });
            }
        }

        if matches.is_empty() {
            matches.push(SearchMatch {
                path: clean_path,
                line: line_number,
                column: 1,
                byte_offset: abs_offset,
                preview,
            });
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_rg_json() {
        let json_line = r#"{"type":"match","data":{"path":{"text":"src/main.rs"},"lines":{"text":"    println!(\"hello\");\n"},"line_number":10,"absolute_offset":200,"submatches":[{"match":{"text":"hello"},"start":14,"end":19}]}}"#;
        let matches = SearchMatch::decode_rg_json_line(json_line);
        assert_eq!(matches.len(), 1);
        let m = &matches[0];
        assert_eq!(m.path, "src/main.rs");
        assert_eq!(m.line, 10);
        assert_eq!(m.column, 15);
        assert_eq!(m.byte_offset, 214);
        assert_eq!(m.preview, "    println!(\"hello\");");
    }
}
