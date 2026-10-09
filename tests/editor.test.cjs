const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const elements=new Map();
const context=vm.createContext({console,JSON,Math,Float32Array,navigator:{userAgent:'Windows'},
 document:{querySelector(s){if(!elements.has(s))elements.set(s,{disabled:false,textContent:'',clientWidth:100,clientHeight:100,getContext:()=>({clearRect(){},fillRect(){},fillText(){}})});return elements.get(s)}},
 AudioContext:class{createBuffer(ch,length,sr){const data=Array.from({length:ch},()=>new Float32Array(length));return {numberOfChannels:ch,length,sampleRate:sr,duration:length/sr,getChannelData:c=>data[c]}}},
 addEventListener(){},window:{},fetch:async()=>({ok:true,json:async()=>({cancelled:false,name:'Vocals_cleaned.wav'})})});
const html=fs.readFileSync('static/index.html','utf8');vm.runInContext(html.match(/<script>([\s\S]*?)<\/script>/)[1],context);
async function run(code){return vm.runInContext(code,context)}
(async()=>{
await run(`bufs={'Vocals.wav':ctx.createBuffer(2,1000,100),'Drums.wav':ctx.createBuffer(2,1000,100)};names=Object.keys(bufs);sid='test';setVocal('Vocals.wav');sel=[1,2];addEdit('mute');`);
assert.equal(await run('edits.length'),1);assert.equal(await run('projectDirty()'),true);
await run(`setVocal('Drums.wav');sel=[3,4];addEdit('fadeout');setVocal('Vocals.wav');`);
assert.equal(await run('edits[0].kind'),'mute');await run('undo();');assert.equal(await run('edits.length'),0);
await run(`setVocal('Drums.wav');setVocal('Vocals.wav');redoE();`);assert.equal(await run('edits[0].kind'),'mute');
await run('saveWav()');assert.equal(await run('JSON.stringify(edits)===savedEdits'),true);assert.equal(await run('projectDirty()'),true);
await run('saveProject()');assert.equal(await run('projectDirty()'),false);
await run('undo()');assert.equal(await run('projectDirty()'),true);await run('redoE()');assert.equal(await run('projectDirty()'),false);
await run(`fetch=async()=>({ok:true,json:async()=>({cancelled:true})});sel=[5,6];addEdit('mute');`);
await run('saveWav()');assert.equal(await run('JSON.stringify(edits)===savedEdits'),false);
await run('saveProject()');assert.equal(await run('projectDirty()'),true);
console.log('PASS: stem edits, undo/redo, WAV save, project save, cancellation');
})().catch(e=>{console.error(e);process.exitCode=1});
