//! Offline port of @aternus/csv-to-xlsx 2.1.5, csv-parse 5.4.0 and SheetJS 0.20.1.
//! Input is UTF-8 decoded like Node; all cells are strings, never formulas.
mod parser;
mod workbook;

pub use parser::parse;
pub use workbook::workbook;

/// Bounds are explicit errors. No row, column, text or output is truncated.
pub const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CELLS: usize = 250_000;
pub const MAX_ROWS: usize = 100_000;
pub const MAX_COLUMNS: usize = 16_384;
pub const MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

pub fn convert(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err("CSV input exceeds 8 MiB resource limit".into());
    }
    let text = String::from_utf8_lossy(bytes);
    workbook(&parse(&text)?)
}
