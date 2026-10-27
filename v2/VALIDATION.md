# Reproducible validation and limits

Validated without opening any native dialog, clipboard, Excel, browser, user file, app window, focus or paste operation. Production file I/O is exercised only inside disposable directories. All SDK routers, Tauri/dialog callbacks, installation/registration boundaries and renderer window hooks are explicit stubs. Both OS verification entries remain **not-run**. Host registration, installation and rebuild are not performed by these tests.

## Commands

From this repository's `v2/` directory:

```sh
python3 scripts/check.py --live
```

`--live` additionally compares every own and beta fixture to their recorded Git pins and audits Git visibility plus an empty staging index. For an extracted Cargo package, omit `--live`; it does not need the original Git directories. Rust crates must first be available in Cargo's local cache; all gates themselves use `--locked --offline`. Node validation is self-contained in byte-exact library fixtures and does not install packages or use network. `check.py` materializes a nested harness from `.in` templates, runs tools, packages the crate, extracts it into a fresh `.checks/extracted-*` directory, and reruns core/adapter/SDK/oracle/renderer/schema checks from that extracted artifact. Test working dirs and build products are excluded from the package. Pinned Cargo manifests are named `Cargo.toml.fixture` to retain their exact bytes while preventing Cargo from silently excluding their enclosing fixtures as nested packages.

The separate disposable V1 checks can be reproduced without modifying any root source or lock:

```sh
python3 scripts/v1-check.py
```

This copies the pinned own source and lock to a new directory under `.checks/`, then runs `npm ci --ignore-scripts --no-audit --no-fund --allow-remote=all`, `npm test` and `npm run typecheck`. Only this preparation step fetches locked npm dependencies; the lock includes the explicit SheetJS 0.20.1 tarball. The option allows that lock's remote tarball in npm 12; no global npm configuration is changed.

Useful individual checks:

```sh
cargo fmt --check
rustfmt --edition 2021 --config skip_children=true --check adapter.rs harness/lib.rs.in harness/contracts.rs.in
python3 scripts/materialize-harness.py
cargo test --locked --offline
cargo test --locked --offline --manifest-path .checks/harness/Cargo.toml
cargo test --locked --offline --manifest-path .checks/harness/Cargo.toml --lib
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --manifest-path .checks/harness/Cargo.toml --all-targets -- -D warnings
cargo build --locked --offline --example oracle-driver
node scripts/oracle.cjs
node scripts/renderer.cjs
node scripts/schema.cjs
python3 scripts/audit.py --live
```

The full runner repeats test/Clippy with `+stable` and `+1.85.0` and Windows `--target x86_64-pc-windows-gnu` check/Clippy on both. It formats added Rust, not pinned beta fixtures. The all-target Clippy gates compile the engine, example, tests, production adapter, normal unchanged NativeSessions path and harness lib/test targets. Integration tests compile the real common/SDK/NativeSessions sources; SDK/common unit tests separately execute the unchanged SDK/common implementation. The pinned full application test suite is not claimed to run.

## Recorded results

Environment: macOS ARM64, Rust stable **1.98.0** (`88d9e12ae`), Rust **1.85.0** (`4d91de4e4`), Node **26.7.0**, Python **3.9.6**. Windows GNU is a cross compile/Clippy target, not a Windows desktop run.

