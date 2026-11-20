# Matriz Mestre de Funcionalidades (`FUNCTIONALITY_MATRIX.md`)

> Fonte estruturada: `docs/features.json`  
> Regra de aprovação: **Somente funcionalidades efetivamente testadas e aprovadas em execução real recebem o estado `APPROVED`.**

| ID | Categoria | Nome | Complexidade | Prioridade | Implementação | Teste | Resultado do Teste | Estado | Última Validação |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `SYNTH-001` | Instruments & Synthesis | Stiff-String Physical Modeling Voice Engine | high | P0 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/synthesis_tests.rs` | Pendente execução | `IMPLEMENTED` | — |
| `SYNTH-002` | Instruments & Synthesis | Sympathetic & Duplex Scale Resonance | high | P1 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/resonance_tests.rs` | Pendente execução | `IMPLEMENTED` | — |
| `SYNTH-003` | Instruments & Synthesis | Mechanical Action & Hammer/Damper/Pedal Noises | medium | P1 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/noise_tests.rs` | Pendente execução | `IMPLEMENTED` | — |
| `SYNTH-004` | Instruments & Synthesis | Tuning, Diapason, Octave Stretching & Historical Temperaments | medium | P0 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/tuning_tests.rs` | Pendente execução | `IMPLEMENTED` | — |
| `SYNTH-005` | Instruments & Synthesis | Pedal State Machine | medium | P0 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/pedal_tests.rs` | Pendente execução | `IMPLEMENTED` | — |
| `SYNTH-006` | Instruments & Synthesis | PRO Per-Note Edit | medium | P2 | `src-tauri/src/synthesis/mod.rs` | `tests/audio/note_edit_tests.rs` | Pendente | `SPECIFIED` | — |
| `INST-001` | Instruments & Presets | Multi-Family Physical Instrument Catalog | high | P1 | `src-tauri/src/instruments/mod.rs` | `tests/integration/instruments_tests.rs` | Pendente | `IMPLEMENTING` | — |
| `INST-002` | Instruments & Presets | Preset Browser, A/B Morphing & Disk Persistence | medium | P1 | `src-tauri/src/presets/mod.rs` | `tests/integration/presets_tests.rs` | Pendente | `IMPLEMENTED` | — |
| `MIDI-001` | MIDI System | Real-Time MIDI Input Discovery, Routing & Hot-Plug | high | P0 | `src-tauri/src/midi/mod.rs` | `tests/integration/midi_tests.rs` | Pendente | `IMPLEMENTING` | — |
| `MIDI-002` | MIDI System | Velocity Curve LUT, Transpose, Channel Filter & Key Modes | medium | P1 | `src-tauri/src/midi/mod.rs` | `tests/integration/midi_tests.rs` | Pendente | `IMPLEMENTING` | — |
| `AUDIO-001` | Audio Processing | Low-Latency Linux Audio Engine & Telemetry | high | P0 | `src-tauri/src/audio/mod.rs` | `tests/performance/audio_perf_tests.rs` | Pendente | `IMPLEMENTED` | — |
| `AUDIO-002` | Audio Processing | Offline WAV Export | low | P1 | `src-tauri/src/audio/mod.rs` | `tests/audio/wav_export_tests.rs` | Pendente | `IMPLEMENTED` | — |
| `FX-001` | Audio Processing | 3-Mic Acoustic Stage, Lid, Stereo Width & Binaural | high | P1 | `src-tauri/src/effects/mod.rs` | `tests/audio/effects_tests.rs` | Pendente | `IMPLEMENTING` | — |
| `FX-002` | Audio Processing | Equalizer, 3 FX Slots, Reverb & Limiter | high | P1 | `src-tauri/src/effects/mod.rs` | `tests/audio/effects_tests.rs` | Pendente | `IMPLEMENTING` | — |
| `UI-001` | UI & Experience | Studio Main Window, 88/105-Key Keyboard & Hotkeys | high | P0 | `src/App.tsx` | `tests/ui/app.test.tsx` | Pendente re-execução | `IMPLEMENTED` | — |
| `UI-002` | UI & Experience | Interactive Editors (Velocity Curve, 2D Stage, Note Edit, MIDI) | high | P1 | `src/components/` | `tests/ui/editors.test.tsx` | Pendente | `SPECIFIED` | — |
| `REA-001` | Reference Research | Complete 215-Parameter LV2 & Manual Extraction | medium | P1 | `docs/REFERENCE_PARAMETERS.json` | `scripts/validate-features.mjs` | Pendente | `SPECIFIED` | — |
