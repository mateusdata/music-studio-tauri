# Agent Handoff Document (`HANDOFF.md`)

## 1. Objetivo Geral
Desenvolver o **Piano Tect Tauri**, um software de piano virtual profissional para Linux baseado em modelagem física (Rust + Tauri 2 + React + TypeScript), com paridade funcional verificável com o Pianoteq 9.3.1.

## 2. Arquitetura Atual
- **Sysroot Linux**: `.sysroot/` gerado por `scripts/setup-dev-sysroot.sh`. Sempre executar comandos Rust/Tauri com `./scripts/with-sysroot.sh ...`.
- **Backend Rust (`src-tauri/src/`)**:
  - `contracts.rs`: contratos compartilhados (`EngineParams`, `EffectType` com todos os 10 FX, `BounceParams`, `NoteFxParams`, `PlayModeParams`, `MidiTransportState`, `InstrumentModel`, `PresetData`, `MidiConfig`, `EngineTelemetry`).
  - `synthesis/mod.rs`: motor físico polifônico (128 vozes, **32 parciais inarmônicos**, tricórdio 1/2/3 cordas com acoplamento longitudinal, decaimento em dois estágios *prompt + aftersound*, tampo harmônico de 4 modos + *bridge knock*, 8 temperamentos, pedais, PRO Note Edit, Mallet Bounce, Per-Note LFO/Vibrato/Tremolo/Smear, Harpa Diatônica & Guitarra).
  - `instruments/mod.rs`: **62 modelos** fisicamente calibrados cobrindo todos os packs do Pianoteq 9 + KIViR Instruments & Bells.
  - `effects/mod.rs`: palco virtual 3-mic, filtro passa-baixas do tampo, largura estéreo, binaural ITD + head-shadow, EQ 3 bandas, **todos os 10 FX do Pianoteq 9** (`Delay`, `Chorus`, `Flanger`, `Phaser`, `Tremolo`, `Wah`, `Fuzz`, `Amp`, `Compressor`, `ParametricEq`), **10 salas de Reverb** (`Concert Hall`, `Cathedral`, `Studio`, `Chamber`, `Taj Mahal`, `Plate`, `Spring`, `Speakers`, `Broceliande`, `Club`) e Limiter brickwall.
  - `audio/mod.rs`: stream `cpal` (PipeWire/ALSA/JACK + fallback headless), transporte MIDI, gravador, CC Learn ao vivo e exportação WAV `hound`.
  - `midi/mod.rs`: parser/escritor SMF `.mid` (Formato 0/1), `midir`, curva de velocity multi-ponto, transposição, 61/88/105 teclas, microtonalidade 16ch e CC mapping.
  - `presets/mod.rs`: **310 presets de fábrica** (5 por instrumento), morphing linear A/B completo e persistência JSON XDG.
- **Frontend (`src/`)**: `App.tsx`, `InstrumentPackBrowser.tsx` (mega-menu 3 colunas por Pack), `VirtualKeyboard.tsx`, `VelocityCurveEditor.tsx`, `MicStageEditor.tsx`, `NoteEditPanel.tsx`, `SavePresetPopover.tsx`, `BounceAndNoteFxPanel.tsx`, `MidiTransportBar.tsx`.

## 3. Resumo do Progresso
- **21/21 funcionalidades `APPROVED`**.
- **55/55 testes Rust** + **33/33 testes Vitest** + `npm run build` + `validate-features.mjs --strict` aprovados.

## 4. Comandos de Execução
- Dev app ao vivo (porta `1430`): `DISPLAY=:0 WAYLAND_DISPLAY=wayland-0 ./scripts/with-sysroot.sh npm run tauri dev`
- Testes Rust: `./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`
- Testes Frontend: `npm test`
- Build Frontend: `npm run build`
- Validação de Matriz: `node scripts/validate-features.mjs --strict`
