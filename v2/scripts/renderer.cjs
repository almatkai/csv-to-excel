// Executes pinned renderer implementation with real React/test renderer.
// Markdown painting, UI primitives, desktop hooks/window and unrelated Grid are headless stubs.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'..'),lib=path.join(root,'fixtures/libraries/node_modules');
const ts=require(path.join(lib,'typescript/lib/typescript.js')),React=require(path.join(lib,'react')),TestRenderer=require(path.join(lib,'react-test-renderer'));const {act}=TestRenderer;
const basedir=path.join(root,'fixtures/beta/src/renderer/src/raycast-api'),cache=new Map();
const desktop=[];const windowStub={addEventListener(){},removeEventListener(){},clearTimeout,setTimeout,tezbar:{setSuppressBlurHide:async(value)=>{desktop.push(value);}}};
const primitives={};for(let key of ['Hint','HintBar','Kbd','ViewHeader'])primitives[key]=({children})=>React.createElement('div',null,children);primitives.cx=(...s)=>s.filter(Boolean).join(' ');
const nav=React.createContext({push(){},pop(){}}),hooks={NavigationContext:nav,setRuntimeContext(){},environment:{},Clipboard:{},getPreferenceValues(){return {}},open(){throw Error('No OS open allowed')},showHUD(){throw Error('No HUD allowed')},showToast(){throw Error('No toast allowed')}};
function load(name){if(cache.has(name))return cache.get(name);let source=fs.readFileSync(path.join(basedir,name+'.tsx'),'utf8');if(name==='form-runtime')source+='\nexport const __test = {collectFormItems};';if(name==='list-runtime')source+='\nexport const __test = {parseListRows};';if(name==='detail-runtime')source+='\nexport const __test = {markdownFromNode};';const m={exports:{}};cache.set(name,m.exports);
 const js=ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText;
 const customRequire=id=>{if(id==='react')return React;if(id==='react/jsx-runtime')return require(path.join(lib,'react/jsx-runtime'));if(id.endsWith('/primitives'))return primitives;if(id.endsWith('/Markdown'))return {Markdown:({value,children,markdown})=>React.createElement('markdown',{value,markdown},children)};if(id.endsWith('/colorConverter'))return new Proxy({},{get:()=>()=>{throw Error('Unrelated color converter')}});if(id==='./hooks')return hooks;if(id==='./grid-runtime')return {GridRuntime:()=>null};if(id.startsWith('./'))return load(id.slice(2));throw Error('Unexpected import '+id)};
 vm.runInNewContext(js,{require:customRequire,exports:m.exports,module:m,window:windowStub,console,setTimeout,clearTimeout,Date,Map,Set,URL},{filename:name+'.tsx'});cache.set(name,m.exports);return m.exports;
}
const form=load('form-runtime'),surface=load('index'),list=load('list-runtime'),detail=load('detail-runtime');
let assertions=0;function equal(a,b){assert.deepEqual(JSON.parse(JSON.stringify(a)),JSON.parse(JSON.stringify(b)));assertions++;}
const field=(value)=>({type:'Form.FilePicker',props:{id:'files',title:'CSV File',allowMultipleSelection:false,value}});
const rootForm=value=>({type:'Form',props:{},children:[field(value)]});
const parsed=form.__test.collectFormItems(rootForm([]));equal(parsed[0].field.id,'files');equal(parsed[0].field.allowMultipleSelection,false);equal(parsed[0].field.type,'Form.FilePicker');
const fields=parsed.map(x=>x.field);equal(form.reconcileFormValues(fields),{files:[]});equal(form.reconcileFormValues(fields,{files:['/newer.csv']},fields),{files:['/newer.csv']});
const changed=form.__test.collectFormItems(rootForm(['/chosen.csv'])).map(x=>x.field);equal(form.reconcileFormValues(changed,{files:['/newer.csv']},fields),{files:['/chosen.csv']});
const omitted=form.__test.collectFormItems(rootForm(undefined)).map(x=>x.field);equal(form.reconcileFormValues(omitted,{files:['/queued.csv']},changed),{files:['/queued.csv']});
const controlled=[{...fields[0],actionId:'files:controlled'}];equal(form.reconcileFormValues(controlled,{files:['/draft.csv']},controlled),{files:[]});
const rows=list.__test.parseListRows({type:'List',children:[{type:'List.Item',props:{id:'one',title:'A',actionIds:['cap']}}]});equal(rows[0].id,'one');equal(rows[0].actionIds,['cap']);
equal(detail.__test.markdownFromNode({type:'Detail',props:{markdown:'plain path'}}),'plain path');
const events=[];let outerBack=0;const props={sessionId:'native-extension:test',extensionId:'com.tezbar.native.csv-to-excel',commandName:'index',title:'Convert CSV to Excel',root:rootForm([]),actions:[{id:'submit:1',title:'Convert',kind:'submit-form'},{id:'choose:1',title:'Choose CSV File',kind:'action'}],onBack:()=>outerBack++,onSearchTextChanged:()=>{},onLoadMore:()=>{},onInvokeAction:(id,values)=>events.push([id,values])};
let renderer;act(()=>{renderer=TestRenderer.create(React.createElement(surface.ExtensionRuntimeSurface,props));});
let file=renderer.root.findByProps({type:'file'});equal(file.props.multiple,false);
act(()=>file.props.onChange({target:{files:[{path:'/test/source.csv',name:'source.csv'}]}}));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(events.pop(),['submit:1',{files:['/test/source.csv']}]);
// A repeated/default form response must not erase a newer local draft.
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,root:rootForm([]),actions:[{id:'submit:2',title:'Convert',kind:'submit-form'}]})));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(events.pop(),['submit:2',{files:['/test/source.csv']}]);
// Stock browser File without .path proves the filename-only beta limitation.
act(()=>renderer.root.findByProps({type:'file'}).props.onChange({target:{files:[{name:'source.csv'}]}}));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(events.pop(),['submit:2',{files:['source.csv']}]);
// Real surface delegates a result's Back to its server capability and Enter to primary action.
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,root:{type:'Detail',props:{markdown:'Done',backActionId:'back:1'}},actions:[{id:'retry:1',title:'Retry Writing Computed XLSX',kind:'action'}]})));
const back=renderer.root.findAllByType('button').find(b=>b.props.children==='Back');act(()=>back.props.onClick());equal(events.pop(),['back:1',undefined]);equal(outerBack,0);
const enter=renderer.root.findAllByType('button').find(b=>b.props.children==='Enter');act(()=>enter.props.onClick());equal(events.pop(),['retry:1',undefined]);
// Back remounts the real form with the saved server draft.
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...props,root:rootForm(['/saved.csv'])})));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(events.pop(),['submit:1',{files:['/saved.csv']}]);
act(()=>renderer.unmount());equal(desktop,[]);
console.log(JSON.stringify({rendererAssertions:assertions,actualReact:'18.3.1',desktopEffects:0}));
// Feed the compiled production adapter's actual wire views through the real renderer.
const wire=JSON.parse(fs.readFileSync(path.join(root,'.checks/adapter-wire.json')));
const actualEvents=[];const actualProps={...props,onInvokeAction:(...a)=>actualEvents.push(a)};
act(()=>renderer=TestRenderer.create(React.createElement(surface.ExtensionRuntimeSurface,{...actualProps,root:wire.initial.root,actions:wire.initial.actions})));
equal(form.__test.collectFormItems(wire.initial.root)[0].field.id,'files');
act(()=>renderer.root.findByProps({type:'file'}).props.onChange({target:{files:[{path:'/test/source.CSV',name:'source.CSV'}]}}));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(actualEvents.pop(),[wire.initial.actions[0].id,{files:['/test/source.CSV']}]);
act(()=>renderer.root.findByProps({type:'file'}).props.onChange({target:{files:[{path:'/newer/queued.csv',name:'queued.csv'}]}}));
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...actualProps,root:wire.error.root,actions:wire.error.actions})));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(actualEvents.pop(),[wire.error.actions[0].id,{files:['/newer/queued.csv']}]);
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...actualProps,root:wire.done.root,actions:wire.done.actions})));
act(()=>renderer.root.findAllByType('button').find(b=>b.props.children==='Back').props.onClick());equal(actualEvents.pop(),[wire.done.root.props.backActionId]);
act(()=>renderer.update(React.createElement(surface.ExtensionRuntimeSurface,{...actualProps,root:wire.back.root,actions:wire.back.actions})));
act(()=>renderer.root.findByType('form').props.onSubmit({preventDefault(){}}));equal(actualEvents.pop(),[wire.back.actions[0].id,{files:['/test/source.csv']}]);
act(()=>renderer.unmount());console.log(JSON.stringify({totalRendererAssertions:assertions,compiledAdapterViews:4,desktopEffects:0}));
