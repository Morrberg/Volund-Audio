# Vocal Cleaner 0.3

Lokalt verktyg för att tysta oönskade vokala inslag i Suno-stems.

## Windows

Dubbelklicka på `vocal-cleaner.exe`. Appen öppnar ett eget fönster och stängs när
fönstret stängs. Microsoft Edge WebView2 Runtime behöver finnas installerad.
Gränssnittet använder en inbäddad webbvy; ljudbehandlingen körs lokalt i Rust.
Inget ljud skickas till en molntjänst. På andra operativsystem öppnas webbläsaren.

1. Öppna Sunos ZIP med alla stems, eller WAV/MP3/FLAC-filer.
2. Alla stems visas samtidigt på en gemensam tidslinje. Klicka på ett spår för att redigera det. Zoom och panorering gäller alla spår. Välj sångspår. Markera, tysta, tona in/ut och lyssna på resultatet.
3. Spåren sorteras efter instrumentnamn (sång, bakgrundssång, trummor, percussion, bas, gitarr, klaviatur, synth, stråkar, blås, effekter och övrigt), med naturlig nummersortering inom gruppen. Kontrollraden följer det aktiva spåret. Byt mellan stems: varje spår behåller sina redigeringar och sin ångrahistorik.
4. **Spara sångspår…** (Ctrl+S) sparar det valda redigerade spåret som WAV,
   32-bit float med originalets samplingsfrekvens och antal kanaler.
   Välj en ny fil, exempelvis `Lead Vocals_cleaned.wav`.
5. Öppna **Metadata för valt spår** för att se stödda textfält, taggformat och antal inbäddade bilder/binära fält. Ändra eller ta bort textfält, lägg till nya, rensa metadata eller återställ från original. Ändringarna skrivs till WAV-exporten och sparas i projektet. Stödda metadatafält skrivs som ID3 i WAV; spelare kan visa olika mycket av dem. Originalets tekniska ljudegenskaper och filer behålls. Projektets originalljud innehåller fortfarande originalmetadata, tillsammans med de sparade metadataändringarna.
6. **Spara projekt…** sparar alla importerade stems, redigeringar och ångrahistorik
   i en `.vcleaner`-fil. Öppna den via samma filväljare för att fortsätta senare.

Att spara WAV sparar inte hela projektet. Projektfiler kan bli stora eftersom
originalens ljuddata ingår. Avbruten spardialog behåller redigeringarna i appen.
Programfönstret varnar vid osparade projektändringar. Stängning sparar inte automatiskt.

Dra = markera, klick = position, hjul = zooma, Skift+hjul = panorera.
Detektionen använder energi och kontextregler, inte en AI-modell.

## Bygga

Installera Rust och kör `cargo build --release`.
Windows-resultat: `target/release/vocal-cleaner.exe`.
GitHub Actions bygger och testar Windows-versionen och laddar upp exe som artifact.
`node tests/editor.test.cjs` testar stembyte, ångra och projektets sparstatus.

Vågformernas toppvärden cachas. Extra minne för själva flerspårsvyn är litet; avkodat ljud dominerar minnesåtgången. Spår med redigeringar får extra ljudbuffertar för förhandslyssning. Spår utan redigeringar återanvänder originalbufferten.

MIT-licens.

## Utseende och ikon

Mörkt och ljust tema kan växlas i programmet. Spårtyperna har olika vågformsfärger, det aktiva spåret markeras tydligt och markerad tid/längd visas vid kontrollerna.

`assets/icon.svg` är ikonens vektorkälla. `icon.ico` bäddas in i Windows-exe via `build.rs`, och `icon.rgba` används för fönsterikonen.

Windows-CI testar gränssnittet och Rust-API:t i headless Microsoft Edge med syntetiskt ljud; skärmbilder av båda temana sparas som artifact. Serverläget som används av testet aktiveras endast via testmiljövariabler. Själva Windows-fönstret och native spardialoger behöver även provköras manuellt.
