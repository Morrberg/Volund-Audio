# Vocal Cleaner

Lokalt, gratis verktyg som hjälper dig hitta och tysta oönskade vokala inslag
("Woooow", "Yeah!", skrik m.m.) i sångspåret från AI-genererad musik (t.ex. Suno stems).

- Körs lokalt på Windows 11, inga molntjänster, ingen GPU krävs.
- En enda `.exe` (Rust). Gränssnittet öppnas i din webbläsare via en lokal server.
- Icke-destruktivt: originalfiler skrivs aldrig över, export ger en ny fil.
- Systemet **föreslår** redigeringar. Inget tas bort utan ditt godkännande.

## Status

Tidig version. Detektionen är heuristisk (energi + kontext: korta, isolerade vokala
block nära låtens början/slut), inte en AI-modell. Säkerhetsprocenten är inte kalibrerad.
Planerat: klassificering med öppna ljudmodeller, FLAC/MP3-export, eget fönster.

## Använda

1. Hämta `vocal-cleaner.exe` under **Releases** (eller **Actions → senaste bygget → Artifacts**).
2. Dubbelklicka. Webbläsaren öppnas.
3. Ladda upp ett vocals-stem, flera stems eller en ZIP (WAV/MP3/FLAC).
4. Klicka **Hitta förslag**, lyssna, **Godkänn** eller **Avvisa**. Markera även manuellt
   i vågformen (Tysta, Fade-in, Fade-out, Ångra/Gör om, A/B mot original).
5. **Exportera sångspår**. Resultatet blir en ny WAV (32-bit float) med samma
   samplingsfrekvens och kanaler. Lägg tillbaka den i din DAW tillsammans med övriga stems.

Reglage: dra = markera, klick = position, mushjul = zooma, Skift+hjul = panorera.

## Bygga själv

Installera Rust (https://rustup.rs), sedan:

```
cargo build --release
```

Resultat: `target/release/vocal-cleaner.exe`. Kör `cargo generate-lockfile` eller bygg en
gång och committa `Cargo.lock`. Att tagga `v0.1.0` och pusha bygger och publicerar
en release automatiskt via GitHub Actions.

## Mappar

- `src/main.rs`, `static/index.html`: Rust-appen (UI inbäddat i exe-filen).
- `prototype-python/`: tidigare Python-prototyp med samma UI (`start.bat`).

## Licens

MIT. Beroenden (axum, symphonia, hound, zip m.fl.) har egna öppna licenser.
