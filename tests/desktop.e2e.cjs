// Tests the real embedded UI and Rust API in Microsoft Edge with synthetic audio on Windows.
const {chromium}=require('../target/ui-tools/node_modules/playwright-core');
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),assert=require('node:assert/strict'),{spawn}=require('node:child_process');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
function chunk(id,data){const h=Buffer.alloc(8);h.write(id);h.writeUInt32LE(data.length,4);return Buffer.concat([h,data,data.length%2?Buffer.alloc(1):Buffer.alloc(0)])}
function wav(freq,amplitude=12000){
 const sr=44100,frames=sr*8,fmt=Buffer.alloc(16),data=Buffer.alloc(frames*4);
 fmt.writeUInt16LE(1,0);fmt.writeUInt16LE(2,2);fmt.writeUInt32LE(sr,4);fmt.writeUInt32LE(sr*4,8);fmt.writeUInt16LE(4,12);fmt.writeUInt16LE(16,14);
 for(let i=0;i<frames;i++){const v=Math.round(Math.sin(i*2*Math.PI*freq/sr)*Math.sin(i/sr*3)**2*amplitude);data.writeInt16LE(v,i*4);data.writeInt16LE(v,i*4+2)}
 const body=Buffer.concat([Buffer.from('WAVE'),chunk('fmt ',fmt),chunk('data',data),chunk('LIST',Buffer.concat([Buffer.from('INFO'),chunk('INAM',Buffer.from('Original title\0'))]))]);
 const h=Buffer.alloc(8);h.write('RIFF');h.writeUInt32LE(body.length,4);return Buffer.concat([h,body]);
}
(async()=>{
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'vocal-cleaner-ui-'));
 const names=['10 Guitar.wav','2 Guitar.wav','1 Lead Vocals.wav','Drums.wav'];
 const paths=names.map((n,i)=>{const p=path.join(temp,n);fs.writeFileSync(p,wav(100+i*75));return p});
 const urlFile=path.join(temp,'server.url');
 const app=spawn(path.resolve('target/release/volund-audio.exe'),[],{env:{...process.env,VOCAL_CLEANER_SERVER_ONLY:'1',VOCAL_CLEANER_TEST_URL_FILE:urlFile},stdio:['ignore','pipe','pipe']});
 let browser,appLog='';app.stdout.on('data',b=>appLog+=b);app.stderr.on('data',b=>appLog+=b);
 try{
  for(let i=0;i<40&&!fs.existsSync(urlFile);i++){if(app.exitCode!==null)break;await pause(250)}
  assert.ok(fs.existsSync(urlFile),'Application server did not start. '+appLog);
  browser=await chromium.launch({channel:'msedge',headless:true});
  const context=await browser.newContext({viewport:{width:1280,height:820}}),page=await context.newPage();
  // A file URL has no backend /icon.svg route, like Tauri's bundled frontend.
  const assetPage=await context.newPage();
  await assetPage.goto(require('node:url').pathToFileURL(path.resolve('static/index.html')).href);
  await assetPage.waitForFunction(()=>{const icon=document.querySelector('.brand img');return icon.complete&&icon.naturalWidth>0});
  assert.ok((await assetPage.locator('.brand img').getAttribute('src')).startsWith('data:image/svg+xml;base64,'));
  await assetPage.close();
  await page.goto(fs.readFileSync(urlFile,'utf8'));
  assert.equal(await page.title(),'Völund Audio');
  assert.equal(await page.locator('h1').textContent(),'Völund Audio');
  assert.equal(await page.locator('.brand p').textContent(),'Smid fram ditt sound');
  const errors=[];page.on('pageerror',e=>{errors.push(e.message);console.error('UI error:',e.message)});
  assert.ok(await page.locator('#startScreen').isVisible());assert.equal(await page.locator('#masterPanel').isVisible(),false);await page.getByRole('button',{name:'Spårredigering',exact:false}).click();
  await page.locator('#f').setInputFiles(paths);
  await page.locator('section.track').nth(3).waitFor();
  assert.deepEqual(await page.locator('section.track header > button').allTextContents(),['1 Lead Vocals.wav','Drums.wav','2 Guitar.wav','10 Guitar.wav']);
  assert.ok(await page.locator('section.track.active #transport').isVisible());
  await page.getByRole('button',{name:'Drums.wav',exact:true}).click();
  assert.equal(await page.locator('section.track.active header > button').textContent(),'Drums.wav');
  assert.ok(await page.locator('section.track.active #transport').isVisible());
  await page.locator('#metadataPanel summary').click();
  await page.getByLabel('Titel',{exact:true}).fill('Ändrad titel – Chaoswrought');
  await page.locator('section.track.active canvas').scrollIntoViewIfNeeded();
  const canvas=await page.locator('section.track.active canvas').boundingBox();
  await page.mouse.move(canvas.x+canvas.width*.2,canvas.y+canvas.height/2);await page.mouse.down();
  await page.mouse.move(canvas.x+canvas.width*.4,canvas.y+canvas.height/2,{steps:6});await page.mouse.up();
  await page.getByRole('button',{name:'Tysta',exact:true}).click();
  const exported=await page.evaluate(async()=>{
    const r=await fetch(`/api/export/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({stem:vocal,edits,metadata:tracks[vocal].metadata})});
    if(!r.ok)throw new Error(await r.text());const bytes=await r.arrayBuffer();
    const sound=await ctx.decodeAudioData(bytes.slice(0)),mid=(sel[0]+sel[1])/2;
    const fd=new FormData();fd.append('files',new Blob([bytes]),'roundtrip.wav');
    const uploaded=await(await fetch('/api/upload',{method:'POST',body:fd})).json();
    const metadata=await(await fetch(`/api/metadata/${uploaded.sid}/roundtrip.wav`)).json();
    return {metadata,duration:sound.duration,middle:sound.getChannelData(0)[Math.round(mid*sound.sampleRate)]};
  });
  assert.equal(exported.metadata.fields.find(f=>f.key==='TrackTitle').value,'Ändrad titel – Chaoswrought');
  assert.ok(Math.abs(exported.duration-8)<.001);assert.equal(Math.abs(exported.middle),0);
  await page.getByRole('button',{name:'1 Lead Vocals.wav',exact:true}).click();await page.getByRole('button',{name:'Drums.wav',exact:true}).click();
  assert.equal(await page.getByLabel('Titel',{exact:true}).inputValue(),'Ändrad titel – Chaoswrought');
  await page.screenshot({path:'target/ui-dark.png',fullPage:true});
  await page.getByRole('button',{name:'Växla ljust och mörkt tema',exact:true}).click();
  await page.screenshot({path:'target/ui-light.png',fullPage:true});
  await page.getByRole('button',{name:'Rensa metadata',exact:true}).click();
  const cleared=await page.evaluate(async()=>{
    const r=await fetch(`/api/export/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({stem:vocal,edits,metadata:tracks[vocal].metadata})});
    if(!r.ok)throw new Error(await r.text());const fd=new FormData();fd.append('files',await r.blob(),'cleared.wav');
    const uploaded=await(await fetch('/api/upload',{method:'POST',body:fd})).json();return (await fetch(`/api/metadata/${uploaded.sid}/cleared.wav`)).json();
  });
  assert.equal(cleared.fields.length,0);assert.equal(cleared.pictures,0);assert.equal(cleared.binary,0);
  // Exercise actual UI handlers against the real mastering backend.

  await page.getByLabel('Nivå Drums.wav',{exact:true}).fill('-3');await page.getByLabel('Nivå Drums.wav',{exact:true}).press('Tab');
  await page.getByRole('button',{name:'Solo Drums.wav',exact:true}).click();
  assert.equal(await page.getByRole('button',{name:'Solo Drums.wav',exact:true}).getAttribute('aria-pressed'),'true');
  await page.evaluate(()=>chooseWorkspace('master'));assert.equal(await page.locator('#tracks').isVisible(),false);
  await page.locator('#masterTitle').fill('Låt 1');await page.locator('#masterArtist').fill('Chaoswrought');
  await page.locator('#masterProfile').selectOption('Vinyl');
  await page.locator('#master-bass_db').fill('-1.5');await page.locator('#master-bass_db').press('Tab');
  await page.locator('#masterProfile').selectOption('CD');
  assert.equal(await page.locator('#master-bits').inputValue(),'16');assert.ok(await page.locator('#master-bits').isDisabled());
  await page.locator('#masterPreview').click();await page.locator('#masterWet').waitFor({state:'visible'});
  await page.waitForFunction(()=>!!masterCache);
  assert.ok((await page.locator('#masterMetrics').textContent()).includes('LUFS'));
  await page.locator('#masterWet').click();await page.waitForTimeout(100);await page.locator('#masterDry').click();
  assert.equal(await page.evaluate(()=>masterWet),false);await page.evaluate(()=>stopMaster());
  const result=await page.evaluate(async()=>{
    const request=masterRequest(profileIds),r=await masterPost('export',request);
    const bytes=Array.from(new Uint8Array(await r.arrayBuffer()));
    // Parse stored ZIP entries in-browser; mastering archive deliberately uses Stored.
    const raw=new Uint8Array(bytes),view=new DataView(raw.buffer);let offset=0,entries=[];
    while(view.getUint32(offset,true)===0x04034b50){const size=view.getUint32(offset+18,true),n=view.getUint16(offset+26,true),extra=view.getUint16(offset+28,true);const name=new TextDecoder().decode(raw.slice(offset+30,offset+30+n));const start=offset+30+n+extra;entries.push({name,start,size});offset=start+size}
    const report=JSON.parse(new TextDecoder().decode(raw.slice(entries.find(e=>e.name==='Mastering-report.json').start,entries.find(e=>e.name==='Mastering-report.json').start+entries.find(e=>e.name==='Mastering-report.json').size)));
    const cd=entries.find(e=>e.name.endsWith('CD.wav')),audio=await ctx.decodeAudioData(raw.slice(cd.start,cd.start+cd.size).buffer);
    const fd=new FormData();fd.append('files',new Blob([raw]),'masters.zip');const uploaded=await(await fetch('/api/upload',{method:'POST',body:fd})).json();
    const metadata=await(await fetch(`/api/metadata/${uploaded.sid}/${encodeURIComponent('Låt 1 - mastered - CD.wav')}`)).json();
    return {names:entries.map(e=>e.name),report,duration:audio.duration,metadata,sideSession:uploaded.sid};
  });
  for(const id of ['Spotify','Vinyl','CD','Kassett'])assert.ok(result.names.includes('Låt 1 - mastered - '+id+'.wav'));
  assert.equal(result.report.reports.length,4);for(const report of result.report.reports)assert.ok(report.output.true_peak_db<=result.report.request.profiles.find(p=>p.id===report.profile).ceiling_db+.001);
  assert.equal(result.metadata.fields.find(f=>f.key==='TrackTitle').value,'Låt 1');assert.equal(result.duration,8);
  const sideResult=await page.evaluate(async sid=>{
    const request={title:'Album',sample_rate:44100,bits:24,tracks:[{stem:'Låt 1 - mastered - Vinyl.wav',side:'A',gap:0},{stem:'Låt 1 - mastered - Kassett.wav',side:'B',gap:2}]};
    const r=await fetch(`/api/master/sides/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(request)});
    // Native dialog is bypassed in CI server mode; resulting ZIP must have both sides.
    if(!r.ok)throw new Error(await r.text());const data=await r.arrayBuffer();return new TextDecoder().decode(data).includes('Side B.wav');
  },result.sideSession);assert.equal(sideResult,true);
  await page.locator('#masterProfile').selectOption('Vinyl');assert.equal(await page.locator('#master-bass_db').inputValue(),'-1.5');
  await page.screenshot({path:'target/ui-mastering.png',fullPage:true});
  const projectBytes=await page.evaluate(async()=>{stashTrack();const r=await fetch(`/api/project/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({version:3,tracks,mastering})});if(!r.ok)throw new Error(await r.text());return Array.from(new Uint8Array(await r.arrayBuffer()))});
  const projectFile=path.join(temp,'reopen.vcleaner');fs.writeFileSync(projectFile,Buffer.from(projectBytes));
  const previousSid=await page.evaluate(()=>sid);await page.evaluate(()=>{mastering.title='Must be restored';mastering.profiles.Vinyl.bass_db=4});
  page.on('dialog',dialog=>dialog.accept());await page.locator('#f').setInputFiles(projectFile);
  await page.waitForFunction(old=>sid!==old&&selectedProfile().id==='Vinyl'&&mastering.title==='Låt 1'&&masterCache===null,previousSid);
  await page.waitForFunction(()=>document.querySelector('#masterTitle').value==='Låt 1');
  assert.equal(await page.locator('#master-bass_db').inputValue(),'-1.5');
  await page.evaluate(()=>chooseWorkspace('editor'));
  assert.ok(await page.locator('#tracks').isVisible());
  assert.equal(await page.getByLabel('Nivå Drums.wav',{exact:true}).inputValue(),'-3');
  assert.equal(await page.getByRole('button',{name:'Solo Drums.wav',exact:true}).getAttribute('aria-pressed'),'true');
  await page.evaluate(()=>chooseWorkspace('editor'));
  await page.getByRole('button',{name:'2 Guitar.wav',exact:true}).click();
  await page.evaluate(()=>{sel=[1,3];draw()});
  await page.locator('#panLeft').click();
  const panned=await page.evaluate(async()=>{
    const preview=editedBuffer(vocal),r=await fetch(`/api/export/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({stem:vocal,edits})});if(!r.ok)throw new Error(await r.text());const sound=await ctx.decodeAudioData(await r.arrayBuffer());let right=0,left=0,error=0;
    for(let i=Math.round(2.1*sound.sampleRate);i<Math.round(2.2*sound.sampleRate);i++){right=Math.max(right,Math.abs(sound.getChannelData(1)[i]));left+=Math.abs(sound.getChannelData(0)[i]);error=Math.max(error,Math.abs(sound.getChannelData(0)[i]-preview.getChannelData(0)[i]))}
    return {right,left,error,vocalEdits:tracks['1 Lead Vocals.wav']?.edits||[],kind:edits.at(-1).kind};
  });assert.equal(panned.right,0);assert.ok(panned.left>0);assert.ok(panned.error<.0001);assert.equal(panned.vocalEdits.length,0);assert.equal(panned.kind,'pan');
  await page.locator('#panCenter').click();assert.ok(await page.evaluate(()=>Math.abs(editedBuffer(vocal).getChannelData(1)[Math.round(2.123*ctx.sampleRate)])>0));
  await page.getByRole('button',{name:'Ångra',exact:true}).click();assert.equal(await page.evaluate(()=>editedBuffer(vocal).getChannelData(1)[Math.round(2.123*ctx.sampleRate)]),0);
  await page.locator('#panSweepRight').click();assert.equal(await page.evaluate(()=>edits.at(-1).pan_to),1);
  await page.locator('#panCycles').fill('3');await page.locator('#panAlternate').click();assert.equal(await page.evaluate(()=>edits.at(-1).pan_cycles),3);
  const panProject=await page.evaluate(async()=>{stashTrack();const r=await fetch(`/api/project/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({version:3,tracks,mastering})});const bytes=await r.arrayBuffer();const fd=new FormData();fd.append('files',new Blob([bytes]),'pan.vcleaner');return(await fetch('/api/upload',{method:'POST',body:fd})).json()});
  assert.equal(panProject.project.tracks['2 Guitar.wav'].edits.at(-1).pan_cycles,3);
  await page.screenshot({path:'target/ui-panning.png',fullPage:true});
  // Cut the stereo mix, leaving every source stem and its edit history untouched.
  await page.evaluate(()=>chooseWorkspace('mix'));
  await page.getByLabel('Start (s)',{exact:true}).fill('1');await page.getByLabel('Slut (s)',{exact:true}).fill('3');
  await page.getByRole('button',{name:'Klipp bort markering',exact:true}).click();
  assert.equal(await page.evaluate(()=>mixAudio.duration),6);
  assert.equal(await page.evaluate(()=>bufs[vocal].duration),8);
  await page.getByLabel('Start (s)',{exact:true}).fill('0.5');await page.getByLabel('Slut (s)',{exact:true}).fill('5.5');
  await page.getByRole('button',{name:'Behåll bara markering',exact:true}).click();
  assert.equal(await page.evaluate(()=>mixAudio.duration),5);
  await page.getByRole('button',{name:'Ångra klipp',exact:true}).click();assert.equal(await page.evaluate(()=>mixAudio.duration),6);
  await page.getByRole('button',{name:'Gör om klipp',exact:true}).click();assert.equal(await page.evaluate(()=>mixAudio.duration),5);
  const mixCheck=await page.evaluate(async()=>{
    const pre=await masterPost('premaster',masterRequest());if(!pre.ok)throw Error(await pre.text());const b=await ctx.decodeAudioData(await pre.arrayBuffer());
    const expected=dryMix();let error=0;for(let i=0;i<b.length;i+=37)error=Math.max(error,Math.abs(b.getChannelData(0)[i]-expected.getChannelData(0)[i]));
    const result=await masterPost('export',masterRequest(profileIds));const raw=new Uint8Array(await result.arrayBuffer()),v=new DataView(raw.buffer);let offset=0,report;
    while(v.getUint32(offset,true)===0x04034b50){const size=v.getUint32(offset+18,true),n=v.getUint16(offset+26,true),extra=v.getUint16(offset+28,true),name=new TextDecoder().decode(raw.slice(offset+30,offset+30+n)),start=offset+30+n+extra;if(name==='Mastering-report.json')report=JSON.parse(new TextDecoder().decode(raw.slice(start,start+size)));offset=start+size}
    return {duration:b.duration,error,durations:report.reports.map(r=>r.output.duration)};
  });assert.equal(mixCheck.duration,5);assert.ok(mixCheck.error<0.0002);assert.deepEqual(mixCheck.durations,[5,5,5,5]);
  const cutBytes=await page.evaluate(async()=>{stashTrack();const r=await fetch(`/api/project/${sid}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({version:3,tracks,mastering})});return Array.from(new Uint8Array(await r.arrayBuffer()))});
  const cutProject=path.join(temp,'cut.vcleaner');fs.writeFileSync(cutProject,Buffer.from(cutBytes));const oldSid=await page.evaluate(()=>sid);
  await page.locator('#f').setInputFiles(cutProject);await page.waitForFunction(old=>sid!==old&&!saving,oldSid);
  await page.evaluate(()=>refreshMix());assert.equal(await page.evaluate(()=>mixAudio.duration),5);
  await page.getByRole('button',{name:'Ångra klipp',exact:true}).click();assert.equal(await page.evaluate(()=>mixAudio.duration),6);
  await page.getByRole('button',{name:'Gör om klipp',exact:true}).click();
  await page.locator('#language').selectOption('en');
  await page.getByRole('button',{name:'Cut selection',exact:true}).waitFor();
  assert.equal(await page.locator('.brand p').textContent(),'Forge your sound');
  assert.equal(await page.locator('#masterProfile option[value="Kassett"]').textContent(),'Cassette');
  assert.equal(await page.evaluate(()=>mastering.profiles.Kassett.id),'Kassett');
  await page.evaluate(()=>chooseWorkspace('editor'));
  assert.equal(await page.getByRole('button',{name:'Drums.wav',exact:true}).textContent(),'Drums.wav');
  await page.evaluate(()=>chooseWorkspace('mix'));
  await page.getByRole('button',{name:'Om / About',exact:true}).click();await page.getByText('A local audio editor for stems, stereo mixing and mastering.',{exact:true}).waitFor();
  assert.ok((await page.locator('#about').textContent()).includes('master smith Völund'));
  await page.getByRole('button',{name:'Close',exact:true}).click();
  await page.screenshot({path:'target/ui-mix-english.png',fullPage:true});
  await page.locator('#language').selectOption('sv');await page.getByRole('button',{name:'Klipp bort markering',exact:true}).waitFor();
  await page.screenshot({path:'target/ui-mix-swedish.png',fullPage:true});
  // A single stereo WAV uses the same cut workflow without importing stems.
  await page.locator('#f').setInputFiles(paths[0]);await page.waitForFunction(()=>names.length===1&&!saving);await page.evaluate(()=>refreshMix());
  await page.getByLabel('Start (s)',{exact:true}).fill('2');await page.getByLabel('Slut (s)',{exact:true}).fill('7');
  await page.getByRole('button',{name:'Behåll bara markering',exact:true}).click();assert.equal(await page.evaluate(()=>mixAudio.duration),5);
  // Album leveling: three complete songs, with an intentional quiet intro.
  const albumPaths=['01 Loud.wav','02 Quiet.wav','03 Intro.wav'].map((name,i)=>{const file=path.join(temp,name);fs.writeFileSync(file,wav(250,[12000,3000,1000][i]));return file});
  await page.evaluate(()=>chooseWorkspace('album'));await page.locator('#sideFiles').setInputFiles(albumPaths);
  await page.waitForFunction(()=>sideTracks.length===3&&!saving);assert.equal(await page.locator('#albumMatch').isChecked(),true);
  await page.getByLabel('Matcha 03 Intro.wav',{exact:true}).uncheck();await page.getByLabel('Justering för 02 Quiet.wav',{exact:true}).fill('-1.5');await page.getByLabel('Justering för 02 Quiet.wav',{exact:true}).press('Tab');
  await page.getByLabel('Sida för 02 Quiet.wav',{exact:true}).selectOption('B');
  await page.locator('#albumCustom').check();await page.locator('#albumTarget').fill('-6');await page.locator('#albumTarget').press('Tab');
  await page.locator('#albumAnalyze').click();await page.waitForFunction(()=>!!albumPlan&&!saving);
  const plan=await page.evaluate(()=>albumPlan);assert.ok(plan.common_attenuation_db>0);assert.equal(plan.tracks[2].matched,false);assert.ok(Math.abs(plan.tracks[0].expected_lufs-plan.tracks[1].expected_lufs-1.5)<.1);
  await page.locator('#albumReport button').nth(1).click();await page.waitForFunction(()=>!!albumSource);await page.evaluate(()=>stopAlbumPreview());
  const albumResult=await page.evaluate(async()=>{
    const parse=raw=>{const v=new DataView(raw.buffer);let offset=0,out={};while(v.getUint32(offset,true)===0x04034b50){const size=v.getUint32(offset+18,true),n=v.getUint16(offset+26,true),extra=v.getUint16(offset+28,true),name=new TextDecoder().decode(raw.slice(offset+30,offset+30+n)),start=offset+30+n+extra;out[name]=raw.slice(start,start+size);offset=start+size}return out};
    const request=sideRequest();request.bits=16;const r=await fetch(`/api/album/export/${sideSession}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(request)});if(!r.ok)throw Error(await r.text());const entries=parse(new Uint8Array(await r.arrayBuffer())),report=JSON.parse(new TextDecoder().decode(entries['Album-report.json']));
    const first=entries[report.outputs[0].file],sound=await ctx.decodeAudioData(first.slice().buffer),v=new DataView(first.buffer);let bits;for(let at=12;at+8<first.length;){const id=new TextDecoder().decode(first.slice(at,at+4)),size=v.getUint32(at+4,true);if(id==='fmt ')bits=v.getUint16(at+22,true);at+=8+size+(size%2)}
    const side=await fetch(`/api/master/sides/${sideSession}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(request)});if(!side.ok)throw Error(await side.text());const sideEntries=parse(new Uint8Array(await side.arrayBuffer())),sideReport=JSON.parse(new TextDecoder().decode(sideEntries['Side-report.json']));
    const fd=new FormData();fd.append('files',new Blob([first]),'album.wav');const uploaded=await(await fetch('/api/upload',{method:'POST',body:fd})).json(),metadata=await(await fetch(`/api/metadata/${uploaded.sid}/album.wav`)).json();
    return {report,duration:sound.duration,bits,metadata,sideReport};
  });assert.equal(albumResult.bits,16);assert.equal(albumResult.duration,8);assert.equal(albumResult.report.outputs.length,3);assert.ok(Math.abs(albumResult.report.outputs[0].metrics.lufs-albumResult.report.outputs[1].metrics.lufs-1.5)<.15);assert.ok(albumResult.report.outputs.every(t=>t.metrics.true_peak_db<=-1));assert.equal(albumResult.metadata.fields.find(f=>f.key==='TrackTitle').value,'Original title');assert.equal(albumResult.sideReport.sides.length,2);
  const albumProjectBytes=await page.evaluate(async()=>{const r=await fetch(`/api/project/${sideSession}`,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({version:3,tracks:{},sideAssembly:sideRequest()})});return Array.from(new Uint8Array(await r.arrayBuffer()))});
  const albumProjectFile=path.join(temp,'album.vcleaner');fs.writeFileSync(albumProjectFile,Buffer.from(albumProjectBytes));const previousAlbum=await page.evaluate(()=>sideSession);await page.locator('#sideFiles').setInputFiles(albumProjectFile);await page.waitForFunction(old=>sideSession!==old&&!saving,previousAlbum);
  assert.equal(await page.getByLabel('Matcha 03 Intro.wav',{exact:true}).isChecked(),false);assert.equal(await page.getByLabel('Justering för 02 Quiet.wav',{exact:true}).inputValue(),'-1.5');assert.equal(await page.locator('#albumTarget').inputValue(),'-6');assert.equal(await page.evaluate(()=>sideDirty()),false);
  await page.locator('#albumAnalyze').click();await page.waitForFunction(()=>!!albumPlan&&!saving);await page.screenshot({path:'target/ui-album-swedish.png',fullPage:true});
  await page.locator('#language').selectOption('en');await page.getByRole('button',{name:'Analyze album levels',exact:true}).waitFor();await page.screenshot({path:'target/ui-album-english.png',fullPage:true});
  await page.locator('#language').selectOption('en');await page.reload();assert.equal(await page.locator('#language').inputValue(),'en');assert.ok(await page.locator('#startScreen').isVisible());await page.evaluate(()=>chooseWorkspace('editor'));await page.getByRole('button',{name:'Save project…',exact:true}).waitFor();
  assert.deepEqual(errors,[]);console.log('PASS: Windows editor UI, sorted stems, moving toolbar, metadata editing/clearing, muted WAV, themes');
 }finally{if(browser)await browser.close();app.kill();fs.rmSync(temp,{recursive:true,force:true})}
})().catch(e=>{console.error(e);process.exitCode=1});
