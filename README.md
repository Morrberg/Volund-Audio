# Völund Audio 0.7

**Smid fram ditt sound.**

Lokal ljudeditor för stems, stereomix och mastering. Uppkallad efter mästersmeden Völund: här smider vi låtar.

Tidigare namn: Vocal Cleaner. Befintliga `.vcleaner`-projekt öppnas och sparas som förut. Ljudformat, projektstruktur och tidigare språk-/temainställningar behålls.

## Windows

Dubbelklicka på `volund-audio.exe`. Appen öppnar ett eget fönster och stängs när
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
Windows-resultat: `target/release/volund-audio.exe`.
GitHub Actions bygger och testar Windows-versionen och laddar upp exe som artifact.
`node tests/editor.test.cjs` testar stembyte, ångra och projektets sparstatus.

Vågformernas toppvärden cachas. Extra minne för själva flerspårsvyn är litet; avkodat ljud dominerar minnesåtgången. Spår med redigeringar får extra ljudbuffertar för förhandslyssning. Spår utan redigeringar återanvänder originalbufferten.

MIT-licens.

## Utseende och ikon

Mörkt och ljust tema kan växlas i programmet. Spårtyperna har olika vågformsfärger, det aktiva spåret markeras tydligt och markerad tid/längd visas vid kontrollerna.

`assets/icon.svg` är ikonens vektorkälla. `icon.ico` bäddas in i Windows-exe via `build.rs`, och `icon.rgba` används för fönsterikonen.

Windows-CI testar gränssnittet och Rust-API:t i headless Microsoft Edge med syntetiskt ljud; skärmbilder av båda temana sparas som artifact. Serverläget som används av testet aktiveras endast via testmiljövariabler. Själva Windows-fönstret och native spardialoger behöver även provköras manuellt.

## 0.4 — mix och masteringstudio

1. Öppna stems och redigera dem. Justera varje spårs nivå (−60 till +12 dB), mute och solo. Dessa val gäller mixlyssning och mastering, och sparas i projektet. En vanlig export av valt spår behåller spårets egna nivåer.
2. Öppna **Masteringstudio**, ange låttitel/artist/album och välj Spotify, Vinyl, CD eller Kassett. Varje profil har separata inställningar för trebandig EQ, kompression, limiter, LUFS-mål, true-peak-gräns, lågskärning, smal djupbas, samplingsfrekvens och bitdjup. Äldre projekt öppnas med standardprofiler.
3. **Analysera & föreslå** mäter hela mixen och den valda profilen. Råd är enkla signalbaserade regler; ingen AI och ingen automatisk ändring av inställningar. LUFS och LRA mäts med Rust-biblioteket `ebur128` (EBU R128); true peak använder bibliotekets interpolerade mätning. Mycket kort eller tyst ljud kan sakna LUFS-värde.
4. **Skapa förhandslyssning** bearbetar hela låten. Växla Före/Efter vid samma position. Volymmatchning sänker den starkare versionen och playback håller även toppmarginal. Ändrad mix eller profil kräver en ny förhandslyssning. Export är inte beroende av förhandslyssningen.
5. **Spara valda versioner som ZIP** ger exempelvis `Låt 1 - mastered - Spotify.wav` och `Låt 1 - mastered - Vinyl.wav`, plus `Mastering-report.json` med inställningar och uppmätta resultat för varje version. **Spara aktuell profil som WAV** sparar en enda version. Windows använder native spardialoger. Projektets titel/artist/album skrivs till masterns WAV-taggar; metadata från en enskild stem kopieras inte till stereomixen.
6. **Spara projekt** bevarar mixval, samtliga masteringprofiler och redigeringar tillsammans med originalljudet. Att exportera WAV/ZIP sparar inte projektet.

Alla profiler utgår från samma mix före mastering. Omsampling använder bandbegränsad, symmetrisk windowed-sinc med fast ljudlängd; mono dupliceras till stereo. Blandade samplingsfrekvenser stöds, fler än två kanaler avvisas. Processen är offline och använder en stem åt gången på serversidan, plus mix och bearbetningsbuffert. Långa projekt och exportarkiv kräver fortfarande RAM. Mixen får vara högst 60 minuter och 64 stems.

Kompressorn är stereolänkad med mjukt knä och 25 ms attack/150 ms release. Limitern är stereolänkad med 5 ms lookahead och 80 ms release. Nivåhöjning begränsas till 12 dB; toppgränsen prioriteras över LUFS-målet. Efter limiter och slutlig TPDF-dither mäts true peak igen; vid behov sänks hela signalen. Rapporten visar faktisk nivå, maximal kompressor/limiterreduktion och säkerhetssänkning. Export använder 16/24-bit PCM, med dither vid slutlig heltalskvantisering; exakt digital tystnad behålls.

### Profiler och praktiska gränser

