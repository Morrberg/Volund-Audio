// Tests the real Windows WebView with synthetic audio. Debugging stays on loopback in CI.
const {chromium}=require('../target/ui-tools/node_modules/playwright-core');
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),assert=require('node:assert/strict'),{spawn}=require('node:child_process');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
function chunk(id,data){const h=Buffer.alloc(8);h.write(id);h.writeUInt32LE(data.length,4);return Buffer.concat([h,data,data.length%2?Buffer.alloc(1):Buffer.alloc(0)])}
function wav(freq){
 const sr=44100,frames=sr*8,fmt=Buffer.alloc(16),data=Buffer.alloc(frames*4);
 fmt.writeUInt16LE(1,0);fmt.writeUInt16LE(2,2);fmt.writeUInt32LE(sr,4);fmt.writeUInt32LE(sr*4,8);fmt.writeUInt16LE(4,12);fmt.writeUInt16LE(16,14);
 for(let i=0;i<frames;i++){const v=Math.round(Math.sin(i*2*Math.PI*freq/sr)*Math.sin(i/sr*3)**2*12000);data.writeInt16LE(v,i*4);data.writeInt16LE(v,i*4+2)}
 const body=Buffer.concat([Buffer.from('WAVE'),chunk('fmt ',fmt),chunk('data',data),chunk('LIST',Buffer.concat([Buffer.from('INFO'),chunk('INAM',Buffer.from('Original title\0'))]))]);
 const h=Buffer.alloc(8);h.write('RIFF');h.writeUInt32LE(body.length,4);return Buffer.concat([h,body]);
}
(async()=>{
 const temp=fs.mkdtempSync(path.join(os.tmpdir(),'vocal-cleaner-ui-'));
 const names=['10 Guitar.wav','2 Guitar.wav','1 Lead Vocals.wav','Drums.wav'];
 const paths=names.map((n,i)=>{const p=path.join(temp,n);fs.writeFileSync(p,wav(100+i*75));return p});
 const app=spawn(path.resolve('target/release/vocal-cleaner.exe'),[],{env:{...process.env,WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9222 --remote-debugging-address=127.0.0.1'},stdio:'ignore'});
 let browser;
 try{
  for(let i=0;i<40;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9222');break}catch{await pause(500)}}
  assert.ok(browser,'WebView2 did not start with the local test debugger');
  const context=browser.contexts()[0];let page;
  for(let i=0;i<30;i++){page=context.pages().find(p=>p.url().includes('127.0.0.1'));if(page)break;await pause(200)}
  assert.ok(page,'Application page did not load');
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.locator('#f').setInputFiles(paths);
  await page.locator('section.track').nth(3).waitFor();
  assert.deepEqual(await page.locator('section.track header button').allTextContents(),['1 Lead Vocals.wav','Drums.wav','2 Guitar.wav','10 Guitar.wav']);
  assert.ok(await page.locator('section.track.active #transport').isVisible());
  await page.getByRole('button',{name:'Drums.wav',exact:true}).click();
  assert.equal(await page.locator('section.track.active header button').textContent(),'Drums.wav');
  assert.ok(await page.locator('section.track.active #transport').isVisible());
  await page.locator('#metadataPanel summary').click();
  await page.getByLabel('Titel',{exact:true}).fill('Ändrad titel – Chaoswrought');
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
  assert.ok(Math.abs(exported.duration-8)<.001);assert.equal(exported.middle,0);
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
  assert.deepEqual(errors,[]);console.log('PASS: native window, sorted stems, moving toolbar, metadata editing/clearing, muted WAV, themes');
 }finally{if(browser)await browser.close();app.kill();fs.rmSync(temp,{recursive:true,force:true})}
})().catch(e=>{console.error(e);process.exitCode=1});
