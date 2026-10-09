const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const elements=new Map();
function element(){return {setAttribute(){},disabled:false,textContent:'',clientWidth:100,clientHeight:100,children:[],classList:{toggle(){}},append(...children){for(const child of children){if(child.parentElement)child.parentElement.children=child.parentElement.children.filter(c=>c!==child);child.parentElement=this;this.children.push(child)}},replaceChildren(){this.children=[]},getBoundingClientRect:()=>({top:0}),getContext:()=>({clearRect(){},fillRect(){},fillText(){}})}}
const context=vm.createContext({console,JSON,Math,Float32Array,navigator:{userAgent:'Windows'},
 document:{createTextNode:text=>({textContent:text}),createElement:element,querySelector(s){if(!elements.has(s))elements.set(s,element());return elements.get(s)}},
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
await run(`buildTracks();draw();`);assert.equal(await run('trackCanvases.size'),2);assert.equal(await run('peakCache.size'),2);
await run(`trackCanvases.get('Drums.wav').onmousedown({offsetX:50,currentTarget:trackCanvases.get('Drums.wav')});`);
assert.equal(await run('vocal'),'Drums.wav');assert.equal(await run('view.b'),10);
assert.equal(await run('sampleGain([{kind:"mute",start:1,end:2,fade:0.01}],150,100,1000)'),0);
assert.equal(await run(`editedBuffer('Vocals.wav')===bufs['Vocals.wav']`),false);
await run(`stashTrack();const reopened=JSON.parse(JSON.stringify(tracks));tracks=reopened;vocal=null;setVocal('Vocals.wav');`);assert.equal(await run('edits.length'),2);
assert.equal(await run("editorPanel.parentElement===trackCanvases.get('Vocals.wav').parentElement"),true);
await run(`const order=['10 Guitar.wav','2 Guitar.wav','Bass.wav','Drums.wav','Backing Vocals.wav','Lead Vocals.wav'];order.sort(compareStems);`);
assert.equal(await run('order.join("|")'),'Lead Vocals.wav|Backing Vocals.wav|Drums.wav|Bass.wav|2 Guitar.wav|10 Guitar.wav');
await run(`metaInfo['Vocals.wav']={fields:[{key:'TrackTitle',value:'Original',editable:true}],pictures:1,binary:0};setMetadata([{key:'TrackTitle',value:'Ändrad titel'}]);`);
assert.equal(await run('metadataFields()[0].value'),'Ändrad titel');
await run(`setVocal('Drums.wav');setVocal('Vocals.wav');`);assert.equal(await run('metadataFields()[0].value'),'Ändrad titel');
await run(`fetch=async(url,options)=>{globalThis.lastRequest=JSON.parse(options.body);return {ok:true,json:async()=>({cancelled:false,name:'saved.wav'})}};saveWav();`);
// Wait for the async save to settle before verifying the saved fingerprint.
await run('Promise.resolve()');assert.equal(await run('lastRequest.metadata.fields[0].value'),'Ändrad titel');
await run('clearMetadata()');assert.equal(await run('metadataFields().length'),0);assert.equal(await run('tracks[vocal].metadata.clear'),true);
await run('resetMetadata()');assert.equal(await run('metadataFields()[0].value'),'Original');
await run(`buildTracks();setVocal('Vocals.wav');`);assert.equal(await run("editorPanel.parentElement===trackCanvases.get('Vocals.wav').parentElement"),true);
await run(`mastering=restoreMaster();mastering.title='Låt 1';mastering.profiles.Vinyl.bass_db=-1.5;mixChange('Drums.wav','gain_db',-3);mixChange('Vocals.wav','solo',true);renderMaster();`);
assert.equal(await run(`masterRequest().tracks.find(t=>t.stem==='Drums.wav').gain_db`),-3);
assert.equal(await run(`masterRequest().tracks.find(t=>t.stem==='Vocals.wav').solo`),true);
assert.equal(await run(`masterRequest(['Vinyl']).profiles[0].bass_db`),-1.5);
await run(`fetch=async(url,options)=>{globalThis.lastRequest=JSON.parse(options.body);return {ok:true,json:async()=>({cancelled:false})}};saveProject();`);
await run('Promise.resolve()');assert.equal(await run('lastRequest.version'),3);assert.equal(await run('lastRequest.mastering.profiles.Vinyl.bass_db'),-1.5);
await run(`mastering=restoreMaster(lastRequest.mastering);tracks=lastRequest.tracks;projectSaved=projectFingerprint();chooseProfile('Vinyl');`);assert.equal(await run('selectedProfile().bass_db'),-1.5);assert.equal(await run('projectDirty()'),true);
await run(`masterNormalize(false);`);assert.equal(await run('selectedProfile().target_lufs'),null);
await run(`chooseProfile('CD');masterPreset('Tryck');`);assert.equal(await run('selectedProfile().sample_rate'),44100);assert.equal(await run('selectedProfile().bits'),16);assert.equal(await run('selectedProfile().target_lufs'),-10);assert.equal(await run('mastering.profiles.Vinyl.bass_db'),-1.5);
await run(`sideTracks=[{stem:'Song.wav',side:'A',gap:2}];sideDurations={'Song.wav':60};document.querySelector('#sideTitle').value='Album';document.querySelector('#sideRate').value='44100';document.querySelector('#sideBits').value='24';sideSaved=JSON.stringify(sideRequest());sideTracks[0].side='B';`);assert.equal(await run('sideDirty()'),true);
console.log('PASS: stem edits, undo/redo, WAV save, project save, cancellation');
})().catch(e=>{console.error(e);process.exitCode=1});
