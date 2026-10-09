# Vocal Cleaner 0.2

Lokalt verktyg för att tysta oönskade vokala inslag i Suno-stems.

## Windows

Dubbelklicka på `vocal-cleaner.exe`. Appen öppnar ett eget fönster och stängs när
fönstret stängs. Microsoft Edge WebView2 Runtime behöver finnas installerad.
Gränssnittet använder en inbäddad webbvy; ljudbehandlingen körs lokalt i Rust.
Inget ljud skickas till en molntjänst. På andra operativsystem öppnas webbläsaren.

1. Öppna Sunos ZIP med alla stems, eller WAV/MP3/FLAC-filer.
2. Alla stems visas samtidigt på en gemensam tidslinje. Klicka på ett spår för att redigera det. Zoom och panorering gäller alla spår. Välj sångspår. Markera, tysta, tona in/ut och lyssna på resultatet.
3. Byt mellan stems: varje spår behåller sina redigeringar och sin ångrahistorik.
4. **Spara sångspår…** (Ctrl+S) sparar det valda redigerade spåret som WAV,
   32-bit float med originalets samplingsfrekvens och antal kanaler.
   Välj en ny fil, exempelvis `Lead Vocals_cleaned.wav`.
5. **Spara projekt…** sparar alla importerade stems, redigeringar och ångrahistorik
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
