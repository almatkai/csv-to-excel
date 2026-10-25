#!/usr/bin/env python3
"""Immutable fixture/provenance audit, optionally against read-only live Git pins."""
from pathlib import Path
import hashlib,json,subprocess,sys
v=Path(__file__).resolve().parent.parent
sha=lambda b:hashlib.sha256(b).hexdigest()
expected=json.loads((v/'fixtures/hashes.json').read_text())
for p,h in expected.items():assert sha((v/p).read_bytes())==h,p
icon=(v/'original-icon.png').read_bytes();assert len(icon)==26498 and sha(icon)=='7e091606ff926192548547eae0bd3743928929c3cb01124c2355cfc4ed1df0e7'
assert (v/'fixtures/original/package.json').exists()
for p in (v/'fixtures').rglob('Cargo.toml'):raise AssertionError('Nested manifest must use .fixture: '+str(p))
for pkg in json.loads((v/'licenses/inventory.json').read_text())['npmFixtures']:
 assert pkg['files'],'No actual license text for '+pkg['name']
 for f in pkg['files']:assert sha((v/f['path']).read_bytes())==f['sha256'],f['path']
if '--live' in sys.argv:
 repo=v.parent;source='718b9ca0b42e00be968491064310ac6e5bc76a93';beta='/Users/almat/Desktop/code/Tezbar APP/Tezbar';pin='fd300e276356587726566026f072563eb9c34cb1'
 assert subprocess.check_output(['git','-C',str(repo),'rev-parse','--show-toplevel'],text=True).strip()==str(repo)
 assert subprocess.check_output(['git','-C',str(repo),'branch','--show-current'],text=True).strip()=='feat/native-v2-rust'
 for p in (v/'fixtures/own').rglob('*'):
  if p.is_file():
   rel=str(p.relative_to(v/'fixtures/own'));assert p.read_bytes()==subprocess.check_output(['git','-C',str(repo),'show',source+':'+rel]),rel
   if rel!='README.md':assert (repo/rel).read_bytes()==p.read_bytes(),rel
 for p in (v/'fixtures/beta').rglob('*'):
  if p.is_file():
   rel=str(p.relative_to(v/'fixtures/beta')).removesuffix('.fixture');assert p.read_bytes()==subprocess.check_output(['git','-C',beta,'show',pin+':'+rel]),rel
 visible=set(subprocess.check_output(['git','-C',str(repo),'ls-files','--cached','--others','--exclude-standard','--','v2'],text=True).splitlines())
 missing=['v2/'+p for p in expected if 'v2/'+p not in visible];assert not missing,missing
 subprocess.run(['git','-C',str(repo),'diff','--cached','--exit-code'],check=True,capture_output=True)
 print(json.dumps({'fixtures':len(expected),'liveSourceAndBeta':'byte-exact','gitVisible':True,'index':'empty'}))
else:print(json.dumps({'fixtures':len(expected),'immutableHashes':'pass','iconBytes':len(icon)}))
