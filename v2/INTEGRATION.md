# Host integration (pending; this task never edits the host)

Target contract: Tezbar beta `fd300e276356587726566026f072563eb9c34cb1`. All host fixture bytes were obtained with `git show`, not by copying the dirty working tree. The host branch `fix/deep-search-state` is outside this implementation's scope. No host registration, profile, catalog, ledger, icon installation or rebuild has been performed.

## Stable vendoring and compiled registration

1. Vendor this `v2/` package at **`src-tauri/native-extensions/csv-to-excel`**. Keep the Cargo library, adapter, exact icon, notices and XLSX templates. For a smaller runtime-only vendor, retain `src`, `Cargo.toml`, `Cargo.lock`, `adapter.rs`, `fixtures/xlsx-template` and their license/provenance notices; change the package include/dev fixture configuration deliberately rather than silently losing validation fixtures.
2. Add `tezbar-csv-to-excel = { path = "native-extensions/csv-to-excel" }` to the host Cargo dependencies. The adapter uses existing host `serde_json`, `base64`, `uuid` v4, `tempfile`, `tokio` sync and `tauri-plugin-dialog` dependencies. It does not need JS/Bun or a converter executable.
3. In `src-tauri/src/native_core/extensions/mod.rs`, register the sibling adapter using exactly:

   ```rust
   #[path = "../../../native-extensions/csv-to-excel/adapter.rs"]
   mod v2_csv_to_excel;
   ```

4. In `extensions/v2_runtime.rs`, add `"csv-to-excel"` to `slug`'s accepted slugs; add a `commands` arm `"csv-to-excel" => super::v2_csv_to_excel::commands(),`; add a `handle` arm:

   ```rust
   "csv-to-excel" => {
       super::v2_csv_to_excel::handle(ctx, command, operation, payload, state).await
   }
   ```

   The native canonical ID is `com.tezbar.native.csv-to-excel`, alias `raycast.csv-to-excel`; command coverage is exactly `["index"]`. The own V1 manifest ID `com.tezbar.csv-to-excel` remains separate; no unrecognized alias or legacy migration is assumed.
5. Append `manifest.json` as one object in `extensions/v2_catalog.json`. Do not add root `author` (the pinned full schema rejects it); credit is in the description and notices. The icon URL is `/extensions/icons/csv-to-excel.png`; copy `original-icon.png` byte-for-byte to **`src/renderer/public/extensions/icons/csv-to-excel.png`**. Verify 26,498 bytes and SHA-256 `7e091606ff926192548547eae0bd3743928929c3cb01124c2355cfc4ed1df0e7` after copying.
6. Extend coverage assertions in `v2_catalog.rs` and the host's runtime/registry tests. Its pinned `items.len() == 11` becomes 12 only if CSV is the sole addition on that base. Assert canonical/alias dispatch, `commands("csv-to-excel") == &["index"]`, schema validity, exactly one Rust command on both declared platforms, unknown command rejection, and both verification states still `not-run`. Reconcile counts against the actual integration branch; do not blindly change unrelated totals.
7. Rebuild the host separately on macOS and Windows. Install the canonical native ID through the host installation flow/CLI, which writes the bundled package, Native V2 marker and installed database entry. A manifest, marker or path dependency alone cannot install executable Rust or establish registry coverage. Keep the V1 package installed for fallback. Test that uninstall invalidates active native sessions. The harness installation stub is not proof of real installation.

## Wire and native boundaries

The production `handle` uses beta's actual signature and SDK `view`/`unchanged`. It handles `extension:run-command`, `extension:invoke-action`, `extension:search-text-changed`, `extension:refresh-session`, `extension:load-more`, `extension:dispose-session`. Launch values are at `argumentValues` and preferences at `payload.preferences`; source declares none and they are intentionally unused. Search, refresh and load-more return unchanged without changing state or issuing IDs. No List/search filtering or arbitrary action shortcuts are advertised.

The Form uses stable field ID **`files`** and real `Form.FilePicker`, `Action` wire kinds `submit-form`/`action`, full-map submit `formValues.files`, and no per-field callbacks. There is no `formValues.value` callback in the V1 source. Error/cancellation replies omit the file value and preserve the renderer's latest draft; initial defaults are renderer-owned and submitted drafts are explicitly restored for Back/native selection. Result Detail sets `root.props.backActionId`. The `Choose CSV File` action invokes the existing `DialogExt::file().add_filter(...).pick_file(...)` callback API, obtains a local Unicode path and handles cancellation. Bare filename submissions are explicit errors; the adapter never guesses a user's file from app cwd. The dialog is a concrete native picker, not a clipboard bridge or JS converter. Its API source and beta's locked versions are pinned in fixtures.

All write responses require an object with `ok:true`, `error` absent/null, and `success` absent/true; null, booleans, contradictory/malformed objects and transport errors fail with sanitized messages. Native publication returns that object after sync/no-clobber persist succeeds. Unconfirmed writes retain Base64 computed bytes and destination, with a new single-use retry capability. Retry cannot read new source data, accept forged output, automatically choose a new name or overwrite a destination. Back discards pending state and invalidates its actions. There is no SDK storage, clipboard read/write, selected-text, paste, open, window hide, HUD or toast effect in this command.

For future effects, actual routes are `clipboard:read [] -> string`, `core:platform:selected-text [] -> string`, SDK copy uses `clipboard:write [text]`, SDK open uses `shell:open [target]`, prepared clipboard paste uses `core:platform:paste []`, and `core:platform:paste-text [text]` already writes/hides/restores focus/waits/pastes. They are not substitutes for source XLSX file output. Pinned `window:hide []` hides then returns null; using it would require a concrete host object-ack fix, not adapter normalization of arbitrary null. This port avoids that unsupported effect. No true HUD/toast/pop-to-root SDK API is asserted; success is a Detail and error is a Form description.

## Harness scope and outstanding QA

`harness/*.in` materializes a standalone crate under `.checks/harness`, including byte-exact beta `common.rs`, `v2_sdk.rs`, `native.rs`, and byte-exact production `adapter.rs`. Tauri AppHandle/Emitter, native dialog callback, router, registration dispatch and installed registry are explicitly labeled stubs. The SDK/common tests and real NativeSessions implementation execute; the full application, plugin implementation, Tauri webview, installer/catalog and platform dispatcher do not. `#[cfg(not(test))]` on the harness NativeSessions module avoids importing unrelated pinned application test suites when Clippy checks the harness lib test target; its normal library/integration target compiles NativeSessions unchanged. Nested manifests are templates because Cargo packaging skips nested packages; pinned host/crate manifests use `.fixture` filenames for the same reason, with their original bytes unchanged.

Real macOS/Windows UI, absolute-path picker behavior, dialog cancellation/focus, permissions, file operations on user directories, Excel opening, Unicode paths, collision behavior under races, actual clipboard formats, paste and window behavior remain **not-run**. The source has no clipboard action; no clipboard or Excel QA is implied by byte comparisons. Keep both schema verification states `not-run` until separately authorized desktop QA records evidence. Host registration/rebuild is also not-run.
