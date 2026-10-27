use crate::{MAX_CELLS, MAX_COLUMNS, MAX_OUTPUT_BYTES, MAX_ROWS};

/// Match JavaScript Object.keys: integer keys first, duplicate keys last-value wins;
/// __proto__ string assignment does not create an own property.
fn headers(row: &[String]) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for (i, h) in row.iter().enumerate() {
        if h == "__proto__" {
            continue;
        }
        if let Some(old) = out.iter_mut().find(|(k, _)| k == h) {
            old.1 = i;
        } else {
            out.push((h.clone(), i));
        }
    }
    fn index(s: &str) -> Option<u32> {
        s.parse::<u32>()
            .ok()
            .filter(|n| *n != u32::MAX && n.to_string() == s)
    }
    out.sort_by(|a, b| match (index(&a.0), index(&b.0)) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    out
}
fn column(mut c: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b'A' + (c % 26) as u8);
        if c < 26 {
            break;
        }
        c = c / 26 - 1;
    }
    out.reverse();
    String::from_utf8(out).expect("ASCII column")
}
fn xml(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&apos;"),
            '"' => out.push_str("&quot;"),
            '\0'..='\u{8}' | '\u{b}'..='\u{1f}' | '\u{fffe}' | '\u{ffff}' => {
                out.push_str(&format!("_x{:04x}_", c as u32))
            }
            _ => out.push(c),
        }
    }
    out
}
fn sheet(rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    if rows.len() > MAX_ROWS
        || rows.iter().any(|r| r.len() > MAX_COLUMNS)
        || rows.iter().map(Vec::len).sum::<usize>() > MAX_CELLS
    {
        return Err("CSV exceeds row/column/cell resource limit".into());
    }
    if let Some(header) = rows.first() {
        if rows.iter().any(|r| r.len() != header.len()) {
            return Err("CSV records have inconsistent columns".into());
        }
    }
    let hdr = if rows.len() > 1 {
        headers(&rows[0])
    } else {
        Vec::new()
    };
    let last_col = column(hdr.len().saturating_sub(1));
    let last_row = rows.len().max(1);
    let range = if hdr.len() <= 1 && last_row == 1 {
        "A1".into()
    } else {
        format!("A1:{last_col}{last_row}")
    };
    // With no records, json_to_sheet([]) always has A1, even if the CSV has a header.
    let range = if rows.len() <= 1 {
        "A1".to_string()
    } else {
        range
    };
    let mut data = String::new();
    if !hdr.is_empty() {
        for (r, row) in rows.iter().enumerate() {
            let row_space = if hdr.iter().any(|(h, c)| {
                if r == 0 {
                    h.contains('\n')
                } else {
                    row[*c].contains('\n')
                }
            }) {
                " xml:space=\"preserve\""
            } else {
                ""
            };
            data.push_str(&format!("<row r=\"{}\"{row_space}>", r + 1));
            for (c, (name, source)) in hdr.iter().enumerate() {
                let value = if r == 0 { name } else { &row[*source] };
                if value.encode_utf16().count() > 32767 {
                    return Err("Text length must not exceed 32767 characters".into());
                }
                let value = xml(value);
                let value = if value.starts_with(crate::parser::js_whitespace)
                    || value.ends_with(crate::parser::js_whitespace)
                    || value.contains('\n')
                {
                    format!("<v xml:space=\"preserve\">{value}</v>")
                } else {
                    format!("<v>{value}</v>")
                };
                let cell_space = if value.contains('\n') {
                    " xml:space=\"preserve\""
                } else {
                    ""
                };
                data.push_str(&format!(
                    "<c r=\"{}{}\" t=\"str\"{cell_space}>{value}</c>",
                    column(c),
                    r + 1
                ));
                if data.len() > MAX_OUTPUT_BYTES {
                    return Err("XLSX exceeds 64 MiB resource limit".into());
                }
            }
            data.push_str("</row>");
            if data.len() > MAX_OUTPUT_BYTES {
                return Err("XLSX exceeds 64 MiB resource limit".into());
            }
        }
    }
    let data = if data.is_empty() {
        "<sheetData/>".into()
    } else {
        format!("<sheetData>{data}</sheetData>")
    };
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\r\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><dimension ref=\"{range}\"/><sheetViews><sheetView workbookViewId=\"0\"/></sheetViews>{data}<ignoredErrors><ignoredError numberStoredAsText=\"1\" sqref=\"{range}\"/></ignoredErrors></worksheet>").into_bytes())
}
const PARTS: [(&str, &[u8]); 10] = [
    (
        "xl/_rels/workbook.xml.rels",
        include_bytes!("../fixtures/xlsx-template/xl/_rels/workbook.xml.rels"),
    ),
    (
        "xl/theme/theme1.xml",
        include_bytes!("../fixtures/xlsx-template/xl/theme/theme1.xml"),
    ),
    (
        "xl/styles.xml",
        include_bytes!("../fixtures/xlsx-template/xl/styles.xml"),
    ),
    ("xl/worksheets/sheet1.xml", b""),
    (
        "xl/metadata.xml",
        include_bytes!("../fixtures/xlsx-template/xl/metadata.xml"),
    ),
    (
        "xl/workbook.xml",
        include_bytes!("../fixtures/xlsx-template/xl/workbook.xml"),
    ),
    (
        "_rels/.rels",
        include_bytes!("../fixtures/xlsx-template/_rels/.rels"),
    ),
    (
        "docProps/app.xml",
        include_bytes!("../fixtures/xlsx-template/docProps/app.xml"),
    ),
    (
        "docProps/core.xml",
        include_bytes!("../fixtures/xlsx-template/docProps/core.xml"),
    ),
    (
        "[Content_Types].xml",
        include_bytes!("../fixtures/xlsx-template/[Content_Types].xml"),
    ),
];
fn u16le(out: &mut Vec<u8>, n: u16) {
    out.extend(n.to_le_bytes());
}
fn u32le(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_le_bytes());
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in bytes {
        c ^= b as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ (0xedb88320u32.wrapping_mul(c & 1));
        }
    }
    !c
}
/// SheetJS defaults to stored ZIP entries, zero DOS dates and no extra fields.
/// This writer reproduces its bytes without relying on OS timestamps or ZIP defaults.
pub fn workbook(rows: &[Vec<String>]) -> Result<Vec<u8>, String> {
    let xml = sheet(rows)?;
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, bytes) in PARTS {
        let bytes = if name == "xl/worksheets/sheet1.xml" {
            xml.as_slice()
        } else {
            bytes
        };
        if out.len() + bytes.len() + name.len() + 30 > MAX_OUTPUT_BYTES {
            return Err("XLSX exceeds 64 MiB resource limit".into());
        }
        let size = bytes.len() as u32;
        let offset = out.len() as u32;
        let crc = crc32(bytes);
        u32le(&mut out, 0x04034b50);
        u16le(&mut out, 20);
        for _ in 0..4 {
            u16le(&mut out, 0);
        }
        u32le(&mut out, crc);
        u32le(&mut out, size);
        u32le(&mut out, size);
        u16le(&mut out, name.len() as u16);
        u16le(&mut out, 0);
        out.extend(name.as_bytes());
        out.extend(bytes);
        u32le(&mut central, 0x02014b50);
        u16le(&mut central, 0);
        u16le(&mut central, 20);
        for _ in 0..4 {
            u16le(&mut central, 0);
        }
        u32le(&mut central, crc);
        u32le(&mut central, size);
        u32le(&mut central, size);
        u16le(&mut central, name.len() as u16);
        for _ in 0..4 {
            u16le(&mut central, 0);
        }
        u32le(&mut central, 0);
        u32le(&mut central, offset);
        central.extend(name.as_bytes());
    }
    let offset = out.len() as u32;
    let len = central.len() as u32;
    out.extend(central);
    u32le(&mut out, 0x06054b50);
    u16le(&mut out, 0);
    u16le(&mut out, 0);
    u16le(&mut out, 10);
    u16le(&mut out, 10);
    u32le(&mut out, len);
    u32le(&mut out, offset);
    u16le(&mut out, 0);
    Ok(out)
}
