# Architecture — Piano Tect Tauri

## 1. Princípios Arquiteturais
1. **Separação estrita entre Thread de Áudio e UI**: A thread de callback de áudio (`cpal`) não realiza alocações dinâmicas no hot-path nem bloqueios em mutexes da UI; atualizações de parâmetros e notas chegam via canal assíncrono (`mpsc`) e telemetria é publicada via `try_write`.
2. **Modularidade com Propriedade Exclusiva (Multiagente)**:
   - `src-tauri/src/contracts.rs`: Tipos canônicos compartilhados (`EngineParams`, `InstrumentModel`, `PresetData`, `MidiConfig`, `EngineTelemetry`, `EngineSnapshot`).
   - `src-tauri/src/synthesis/`: Motor modal/waveguide de síntese física, temperamentos e polifonia.
   - `src-tauri/src/instruments/`: Perfis acústicos e parâmetros de fábrica de cada família de instrumento.
   - `src-tauri/src/effects/`: Palco virtual 3D / binaural, equalizador paramétrico, slots de efeitos, reverb e limiter.
   - `src-tauri/src/audio/`: Supervisor de stream `cpal` (PipeWire/ALSA/JACK), fallback headless DSP e exportação WAV (`hound`).
   - `src-tauri/src/midi/`: Descoberta e hot-plug `midir`, parser MIDI, curva de velocidade, CC Learn e transposição.
   - `src-tauri/src/presets/`: Presets de fábrica, morphing A/B e persistência JSON em XDG (`~/.local/share/piano-tect-tauri/presets`).
   - `src-tauri/src/commands/`: Ponte IPC Tauri 2 (`#[tauri::command]`).
   - `src/`: Interface React + TypeScript + Vite.

## 2. Fluxo de Sinal (Signal Flow)

```text
MIDI Input (midir) / Teclado Virtual (UI)
   │  (Velocity Curve, Transpose, Channel Filter, CC64/66/67/69, Pitch Bend)
   ▼
PianoSynthEngine (src-tauri/src/synthesis/)
   │  [128 Voices • 8 Stiff-String Partials • Hammer Felt Filter • Strike Point]
   │  [Unison Trichord • Sympathetic & Duplex Resonance • Damper/Hammer/Pedal Noise]
   ▼
EffectsChain (src-tauri/src/effects/)
   │  [Lid Attenuation -> Stereo Width / Binaural Crossfeed -> 3-Band EQ]
   │  [FX Slots (Delay / Chorus / Tremolo / Comp) -> Stereo Reverb -> Limiter]
   ▼
Audio Output (cpal PipeWire/ALSA/JACK) & Telemetry + WAV Export (hound)
```
