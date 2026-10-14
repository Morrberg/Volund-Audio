const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const elements=new Map();
function element(){return {setAttribute(){},disabled:false,textContent:'',clientWidth:100,clientHeight:100,children:[],classList:{toggle(){}},append(...children){for(const child of children){if(child.parentElement)child.parentElement.children=child.parentElement.children.filter(c=>c!==child);child.parentElement=this;this.children.push(child)}},replaceChildren(...children){for(const child of this.children)child.parentElement=null;this.children=[];this.append(...children)},getBoundingClientRect:()=>({top:0}),getContext:()=>({clearRect(){},fillRect(){},fillText(){}})}}
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
// Selection panning preserves the other stem, stereo channels and undo history.
await run(`bufs={'Synth.wav':ctx.createBuffer(2,1000,1000),'Vocals.wav':ctx.createBuffer(2,1000,1000)};names=Object.keys(bufs);tracks={};vocal=null;for(const n of names){bufs[n].getChannelData(0).fill(.2);bufs[n].getChannelData(1).fill(.3)}buildTracks();setVocal('Synth.wav');sel=[.2,.8];addPan('left');`);
assert.equal(await run(`editedBuffer('Synth.wav').getChannelData(1)[500]`),0);
assert.ok(Math.abs(await run(`editedBuffer('Synth.wav').getChannelData(0)[500]`)-.5)<1e-6);
assert.ok(Math.abs(await run(`editedBuffer('Synth.wav').getChannelData(1)[100]`)-.3)<1e-6);
assert.equal(await run(`editedBuffer('Vocals.wav')===bufs['Vocals.wav']`),true);
await run(`addPan('center');`);assert.ok(Math.abs(await run(`editedBuffer('Synth.wav').getChannelData(1)[500]`)-.3)<1e-6);
await run(`undo();`);assert.equal(await run(`editedBuffer('Synth.wav').getChannelData(1)[500]`),0);
await run(`redoE();`);assert.ok(Math.abs(await run(`editedBuffer('Synth.wav').getChannelData(1)[500]`)-.3)<1e-6);
await run(`undo();setVocal('Vocals.wav');setVocal('Synth.wav');sel=[.2,.8];addPan('lr');`);
assert.ok(await run(`editedBuffer('Synth.wav').getChannelData(0)[220]>editedBuffer('Synth.wav').getChannelData(1)[220]`));
assert.ok(await run(`editedBuffer('Synth.wav').getChannelData(1)[779]>editedBuffer('Synth.wav').getChannelData(0)[779]`));
await run(`document.querySelector('#panCycles').value=2;addPan('alternate');`);assert.equal(await run('edits.at(-1).pan_cycles'),2);
await run(`stashTrack();const panManifest=JSON.parse(JSON.stringify(tracks));tracks=panManifest;vocal=null;setVocal('Synth.wav');`);assert.equal(await run('edits.at(-1).kind'),'pan');
await run(`const beforeRemove=edits.length;document.querySelector('#panEdits').children[0].children[1].onclick();`);assert.equal(await run('edits.length'),await run('beforeRemove-1'));await run('undo()');assert.equal(await run('edits.length'),await run('beforeRemove'));
await run(`bufs['Mono.wav']=ctx.createBuffer(1,1000,1000);bufs['Mono.wav'].getChannelData(0).fill(.2);names.push('Mono.wav');setVocal('Mono.wav');sel=[.2,.8];addPan('right');`);
assert.equal(await run(`editedBuffer('Mono.wav').numberOfChannels`),2);assert.equal(await run(`editedBuffer('Mono.wav').getChannelData(0)[500]`),0);
assert.ok(Math.abs(await run(`editedBuffer('Mono.wav').getChannelData(1)[500]`)-.2*Math.SQRT2)<1e-6);
await run(`sideTracks=[{stem:'Song.wav',side:'A',gap:2}];sideDurations={'Song.wav':60};document.querySelector('#sideTitle').value='Album';document.querySelector('#sideRate').value='44100';document.querySelector('#sideBits').value='24';sideSaved=JSON.stringify(sideRequest());sideTracks[0].side='B';`);assert.equal(await run('sideDirty()'),true);
// Rebuild detaches the active editor. Real querySelector cannot find its controls until reattached.
await run(`setVocal('Synth.wav');buildTracks();const realQuery=document.querySelector;let missingDetachedControls=false;document.querySelector=s=>{if(s==='#pp'&&!editorPanel.parentElement?.parentElement){missingDetachedControls=true;return null}return realQuery(s)};setVocal('Vocals.wav');document.querySelector=realQuery;`);
assert.equal(await run('missingDetachedControls'),false);
// Mix cuts have their own undo/redo and never alter stem timing.
await run(`mastering=restoreMaster();tracks={};vocal=null;names=['One.wav'];bufs={'One.wav':ctx.createBuffer(2,1000,1000)};for(let i=0;i<1000;i++){bufs['One.wav'].getChannelData(0)[i]=i;bufs['One.wav'].getChannelData(1)[i]=-i}buildTracks();setVocal('One.wav');refreshMix();$('#mixStart').value=.2;$('#mixEnd').value=.4;cutMix('cut');`);
assert.equal(await run('mixAudio.length'),800);assert.equal(await run('mixAudio.getChannelData(0)[300]'),500);assert.equal(await run('mixAudio.getChannelData(1)[300]'),-500);assert.equal(await run('bufs[vocal].length'),1000);assert.equal(await run('edits.length'),0);
await run('undoMix()');assert.equal(await run('mixAudio.length'),1000);await run('redoMix()');assert.equal(await run('mixAudio.length'),800);
await run(`$('#mixStart').value=.1;$('#mixEnd').value=.6;cutMix('keep');`);assert.equal(await run('mixAudio.length'),500);assert.equal(await run('mixAudio.getChannelData(0)[200]'),500);
assert.equal(await run('masterRequest().mix_edits.length'),2);
await run('mastering=restoreMaster(JSON.parse(JSON.stringify(mastering)));refreshMix()');assert.equal(await run('mixAudio.length'),500);await run('undoMix()');assert.equal(await run('mixAudio.length'),800);
await run(`$('#mixStart').value=0;$('#mixEnd').value=.8;cutMix('cut');`);assert.equal(await run('mixAudio.length'),800);assert.equal(await run('mastering.mixEdits.length'),1);
await run(`setLanguage('en')`);assert.equal(await run(`t('Klipp bort markering')`),'Cut selection');assert.equal(await run(`t('Spara projekt…')`),'Save project…');await run(`setLanguage('sv')`);assert.equal(await run(`t('Klipp bort markering')`),'Klipp bort markering');
await run(`$('#albumMatch').checked=true;$('#albumCustom').checked=true;$('#albumTarget').value=-16;$('#albumCeiling').value=-2;sideTracks=[{stem:'Quiet.wav',side:'A',gap:0,match_level:false,offset_db:-3}];`);
assert.equal(await run('sideRequest().level_matching.target_lufs'),-16);assert.equal(await run('sideRequest().tracks[0].match_level'),false);assert.equal(await run('sideRequest().tracks[0].offset_db'),-3);
await run(`sideSaved=JSON.stringify(sideRequest());albumPlan={};sideChanged()`);assert.equal(await run('albumPlan'),null);assert.equal(await run('sideDirty()'),false);
await run(`$('#albumMatch').checked=false;sideChanged()`);assert.equal(await run('sideDirty()'),true);assert.equal(await run(`$('#albumTarget').disabled`),true);
console.log('PASS: stem edits, undo/redo, WAV save, project save, cancellation');
})().catch(e=>{console.error(e);process.exitCode=1});
