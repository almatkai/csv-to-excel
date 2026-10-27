// Executes unchanged own TS command against actual locked conversion libraries.
// Raycast UI calls are recording stubs. Files live only in a disposable test dir.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict'),cp=require('node:child_process');
const root=path.resolve(__dirname,'..'),lib=path.join(root,'fixtures/libraries/node_modules');
const ts=require(path.join(lib,'typescript/lib/typescript.js'));
const converter=require(path.join(lib,'@aternus/csv-to-xlsx'));

fs.mkdirSync(path.join(root,'.checks'),{recursive:true});const outdir=fs.mkdtempSync(path.join(root,'.checks/oracle-'));
const logs=[];
const raycast={Form:{FilePicker:'FilePicker'},ActionPanel:'ActionPanel',Action:{SubmitForm:'SubmitForm'},Icon:{Hammer:'hammer'},Toast:{Style:{Failure:'failure'}},showToast:(...x)=>logs.push(['toast',...x]),showHUD:async(...x)=>logs.push(['hud',...x]),popToRoot:async()=>logs.push(['pop'])};raycast.Form=Object.assign('Form',raycast.Form);
// React JSX createElement is only a recording UI boundary, source handler is unchanged.
const React={createElement:(type,props,...children)=>({type,props:{...props,children}})};
let source=fs.readFileSync(path.join(root,'fixtures/own/src/index.tsx'),'utf8');
const js=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.React,esModuleInterop:true}}).outputText;
const moduleObject={exports:{}};
vm.runInNewContext(js,{require:(id)=>id==='@raycast/api'?raycast:id==='@aternus/csv-to-xlsx'?converter:require(id),exports:moduleObject.exports,module:moduleObject,React,Error,console:{error(){}},Math:Object.assign(Object.create(Math),{random:()=>0.5})});
const command=moduleObject.exports.default();
const submit=command.props.actions.props.children[0].props.onSubmit;
const cases=[
 ['empty',''],['header','a,b\n'],['single','a\n1'],['basic','a,b\n1,2\n'],['quoted','a,b\r\n"a,b","a""b"\r\n'],
 ['newline','a,b\n"line\nline","CR\r\nLF"\n'],['bare-cr','a,b\r1,2\r'],['trim',' a , b \n 1 , 2 \t\n'],['quoted-trim','a,b\n " 1 " , "2 " \n'],
 ['blank','\n'],['blank-row','a,b\n\n'],['one-blank-row','a\n\n'],['extra-column','a,b\n1,2,3'],['missing-column','a,b\n1'],
 ['trailing','a,b,\n1,2,\n'],['duplicated','a,a,b\n1,2,3'],['numeric-headers','b,10,2,01,0,4294967295\na,b,c,d,e,f'],
 ['prototype','__proto__,constructor,toString\na,b,c'],['proto-only','__proto__\nx\ny'],['empty-header',',a,\n1,2,3'],
 ['semicolon','a;b\n1;2'],['tabs','a\tb\n1\t2'],['bom','\ufeffa,b\n1,2'],['bom-quoted','\ufeff"a",b\n1,2'],
 ['unicode','a,б\nҚазақ 😀,日本'],['formulas','a,b,c,d\n=A1,+1,-1,@SUM(A1)'],['xml','<h>,b\n"<&>\'\"\"",_x000a_'],
 ['controls','a,b\n"\u0000\u0001\u000b\u000c\u001f\ufffe\uffff", "\t\n\r"'],['space','a,b\n" ","\u00a0\ufeff"'],
 ['mixed-lines','a,b\r\n1,2\n3,4\r\n'],['mixed-lines2','a,b\n1,2\r3,4'],['quoted-bad','a\n"x"z'],['quoted-after-space','a\n"x" z'],['unclosed','a\n"x'],['opening','a\nx"y'],
 ['header-opening','a"b,c'],['empty-quoted','a,b\n"",""'],['long-cell','a\n'+ 'x'.repeat(32768)],['max-cell','a\n'+ '😀'.repeat(16383)+'x'],
 ['whitespace-only',' \t'],['vertical-tab','a\n\u000bx\u000b'],['formfeed','a\n\fx\f'],['unicode-trim','a\n\u00a0x\u00a0'],['bom-only','\ufeff'],
 ['nul-after-quote','a\n"x"\0'],['quoted-crlf-error','a,b\r\n"x\r\ny"z,2'],
];
cases.push(['invalid-utf8',Buffer.from([97,44,98,10,0xf0,0x80,0x80,0x80,44,0xff])],['truncated-utf8',Buffer.from([97,10,0xe2,0x82])]);
// Deterministic generated valid records and malformed mutations exercise real libraries.
let seed=0x13579;const rand=n=>(seed=(Math.imul(seed,1664525)+1013904223)>>>0)%n;
const values=['','  x  ','a,b','a"b','\r\n','\t','日本😀','\ufeff','<>&\'"','=1+2','+12','-3','@A1','0','\0'];
for(let i=0;i<160;i++){const width=1+rand(7),height=1+rand(5),eol=['\n','\r\n','\r'][rand(3)];const headers=Array.from({length:width},(_,j)=>i%9===0?String(rand(5)):i%13===0?'__proto__':'h'+j);const rows=[headers,...Array.from({length:height},()=>Array.from({length:width},()=>values[rand(values.length)]))];const csv=rows.map(r=>r.map(v=>'"'+v.replaceAll('"','""')+'"').join(',')).join(eol)+(rand(2)?eol:'');cases.push(['generated-'+i,csv]);}
for(let i=0;i<80;i++){const chars=['a',',','"','\r','\n',' ','\t','\0',';'];cases.push(['malformed-'+i,Array.from({length:1+rand(45)},()=>chars[rand(chars.length)]).join('')]);}
(async()=>{
 const expected=[];
 for(let i=0;i<cases.length;i++){
  const [name,csv]=cases[i],d=path.join(outdir,String(i));fs.mkdirSync(d,{recursive:true});const input=path.join(d,'source.csv'),output=path.join(d,'source.xlsx');fs.writeFileSync(input,csv);logs.length=0;await submit({files:[input]});
  let result;if(fs.existsSync(output)){const bytes=fs.readFileSync(output);assert.deepEqual(logs,[['hud',`✅ Converted to the «${d}» directory`],['pop']]);result={bytes:[...bytes]};}else{assert.equal(logs[0]?.[0],'toast');result={error:logs[0]?.[3]};}
  expected.push({name,input:[...Buffer.from(csv)],...result});
 }
 // Own source validations, exact extension case, first-only selection and one-shot collision.
 for(const [values,message] of [[{files:[]},'You need choose a CSV file'],[{files:['a.CSV']},'Failed to convert this CSV file'],[{files:[path.join(outdir,'missing.csv')]},'Failed to convert this CSV file']]){logs.length=0;await submit(values);assert.equal(logs[0][2],message);}
 const c=path.join(outdir,'collision');fs.mkdirSync(c,{recursive:true});fs.writeFileSync(c+'/source.csv','a\nx');fs.writeFileSync(c+'/source.xlsx','unchanged');logs.length=0;await submit({files:[c+'/source.csv','ignored']});assert(fs.existsSync(c+'/source_500.xlsx'));assert.equal(fs.readFileSync(c+'/source.xlsx','utf8'),'unchanged');logs.length=0;await submit({files:[c+'/source.csv']});assert.match(logs[0][3],/already exists/);
 const executable=process.argv[2]||path.join(root,'target/debug/examples/oracle-driver');
 const rust=cp.spawnSync(executable,[],{input:expected.map(c=>JSON.stringify({bytes:c.input})).join('\n')+'\n',maxBuffer:128*1024*1024,encoding:'utf8'});assert.equal(rust.status,0,rust.stderr);const actual=rust.stdout.trim().split('\n').map(JSON.parse);
 let failures=[];for(let i=0;i<expected.length;i++){const e=expected[i],a=actual[i];try{assert.deepEqual(a,e.error!==undefined?{error:e.error}:{bytes:e.bytes});}catch{failures.push({name:e.name,expected:e.error||Buffer.from(e.bytes).length,actual:a.error||a.bytes?.length,firstDiff:e.bytes&&a.bytes?e.bytes.findIndex((v,i)=>v!==a.bytes[i]):undefined});}}
 fs.writeFileSync(path.join(root,'.checks/oracle-results.json'),JSON.stringify({cases:expected.length,failures},null,2));
 // Compact golden corpus keeps exact own source outputs for offline Rust tests.
 const golden=path.join(root,'fixtures/golden');fs.mkdirSync(golden,{recursive:true});for(let i=0;i<expected.length;i++){let e=expected[i];fs.writeFileSync(path.join(golden,e.name+'.csv'),Buffer.from(e.input));fs.writeFileSync(path.join(golden,e.name+(e.error!==undefined?'.error':'.xlsx')),e.error!==undefined?e.error:Buffer.from(e.bytes));}
 console.log(JSON.stringify({cases:expected.length,sourceBoundaryChecks:6,failures},null,2));if(failures.length)process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1});
