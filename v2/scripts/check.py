#!/usr/bin/env python3
"""Run non-desktop gates. Never stages, commits, pushes or changes a host profile."""
from pathlib import Path
import subprocess,json,sys,tarfile,tempfile
v=Path(__file__).resolve().parent.parent
out=v/'.checks';out.mkdir(exist_ok=True)
results=json.loads((out/"gates.json").read_text()) if "--extracted-only" in sys.argv and (out/"gates.json").exists() else []
def run(name,args,cwd=v):
    print(name,flush=True)
    p=subprocess.run(args,cwd=cwd,capture_output=True,text=True)
    (out/(name+'.log')).write_text(p.stdout+p.stderr)
    results[:]=[r for r in results if r['gate']!=name]
    results.append({'gate':name,'exit':p.returncode})
    (out/'gates.json').write_text(json.dumps(results,indent=2)+'\n')
    if p.returncode:print((p.stdout+p.stderr)[-6000:]);sys.exit(p.returncode)
if '--extracted-only' not in sys.argv:
    run('materialize',['python3','scripts/materialize-harness.py'])
    run('format-engine',['cargo','fmt','--check'])
    run('format-adapter',['rustfmt','--edition','2021','--config','skip_children=true','--check','adapter.rs','harness/lib.rs.in','harness/contracts.rs.in'])
    for tc in ['stable','1.85.0']:
        for project,manifest in [('engine','Cargo.toml'),('adapter','.checks/harness/Cargo.toml')]:
            run(tc+'-'+project+'-test',['cargo','+'+tc,'test','--locked','--offline','--manifest-path',manifest])
            if project=='adapter':run(tc+'-sdk-common-test',['cargo','+'+tc,'test','--locked','--offline','--manifest-path',manifest,'--lib'])
            run(tc+'-'+project+'-clippy',['cargo','+'+tc,'clippy','--locked','--offline','--manifest-path',manifest,'--all-targets','--','-D','warnings'])
            for command in ['check','clippy']:
                args=['cargo','+'+tc,command,'--locked','--offline','--manifest-path',manifest,'--all-targets','--target','x86_64-pc-windows-gnu']
                if command=='clippy':args+=['--','-D','warnings']
                run(tc+'-'+project+'-windows-'+command,args)
    run('build-oracle-driver',['cargo','build','--locked','--offline','--example','oracle-driver'])
    run('source-oracle',['node','scripts/oracle.cjs'])
    run('renderer',['node','scripts/renderer.cjs'])
    run('schema',['node','scripts/schema.cjs'])
    run('audit',['python3','scripts/audit.py']+(['--live'] if '--live' in sys.argv else []))
    run('package',['cargo','package','--allow-dirty','--locked','--offline'])
else:
    run('package',['cargo','package','--allow-dirty','--locked','--offline'])
crate=v/'target/package/tezbar-csv-to-excel-0.1.0.crate'
extract=Path(tempfile.mkdtemp(prefix='extracted-',dir=out))
with tarfile.open(crate) as archive:
    for f in archive.getmembers():assert (extract/f.name).resolve().is_relative_to(extract.resolve()),f.name
    assert all(f.isfile() or f.isdir() for f in archive.getmembers()),'Unexpected archive link'
    archive.extractall(extract)
e=extract/'tezbar-csv-to-excel-0.1.0'
required=list((v/'fixtures').rglob('*'))+[v/'adapter.rs',v/'harness/Cargo.toml.in',v/'harness/Cargo.lock.in',v/'original-icon.png']
for p in required:
    if p.is_file():assert (e/p.relative_to(v)).read_bytes()==p.read_bytes(),str(p)
assert not (e/'validation').exists() and not (e/'.checks').exists(),'Test working dirs leaked into package'
run('extracted-audit',['python3','scripts/audit.py'],e)
run('extracted-engine',['cargo','test','--locked','--offline'],e)
run('extracted-materialize',['python3','scripts/materialize-harness.py'],e)
run('extracted-adapter',['cargo','test','--locked','--offline','--manifest-path','.checks/harness/Cargo.toml'],e)
run('extracted-sdk',['cargo','test','--locked','--offline','--manifest-path','.checks/harness/Cargo.toml','--lib'],e)
run('extracted-build-oracle',['cargo','build','--locked','--offline','--example','oracle-driver'],e)
run('extracted-oracle',['node','scripts/oracle.cjs'],e)
run('extracted-renderer',['node','scripts/renderer.cjs'],e)
run('extracted-schema',['node','scripts/schema.cjs'],e)
print(json.dumps({'gates':len(results),'allPassed':True,'crateBytes':crate.stat().st_size,'requiredFilesByteExact':sum(p.is_file() for p in required),'desktopQA':'not-run'}))
