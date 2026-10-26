# HEAD OF STATE

3D-Catch-Wrestling in Rust. Zwei Wrestler, ein Ringrichter, ein voller Saal.
**Der Präsident** (rot/gold) gegen **Der Kanzler** (blau/silber) – gewonnen wird durch Pinfall (3er-Zählung).

Alles ist prozedural erzeugt: Körpermodelle (19-Gelenk-Skelett mit Muskelformen), Animationen
(Keyframes + prozedurale Lauf-/Atembewegung), Ring mit federnden Seilen, Arena mit ~2500 Zuschauern,
Licht-Traverse, Videowand und alle Sounds (Glocke, Treffer, Slams, Publikum) – keine externen Assets
ausser der eingebetteten Schrift.

## Starten

- **Windows:** `windows/HeadOfState.exe` doppelklicken (keine Installation, keine DLLs nötig).
- **Linux:** `linux/head_of_state` ausführen (`chmod +x` falls nötig). Benötigt OpenGL + ALSA.

## Spielmodi

- 1 Spieler vs CPU (Leicht / Mittel / Schwer)
- 2 Spieler lokal an einer Tastatur
- CPU vs CPU (Zuschauen) – läuft auch im Hintergrund des Hauptmenüs

## Steuerung

| Aktion | Spieler 1 | Spieler 2 |
|---|---|---|
| Bewegen | W A S D | Pfeiltasten |
| Rennen | Shift links | Shift rechts |
| Schlag (am Boden: Stampfer, im Lauf: Clothesline) | J | Num1 / , |
| Tritt (Big Boot, am Boden: Knee Drop) | K | Num2 / . |
| Griff → Clinch | L | Num3 / - |
| Pin (liegender Gegner auf dem Rücken) / Posen | Leertaste | Num0 / Ctrl rechts |
| Blocken (halten) | I / Ctrl links | Num5 / M |

**Im Clinch:** Schlag = Body Slam · Tritt = Vertical Suplex · Griff = Irish Whip (Gegner prallt aus den Seilen zurück – dann zuschlagen!)
**Finisher:** Finisher-Leiste voll (blau → gold) + Pin-Taste im Clinch = **HEAD OF STATE** (DDT).
Der Verteidiger kann im Clinch mit Schlag/Tritt kontern. Am Boden und im Pin: Tasten hämmern zum Aufstehen bzw. Befreien.

Kamera: Q/E oder rechte Maustaste ziehen, Mausrad = Zoom. F1 Steuerung, P Pause, ESC Menü.

## Selbst bauen

```
cargo build --release
```
Abhängigkeit: [macroquad 0.4](https://github.com/not-fl3/macroquad). Unter Linux werden
`libasound2-dev`, `libx11-dev`, `libxi-dev`, `libgl1-mesa-dev` benötigt.
Windows-Exe nativ: `cargo build --release` mit der Rust-MSVC- oder GNU-Toolchain.

## Code-Aufbau

- `src/mesh.rs` – prozeduraler Mesh-Builder (Drehkörper, Ellipsoide, Röhren, Boxen)
- `src/model.rs` – Körpermodelle der Wrestler und des Ringrichters
- `src/anim.rs` – Skelett, Vorwärtskinematik, Posen, alle Animations-Clips inkl. Paar-Würfe
- `src/wrestler.rs` – Zustandsmaschine eines Wrestlers, Eingaben, KI
- `src/game.rs` – Kampf-Logik, Treffer, Clinch, Würfe, Pinfall, Ringrichter, Kamera, Rendering
- `src/arena.rs` – Ring, Seil-Physik, Halle, Publikum, Texturen
- `src/render.rs` – GLSL-Shader (Spotlights, Glanz, Fog, Tonemapping, Bodenschatten)
- `src/audio.rs` – Sound-Synthese
- `src/hud.rs` – HUD, Menü

Schrift: Poppins Bold (SIL Open Font License, siehe `assets/OFL.txt`).
