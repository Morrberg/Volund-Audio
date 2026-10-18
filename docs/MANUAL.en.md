# Völund Audio – User Manual

<p align="center"><img src="../assets/volund-emblem.jpg" width="360" alt="Völund's winged emblem in antique bronze"></p>

**For the 0.95 development preview · Forge your sound.**

This manual uses the English interface labels. Choose **English** at the top of the app to follow the instructions. The interface is also available in Swedish.

[Back to README](../README.md) · [Svensk manual](MANUAL.sv.md)

<a id="contents"></a>
## Contents

1. [Understanding the workflow](#workflow)
2. [Starting the app and opening files](#starting)
3. [Timeline and playback](#timeline)
4. [Muting, fading, and undoing](#editing)
5. [Finding suggestions for unwanted sounds](#suggestions)
6. [Panning a selected region](#panning)
7. [Balancing track levels](#levels)
8. [Trimming the stereo mix](#trimming)
9. [Viewing and editing metadata](#metadata)
10. [Mastering a song](#mastering)
11. [Exporting masters](#master-export)
12. [Matching album loudness](#album)
13. [Compiling sides A and B](#sides)
14. [Saving and reopening projects](#projects)
15. [Troubleshooting and limitations](#troubleshooting)
16. [Glossary](#glossary)
17. [Trim Silence and De-clip](#trim-silence-and-de-clip)
18. [Track volume, global analysis, and DC offset](#volume-global-analysis-and-dc-offset)
19. [Workspaces and vinyl restoration](#workspaces-and-vinyl-restoration)
20. [Easy Podcast mode](#easy-podcast)

<a id="workflow"></a>
## 1. Understanding the workflow

Völund Audio has three levels of work:

| Level | What you work with | Example |
|---|---|---|
| **Stems** | Separate instrument and vocal tracks from one song. | Mute an extra vocal sound or pan a synth part. |
| **Mix and mastering** | The tracks combined into a stereo song. | Shorten the intro, adjust the tone, and create different editions. |
| **Album** | Several finished stereo songs of the same edition type. | Match the loudness of the CD versions of eight songs. |

**Work on one song at a time in the stem view; use the album section for multiple finished songs.** If you open two different songs as stems, they will be summed and play at the same time.

A typical workflow is: **edit stems → balance the mix → trim the mix → master → match album levels**. Save a song project while you work, and save a separate side project for the album.

<a id="starting"></a>
## 2. Starting the app and opening files

### Windows

1. Extract the application ZIP and start `volund-audio.exe`.
2. The app requires Microsoft Edge WebView2 Runtime. If the window does not open, check that it is installed.
3. Choose the language and light or dark theme at the top. **About** shows the app version and information.

Windows SmartScreen may warn about the unsigned app. Check that the file came from the intended build in the project's [GitHub Actions](https://github.com/Morrberg/Volund-Audio/actions). If you have verified and trust the build, you can choose **More info → Run anyway** if Windows offers that option. You do not need to disable SmartScreen generally. See also [Microsoft's explanation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

### Importing a song

Use **Open stems / project** for:

- WAV, MP3, or FLAC files, one or more files from the same song;
- a ZIP archive, for example one containing Suno stems;
- a saved song project with the `.vcleaner` extension.

You can open a ZIP file directly. Imported tracks are shown and sorted by the likely track type inferred from the filename: vocals first, followed by drums, bass, guitar, keyboards, and other types. Numbers are sorted naturally, so `2 Guitar` comes before `10 Guitar`. This sorting looks at the filename, not at which instrument can actually be heard.

Use unique filenames. ZIP import removes directory paths, so two files both named `Vocals.wav` will not become two distinct tracks.

You can also open a single WAV here for trimming or mastering. For an album, use **Open finished songs** in the album section instead; see [section 12](#album).

<a id="timeline"></a>
## 3. Timeline and playback

Click a track to make it active. Its waveform becomes larger and its control row moves with it. Changing the active track does not remove edits from other tracks.

| Action | What it does |
|---|---|
| Drag across the waveform | Select a time range on the active track. |
| Click without dragging | Set the playback position and clear the selection. |
| Mouse wheel over a stem waveform | Zoom around the pointer. All stems share the same time view. |
| Shift + mouse wheel | Move the visible time view horizontally. This does not change stereo placement. |
| **Play** / **Pause** | Start or pause playback from the selected position. |
| **Stop** in the stem controls | Stop and return to the beginning. |
| **Play selection** | Listen to the selected time range. |
| **Loop selection** | Repeat the selection during normal playback. |

Playback uses all audible stems. To hear only one track, choose **Solo** on that track. You can solo more than one track at a time.

<a id="editing"></a>
## 4. Muting, fading, and undoing

Select a region and choose:

| Control | Result |
|---|---|
| **Mute** | Silences the region and applies short, smooth transitions at its edges. The track length stays the same. |
| **Fade-in** | Fades the sound up across the selection. |
| **Fade-out** | Fades the sound down across the selection. |
| **Noise gate** | Attenuates quiet sections below the selected dBFS threshold. Set the threshold and choose Noise gate; with no selection, it processes the full track. Smooth attack and release reduce clicks. |
| **Undo / Redo** | Changes the edit history for the active track. |
| **Showing: Edited / Original** | Switches the stem waveform and stem playback between edited and original audio. |

Switching to **Original** does not remove edits or disable them for export or mastering. To change what is exported, use **Undo** or redo the edit.

The noise gate works best when noise occurs between drum hits. It cannot distinguish noise from drum sound that overlaps it, and a threshold set too high can attenuate quiet hits and cymbal tails. Adjust it while listening and use Undo to compare.

### Example: removing an extra “Wrooow”

1. Open the song's Suno ZIP and select the vocal track.
2. Turn on **Solo** to hear it more clearly.
3. Zoom in and select the unwanted sound, for example from 0.8 to 2.1 seconds.
4. Choose **Mute**, listen, and undo if the selection was wrong.
5. Turn off **Solo** and listen with the instruments again.
6. Save the project. Export the track too if you want to continue in another music app.

Muting leaves a gap in the vocal track instead of moving the rest of the vocals. This keeps the vocals aligned with the drums and instruments.

<a id="suggestions"></a>
## 5. Finding suggestions for unwanted sounds

**Find suggestions** scans the active track for short, isolated sounds near the beginning or end. It uses audio energy and silent gaps. It does not understand lyrics or know whether a sound is musically intentional.

For each suggestion, choose **Listen**, **Accept (mute)**, or **Dismiss**. Accepting adds a normal mute edit that can be undone. Searching by itself does not change the audio.

A suggestion might be an intentional shout or a real vocal phrase, so listen before accepting it. If no suggestions appear, that does not mean the entire track is free of unwanted sounds.

<a id="panning"></a>
## 6. Panning a selected region

Panning affects all audio in the selection on **the active track only**. For example, panning a synth track does not change the vocal track.

| Choice | What it does |
|---|---|
| **Left / Right** | Place the region on the selected side. |
| **Center** | Keep or restore the region's natural stereo image; this does not convert it to mono. |
| **Position + Place** | Choose a fixed position between the sides with the slider. |
| **Left → right / Right → left** | Move the sound across the stereo image during the selection. |
| **Alternate left ↔ right** | Move the sound back and forth for a chosen number of full cycles, 1–32, during the selection. |

Transitions at the edges are smooth. Pans are listed, can be removed individually, and appear as purple lines in the waveform. If pans overlap, the later pan takes precedence with smooth transitions.

### Example: drums moving from side to side

Select 20–24 seconds in the drum track, choose **Cycles: 2**, then choose **Alternate left ↔ right**. The drums make two complete passes across the stereo field over those four seconds. Other tracks keep their stereo placement.

To place different drum hits on different sides instead, select the hits individually and choose left or right.

Both channels of a stereo track can be moved to the same side. Summing them can increase peaks, so lower the track level if needed. If several sounds occur at the same time in one stem, panning cannot separate them.

<a id="levels"></a>
## 7. Balancing track levels

Each track has a level control in dB, **Mute**, and **Solo**.

- **0 dB** keeps the level unchanged. A negative value lowers it and a positive value raises it; the range is −60 to +12 dB.
- **Mute** removes the entire track from mix playback and mix export. This differs from muting a selected region, which affects only the selected audio.
- **Solo** makes only soloed, unmuted tracks audible. A muted track remains silent even if Solo is selected.

**Example:** lower the vocals to −2 dB if they sit too far forward, or lower the guitars to −1 dB if the mix feels too dense. These are examples; listen to your own song.

Level, mute, and solo settings carry through to the stereo mix and mastering. Check that no temporary solo settings remain before exporting. **Export selected track…** uses the track edits, but does not include these mix levels or mute/solo settings.

<a id="trimming"></a>
## 8. Trimming the stereo mix

Open **Mix · trim before mastering** and choose **Refresh mix** when the source has changed. This view combines your edited stems using their mix levels, mute, and solo settings. A single imported WAV also works.

1. Drag across the mix waveform or enter **Start (s)** and **End (s)**.
2. Choose **Remove selection** to remove the selected time and join what remains.
3. Choose **Keep selection only** to remove everything outside the selection.
4. Use **Undo trim / Redo trim** for the mix's separate edit history.

**Example:** a mix is 180 seconds long. Remove 0–4 seconds to shorten the intro; the mix is now 176 seconds. Make the next trim using the new, shorter timeline.

Trims affect the same stereo mix before **all** mastering profiles. The stems' own timelines do not change. Short fades of about 5 ms at cuts and trim edges reduce the risk of clicks without adding extra overlap or shortening. You cannot trim away the entire mix.

**Play mix** plays the selection if there is one, otherwise the full mix. **Save edited mix…** creates a 24-bit stereo WAV before mastering at the sample rate selected in the mastering profile. This also applies when the CD profile is selected; the CD master is exported separately as 16-bit audio.

<a id="metadata"></a>
## 9. Viewing and editing metadata

Open **Metadata for selected track**. Depending on the source and supported format, the panel can show title, artist, album, date, genre, track number, comment, ISRC, and lyrics.

You can edit values, **Add field**, remove a field, **Clear metadata**, or **Restore from original**. The panel also shows information about embedded images and fields that are read-only.

Changes apply to the stem's exported WAV and are saved in the project. Supported WAV tags are preserved, but other players may display different subsets. Clearing metadata also removes original tags and embedded images from the exported file.

**The original file is not changed.** A project still contains the original audio and its original metadata even if you chose to clear metadata from the export.

The title, artist, and album fields in the mastering mix are set in **Mastering studio**. These are separate from the metadata on an individual stem. Album exports of finished songs preserve their supported original metadata.

<a id="mastering"></a>
## 10. Mastering a song

### Start with a balanced mix

1. Check the stems, levels, mute/solo settings, and any mix trims.
2. Open **Mastering studio · one mix, multiple editions**.
3. Enter the song title, artist, and album.
4. Choose a **Profile to edit**. Each profile has its own settings, saved in the song project.

All profiles start from the same mix **before mastering**. The Vinyl version is not processed from the Spotify version.

### Default profile settings

The table shows the app's starting points after **Reset profile**, not universal delivery requirements.

| Setting | Spotify | Vinyl | CD | Cassette |
|---|---|---|---|---|
| Sample rate | 44.1 kHz | 44.1 kHz | 44.1 kHz, locked | 44.1 kHz |
| Bit depth | 24 bit | 24 bit | 16 bit, locked | 24 bit |
| LUFS target | −14 | Off | −14 | Off |
| True peak max | −1 dBTP | −3 dBTP | −1 dBTP | −1 dBTP |
| Compression | 15% | 0% | 15% | 15% |
| Limiter | On | Off | On | On |
| Narrow low bass | Off | Below 120 Hz | Off | Off |
| Low cut | Off | 20 Hz | Off | Off |

EQ starts at 0 dB in every profile. Outside the CD profile, you can change the export sample rate and bit depth. Available mastering sample rates are 44.1, 48, 88.2, and 96 kHz; bit depths are 16 or 24 bit.

### Controls

| Control | What it does |
|---|---|
| **Match loudness to LUFS target** | Tries to reach the selected integrated loudness target. When off, this target-based level adjustment is not applied. |
| **True peak max** | Sets the peak limit. It takes priority over the LUFS target. |
| **Bass / Mid / Treble** | Adjusts the mix tone with a simple three-band EQ. |
| **Compression** | Reduces the dynamic range of the stereo mix. Start gently if the material is already heavily compressed. |
| **Limiter** | Limits peaks. Even with the limiter off, the entire signal can be turned down to keep within the peak limit. |
| **Narrow low bass below** | Reduces stereo width in the lowest frequencies. Set to 0 to turn it off. |
| **Low cut** | Attenuates very low frequencies. Set to 0 to turn it off. |
| **Gentle / Balanced / Punch** | Sets compression to 0/25/55%. For Spotify and CD it also sets the LUFS target to −16/−14/−10 and peak limit to −1/−1/−2 dBTP. |
| **Reset profile** | Restores the selected profile's defaults. |

Quick settings do not reset all other controls, such as EQ. Changing Vinyl or Cassette does not affect Spotify or CD.

### Analyze, listen, and adjust

**Analyze & suggest** shows measurements and simple advice. It does not change settings automatically. **LUFS** describes loudness, **true peak** describes peak level, and **LRA** describes loudness variation. Very short or quiet audio may not have a LUFS value.

Choose **Create preview**, then switch between **Before** and **After** at the same position. **Match volume for comparison** reduces the loudness difference so it is easier to judge tone and dynamics. This is for listening only and does not change the export level.

When the mix, trims, or profile change, create the preview again. You can export without making a preview; it is not required.

**Example for a dense metal song:** try **Gentle**, create a preview, and listen to the snare attack and cymbals. Increase compression only if it improves the result. If the app reports that the LUFS target cannot be reached, choose a lower target or accept the measured level. The app prioritizes the peak limit and caps level increase at +12 dB.

<a id="master-export"></a>
## 11. Exporting masters

Select the editions under **Export** and choose **Save selected versions as ZIP…**. For example:

```text
Song 1 - mastered - Spotify.wav
Song 1 - mastered - Vinyl.wav
Song 1 - mastered - CD.wav
Song 1 - mastered - Cassette.wav
Mastering-report.json
```

**Save current profile as WAV…** exports only the selected profile. The title, artist, and album from the mastering studio are written to WAV tags. The ZIP report contains settings, measurements, and any warnings.

Master export uses 16/24-bit PCM with final dithering. Exporting an MP3 source as WAV does not add detail beyond the original.

| Edition | What the export is |
|---|---|
| **Spotify / streaming** | A WAV using the selected profile. Follow your distributor's current delivery requirements. |
| **Vinyl** | A source file for the cutting engineer. Side length, track order, and harsh treble need to be assessed there; the app does not apply RIAA encoding or simulate a cutting lathe. |
| **CD** | 16-bit/44.1 kHz WAV. DDP, PQ indices, and CD-Text are not included. |
| **Cassette** | A digital source for duplication. No Dolby encoding or tape simulation is applied. |

Choose new filenames or a new location when exporting. Exporting does not replace **Save project…**.

<a id="album"></a>
## 12. Matching album loudness

### Open the right songs

1. Open **Album · loudness matching and sides A/B**.
2. Choose **Open finished songs** and load complete WAV/FLAC songs, a ZIP, or a saved side project.
3. Use songs from the same edition type, for example all Spotify versions. Make a separate album pass for Vinyl, CD, and Cassette.
4. Enter the edition name, sample rate, and bit depth. For a CD audio export, choose **44.1 kHz and 16 bit** here; the album section does not have a locked CD profile.

A new file import starts with loudness matching on. Older side projects without this setting open with matching off, so it is not enabled unexpectedly.

### How matching works

When **Match perceived loudness of songs** is enabled, each matched song gets a constant level adjustment. The song's internal dynamics and tone remain unchanged: no new EQ, compressor, or limiter is applied.

The automatic target is the **median** of matched songs with measurable LUFS. You can choose a **Custom LUFS target** instead. **Analyze album levels** shows the level before, the calculated level after, and the adjustment in dB. The Before/After buttons play the selected song; here you hear the level difference itself.

### Example: three songs with different loudness

Assume there is enough peak headroom for the adjustments:

| Song | Before | Adjustment | After |
|---|---:|---:|---:|
| Song 1 | −10 LUFS | −6 dB | −16 LUFS |
| Song 2 | −16 LUFS | 0 dB | −16 LUFS |
| Song 3 | −22 LUFS | +6 dB | −16 LUFS |

The median is −16 LUFS. A separate intro at −28 LUFS can be left quieter by clearing its **Match** checkbox. It is also excluded from the automatic target calculation.

You can also set a custom dB adjustment for each song, for example **−3 dB** for a song that should intentionally be quieter after matching. The range is −12 to +12 dB. Songs without measurable LUFS, such as digital silence, are not matched automatically.

### When the whole album needs to be turned down

If a song would otherwise exceed the peak limit or the total +12 dB level increase, **all songs are turned down by the same amount**. This preserves the intended balance instead of making only the limited song too quiet.

For example, if the common reduction is 3 dB, the three songs in the table become −19 LUFS. An excluded intro at −28 LUFS becomes −31 LUFS. **Excluding a song means it receives no automatic loudness match; it is still included in the album's shared safety reduction.** Custom dB adjustments can still apply when automatic matching is off.

### Export and save

**Save level-matched songs as ZIP…** creates numbered WAV files in album order, such as `01 - Song 1 - mastered - CD - album.wav`, and `Album-report.json`. The report contains values measured from the files after conversion and dithering. Supported original metadata is preserved.

The results table is a calculation; the export report contains measurements of the finished files. LUFS helps even out levels, but does not guarantee identical perceived loudness. Listen to transitions and fine-tune if needed. Differences in EQ, density, and arrangement may still be audible.

Save your work with **Save side project…**. See [the project overview](#projects).

<a id="sides"></a>
## 13. Compiling sides A and B

In the album track list, choose **Side A** or **Side B** for each song. Move songs with the arrows. The order within each side follows the list.

The number field next to the side selection sets the gap **before the song**, from 0 to 30 seconds. The first song on a side has no extra leading gap. The list also shows the estimated side length.

**Example:** choose three songs for A and three for B. Set a two-second gap before each song that should have one; the first song's gap field is ignored on each side.

**Save sides A/B and track lists as ZIP…** creates a WAV for each used side, timed track lists, and `Side-report.json`. The same loudness matching and exclusions are used as for separate album track exports. If the side transitions themselves need more peak headroom, both sides are turned down by the same amount; the report shows this.

Use the relevant Vinyl or Cassette version as the source and confirm format and side length with the manufacturer. The app's 60-minute-per-side limit is a technical limit, not a recommendation for vinyl side length.

<a id="projects"></a>
## 14. Saving and reopening projects

| What you want to save | Use | Reopen with |
|---|---|---|
| A song's stems, edits, undo/redo history, metadata changes, mix levels, mix trims, mastering profiles, and optional podcast jingles | **Save project…** | **Open stems / project** |
| An album's source songs, order, sides, gaps, matching target, exclusions, and level adjustments | **Save side project…** | **Open finished songs** in the album section |
| A finished audio file for another app or recipient | The relevant WAV or ZIP export | Import the audio file where you need it |

Both project types use the `.vcleaner` extension and contain the source audio, so they can be large. Older projects from Vocal Cleaner are still supported.

**Ctrl+S saves the project; Ctrl+Shift+E exports the active track in the selected format.** Choose WAV, FLAC, MP3 (320 kbps), or AAC in an M4A file (256 kbps). WAV and FLAC are lossless; MP3 and AAC are lossy. Metadata edits are written to WAV only. In the album workspace, Ctrl+S saves the side project. Ctrl+S opens the dialog where you choose where to save the project file. You can also use the project buttons. Exporting a track or master does not save the project's edits and can therefore leave unsaved project changes.

Canceling a save dialog keeps your work in the app. The close warning does not save anything automatically. Version 0.9 has no automatic project saving or crash recovery.

<a id="troubleshooting"></a>
## 15. Troubleshooting and limitations

| Symptom | Check or try |
|---|---|
| Only one instrument is included in the mix | Turn off temporary **Solo** settings and check **Mute**. |
| A region is silent but the song is not shorter | **Mute** preserves the timing. Use **Remove selection** in the mix view to shorten the full song. |
| The mix waveform is empty after a change | Choose **Refresh mix**. Earlier mix data is invalidated when audio or the profile changes. |
| Mastering Before/After buttons are inactive | Choose **Create preview** again after making changes. |
| The exported mix does not include your mix levels | Check that you exported a mix or master, not **Export selected track…**. |
| A LUFS target is not reached | Read the measurements and warnings. The peak limit and maximum level increase may constrain the result. |
| An excluded intro is still turned down | The album's shared safety reduction also applies to excluded songs. |
| Panning affects several sounds | They are in the same stem and time range. The tool moves all audio in that selection. |
| No sound suggestions are found | Select the region manually. The detector only looks for certain short, isolated sounds near the beginning or end. |
| Metadata does not appear in another player | Players support different WAV tags. Also check whether the field could be exported. |
| Edits are missing when you continue | Open the project file, not only the original ZIP or exported WAV. |
| The app feels slow | Fewer or shorter audio files reduce memory use. Audio is decoded into working buffers even when the source is compressed. |

### Technical limits

- A mastering mix supports up to **64 stems** and **60 minutes**. Mono and stereo are supported; multichannel mastering is rejected.
- The album list supports up to **100 songs**. A compiled side can be at most **60 minutes**.
- Uploads to the local audio server are limited to **2 GiB** per request. Decoded audio and exports can require much more memory than the ZIP file size suggests.
- As a rough comparison, five minutes of stereo at 44.1 kHz requires about **101 MiB** for one 32-bit working buffer alone. Editing, mixing, and exporting may require additional buffers.
- **Save edited mix…** may reject PCM export if mix samples exceed 0 dBFS. Lower the track levels in that case. Stem export is 32-bit float at the original sample rate and is not automatically peak-limited. Panning can turn a mono track into stereo.
- Edits do not change source files on your computer. However, a save dialog can overwrite an existing file if you choose the same filename; choose new names to keep the originals.

<a id="glossary"></a>
## 16. Glossary

| Term | Explanation |
|---|---|
| **Stem** | A separate vocal or instrument track from a song. |
| **Mix** | Tracks combined with their levels and stereo positions. |
| **Master** | A processed final version of a mix for a particular edition. |
| **Panning** | Placing sound between the left and right channels. |
| **dB** | A relative level measurement. In the track controls, 0 dB means no level change. |
| **LUFS** | A loudness measurement. A less negative value is louder. |
| **True peak / dBTP** | An estimate of peak level, including between digital samples. |
| **LRA** | Loudness Range: a measurement of loudness variation. |
| **Compressor** | Reduces the difference between louder and quieter sections. |
| **Limiter** | Restricts high peaks. |
| **Sample rate / kHz** | The number of audio samples per second in a file. |
| **Bit depth / PCM** | Resolution and integer format for digital audio, such as 16- or 24-bit PCM. |
| **Dither** | Very low-level noise used during final integer quantization to reduce quantization artifacts. |

[Back to contents](#contents) · [Back to README](../README.md) · [Svensk manual](MANUAL.sv.md)

<a id="trim-silence-and-de-clip"></a>
## 17. Trim Silence and De-clip

Trim Silence is available in mix editing. The threshold is measured as an absolute sample level in dBFS across all channels: a region is kept if either channel contains audio. Defaults are −60 dBFS, at least 0.5 seconds of silence, and a 0.1-second margin around audio. The function removes silent regions and shortens the mix; the original stems and their alignment are preserved. A completely silent mix is kept. Listen to the result, especially for quiet reverb tails. Each removed region can be restored with **Undo trim**. Trims are saved in the project and used for mix export and mastering.

De-clip (spline) is available in the track tools. It processes the selection, or the entire active track if there is no selection. A cubic Hermite spline uses intact neighboring samples and their slopes to estimate short, flattened peaks. Conservative detection requires 2–64 consecutive samples with the same sign and an absolute level of at least 0.9999. Long damage, file edges, and uncertain reconstructions are left unchanged. If reconstructed peaks exceed full scale, the entire processed region is lowered equally in all channels until its sample peak is 0.999. This can cause a level change at the selection boundary, so processing the whole track is preferable. This is not a true-peak limiter. Preview and export use the same algorithm. Undo restores the previous audio, and source files are never overwritten.

The spline method can help with short, hard digital clipping. It does not generally repair AI artifacts, soft distortion, or clipping that was later normalized to a lower level. This is a simple in-house implementation, not the algorithm used by RX.

<a id="volume-global-analysis-and-dc-offset"></a>
## 18. Track volume, global analysis, and DC offset

Each track has a volume slider from −60 to +12 dB next to its numeric field. Both controls change the same saved mix level. During stem playback, gain changes smoothly without restarting playback. The control affects the mix and mastering; **Export selected track** saves track edits without the mix gain.

**Analyze & suggest** shows global analysis before and after mastering: the highest True Peak in a 100 ms detector interval, the highest and lowest finite short-term LUFS values over three seconds, and a distribution in 1 LU-wide bins. LUFS is read every 100 ms, so the windows overlap: percentages describe measurement windows, not independent sections of the song. Files shorter than three seconds have no short-term values. Digital silence without a finite value is excluded from the minimum. True Peak filter delay is not compensated, so the time is a guide, not an exact sample index. Times refer to the edited mix after trimming. The full measurement series, distribution, and each channel's average DC offset are included in `Mastering-report.json`.

**Remove DC offset · 2 Hz** is optional per mastering profile and is off by default, including for older projects. A normalized first-order high-pass filter with separate channel state runs before EQ, compression, and limiting in both preview and master export. The setting is saved in the project and in the report's profile settings. Regular mix export and source files are not affected. The filter has a settling time and can change peak level; improved headroom is not guaranteed.

<a id="workspaces-and-vinyl-restoration"></a>
## 19. Workspaces and vinyl restoration

The start view offers **Track editing**, **Mix editing**, **Mastering**, **Vinyl restoration**, and **Album & record sides**. Only the selected workspace is shown. Choosing a workspace returns to the start view without closing the project. This is not a new session: imported files and settings remain available across workspaces.

Vinyl restoration processes the selected source track before track edits, mixing, and mastering. Enable the features you want and choose **Apply & prepare preview**. The same Rust processing is used for preview, export of the selected track, split-track export, and mastering. Settings are saved in the project. When reopened, the preview is prepared again from the original audio. **Reset restoration** removes the entire restoration process. Source files are never changed.

In Vinyl restoration, use **Cut selection** beside **Mute** to remove a selected section before track detection and split export. **Undo** restores the latest cut. The cut's original time position is saved in the project so track markers remain aligned.

### Separating a mix into instruments and vocals

If an old vinyl release has an unbalanced mix, you can first import a single stereo recording and choose **Separate recorded mix**. Demucs-RS creates four new tracks: drums, bass, other, and vocals. Völund mutes the original mix and selects the vocal track; then adjust the track levels in Mix editing. The original file remains available.

This optional feature runs only when you start separation. The first time, the app asks for confirmation and Demucs-RS downloads model weights from Hugging Face. The weights are not included in the app or test package, and your audio is not sent to Hugging Face. The Windows test package contains the separate engine alongside Völund. The engine is currently not packaged for Linux or macOS.

Separation can take longer than the vinyl filters and uses your computer's graphics processor when Demucs-RS can access it. The result is an estimate of the instruments; bleed and artifacts may occur. Listen to each stem and compare it with the original mix before exporting.

De-click uses a local impulse detector with about 2 ms of context and a limited repair width. Detected short impulses are replaced by linear interpolation. De-crackle is an experimental, more sensitive second pass for shorter impulses, not a full-band or spectral crackle model. Start with sensitivity 3; high settings can affect musical attacks. Compare **Original** and **Restored** at the same position. Playback in this view isolates the selected track, and **Original** plays the imported audio without track edits.

De-rumble is a fourth-order Butterworth high-pass filter (24 dB/octave), built from two sections with the correct distinct Q values. Choose 20 or 25 Hz. **Mono bass** filters the side channel with a second-order high-pass at 120 Hz and preserves the mid channel. It gradually reduces low-frequency stereo content; it is not an exact cutoff or an automatic correction for azimuth, polarity, or channel delay.

**Find tracks** uses the highest RMS level in any channel over 20 ms windows. Defaults are −45 dBFS and at least 1.5 seconds of silence. A marker is placed in the middle of each internal pause, at least five seconds from file edges and the previous marker. Silence at the beginning or end does not create tracks. Quiet music can produce false suggestions, and crackly pauses can be missed. Markers appear on the waveform, can be moved with the seconds field or removed, and are saved in the project. **Add marker at play position** also works while playback is paused.

**Export split tracks** creates a ZIP with numbered 32-bit float WAV files and `Tracks.json`. The boundaries split the audio continuously: no samples or pauses are removed and no fade is added. Check markers and audio levels before delivery; the float format can preserve peaks above 0 dBFS. **Export selected track** creates a restored file without splitting. The timeline keeps its original duration.

[Back to contents](#contents) · [Back to README](../README.md) · [Svensk manual](MANUAL.sv.md)


<a id="easy-podcast"></a>
## 20. Easy Podcast mode

Easy Podcast keeps the controls focused on a spoken-word session. It is designed for a recording with one audio file per microphone.

### Prepare the recordings

1. Record all microphones at the same time in one session.
2. Export one mono or stereo WAV per microphone from the same start point. Keep the full session length and any silence at the beginning; do not trim each file separately.
3. Open all the WAV files together in Völund Audio, then choose **Easy Podcast** from the start screen.

The tracks share one timeline. Cutting a selected passage removes that time range from the whole programme. Muting a passage silences it in the mix but keeps the programme length. Individual track buttons mute a voice or let you listen to one voice by itself. The mode does not move tracks to correct files that start at different times; ask the recording leader to prepare those files from a common start point.

### Add an opening or closing jingle

Choose a WAV file under **Jingle at beginning** or **Jingle at end**. Either one is optional. The opening jingle plays before the recorded voices; the closing jingle plays after them. Supported files are standard RIFF/WAVE, mono or stereo, up to 30 seconds and 12 MB each. Select **Remove jingle** to clear one.

The jingles are included in podcast playback, mastering previews, and exported mixes and masters. They are saved inside the <code>.vcleaner</code> project, so save the project if you want to reopen or change the jingles later. Mix edits apply to the full programme, including the jingles.

When the edit is ready, choose **Done – go to mastering** and use the usual mastering controls to make the final WAV export.