- **Spotify:** standard −14 LUFS / −1 dBTP, justerbart. −14 LUFS är en rekommenderad utgångspunkt, inget obligatoriskt leveransmål. En starkare master ger råd om −2 dBTP-marginal. Källan är Spotifys [masteringråd](https://support.spotify.com/us/artists/article/loudness-normalization/).
- **Vinyl:** inget LUFS-mål som standard, ingen kompression eller limiter, −3 dBTP, lågskärning vid 20 Hz och smalare stereo under 120 Hz. Inga skarpa påståenden om "vinylsäker" export: sidlängd, låtordning, sibilans och skärverkets bedömning återstår. Funktionen är inte en RIAA-encoder eller en skärsimulator. Stereo i basen och diskantenergi är enkla indikatorer, ingen sibilansdetektor.
- **CD:** 16 bit / 44,1 kHz med dither. Filerna är WAV-ljud; DDP, PQ-index, CD-Text och en färdig pressningsmaster ingår inte.
- **Kassett:** inget LUFS-mål som standard. 24 bit / 44,1 kHz som justerbar utgångspunkt. Ingen tapesimulering, Dolby-kodning eller förkompensation som kan bli fel för dupliceringsföretagets utrustning. Kontrollera [leveranskrav hos tillverkaren](https://www.duplication.com/upload.php).

### Album: sida A och B

Öppna färdiga WAV/FLAC-masterfiler i **Album** via dess egen filväljare. Välj ordning, sida och sekunder före varje låt (sidans första låt får inget automatiskt mellanrum). Export ger en WAV per använd sida, tidsatt låtlista och JSON-rapport. Ingen ny EQ, kompression eller normalisering görs; omsampling och slutlig dither kan behövas. Sammanställningen sänks som helhet endast om −1 dBTP överskrids. Detta är ett leveransunderlag, inte en garanterad skär- eller dupliceringsmaster. Spara ett **sidprojekt** och återöppna det med Albumets filväljare för att bevara ordning, sidor, mellanrum och originalljud. Vinyl/kassett bör sammanställas från respektive masterversion, inte streamingmastern.

Testerna täcker referens-LUFS, true peak även vid transienter/hög diskant, omsamplingens nivå och aliasdämpning, stereo, tystade partier, profilers PCM-format, filnamn, ZIP-rapporter och sidexport. Windows UI-testet kontrollerar profilval, mixkontroller, förhandslyssning, samtliga fyra exporter, metadata och projektåteröppning. Native programfönster/spardialoger behöver fortfarande provköras manuellt.

## 0.5 — panorering av markeringar

Markera ett parti i ett spår och välj **Vänster**, **Center**, **Höger**, en glidning mellan sidorna eller **Pendla vänster ↔ höger** (1–32 hela varv över markeringen). Reglaget ger valfri fast position mellan sidorna. Bara det markerade partiet på det valda spåret påverkas. Markeringens första/sista 10 ms övergår mjukt från/till spårets tidigare ljudbild. Andra spår ändras inte.

Panorering sparas som en redigering, med ångra/gör om och en lista där en enskild panorering kan tas bort. Vid överlapp används den senaste panoreringen i överlappet; Center kan därför återställa ett redan panorerat parti. Center behåller originalets naturliga stereobild, det gör inte stereoljud till mono. Vågformen visar båda kanalernas toppar tillsammans; lila markeringar visar panorerade partier.

Stereopan använder equal-power-crossfeed: båda originalkanalerna kan flyttas till samma sida utan att den ena kanalens innehåll kastas bort. Mono med panorering exporteras som stereo med equal-power-gain (center motsvarar originalets mono i båda högtalarna). Utan panorering behåller vanlig stemexport originalets kanalantal. Summering till en sida kan höja topparna; sänk spårnivån vid behov och kontrollera masteringens mätvärden. Ljud som redan är i motfas kan släcka ut varandra vid summering.

Samma beräkningar används i vanlig förhandslyssning, WAV-stemexport, masteringens mix och samtliga masterprofiler. Vinylprofilens inställning för smal djupbas kan minska panoreringens bredd i de lägsta frekvenserna.


## 0.6 — klipp i mixen, språk och Om

Öppna **Mix · klipp före mastering** efter att du justerat stems. En ensam WAV kan öppnas via samma import. Dra i mixens vågform eller ange start/slut i sekunder. **Klipp bort markering** tar bort tiden och sammanfogar resten; **Behåll bara markering** trimmar bort allt utanför. **Ångra klipp / Gör om klipp** är en egen historik. Klippen använder den aktuella mixens tidspositioner och 5 ms toningar vid skarvar/trimkanter utan att ändra den beräknade längden ytterligare.

Stems behåller sin längd, synk och redigeringar. Mixens klipp används före alla masteringprofiler, inklusive Före/Efter och analys. **Spara redigerad mix** exporterar en 24-bitars stereo-WAV före mastering vid den valda profilens samplingsfrekvens. Sänk spårnivåerna om PCM-exporten rapporterar toppar över 0 dBFS. Spara projekt för att behålla klipp, ångra/gör om och originalfiler. Äldre projekt öppnas utan mixklipp.

Välj **Svenska / English** i överkanten; språkvalet sparas på datorn. Filnamn och metadata ändras inte när du byter språk. **Om / About** visar version, syfte och licens.

English: Open **Mix · cut before mastering**, select a region and choose **Cut selection** or **Keep selection only**. Undo/redo affects only mix cuts. Original stems remain synchronized. Cuts are saved in the project and applied before every mastering profile. Export the edited premaster as a 24-bit stereo WAV or export the separate mastered versions.


## English

**Völund Audio — Forge your sound.** A local stem editor, stereo mix editor and mastering tool, named after the legendary master smith Völund. Previously called Vocal Cleaner. Run `volund-audio.exe` on Windows. Existing `.vcleaner` projects remain compatible.
