use crate::{MAX_CELLS, MAX_COLUMNS, MAX_INPUT_BYTES, MAX_ROWS};

fn trim_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | b'\n' | 0x0c)
}
pub(crate) fn js_whitespace(c: char) -> bool {
    matches!(c, '\t'..='\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
fn js_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\0'..='\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Returns physical records before csv-parse's object mapping. Header/BOM is retained.
pub fn parse(text: &str) -> Result<Vec<Vec<String>>, String> {
    if text.len() > MAX_INPUT_BYTES * 3 {
        return Err("Decoded CSV exceeds resource limit".into());
    }
    let bytes = text.as_bytes();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row = Vec::new();
    let mut field = Vec::new();
    let (mut quoted, mut was_quoted) = (false, false);
    let mut delimiter: &[u8] = &[];
    let (mut i, mut line, mut pending_line, mut cells) = (0, 1, false, 0);
    while i < bytes.len() {
        if pending_line {
            line += 1;
            pending_line = false;
        }
        if !quoted && delimiter.is_empty() {
            delimiter = if bytes[i..].starts_with(b"\r\n") {
                b"\r\n"
            } else if bytes[i] == b'\n' {
                b"\n"
            } else if bytes[i] == b'\r' {
                b"\r"
            } else {
                b""
            };
        }
        let b = bytes[i];
        if b == b'\r' || b == b'\n' {
            pending_line = true;
        }
        if b == b'"' {
            if quoted {
                if bytes.get(i + 1) == Some(&b'"') {
                    field.push(b'"');
                    i += 2;
                    continue;
                }
                // A closing quote can discover the first record delimiter, even at lookahead.
                if delimiter.is_empty() {
                    delimiter = if bytes[i + 1..].starts_with(b"\r\n") {
                        b"\r\n"
                    } else if bytes.get(i + 1) == Some(&b'\n') {
                        b"\n"
                    } else if bytes.get(i + 1) == Some(&b'\r') {
                        b"\r"
                    } else {
                        b""
                    };
                }
                let next = bytes.get(i + 1).copied().unwrap_or(0);
                if next == 0
                    || next == b','
                    || trim_byte(next)
                    || (!delimiter.is_empty() && bytes[i + 1..].starts_with(delimiter))
                {
                    quoted = false;
                    was_quoted = true;
                    i += 1;
                    continue;
                }
                return Err(format!("Invalid Closing Quote: got \"{}\" at line {line} instead of delimiter, record delimiter, trimable character (if activated) or comment", char::from(next)));
            }
            if !field.is_empty() {
                let column = match rows.first() {
                    Some(header) => header
                        .get(row.len())
                        .map(|s| js_string(s))
                        .unwrap_or_else(|| "null".into()),
                    None => row.len().to_string(),
                };
                let value = String::from_utf8_lossy(&field);
                let bom = if value == "\u{feff}" {
                    " (utf8 bom)"
                } else {
                    ""
                };
                return Err(format!("Invalid Opening Quote: a quote is found on field {column} at line {line}, value is {}{bom}",js_string(&value)));
            }
            quoted = true;
            i += 1;
            continue;
        }
        if !quoted && (b == b',' || (!delimiter.is_empty() && bytes[i..].starts_with(delimiter))) {
            finish_field(&mut field, &mut row, was_quoted)?;
            was_quoted = false;
            if b == b',' {
                i += 1;
                continue;
            }
            finish_row(&mut rows, &mut row, line, &mut cells)?;
            i += delimiter.len();
            continue;
        }
        if quoted || (!was_quoted && (!field.is_empty() || !trim_byte(b))) {
            field.push(b);
        } else if was_quoted && !trim_byte(b) {
            return Err(format!(
                "Invalid Closing Quote: found non trimable byte after quote at line {line}"
            ));
        }
        i += 1;
    }
    if quoted {
        return Err(format!(
            "Quote Not Closed: the parsing is finished with an opening quote at line {line}"
        ));
    }
    if was_quoted || !row.is_empty() || !field.is_empty() {
        finish_field(&mut field, &mut row, was_quoted)?;
        finish_row(&mut rows, &mut row, line, &mut cells)?;
    }
    Ok(rows)
}
fn finish_field(field: &mut Vec<u8>, row: &mut Vec<String>, quoted: bool) -> Result<(), String> {
    if row.len() == MAX_COLUMNS {
        return Err("CSV exceeds 16384-column resource limit".into());
    }
    let text =
        String::from_utf8(std::mem::take(field)).map_err(|_| "Internal UTF-8 boundary error")?;
    let text = if quoted {
        text
    } else {
        text.trim_end_matches(js_whitespace).to_string()
    };
    row.push(text);
    Ok(())
}
fn finish_row(
    rows: &mut Vec<Vec<String>>,
    row: &mut Vec<String>,
    line: usize,
    cells: &mut usize,
) -> Result<(), String> {
    if let Some(header) = rows.first() {
        if header.len() != row.len() {
            return Err(format!(
                "Invalid Record Length: columns length is {}, got {} on line {line}",
                header.len(),
                row.len()
            ));
        }
    }
    *cells += row.len();
    if rows.len() >= MAX_ROWS || *cells > MAX_CELLS {
        return Err("CSV exceeds row/cell resource limit".into());
    }
    rows.push(std::mem::take(row));
    Ok(())
}
