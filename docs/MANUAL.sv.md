# Völund Audio – användarmanual

<p align="center"><img src="../assets/volund-emblem.jpg" width="360" alt="Vingad Völund-symbol i antik brons"></p>

**För version 0.9 · Smid fram ditt sound.**

Manualen använder programmets svenska knappnamn. Välj **Svenska** i överkanten om du vill följa instruktionerna ord för ord. Gränssnittet kan också visas på engelska.

[Till README](../README.md)

<a id="innehall"></a>
## Innehåll

1. [Förstå arbetsflödet](#arbetsflode)
2. [Starta och öppna filer](#starta)
3. [Tidslinje och uppspelning](#tidslinje)
4. [Tysta, tona och ångra](#redigera)
5. [Hitta förslag på oönskade ljud](#forslag)
6. [Panorera ett markerat parti](#panorera)
7. [Balansera spårens nivåer](#balansera)
8. [Klippa och trimma stereomixen](#klippa)
9. [Visa och ändra metadata](#metadata)
10. [Mastera en låt](#mastera)
11. [Exportera masterversioner](#masterexport)
12. [Nivåmatcha ett album](#album)
13. [Sammanställa sida A och B](#sidor)
14. [Spara och återöppna projekt](#projekt)
15. [Felsökning och begränsningar](#felsokning)
16. [Ordlista](#ordlista)

<a id="arbetsflode"></a>
## 1. Förstå arbetsflödet

Völund Audio har tre arbetsnivåer:

| Nivå | Du arbetar med | Exempel |
|---|---|---|
| **Stems** | Separata instrument- och sångspår från en låt. | Tysta ett extra sångljud eller panorera ett syntparti. |
| **Mix och mastering** | Summan av spåren som en stereolåt. | Korta introt, justera klang och skapa olika utgåvor. |
| **Album** | Flera färdiga stereolåtar från samma utgåvetyp. | Jämna ut ljudstyrkan mellan CD-versionerna av åtta låtar. |

**En låt i taget i stemsvyn; flera kompletta låtar i albumdelen.** Om du öppnar två olika låtar som stems summeras de och spelar samtidigt.

Ett lämpligt flöde är: **redigera stems → balansera mixen → klipp mixen → mastera → nivåmatcha albumet**. Spara ett låtprojekt under arbetet och ett separat sidprojekt för albumet.

<a id="starta"></a>
## 2. Starta och öppna filer

### Windows

1. Packa upp program-ZIP:en och starta `volund-audio.exe`.
2. Programmet behöver Microsoft Edge WebView2 Runtime. Om fönstret inte kan öppnas, kontrollera att den är installerad.
3. Välj språk och ljust eller mörkt tema i överkanten. **Om / About** visar version och information om programmet.

Windows SmartScreen kan reagera på det osignerade programmet. Kontrollera att filen kommer från det avsedda bygget i projektets [GitHub Actions](https://github.com/Morrberg/Vocal-cleaner/actions). För ett bygge du har verifierat och litar på kan du välja **Mer information → Kör ändå**, om Windows erbjuder det. Du behöver inte stänga av SmartScreen generellt. Se även [Microsofts beskrivning](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

### Importera en låt

Använd **Öppna stems / projekt** för:

- WAV, MP3 eller FLAC, en eller flera filer från samma låt;
- ZIP med exempelvis Sunos stems;
- ett sparat låtprojekt med ändelsen `.vcleaner`.

Du kan öppna ZIP-filen direkt. Alla importerade spår visas och sorteras efter filnamnets troliga spårtyp: sång först, därefter bland annat trummor, bas, gitarr och klaviatur. Siffror sorteras naturligt, exempelvis `2 Guitar` före `10 Guitar`. Sorteringen analyserar namnet, inte vilket instrument som faktiskt hörs.

Använd unika filnamn. ZIP-importen tar bort katalogsökvägar; två filer som båda heter `Vocals.wav` blir därför inte två skilda spår.

En ensam WAV kan också öppnas här för klippning eller mastering. För ett album använder du i stället **Öppna färdiga låtar** i albumdelen; se [avsnitt 12](#album).

<a id="tidslinje"></a>
## 3. Tidslinje och uppspelning

Klicka på ett spår för att göra det aktivt. Dess vågform blir större och kontrollraden flyttar med. Att byta aktivt spår tar inte bort redigeringarna i de andra spåren.

| Åtgärd | Funktion |
|---|---|
| Dra över vågformen | Markera ett tidsområde på det aktiva spåret. |
| Klicka utan att dra | Välj uppspelningsposition och ta bort markeringen. |
| Mushjul över en stemvågform | Zooma kring muspekaren. Alla stems delar samma tidsvy. |
| Skift + mushjul | Flytta den synliga tidsvyn i sidled. Detta ändrar inte stereoplaceringen. |
| **Spela** / **Paus** | Starta eller pausa uppspelningen från vald position. |
| **Stopp** i stemkontrollerna | Stoppa och återgå till början. |
| **Spela markering** | Lyssna på det markerade tidsområdet. |
| **Loop markering** | Vid vanlig uppspelning upprepas markeringen. |

Uppspelningen använder alla hörbara stems. Vill du höra endast ett spår väljer du **Solo** på det spåret. Flera spår kan vara solo samtidigt.

<a id="redigera"></a>
## 4. Tysta, tona och ångra

Markera ett parti och välj:

| Kontroll | Resultat |
|---|---|
| **Tysta** | Gör partiet tyst och använder korta mjuka övergångar kring kanterna. Spårets längd behålls. |
| **Fade-in** | Tonar upp ljudet genom markeringen. |
| **Fade-out** | Tonar ned ljudet genom markeringen. |
| **Ångra / Gör om** | Ändrar redigeringshistoriken för det aktiva spåret. |
| **Visar: Redigerad / Original** | Växlar stemvågformer och stemlyssning mellan redigerat och ursprungligt ljud. |

Växlingen till **Original** tar inte bort redigeringarna och stänger inte av dem i export eller mastering. Vill du ändra vad som exporteras använder du **Ångra** eller gör om redigeringen.

### Exempel: ta bort ett extra ”Wrooow”

1. Öppna låtens Suno-ZIP och välj sångspåret.
2. Aktivera **Solo** för att lättare höra vad som händer.
3. Zooma in och markera det oönskade ljudet, exempelvis 0,8–2,1 sekunder.
4. Välj **Tysta**, lyssna och ångra om markeringen blev fel.
5. Stäng av **Solo** och lyssna med instrumenten igen.
6. Spara projektet. Exportera även spåret om du vill fortsätta i ett annat musikprogram.

Tystning lämnar en lucka i sångspåret i stället för att flytta resten av sången. Därför behåller sången sin placering mot trummor och instrument.

<a id="forslag"></a>
## 5. Hitta förslag på oönskade ljud

**Hitta förslag** söker på det aktiva spåret efter korta, isolerade ljud nära början eller slutet. Funktionen använder ljudenergi och tysta mellanrum. Den förstår inte texten eller om ett ljud är musikaliskt önskat.

För varje förslag kan du välja **Lyssna**, **Godkänn (tysta)** eller **Avvisa**. Godkännande lägger till en vanlig tystning som kan ångras. Sökningen i sig ändrar inget.

Ett förslag kan vara ett önskat rop eller en riktig sångfras. Lyssna därför innan du godkänner. Inga förslag betyder inte att hela spåret är fritt från oönskade ljud.

<a id="panorera"></a>
## 6. Panorera ett markerat parti

Panoreringen gäller allt ljud i markeringen på **just det aktiva spåret**. Sångspåret ändras exempelvis inte när du panorerar ett syntspår.

| Val | Funktion |
|---|---|
| **Vänster / Höger** | Placera partiet på vald sida. |
| **Center** | Behåll eller återställ partiets naturliga stereobild; det är inte en omvandling till mono. |
| **Position + Placera** | Välj en fast mellanposition med reglaget. |
| **Vänster → höger / Höger → vänster** | Låt ljudet glida över stereobilden under markeringen. |
| **Pendla vänster ↔ höger** | Låt ljudet pendla ett valt antal hela varv, 1–32, under markeringen. |

Övergångarna vid kanterna är mjuka. Panoreringarna visas i en lista där de också kan tas bort separat, och markeras med lila streck i vågformen. Vid överlapp får den senare panoreringen företräde med mjuka övergångar.

### Exempel: trummor som pendlar

Markera 20–24 sekunder i trumspåret, välj **Varv: 2** och **Pendla vänster ↔ höger**. Under de fyra sekunderna gör trummorna två hela pendlingar. Övriga spår behåller sina stereoplaceringar.

Vill du i stället placera olika trumslag på olika sidor, markera slagen var för sig och välj vänster respektive höger.

Båda kanalerna i ett stereospår kan flyttas till samma sida. Summeringen kan höja topparna; sänk spårnivån vid behov. Om flera ljud ligger samtidigt i samma stem går de inte att separera med panoreringen.

<a id="balansera"></a>
## 7. Balansera spårens nivåer

Varje spår har en nivåkontroll i dB, **Mute** och **Solo**.

- **0 dB** behåller nivån. Ett negativt värde sänker och ett positivt höjer; intervallet är −60 till +12 dB.
- **Mute** tar bort hela spåret från mixlyssning och mixexport. Det skiljer sig från **Tysta**, som redigerar en markering.
- **Solo** gör att bara solovalda, omutade spår hörs. Ett mutat spår hörs inte även om Solo är valt.

**Exempel:** sänk sången till −2 dB om den ligger för långt fram och gitarrerna till −1 dB om mixen blir för tät. Värdena är bara exempel; lyssna på din egen låt.

Nivå, mute och solo följer med till stereomixen och mastering. Kontrollera därför att inga tillfälliga soloval finns kvar före export. **Exportera valt spår som WAV…** använder spårets redigeringar, men bakar inte in dessa mixnivåer eller mute/solo.

<a id="klippa"></a>
## 8. Klippa och trimma stereomixen

Öppna **Mix · klipp före mastering** och välj **Uppdatera mix** när underlaget har ändrats. Vyn summerar dina redigerade stems med deras mixnivåer, mute och solo. En ensam importerad WAV fungerar också.

1. Dra över mixvågformen eller ange **Start (s)** och **Slut (s)**.
2. Välj **Klipp bort markering** för att ta bort tiden och foga ihop resten.
3. Välj **Behåll bara markering** för att ta bort allt utanför området.
4. Använd **Ångra klipp / Gör om klipp** för mixens separata historik.

**Exempel:** en mix är 180 sekunder. Klipp bort 0–4 sekunder för att korta introt; mixen blir 176 sekunder. Nästa klipp anges på den nya, kortare tidslinjen.

Klippen påverkar samma stereomix före **alla** masteringprofiler. Stemsens egna tidslinjer ändras inte. Korta toningar på cirka 5 ms vid skarvar och trimkanter minskar risken för klick utan att lägga till ytterligare överlapp eller förkortning. Hela mixen kan inte klippas bort.

**Spela mix** spelar markeringen om en sådan finns, annars hela mixen. **Spara redigerad mix…** ger en 24-bitars stereo-WAV före mastering vid den valda masteringprofilens samplingsfrekvens. Det gäller även om CD-profilen är vald; CD-mastern exporteras separat som 16 bit.

<a id="metadata"></a>
## 9. Visa och ändra metadata

Öppna **Metadata för valt spår**. Här visas bland annat titel, artist, album, datum, genre, spårnummer, kommentar, ISRC och låttext när de finns och formatet stöds.

Du kan redigera värden, **Lägg till fält**, ta bort ett fält, **Rensa metadata** eller **Återställ från original**. Panelen visar även information om inbäddade bilder och fält som bara kan läsas.

Ändringarna gäller stemmens exporterade WAV och sparas i projektet. WAV-stödda taggar följer med; andra spelare kan visa olika mycket av dem. Rensning tar även bort originaltaggar och inbäddade bilder från den exporterade filen.

**Originalfilen ändras inte.** Ett projekt innehåller fortfarande originalljudet med dess originalmetadata, även om du har valt att rensa exportens metadata.

Masteringmixens titel, artist och album fylls i i **Masteringstudio**. Dessa fält är separata från metadata på ett enskilt stem. Albumexport av färdiga låtar bevarar deras stödda originalmetadata.

<a id="mastera"></a>
## 10. Mastera en låt

### Börja med en balanserad mix

1. Kontrollera stems, nivåer, mute/solo och eventuella mixklipp.
2. Öppna **Masteringstudio · en mix, flera utgåvor**.
3. Fyll i låttitel, artist och album.
4. Välj **Profil att redigera**. Varje profil har egna inställningar som sparas i låtprojektet.

Alla profiler utgår från samma mix **före mastering**. Vinylversionen är alltså inte en vidarebearbetning av Spotify-versionen.

### Profilernas standardvärden

Tabellen visar programmets utgångspunkter efter **Återställ profilen**, inte universella leveranskrav.

| Inställning | Spotify | Vinyl | CD | Kassett |
|---|---|---|---|---|
| Samplingsfrekvens | 44,1 kHz | 44,1 kHz | 44,1 kHz, låst | 44,1 kHz |
| Bitdjup | 24 bit | 24 bit | 16 bit, låst | 24 bit |
| LUFS-mål | −14 | Av | −14 | Av |
| True peak max | −1 dBTP | −3 dBTP | −1 dBTP | −1 dBTP |
| Kompression | 15 % | 0 % | 15 % | 15 % |
| Limiter | På | Av | På | På |
| Smal djupbas | Av | Under 120 Hz | Av | Av |
| Lågskärning | Av | 20 Hz | Av | Av |

EQ börjar på 0 dB i samtliga profiler. Utanför CD-profilen kan exportens samplingsfrekvens och bitdjup ändras. Tillgängliga masteringfrekvenser är 44,1, 48, 88,2 och 96 kHz; bitdjupen är 16 eller 24 bit.

### Kontrollerna

| Kontroll | Vad den gör |
|---|---|
| **Anpassa ljudstyrka till LUFS-mål** | Försöker nå valt integrerat ljudstyrkemål. Avstängt betyder att denna målstyrda nivåanpassning uteblir. |
| **True peak max** | Sätter toppgränsen. Den prioriteras framför LUFS-målet. |
| **Bas / Mellanregister / Diskant** | Justerar mixens klang med en enkel trebandig EQ. |
| **Kompression** | Jämnar ut dynamiken i stereomixen. Börja försiktigt om materialet redan är hårt komprimerat. |
| **Limiter** | Begränsar toppar. Även utan limiter kan hela signalen sänkas för att hålla toppgränsen. |
| **Smal djupbas under** | Minskar stereobredden i de lägsta frekvenserna. 0 stänger av funktionen. |
| **Lågskärning** | Dämpar mycket låga frekvenser. 0 stänger av funktionen. |
| **Varsam / Balanserad / Tryck** | Sätter kompressionen till 0/25/55 %. För Spotify och CD ändras också LUFS-målet till −16/−14/−10 och toppgränsen till −1/−1/−2 dBTP. |
| **Återställ profilen** | Återställer den valda profilens standardvärden. |

Snabbvalen återställer inte alla övriga inställningar, exempelvis EQ. Att ändra Vinyl eller Kassett påverkar inte Spotify eller CD.

### Analysera, lyssna och justera

**Analysera & föreslå** visar mätvärden och enkla råd. Knappen ändrar inte inställningarna automatiskt. **LUFS** beskriver ljudstyrka, **true peak** toppnivå och **LRA** variation i ljudstyrkan. Mycket kort eller tyst ljud kan sakna ett LUFS-värde.

Välj **Skapa förhandslyssning** och växla sedan mellan **Före** och **Efter** på samma position. Med **Matcha volym vid jämförelse** minskas volymskillnaden så att du lättare kan bedöma klang och dynamik. Det är en lyssningsfunktion och ändrar inte exportens nivå.

När mix, klipp eller profil ändras behöver förhandslyssningen skapas på nytt. Export kan ändå göras direkt; en förhandslyssning är inget krav.

**Exempel för en tät metallåt:** prova **Varsam**, skapa förhandslyssning och lyssna på virvelns attack och cymbalerna. Öka kompressionen bara om resultatet blir bättre. Om programmet rapporterar att LUFS-målet inte nås kan du välja ett lägre mål eller acceptera den uppmätta nivån. Programmet prioriterar toppgränsen och begränsar nivåhöjningen till +12 dB.

<a id="masterexport"></a>
## 11. Exportera masterversioner

Markera utgåvorna under **Exportera** och välj **Spara valda versioner som ZIP…**. Exempel:

```text
Låt 1 - mastered - Spotify.wav
Låt 1 - mastered - Vinyl.wav
Låt 1 - mastered - CD.wav
Låt 1 - mastered - Kassett.wav
Mastering-report.json
```

**Spara aktuell profil som WAV…** exporterar bara den valda profilen. Titel, artist och album från masteringstudion skrivs till WAV-taggarna. ZIP-rapporten innehåller inställningar, mätvärden och eventuella varningar.

Masterexport använder 16/24-bit PCM med slutlig dither. Att källan är MP3 gör inte en WAV-export mer detaljrik än originalet.

| Utgåva | Vad exporten är |
|---|---|
| **Spotify / streaming** | En WAV enligt den valda profilen. Följ distributörens aktuella leveranskrav. |
| **Vinyl** | Ett underlag för skärverket. Sidlängd, låtordning och vass diskant behöver bedömas där; programmet gör ingen RIAA-kodning eller skärsimulering. |
| **CD** | 16 bit/44,1 kHz WAV. DDP, PQ-index och CD-Text ingår inte. |
| **Kassett** | Ett digitalt underlag för duplicering. Ingen Dolby-kodning eller tapesimulering läggs till. |

Välj nya filnamn eller en ny plats vid export. Exporten ersätter inte **Spara projekt…**.

<a id="album"></a>
## 12. Nivåmatcha ett album

### Öppna rätt låtar

1. Öppna **Album · nivåmatchning och sida A/B**.
2. Välj **Öppna färdiga låtar** och läs in kompletta WAV/FLAC-låtar, en ZIP eller ett sparat sidprojekt.
3. Använd låtar från samma utgåvetyp, exempelvis alla Spotify-versioner. Gör en separat albumomgång för Vinyl, CD respektive Kassett.
4. Ange utgåvans namn, samplingsfrekvens och bitdjup. För en CD-ljudexport väljer du här själv **44,1 kHz och 16 bit**; albumdelen har ingen låst CD-profil.

En ny filimport börjar med nivåmatchning på. Äldre sidprojekt utan inställningen öppnas med matchning av, så att den inte läggs till oväntat.

### Så fungerar matchningen

Med **Matcha låtarnas upplevda ljudstyrka** aktiverat får varje matchad låt en konstant nivåändring. Låtens interna dynamik och klang behålls: ingen ny EQ, kompressor eller limiter läggs till.

Det automatiska målet är **medianen** av de matchade låtar som har mätbar LUFS. Du kan i stället välja **Eget LUFS-mål**. **Analysera albumnivåer** visar nivå före, beräknad nivå efter och justering i dB. Före/efter-knapparna spelar respektive låt; här får du höra själva nivåskillnaden.

### Exempel: tre låtar med olika ljudstyrka

Anta att det finns tillräcklig toppmarginal för justeringarna:

| Låt | Före | Justering | Efter |
|---|---:|---:|---:|
| Låt 1 | −10 LUFS | −6 dB | −16 LUFS |
| Låt 2 | −16 LUFS | 0 dB | −16 LUFS |
| Låt 3 | −22 LUFS | +6 dB | −16 LUFS |

Medianen är −16 LUFS. Ett separat intro på −28 LUFS kan lämnas lugnare genom att avmarkera dess **Matcha**. Det undantas då också från beräkningen av det automatiska målet.

Du kan dessutom ange en egen dB-justering per låt, exempelvis **−3 dB** för en låt som medvetet ska vara svagare efter matchningen. Intervallet är −12 till +12 dB. Låtar utan mätbar LUFS, exempelvis digital tystnad, matchas inte automatiskt.

### När hela albumet behöver sänkas

Om en låt annars skulle överskrida toppgränsen eller +12 dB total nivåhöjning sänks **alla låtar lika mycket**. Då bevaras den avsedda balansen i stället för att bara den begränsade låten blir för svag.

Om sänkningen exempelvis är 3 dB blir de tre låtarna i tabellen −19 LUFS. Ett undantaget intro på −28 LUFS blir då −31 LUFS. **Undantagen låt betyder alltså ingen automatisk nivåmatchning, inte undantag från albumets gemensamma säkerhetssänkning.** Egna dB-justeringar kan fortfarande tillämpas även när den automatiska matchningen är avstängd.

### Exportera och spara

**Spara nivåmatchade låtar som ZIP…** ger numrerade WAV-filer i albumordning, till exempel `01 - Låt 1 - mastered - CD - album.wav`, och `Album-report.json`. Rapporten innehåller faktiskt uppmätta värden efter konvertering och dither. Stödda originalmetadata bevaras.

Resultattabellen är en beräkning, exportens rapport är mätningen av de färdiga filerna. LUFS är ett stöd för att jämna ut nivån, inte en garanti för identisk subjektiv ljudstyrka. Lyssna på övergångarna och finjustera vid behov. Skillnader i EQ, täthet och arrangemang kan fortfarande höras.

Spara arbetet med **Spara sidprojekt…**. Se [projektöversikten](#projekt).

<a id="sidor"></a>
## 13. Sammanställa sida A och B

I albumets låtlista väljer du **Sida A** eller **Sida B** för varje låt. Flytta låtarna med pilarna. Ordningen inom respektive sida följer listan.

Sifferfältet intill sidvalet anger sekunder **före låten**, mellan 0 och 30. Sidans första låt får inget extra inledande mellanrum. Listan visar också beräknad sidlängd.

**Exempel:** välj tre låtar på A och tre på B. Sätt 2 sekunder före varje låt som ska ha ett mellanrum; första låtens fält ignoreras på respektive sida.

**Spara sida A/B och låtlistor som ZIP…** ger en WAV per använd sida, tidsatta låtlistor och `Side-report.json`. Samma nivåmatchning och undantag används som vid export av separata albumlåtar. Om själva skarvarna kräver ytterligare toppmarginal sänks båda sidor lika mycket; det framgår av rapporten.

Använd respektive vinyl- eller kassettversion som underlag och stäm av format och sidlängd med tillverkaren. Programmets gräns på 60 minuter per sida är en teknisk gräns, inte ett råd om hur lång en vinylsida bör vara.

<a id="projekt"></a>
## 14. Spara och återöppna projekt

| Vad du vill spara | Använd | Öppna igen med |
|---|---|---|
| En låts stems, redigeringar, ångra/gör om, metadataändringar, mixnivåer, mixklipp och masteringprofiler | **Spara projekt…** | **Öppna stems / projekt** |
| Ett albums originallåtar, ordning, sidor, mellanrum, matchningsmål, undantag och nivåjusteringar | **Spara sidprojekt…** | **Öppna färdiga låtar** i albumdelen |
| En färdig ljudfil för ett annat program eller en mottagare | Respektive WAV- eller ZIP-export | Importera ljudfilen där den ska användas |

Båda projekttyperna använder ändelsen `.vcleaner` och innehåller originalens ljuddata. De kan därför bli stora. Äldre projekt från Vocal Cleaner stöds fortfarande.

**Ctrl+S sparar projektet; Ctrl+Shift+E exporterar det aktiva spåret som WAV.** I albumets arbetsområde sparar Ctrl+S sidprojektet. Ctrl+S öppnar dialogrutan där du väljer var projektfilen ska sparas. Du kan också använda projektknapparna. Att exportera ett spår eller en master sparar inte projektets redigeringar och kan därför lämna osparade projektändringar.

Avbruten spardialog behåller arbetet i programmet. Stängningsvarningen sparar däremot inget automatiskt. Det finns ingen automatisk projektsparning eller kraschåterställning i version 0.9.

<a id="felsokning"></a>
## 15. Felsökning och begränsningar

| Symtom | Kontrollera eller prova |
|---|---|
| Bara ett instrument kommer med i mixen | Stäng av tillfälliga **Solo** och kontrollera **Mute**. |
| Ett parti blir tyst men låten blir inte kortare | **Tysta** behåller tiden. Använd mixvyns **Klipp bort markering** för att korta hela låten. |
| Mixvågformen är tom efter en ändring | Välj **Uppdatera mix**. Tidigare mixunderlag ogiltigförklaras när ljud eller profil ändras. |
| Före/efter-knapparna i mastering är inaktiva | Välj **Skapa förhandslyssning** igen efter ändringar. |
| Den exporterade mixen saknar dina mixnivåer | Kontrollera att du exporterade mix eller master, inte **Exportera valt spår som WAV…**. |
| Ett LUFS-mål nås inte | Läs mätvärden och varningar. Toppgräns och maximal nivåhöjning kan begränsa resultatet. |
| Ett undantaget intro sänks ändå | Albumets gemensamma säkerhetssänkning gäller även undantagna låtar. |
| En panorering påverkar flera ljud | De ligger i samma stem och tidsområde. Verktyget flyttar allt ljud i den markeringen. |
| Inga ljudförslag hittas | Markera manuellt. Detektorn söker bara vissa korta, isolerade ljud nära början/slutet. |
| Metadata syns inte i en annan spelare | Spelare stöder olika WAV-taggar. Kontrollera även att fältet kunde exporteras. |
| Redigeringarna saknas när du fortsätter | Öppna projektfilen, inte bara original-ZIP:en eller den exporterade WAV-filen. |
| Programmet känns tungt | Färre eller kortare ljudfiler minskar minnesbehovet. Ljudet avkodas till arbetsbuffertar även om källan är komprimerad. |

### Tekniska ramar

- Masteringmixen stöder högst **64 stems** och **60 minuter**. Mono och stereo stöds; mastering av flerkanalsljud avvisas.
- Albumlistan stöder högst **100 låtar**. En sammanställd sida får vara högst **60 minuter**.
- Filuppladdningen till den lokala ljudservern har en gräns på **2 GiB** per begäran. Avkodat ljud och exporter kan kräva betydligt mer arbetsminne än ZIP-filens storlek antyder.
- Som storleksjämförelse kräver fem minuters stereo vid 44,1 kHz ungefär **101 MiB** bara för en 32-bitars arbetsbuffert. Redigering, mixning och export kan kräva fler buffertar.
- **Spara redigerad mix…** kan avvisa PCM-export om mixens samples överstiger 0 dBFS. Sänk då spårnivåerna. Stemexport är däremot 32-bit float vid originalets samplingsfrekvens och är inte automatiskt toppbegränsad. Panorering kan göra ett monospår till stereo.
- Originalfilerna på datorn ändras inte av redigeringarna. En spardialog kan dock skriva över en befintlig fil om du själv väljer samma filnamn; välj nya namn för att behålla originalen.

<a id="ordlista"></a>
## 16. Ordlista

| Begrepp | Förklaring |
|---|---|
| **Stem** | Ett separat sång- eller instrumentspår från en låt. |
| **Mix** | Spåren summerade med deras nivåer och stereoplaceringar. |
| **Master** | En bearbetad slutversion av mixen för en viss utgåva. |
| **Panorering** | Var ljudet placeras mellan vänster och höger kanal. |
| **dB** | Ett relativt nivåmått. I spårkontrollerna betyder 0 dB ingen nivåändring. |
| **LUFS** | Ett mått på ljudstyrka. Ett mindre negativt värde är starkare. |
| **True peak / dBTP** | En uppskattning av toppnivån, även mellan digitala samples. |
| **LRA** | Loudness Range: ett mått på variationen i ljudstyrka. |
| **Kompressor** | Minskar skillnaden mellan starkare och svagare partier. |
| **Limiter** | Begränsar höga toppar. |
| **Samplingsfrekvens / kHz** | Hur många ljudsamples per sekund en fil har. |
| **Bitdjup / PCM** | Upplösning och heltalsformat för digitalt ljud, exempelvis 16- eller 24-bit PCM. |
| **Dither** | Ett mycket svagt brus som används vid slutlig heltalskvantisering för att minska kvantiseringsartefakter. |

[Tillbaka till innehållet](#innehall) · [Till README](../README.md)


## Trim Silence och De-clip (Tauri)

Trim Silence finns i mixredigeringen. Tröskeln mäts som absolut samplingsnivå i dBFS i alla kanaler: en kanal med ljud räcker för att behålla partiet. Standard: −60 dBFS, minst 0,5 sekunders tystnad och 0,1 sekunds marginal runt ljud. Funktionen tar bort tysta partier och förkortar mixen; stem-original och deras inbördes synk bevaras. Helt tysta mixar behålls. Lyssna igenom resultatet, särskilt svaga efterklanger. Varje borttaget parti kan återställas med Ångra klipp. Klippen sparas i projektet och används vid mixexport/mastering.

De-clip (spline) finns i spårverktygen och behandlar markeringen, eller hela valt spår om markering saknas. En kubisk Hermite-spline använder intakta grannsamplingar och deras lutningar för att uppskatta korta avplattade toppar. Den konservativa detekteringen kräver 2–64 på varandra följande samplingar med samma tecken och absolut nivå minst 0,9999. Långa skador, filkanter och osäkra rekonstruktioner lämnas orörda. När återskapade toppar överstiger full skala sänks hela behandlade partiet lika i alla kanaler till samplingspeak 0,999. Detta kan ge en nivåskillnad vid markeringens gräns; använd helst hela spåret. Det är inte en true-peak-limiter. Förhandslyssning och export använder motsvarande algoritm. Ångra återställer behandlingen och originalfiler skrivs aldrig över.

Spline-metoden kan hjälpa kort hård digital klippning. Den reparerar inte generellt AI-artefakter, mjuk distorsion eller klippning som senare normaliserats till lägre nivå. Funktionen är en enkel egen implementation, inte RX:s algoritm.


## Volymreglage, global analys och DC offset

Varje spår har ett volymreglage mellan −60 och +12 dB bredvid sifferfältet. Båda ändrar samma sparade mixnivå. Under stemuppspelning ändras förstärkningen mjukt utan att uppspelningen startas om. Reglaget påverkar mixen och mastering; Exportera valt spår som WAV sparar spårredigeringarna utan mixförstärkning.

Analysera & föreslå visar global analys före och efter mastering: högsta True Peak i ett 100 ms-detektorintervall, högsta/lägsta ändliga korttids-LUFS över tre sekunder samt fördelning i 1 LU-breda intervall. LUFS läses var 100 ms, så fönstren överlappar: procenten beskriver mätfönster, inte oberoende låtpartier. Filer kortare än tre sekunder saknar korttidsvärden. Digital tystnad utan ändligt värde undantas från minimum. True Peak-filterfördröjningen är inte kompenserad; tiden är vägledande, inte ett exakt samplingsindex. Tiderna gäller den redigerade mixen efter klipp. Full mätserie, fördelning och kanalernas genomsnittliga DC-förskjutning ingår i Mastering-report.json.

Ta bort DC offset · 2 Hz är valbart per masteringprofil och avstängt som standard även för äldre projekt. Ett normaliserat första ordningens högpassfilter med separat kanaltillstånd körs före EQ, kompressor och limiter i både förhandslyssning och masterexport. Inställningen sparas i projektet och rapportens profilinställningar. Vanlig mixexport och originalfiler påverkas inte. Filtret har insvängningstid och kan ändra toppnivån; förbättrat headroom är inte garanterat.


## Arbetsområden och vinylrestaurering

Startvyn erbjuder Spårredigering, Mixredigering, Mastering, Vinylrestaurering och Album & skivsidor. Bara valt arbetsområde visas. Välj arbetsområde återgår till startsidan utan att lämna projektet. Vyn är inte en ny session: importerade filer och inställningar följer med mellan arbetsområdena.

Vinylrestaurering behandlar valt originalspår före spårredigeringar, mixning och mastering. Aktivera önskade funktioner och tryck Tillämpa & förbered lyssning. Samma Rust-behandling används vid förhandslyssning, export av valt spår, uppdelad export och mastering. Inställningarna sparas i projektet; vid återöppning förbereds lyssningen på nytt från originalljudet. Återställ restaurering tar bort hela restaureringsbehandlingen. Originalfilerna ändras aldrig.

### Separera en mix i instrument och sång

Om en gammal vinylutgåva har en obalanserad mix kan du först importera en ensam stereoinspelning och trycka **Separera inspelad mix**. Demucs-RS skapar fyra nya spår: trummor, bas, övrigt och sång. Völund mutar originalmixen och väljer sångspåret; justera sedan spårens nivåer i Mixredigering. Originalfilen finns kvar.

Den valfria funktionen kräver att du själv startar separationen. Första gången visas en fråga och Demucs-RS hämtar modellvikterna från Hugging Face. Vikterna ingår inte i program- eller testpaketet och ljudfilen skickas inte till Hugging Face. Windows-testpaketet innehåller den separata motorn bredvid Völund. Motorn paketeras för närvarande inte för Linux eller macOS.

Separeringen kan ta längre tid än vinylfiltren och använder datorns grafikprocessor när Demucs-RS har tillgång till den. Resultatet är en uppskattning av instrumenten; läckage och artefakter kan förekomma. Lyssna på varje stämma och jämför med originalmixen innan export.

De-click använder en lokal impulsdetektor med cirka 2 ms omgivning och begränsad reparationsbredd. Markerade korta impulser ersätts med linjär interpolation. De-crackle är ett experimentellt, känsligare andra pass för kortare impulser, inte en fullständig bredbands- eller spektral knastermodell. Börja med känslighet 3; höga värden kan påverka musikattacker. Jämför Original och Restaurerat vid samma position. Lyssning i denna vy isolerar valt spår och Original spelar det importerade ljudet utan spårredigeringar.

De-rumble är ett fjärde ordningens Butterworth-högpass (24 dB/oktav), uppbyggt av två sektioner med rätt skilda Q-värden. Välj 20 eller 25 Hz. Monobas filtrerar sidokanalen med ett andra ordningens högpass vid 120 Hz och bevarar mittkanalen. Det minskar lågfrekevent stereoinnehåll gradvis; det är ingen exakt gräns eller automatisk korrigering av azimut, polaritet eller kanalfördröjning.

Hitta låtar använder högsta RMS-nivån i någon kanal över 20 ms-fönster. Standard: −45 dBFS och minst 1,5 sekunders paus. En markör placeras mitt i varje intern paus, med minst fem sekunder till filkanter och föregående markör. Inledande/slutlig tystnad skapar inte låtar. Lugn musik kan ge felaktiga förslag och knastriga pauser kan missas. Markörerna visas på vågformen, kan flyttas med sekundfält eller tas bort, och sparas i projektet. Lägg markör vid spelposition fungerar även vid pausad uppspelning.

Exportera uppdelade låtar skapar en ZIP med numrerade 32-bit float WAV-filer och Tracks.json. Gränserna delar ljudet sammanhängande: inga samplingar eller pauser tas bort och ingen fade läggs till. Kontrollera markörerna och ljudnivån före leverans; floatformatet kan bevara toppar över 0 dBFS. Exportera valt spår som WAV ger en restaurerad fil utan uppdelning. Tidslinjen behåller sin ursprungliga längd.
