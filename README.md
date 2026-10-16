<p align="center"><img src="assets/volund-emblem.jpg" width="300" alt="Völund Audio: vingat Völund-symbol i antik brons"></p>

# Völund Audio

**Smid fram ditt sound.**

En lokal ljudeditor för stems, stereomix, mastering och vinylrestaurering. Namnet kommer från mästersmeden Völund: här smider vi låtar.

**Version 0.9.0 · Tauri-port under test för Windows, macOS och Linux · Svenskt och engelskt gränssnitt · MIT-licens**

[Kom igång](#kom-igång) · [Läs manualen](docs/MANUAL.sv.md) · [Vinylrestaurering och låtuppdelning](docs/MANUAL.sv.md#arbetsomraden-och-vinylrestaurering) · [Arbeta med ett album](docs/MANUAL.sv.md#album) · [Bygg från källkod](#bygga-och-testa)

## Vad kan programmet göra?

| Del | Funktioner |
|---|---|
| **Stems** | Öppna Suno-ZIP, WAV, MP3 eller FLAC. Visa alla spår på samma tidslinje. Tysta, tona och panorera markerade partier med ångra/gör om per spår. |
| **Mix** | Justera nivå, mute och solo per spår. Klipp bort eller behåll delar av den sammanlagda stereomixen före mastering. |
| **Masteringstudio** | Separata profiler för Spotify, Vinyl, CD och Kassett. EQ, kompressor, limiter, LUFS/true-peak-mätning och före/efter-lyssning. |
| **Album** | Nivåmatcha färdiga låtar med valbara undantag och egna justeringar. Exportera separata låtar eller sammanställ sida A/B med låtlistor. |
| **Vinylrestaurering** | Minska klick, knaster och rumble. Hitta tystnader som möjliga låtgränser, justera markörerna och exportera uppdelade spår. Separera även en stereomix i trummor, bas, övrigt och sång med Demucs-RS. |
| **Metadata och projekt** | Visa, ändra och rensa metadata. Spara originalfiler och redigeringar i ett återöppningsbart projekt. |

Ljudbehandlingen körs lokalt på datorn. Demucs är en separat, valfri funktion: Windows-testpaketet innehåller motorn men inte modellvikterna. När du själv startar separationen hämtas vikterna från Hugging Face; ljudet skickas inte dit. Motorn paketeras för närvarande bara med Windows-testpaketet. Ingen AI-prenumeration behövs. Förslagen för oönskade ljud och masteringråden bygger på signalanalys, inte generativ AI.

## Kom igång

1. Öppna [GitHub Actions](https://github.com/Morrberg/Vocal-cleaner/actions) och välj en lyckad körning för den version du vill använda.
2. Hämta **Volund-Audio-Windows** under **Artifacts** och packa upp ZIP-filen. GitHub kan kräva inloggning för nedladdningen.
3. Starta `volund-audio.exe`. Windows-versionen använder ett eget programfönster med inbäddad webbvy och kräver [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
4. Välj **Öppna stems / projekt** och öppna stems för **en låt**, exempelvis Sunos ZIP-fil.
5. Klicka på ett spår, markera ett parti och välj exempelvis **Tysta**. Lyssna på resultatet och välj **Spara projekt…** för att kunna fortsätta senare.

Programmet är osignerat, så Windows SmartScreen kan visa en varning. Se [manualens startavsnitt](docs/MANUAL.sv.md#starta) för hur du kontrollerar källan och går vidare med ett betrott bygge.

## Ett vanligt arbetsflöde

1. **Redigera stems:** ta bort ett extra sångljud genom att tysta just det partiet; de andra spåren behåller sin synk.
2. **Forma mixen:** balansera spårnivåerna och lägg eventuell panorering. Klipp intro eller outro i **Mix · klipp före mastering**.
3. **Skapa utgåvor:** justera masteringprofilerna, jämför före/efter och exportera de versioner du behöver.
4. **Samla albumet:** öppna färdiga låtar från **samma profil** i albumdelens egen filväljare. Nivåmatcha och exportera låtar eller sida A/B.

Albumdelen använder kompletta låtar som underlag. Att öppna flera låtar som stems i huvudvyn skulle i stället lägga dem ovanpå varandra i en mix.

## Spara rätt sak

| Knapp | Resultat |
|---|---|
| **Spara valt spår…** / **Ctrl+S** | En redigerad WAV av det aktiva spåret. |
| **Spara redigerad mix…** | En stereomix före mastering. |
| **Spara aktuell profil som WAV…** / **Spara valda versioner som ZIP…** | Färdiga masterfiler; ZIP-exporten innehåller även en mätrapport. |
| **Spara projekt…** | Originalljud, stemredigeringar, mixklipp, mixnivåer och masteringinställningar i en `.vcleaner`-fil. |
| **Spara sidprojekt…** | Albumets originallåtar, ordning, sidor och nivåmatchningsinställningar i en `.vcleaner`-fil. |

**Att exportera ljud sparar inte projektet.** Programmet har ingen automatisk projektsparning eller kraschåterställning. Originalfilerna på datorn skrivs inte över av redigeringarna; välj ett nytt filnamn när du exporterar.

## Manual och exempel

[Den svenska manualen](docs/MANUAL.sv.md) går igenom kontrollerna och innehåller exempel på att:

- tysta ett oönskat ”Wrooow” utan att rubba stemsens synk;
- låta trummor pendla mellan vänster och höger;
- korta ett intro i mixen före alla masterversioner;
- exportera samma låt för flera format;
- nivåmatcha ett album och behålla ett avsiktligt lugnare intro.

Profilernas standardvärden är utgångspunkter, inte garantier för leverans till en viss tjänst eller tillverkare. Vinyl och kassett behöver anpassas till mottagarens krav. CD-exporten är WAV-ljud, inte en komplett DDP-master. Albumnivåmatchningen jämnar ut ljudstyrka; den gör ingen automatisk matchning av EQ eller klang.

## Bygga och testa

Installera Rust och Node.js för respektive kontroll:

```sh
cargo test
node tests/editor.test.cjs
cargo build --release
```

`cargo build --release` bygger Tauri-skrivbordsappen för Windows, Linux och macOS. Linux kräver Tauri/WebKitGTK-systempaketen som anges i GitHub Actions-flödet. FreeBSD använder webbläsarfallbacken tills skrivbordsbygget har verifierats där.

[GitHub Actions-flödet](.github/workflows/rust.yml) kör Rust-tester på Linux och Windows samt gränssnitts- och exporttester i Microsoft Edge på Windows. Flödet sparar även skärmbilder som artifact. Det inbäddade Windows-fönstret och de vanliga spardialogerna behöver kompletteras med manuell provkörning.

## Projektet

Programmet hette tidigare **Vocal Cleaner**. Äldre `.vcleaner`-projekt stöds fortfarande. GitHub-repot behåller tills vidare namnet `Vocal-cleaner`.

Koden omfattas av [MIT-licensen](LICENSE). Programikonen finns i `icons/icon.png` och `icons/icon.ico`; den större Völund-illustrationen finns i `assets/volund-emblem.jpg`.
