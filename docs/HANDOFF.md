# Agent Handoff Document (`HANDOFF.md`)

## 1. Objetivo Geral
Desenvolver o **Piano Tect Tauri**, um software de piano virtual profissional para Linux baseado em modelagem física (Rust + Tauri 2 + React + TypeScript), buscando paridade funcional com o Pianoteq 9.3.1 com código próprio e testes objetivos.

## 2. Arquitetura Atual
- **Sysroot Linux**: `.sysroot/` gerado por `scripts/setup-dev-sysroot.sh`. Sempre executar comandos Rust/Tauri com `./scripts/with-sysroot.sh ...`.
- **Backend Rust (`src-tauri/src/`)**:
  - `contracts.rs`: contratos compartilhados (`EngineParams`, `BounceParams`, `NoteFxParams`, `PlayModeParams`, `MidiTransportState`, `InstrumentModel`, `PresetData`, `MidiConfig`, `EngineTelemetry`).
  - `synthesis/mod.rs`: motor físico polifônico (128 vozes, 8 parciais inarmônicos, martelo por velocity, strike point, 8 temperamentos, 7+ pedais, PRO Note Edit, Mallet Bounce, Per-Note LFO/Vibrato/Tremolo/Smear, modo Harpa Diatônica 7 pedais & Guitarra, ruídos mecânicos, ressonância simpática & duplex).
  - `instruments/mod.rs`: 32 modelos fisicamente calibrados em 8 categorias (96 presets de fábrica).
  - `effects/mod.rs`: palco virtual 3-mic (compensação nível/delay), filtro passa-baixas do tampo (`lid_position`), largura estéreo, binaural ITD + head-shadow, EQ 3 bandas, FX1..3 (Delay, PEQ, Chorus, Tremolo, Compressor), Reverb (pre-delay, early reflections, room size, tone, highpass), Limiter brickwall.
  - `audio/mod.rs`: stream `cpal` (PipeWire/ALSA/JACK + fallback headless), transporte MIDI em thread dedicada com velocidade `0.25x..2.0x`, gravador, CC Learn ao vivo, microtonalidade 16ch e exportação WAV `hound`.
  - `midi/mod.rs`: parser/escritor SMF `.mid` (Format 0/1), `midir`, curva de velocity multi-ponto, transposição, 61/88/105 teclas, microtonalidade 16ch, CC mapping, sequência demo e diff hot-plug.
  - `presets/mod.rs`: 96 presets de fábrica, morphing linear A/B completo e persistência JSON XDG.
- **Frontend (`src/`)**: `App.tsx`, `VirtualKeyboard.tsx`, `ParameterSlider.tsx`, `VelocityCurveEditor.tsx`, `MicStageEditor.tsx`, `NoteEditPanel.tsx`, `SavePresetPopover.tsx`, `BounceAndNoteFxPanel.tsx`, `MidiTransportBar.tsx` + tema escuro WebKitGTK corrigido em `src/styles/app.css`.

## 3. Resumo do Progresso
- **Ciclo 1 e Ciclo 2 concluídos**: **21/21 funcionalidades `APPROVED`**.
- **51/51 testes Rust** + **31/31 testes Vitest** + `npm run build` + `validate-features.mjs --strict` aprovados.

## 4. Comandos de Execução
- Dev app ao vivo (porta `1430`): `DISPLAY=:0 WAYLAND_DISPLAY=wayland-0 ./scripts/with-sysroot.sh npm run tauri dev`
- Testes Rust: `./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`
- Testes Frontend: `npm test`
- Build Frontend: `npm run build`
- Validação de Matriz: `node scripts/validate-features.mjs --strict`

## 5. Próxima Ação Exata
- Consultar `docs/NEXT_TASK.md` para o Ciclo 3 opcional (`.scl`/`.kbm`, impulsos `.wav` customizados, `.mid` file picker, empacotamento Linux).
