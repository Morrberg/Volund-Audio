"""Vocal Cleaner - lokal backend (FastAPI). Originalfiler skrivs aldrig över."""
import io, uuid, zipfile
from pathlib import Path
import numpy as np, soundfile as sf
from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

ROOT = Path(__file__).parent
DATA = ROOT / "workdir"; DATA.mkdir(exist_ok=True)
EXT = {".wav", ".mp3", ".flac"}
app = FastAPI()


def sess(sid: str) -> Path:
    p = DATA / Path(sid).name
    if not p.is_dir():
        raise HTTPException(404, "Okänd session")
    return p


@app.post("/api/upload")
async def upload(files: list[UploadFile] = File(...)):
    sid = uuid.uuid4().hex[:8]; d = DATA / sid; d.mkdir()
    for f in files:
        name, raw = Path(f.filename).name, await f.read()
        if name.lower().endswith(".zip"):
            with zipfile.ZipFile(io.BytesIO(raw)) as z:
                for m in z.namelist():
                    n = Path(m).name
                    if Path(n).suffix.lower() in EXT:
                        (d / n).write_bytes(z.read(m))
        elif Path(name).suffix.lower() in EXT:
            (d / name).write_bytes(raw)
    stems = sorted(p.name for p in d.iterdir() if p.is_file())
    if not stems:
        raise HTTPException(400, "Inga ljudfiler hittades")
    vocal = next((s for s in stems if "vocal" in s.lower() or "sång" in s.lower()), None)
    return {"sid": sid, "stems": stems, "vocal": vocal}


@app.get("/api/stem/{sid}/{name}")
def stem(sid: str, name: str):
    return FileResponse(sess(sid) / Path(name).name)


@app.get("/api/analyze/{sid}/{name}")
def analyze(sid: str, name: str, edge: float = 20.0, max_len: float = 4.0, min_gap: float = 1.5):
    """Energibaserad aktivitetsdetektering + kontextregler. Returnerar FÖRSLAG, ändrar inget."""
    y, sr = sf.read(sess(sid) / Path(name).name, always_2d=True, dtype="float32")
    m = y.mean(1); total = len(m) / sr
    hop = int(sr * 0.02); n = len(m) // hop
    rms = np.sqrt((m[: n * hop].reshape(n, hop) ** 2).mean(1)) + 1e-9
    db = 20 * np.log10(rms)
    act = db > max(db.max() - 45, -60)
    runs, i = [], 0
    while i < n:                                   # sammanhängande aktiva block
        if act[i]:
            j = i
            while j < n and act[j]: j += 1
            runs.append([i * 0.02, j * 0.02]); i = j
        else:
            i += 1
    segs = []                                      # slå ihop block med glapp < 0.4 s
    for r in runs:
        if segs and r[0] - segs[-1][1] < 0.4: segs[-1][1] = r[1]
        else: segs.append(r)
    out = []
    for k, (a, b) in enumerate(segs):
        first, last = k == 0, k == len(segs) - 1
        gb = a - segs[k - 1][1] if not first else None   # glapp före (None = spårets början)
        ga = segs[k + 1][0] - b if not last else None    # glapp efter (None = spårets slut)
        dur = b - a
        near_edge = a < edge or b > total - edge
        inner_ok = ((first and ga is not None and ga >= min_gap) or
                    (last and gb is not None and gb >= min_gap) or
                    (gb is not None and ga is not None and gb >= min_gap and ga >= min_gap))
        if dur < max_len and near_edge and inner_ok:
            gap = min(g for g in (gb, ga) if g is not None)
            conf = float(np.clip(0.4 + 0.3 * min(gap / 5, 1) + 0.3 * (1 - dur / max_len), 0, 0.95))
            pos = "i början" if a < edge else "i slutet"
            out.append({"start": round(a, 2), "end": round(b, 2), "confidence": round(conf, 2),
                        "desc": f"Kort isolerat vokalt ljud {pos} ({dur:.1f} s, {gap:.1f} s tystnad intill)"})
    return {"duration": total, "suggestions": out,
            "note": "Heuristiska förslag (energi + kontext). Confidence är inte kalibrerad."}


class Edit(BaseModel):
    kind: str = "mute"       # mute | fadein | fadeout
    start: float
    end: float
    fade: float = 0.01


class ExportReq(BaseModel):
    stem: str
    edits: list[Edit]


@app.post("/api/export/{sid}")
def export(sid: str, r: ExportReq):
    p = sess(sid) / Path(r.stem).name
    info = sf.info(p)
    y, sr = sf.read(p, always_2d=True, dtype="float32")
    for e in r.edits:                              # samma matematik som förhandsvisningen i index.html
        a, b = max(0, round(e.start * sr)), min(len(y), round(e.end * sr)); n = b - a
        if n <= 0: continue
        if e.kind == "fadein": y[a:b] *= (np.arange(n) / n)[:, None]
        elif e.kind == "fadeout": y[a:b] *= (1 - np.arange(n) / n)[:, None]
        else:
            f = min(round(e.fade * sr), a, len(y) - b)
            if f > 0:
                y[a - f:a] *= (1 - np.arange(f) / f)[:, None]
                y[b:b + f] *= (np.arange(f) / f)[:, None]
            y[a:b] = 0
    od = sess(sid) / "export"; od.mkdir(exist_ok=True)
    out = od / f"{p.stem}_cleaned{p.suffix}"
    try:
        sf.write(out, y, sr, subtype=info.subtype, format=info.format)
    except Exception:                              # t.ex. MP3-skrivning saknas i libsndfile -> WAV
        out = od / f"{p.stem}_cleaned.wav"; sf.write(out, y, sr, subtype="PCM_24")
    return FileResponse(out, filename=out.name)


app.mount("/", StaticFiles(directory=ROOT / "static", html=True))
