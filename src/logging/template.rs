use std::collections::HashMap;

/// Apply placeholder substitution to a template string.
/// Replaces `{{key}}` with `vars[key]`.
/// Single-pass scan: O(template.len()) time, O(1) lookups per placeholder.
pub fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut last_end = 0;
    let mut i = 0;

    while i < bytes.len() {
        // Look for opening {{
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Add text before placeholder
            result.push_str(&template[last_end..i]);

            // Find closing }}
            let start = i + 2;
            let mut j = start;
            while j + 1 < bytes.len() {
                if bytes[j] == b'}' && bytes[j + 1] == b'}' {
                    let key = &template[start..j];
                    if let Some(value) = vars.get(key) {
                        result.push_str(value);
                    } else {
                        // Key not found, preserve original placeholder
                        result.push_str(&template[i..j + 2]);
                    }
                    i = j + 2;
                    last_end = i;
                    // Move past this placeholder and continue scanning
                    break;
                }
                j += 1;
            }

            // no closing found, add rest of string and exit
            if j + 1 >= bytes.len() {
                result.push_str(&template[i..]);
                return result;
            }
        } else {
            i += 1;
        }
    }

    result.push_str(&template[last_end..]);
    result
}
