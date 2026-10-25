#!/usr/bin/env python3
"""Materialize nested crate manifests omitted by cargo package; entirely self-contained."""
from pathlib import Path
import shutil
v=Path(__file__).resolve().parent.parent
out=v/'.checks/harness'
(out/'src/native_core/extensions').mkdir(parents=True,exist_ok=True)
(out/'tests').mkdir(exist_ok=True)
for name in ['Cargo.toml','Cargo.lock']:
    source=v/'harness'/(name+'.in')
    if source.exists():shutil.copyfile(source,out/name)
shutil.copyfile(v/'harness/lib.rs.in',out/'src/lib.rs')
for p in ['common.rs','extensions/v2_sdk.rs','extensions/native.rs']:
    shutil.copyfile(v/'fixtures/beta/src-tauri/src/native_core'/p,out/'src/native_core'/p)
shutil.copyfile(v/'adapter.rs',out/'src/native_core/extensions/v2_csv_to_excel.rs')
shutil.copyfile(v/'harness/contracts.rs.in',out/'tests/contracts.rs')
print(out/'Cargo.toml')
