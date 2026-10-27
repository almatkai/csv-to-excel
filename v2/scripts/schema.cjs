const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),Ajv2020=require(path.join(root,'fixtures/libraries/node_modules/ajv/dist/2020')).default;
const schema=JSON.parse(fs.readFileSync(path.join(root,'fixtures/beta/schemas/extension-v2.schema.json'))),manifest=JSON.parse(fs.readFileSync(path.join(root,'manifest.json')));
const validate=new Ajv2020({allErrors:true,strict:true}).compile(schema);assert(validate(manifest),JSON.stringify(validate.errors));
let negatives=0;function reject(change){const m=structuredClone(manifest);change(m);assert(!validate(m),'Invalid manifest accepted');negatives++;}
for(const k of schema.required)reject(m=>delete m[k]);
reject(m=>m.schemaVersion=1);reject(m=>m.author='unsupported');reject(m=>m.runtime='javascript');reject(m=>m.id='raycast.csv-to-excel');reject(m=>m.slug='../escape');reject(m=>m.icon='assets/command-icon.png');reject(m=>m.commands=[]);reject(m=>m.commands[0].runtime='bun');reject(m=>m.commands[0].platforms=[]);reject(m=>m.commands[0].platforms=['macos']);reject(m=>m.commands[0].platforms=['macOS','macOS']);reject(m=>m.replaces=[]);reject(m=>m.replaces=['com.tezbar.csv-to-excel']);reject(m=>m.verification=[m.verification[0]]);reject(m=>m.verification[0].state='tested');reject(m=>m.preferences={});
assert.equal(manifest.commands[0].name,'index');assert.equal(manifest.id,'com.tezbar.native.csv-to-excel');assert.deepEqual(manifest.verification.map(v=>v.state),['not-run','not-run']);
console.log(JSON.stringify({fullPinnedSchema:true,positive:1,negative:negatives}));
