#!/usr/bin/env python3
"""Disposable V1 npm checks. Root source and root lock are never changed."""
from pathlib import Path
import tempfile,shutil,subprocess,hashlib,json
v=Path(__file__).resolve().parent.parent
checks=v/'.checks';checks.mkdir(exist_ok=True)
d=Path(tempfile.mkdtemp(prefix='v1-',dir=checks))
for p in (v/'fixtures/own').rglob('*'):
 if p.is_file():
  dest=d/p.relative_to(v/'fixtures/own');dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest)
lock=(d/'package-lock.json').read_bytes()
for args in [['npm','ci','--ignore-scripts','--no-audit','--no-fund','--allow-remote=all'],['npm','test'],['npm','run','typecheck']]:subprocess.run(args,cwd=d,check=True)
assert (d/'package-lock.json').read_bytes()==lock
for p in (v/'fixtures/own').rglob('*'):
 if p.is_file():assert (d/p.relative_to(v/'fixtures/own')).read_bytes()==p.read_bytes()
print(json.dumps({'v1Smoke':1,'typecheck':'pass','copiedSourceAndLock':'byte-exact','directory':str(d)}))
