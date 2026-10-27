//! Production compiled-in adapter. Host imports this sibling file; JSON cannot install it.
use crate::native_core::{
    common::{CoreContext, CoreResult},
    extensions::v2_sdk as sdk,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tauri_plugin_dialog::DialogExt;
use tezbar_csv_to_excel::{convert, MAX_INPUT_BYTES};

pub const SLUG: &str = "csv-to-excel";
pub const ID: &str = "com.tezbar.native.csv-to-excel";
pub fn commands() -> &'static [&'static str] {
    &["index"]
}
static NONCE: AtomicU64 = AtomicU64::new(1);
fn capability(kind: &str) -> String {
    format!(
        "csv-to-excel:{kind}:{}:{}:{}",
        std::process::id(),
        NONCE.fetch_add(1, Ordering::Relaxed),
        uuid::Uuid::new_v4()
    )
}
/// The test boundary records every read/write and never touches desktop services.
/// Production conversion and filesystem publication remain native Rust.
pub trait FilePort {
    fn exists(&mut self, path: &Path) -> Result<bool, String>;
    fn is_file(&mut self, path: &Path) -> Result<bool, String>;
    fn read(&mut self, path: &Path) -> Result<Vec<u8>, String>;
    fn suffix(&mut self) -> u16;
    fn publish(&mut self, path: &Path, bytes: &[u8]) -> CoreResult;
}
pub struct NativeFiles;
impl FilePort for NativeFiles {
    fn exists(&mut self, path: &Path) -> Result<bool, String> {
        path.try_exists()
            .map_err(|_| "Could not inspect selected file".into())
    }
    fn is_file(&mut self, path: &Path) -> Result<bool, String> {
        std::fs::symlink_metadata(path)
            .map(|m| m.is_file())
            .map_err(|_| "Could not inspect selected file".into())
    }
    fn read(&mut self, path: &Path) -> Result<Vec<u8>, String> {
        let file = std::fs::File::open(path).map_err(|_| "Could not read selected CSV file")?;
        let mut bytes = Vec::new();
        file.take((MAX_INPUT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "Could not read selected CSV file")?;
        if bytes.len() > MAX_INPUT_BYTES {
            return Err("CSV input exceeds 8 MiB resource limit".into());
        }
        Ok(bytes)
    }
    fn suffix(&mut self) -> u16 {
        // Rejection sampling gives the same uniform integer range [1,1000) as V1.
        loop {
            let n = uuid::Uuid::new_v4().as_u128();
            let n = (n & 0xffff) as u16;
            if n < 64_935 {
                return n % 999 + 1;
            }
        }
    }
    fn publish(&mut self, path: &Path, bytes: &[u8]) -> CoreResult {
        let parent = path.parent().ok_or("Output directory is unavailable")?;
        // Atomic no-clobber publication avoids V1's exists/write race and partial files.
        let mut file = tempfile::NamedTempFile::new_in(parent)
            .map_err(|_| "Could not create XLSX in the selected directory")?;
        file.write_all(bytes)
            .map_err(|_| "Could not write XLSX file")?;
        file.flush().map_err(|_| "Could not flush XLSX file")?;
        file.as_file()
            .sync_all()
            .map_err(|_| "Could not sync XLSX file")?;
        file.persist_noclobber(path).map_err(|e| {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                format!("destination \"{}\" already exists.", path.display())
            } else {
                "Could not publish XLSX file".into()
            }
        })?;
        Ok(json!({"ok":true}))
    }
}
/// Never normalize null, booleans, contradictory or malformed effect responses.
pub fn require_ack(result: CoreResult) -> Result<(), String> {
    let value = result.map_err(|_| "File write could not be confirmed; retry deliberately")?;
    let valid = value.as_object().is_some_and(|o| {
        o.get("ok") == Some(&Value::Bool(true))
            && o.get("error").is_none_or(Value::is_null)
            && o.get("success").is_none_or(|v| v == &Value::Bool(true))
    });
    if valid {
        Ok(())
    } else {
        Err("File write could not be confirmed; retry deliberately".into())
    }
}
fn optional_values(payload: &Value) -> Result<(), String> {
    if payload
        .get("formValues")
        .is_some_and(|v| !v.is_null() && !v.is_object())
    {
        return Err("Expected formValues object".into());
    }
    Ok(())
}
fn action(id: &Value, title: &str, kind: &str) -> Value {
    json!({"id":id,"title":title,"kind":kind})
}
fn form(state: &Value, message: Option<&str>) -> Value {
    let mut children = vec![
        json!({"type":"Form.FilePicker","props":{"id":"files","title":"CSV File","allowMultipleSelection":false,"info":"Use Actions → Choose CSV File to select a file with its full path."}}),
    ];
    if let Some(text) = message {
        children.push(json!({"type":"Form.Description","props":{"title":"Failed to convert this CSV file","text":text}}));
    }
    sdk::view(
        "Convert CSV to Excel",
        json!({"type":"Form","props":{},"children":children}),
        vec![
            action(&state["submit"], "Convert", "submit-form"),
            action(&state["choose"], "Choose CSV File", "action"),
        ],
    )
}
fn restored_form(state: &Value, message: Option<&str>) -> Value {
    let mut view = form(state, message);
    view["root"]["children"][0]["props"]["value"] = state["draft"].clone();
    view
}
fn rotate_form(state: &mut Value) {
    state["submit"] = json!(capability("submit"));
    state["choose"] = json!(capability("choose"));
}
fn markdown_literal(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if "\\`*_{}[]<>()#+-.!|".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
fn outcome(state: &Value, message: Option<&str>) -> Value {
    let (markdown, actions) = if state["pending"].is_object() {
        (
            message
                .unwrap_or("File write could not be confirmed; retry deliberately")
                .to_string(),
            vec![
                action(&state["retry"], "Retry Writing Computed XLSX", "action"),
                action(&state["back"], "Back", "action"),
            ],
        )
    } else {
        (
            format!(
                "✅ Converted to the «{}» directory",
                markdown_literal(state["completed"]["directory"].as_str().unwrap_or(""))
            ),
            vec![action(&state["back"], "Back", "action")],
        )
    };
    sdk::view(
        "Convert CSV to Excel",
        json!({"type":"Detail","props":{"markdown":markdown,"backActionId":state["back"]}}),
        actions,
    )
}
fn form_files(payload: &Value) -> Result<Vec<String>, String> {
    let form = payload
        .get("formValues")
        .and_then(Value::as_object)
        .ok_or("Expected formValues object")?;
    let files = form
        .get("files")
        .and_then(Value::as_array)
        .ok_or("Expected files array")?;
    if files.len() > 128 {
        return Err("Too many selected file paths".into());
    }
    files
        .iter()
        .map(|f| {
            f.as_str()
                .filter(|s| s.len() <= 32768 && !s.contains('\0'))
                .map(str::to_owned)
                .ok_or_else(|| "Invalid selected file path".into())
        })
        .collect()
}
fn prepare(port: &mut dyn FilePort, files: &[String]) -> Result<(PathBuf, Vec<u8>), String> {
    let file = files
        .first()
        .filter(|s| !s.is_empty())
        .ok_or("You need choose a CSV file")?;
    let file = Path::new(file);
    if file.extension().and_then(|s| s.to_str()) != Some("csv") {
        return Err("File extension is not CSV".into());
    }
    if !file.is_absolute() {
        return Err("Selected file path is not absolute; use Actions → Choose CSV File".into());
    }
    if !port.exists(file)? || !port.is_file(file)? {
        return Err("File not found".into());
    }
    let name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid CSV file name")?;
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    let mut destination = dir.join(format!("{name}.xlsx"));
    if port.exists(&destination)? {
        let suffix = port.suffix();
        if !(1..1000).contains(&suffix) {
            return Err("Invalid collision suffix".into());
        }
        destination = dir.join(format!("{name}_{suffix}.xlsx"));
    }
    if port.exists(&destination)? {
        return Err(format!(
            "destination \"{}\" already exists.",
            destination.display()
        ));
    }
    let bytes = convert(&port.read(file)?)?;
    Ok((destination, bytes))
}
fn publish(port: &mut dyn FilePort, state: &mut Value) -> CoreResult {
    let destination = state["pending"]["destination"]
        .as_str()
        .ok_or("No pending output")?
        .to_string();
    let encoded = state["pending"]["bytes"]
        .as_str()
        .ok_or("Pending output is invalid")?;
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "Pending output is invalid")?;
    let result = require_ack(port.publish(Path::new(&destination), &bytes));
    if let Err(message) = result {
        return Ok(outcome(state, Some(&message)));
    }
    state["completed"] =
        json!({"destination":destination,"directory":state["pending"]["directory"]});
    state
        .as_object_mut()
        .ok_or("Invalid adapter state")?
        .remove("pending");
    state
        .as_object_mut()
        .ok_or("Invalid adapter state")?
        .remove("retry");
    Ok(outcome(state, None))
}
async fn choose_file(ctx: &CoreContext) -> Result<Option<String>, String> {
    let app = ctx
        .app
        .as_ref()
        .ok_or("Native file selection requires a desktop session")?;
    let (send, receive) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("CSV File", &["csv"])
        .pick_file(move |file| {
            let result = file
                .map(|p| {
                    p.into_path()
                        .map_err(|_| "Selected file is not a local path".to_string())
                        .and_then(|p| {
                            p.into_os_string()
                                .into_string()
                                .map_err(|_| "Selected path is not Unicode".to_string())
                        })
                })
                .transpose();
            let _ = send.send(result);
        });
    receive
        .await
        .map_err(|_| "Native file dialog closed unexpectedly")?
}
pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command == "index"
        && operation == "extension:invoke-action"
        && payload["actionId"]
            .as_str()
            .is_some_and(|id| state["choose"].as_str() == Some(id))
        && !state["pending"].is_object()
        && !state["completed"].is_object()
    {
        optional_values(payload)?;
        rotate_form(state);
        match choose_file(ctx).await {
            Ok(Some(path)) => {
                state["draft"] = json!([path]);
                return Ok(restored_form(state, None));
            }
            Ok(None) => return Ok(form(state, None)),
            Err(message) => return Ok(form(state, Some(&message))),
        }
    }
    handle_with(&mut NativeFiles, command, operation, payload, state)
}
/// Synchronous bounded conversion/publication contains no await between write and checkpoint.
/// Cancellation cannot drop the acknowledgement after publication and before state commit.
pub fn handle_with(
    port: &mut dyn FilePort,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != "index" {
        return Err("Unknown CSV to Excel command".into());
    }
    match operation {
        "extension:run-command" => {
            *state = json!({"draft":[],"submit":capability("submit"),"back":capability("back"),"choose":capability("choose")});
            Ok(form(state, None))
        }
        "extension:dispose-session" => {
            *state = Value::Null;
            Ok(sdk::unchanged())
        }
        "extension:refresh-session" | "extension:search-text-changed" | "extension:load-more" => {
            Ok(sdk::unchanged())
        }
        "extension:invoke-action" => {
            let id = payload["actionId"]
                .as_str()
                .ok_or("Missing action capability")?;
            optional_values(payload)?;
            if state["back"].as_str() == Some(id)
                && (state["pending"].is_object() || state["completed"].is_object())
            {
                state
                    .as_object_mut()
                    .ok_or("Invalid adapter state")?
                    .remove("pending");
                state
                    .as_object_mut()
                    .ok_or("Invalid adapter state")?
                    .remove("completed");
                state
                    .as_object_mut()
                    .ok_or("Invalid adapter state")?
                    .remove("retry");
                state["back"] = json!(capability("back"));
                rotate_form(state);
                return Ok(restored_form(state, None));
            }
            if state["retry"].as_str() == Some(id) && state["pending"].is_object() {
                state["retry"] = json!(capability("retry"));
                return publish(port, state);
            }
            if state["submit"].as_str() != Some(id)
                || state["pending"].is_object()
                || state["completed"].is_object()
            {
                return Err("Stale or unknown action capability".into());
            }
            // Validate the complete event before consuming its legitimate capability.
            let files = form_files(payload)?;
            state["draft"] = json!(files);
            rotate_form(state);
            let (destination, bytes) = match prepare(port, &files) {
                Ok(value) => value,
                Err(message) => return Ok(form(state, Some(&message))),
            };
            let directory = destination
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_string_lossy();
            state["pending"] = json!({"destination":destination.to_string_lossy(),"directory":directory,"bytes":STANDARD.encode(bytes)});
            state["retry"] = json!(capability("retry"));
            publish(port, state)
        }
        _ => Err("Unsupported CSV to Excel operation".into()),
    }
}