- **288** own TypeScript command executions through the real locked @aternus/csv-to-xlsx 2.1.5, csv-parse 5.4.0, SheetJS 0.20.1 and filesystem dependencies, compared against Rust. Success compares the **entire XLSX archive bytes**, including XML, ZIP order, CRC, zero dates, empty cells and whitespace attributes. Failures compare the exact parser message. Six source boundary checks cover empty/wrong/missing selections, first-file behavior and one-shot destination collisions. Deterministic generated records and malformed inputs supplement quoted/escaped cells, embedded CR/LF/CRLF, mixed endings, BOM/Unicode, invalid UTF-8, blank/malformed rows, duplicate/numeric/prototype headers, trailing separators, semicolon/tab inputs, formula-like text, controls/XML and UTF-16 limits. The script transpiles the unchanged own TS file with the pinned compiler; only Raycast UI/effect functions and randomness are recording boundaries. There is no handwritten JS converter oracle.
- **3 engine tests**: exact 288-case golden corpus and explicit input/record/cell/column/text/output resource-limit failures. No truncation or XLSX format substitution.
- **14 adapter contract tests**: malformed/forged/cross-session/stale/discarded capabilities, bound output and deliberate retry, strict positive/negative/contradictory acknowledgements and sanitized transport errors, saved drafts and Back, one-shot collisions, no-clobber native publication, native dialog callback stub cancellation/path/headless error, actual NativeSessions installation/lifecycle/capacity/clear/uninstall routing, no desktop routes, and SDK unknown-key/atomic/JSON/typed-corruption behavior. Storage tests check the SDK contract only; this extension creates no store or migration.
- **5 unchanged SDK/common unit tests**, including concurrent atomic storage, corruption rejection, browser encoding and actual copy/open route names under recording routers.
- **25 renderer assertions** using actual pinned Form/List/Detail parsing, reconciliation and Surface action routing with real React **18.3.1** and react-test-renderer. Four views come from the compiled production adapter. Checks cover FilePicker full paths and basename fallback, submit full-map values, queued draft preservation on repeated error replies, Detail Back capability, primary action and saved draft restoration. Markdown painting, UI primitives, unrelated Grid/color functions and desktop hooks are stubs. This is not visual or desktop QA.
- Full pinned schema validator: **1 accepted manifest + 26 rejected variants** using AJV 2020-12. Root author, old runtime/ID, malformed commands/platforms/icon/replacements and verification are rejected.
- Stable and 1.85 tests, formatting, strict all-target Clippy, Windows GNU check/Clippy: passed for engine and concrete adapter harness. Windows plugin/UI implementation and application linking are not covered.
- Cargo packaging and **extracted offline** core/adapter/SDK/source oracle/renderer/schema checks pass; required fixtures, adapter, nested manifest/lock templates and icon are byte-exact inside the archive. No `.checks`, `validation` or `target` paths are shipped.
- Disposable V1 smoke test: **1 passed**; original V1 typecheck passed with the unchanged root lock/source. No root dependency installation is performed.
- Exact icon: **26,498 bytes**, SHA-256 **`7e091606ff926192548547eae0bd3743928929c3cb01124c2355cfc4ed1df0e7`**. Immutable fixtures and actual license text hashes are checked by `scripts/audit.py`. Root dist/node_modules ignores are explicitly overridden only for intentional library fixtures in `v2/.gitignore`, and Git visibility is audited without staging. Root LICENSE formatting conflict and original extension-specific license gap are recorded in NOTICE rather than inferred away.

Initial XERJ prefix `ax-35f8c1cf566a` indexed all three own source files, 51 records, and exited 3 (`completed-with-junk`, two skipped non-source files). Exact source/lock and beta reads plus `rg` supplied exhaustive evidence. Final index status is recorded in provenance; any incomplete refresh is not evidence that missing code is absent. No XERJ service restart or schema change is performed.

## Remaining integration and desktop gaps

Host module/dependency/runtime/catalog/icon installation, coverage assertions, real installer/profile behavior and rebuild are pending. Native dialog callbacks are compiled against a labeled API stub, not the real Tauri plugin binary; its beta-locked API source is pinned. macOS and Windows picker behavior, permissions, Unicode paths, filesystem races/slow filesystems, session disposal while a native dialog remains open, Excel interoperability, UI layout, focus/window/paste/clipboard formats and actual OS effects remain **not-run**. The source has no clipboard/paste/open action, so no rich-clipboard representation is replaced with text or a path. Source HUD/toast/pop-to-root is a documented Form/Detail adaptation. No desktop QA success is claimed from headless or cross compilation results.
