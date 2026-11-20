# Test Results (`TEST_RESULTS.md`)

Registro objetivo de execuções reais após a conclusão e integração do **Ciclo 2**.

## Execução Integrada — 2026-11-20 (Ciclo 2 Completo no `main`)

### 1. Rust Backend & DSP (`./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`)
- **Status**: **PASSED (51 passed, 0 failed, 0 ignored)** em `2.19s`
- **Cobertura observada por módulo**:
  - `synthesis::tests` (17 testes): inclui `test_sympathetic_and_duplex_scale_resonance_isolation`, `test_hammer_damper_and_pedal_mechanical_noises_isolation`, `test_mallet_bounce_multi_strike_decay`, `test_notefx_lfo_vibrato_tremolo_and_smear`, `test_diatonic_harp_pedals_and_guitar_mode`, além dos 12 testes do Ciclo 1 (diapasão, 8 temperamentos, velocidade, 5 testes de pedais, PRO note edit, sample rate/buffers, polifonia 64 vozes).
  - `audio::tests` (5 testes): inclui `test_midi_transport_recorder_and_cc_learn`, `test_render_dsp_slice_64_voice_polyphony_and_dynamic_buffers`, `test_audio_manager_pedals_and_preset_morph`, `test_offline_wav_export_via_hound`, `test_offline_wav_export_with_selected_instrument`.
  - `effects::tests` (10 testes): inclui `test_lid_position_spectral_attenuation`, `test_stereo_width_mono_to_wide_separation`, `test_binaural_mode_interaural_crossfeed_and_head_angle`, além dos 7 testes de microfones, EQ, Delay, Chorus/Tremolo, Compressor, Reverb e Limiter.
  - `midi::tests` (13 testes): inclui `test_smf_midi_file_roundtrip`, `test_smf_format1_running_status_tempo_and_errors`, `test_live_or_loopback_midi_stream_dispatch`, além dos 10 testes de parsing, curva de velocity, transposição, microtonalidade 16ch, CC mapping e hot-plug diff.
  - `instruments::tests` (4 testes): 32 modelos, 8 categorias, limites físicos e diferenciação acústica.
  - `presets::tests` (2 testes): morphing A/B (incluindo `bounce`, `note_fx` e `guitar_body`) e round-trip JSON.

### 2. Frontend React/TS & Contratos (`npm test` via Vitest v3.2.7)
- **Status**: **PASSED (31 passed, 0 failed)** em `1.73s`
- **Arquivos**:
  - `tests/unit/engine_contracts.test.ts`: 16/16 aprovados (contratos Ciclo 1 + Ciclo 2, matemática de harpa 7 pedais e decaimento geométrico de Mallet Bounce).
  - `tests/ui/app.test.tsx`: 7/7 aprovados.
  - `src/components/StudioComponents.test.tsx`: 8/8 aprovados (incluindo `BounceAndNoteFxPanel` e `MidiTransportBar`).

### 3. Build de Produção (`npm run build` via `tsc && vite build`)
- **Status**: **PASSED** (`exit 0`, 1599 módulos transformados em `999ms`; CSS final `15.14 kB` com `color-scheme: dark` e `appearance: none` no `<select>`).

### 4. Validador da Matriz (`node scripts/validate-features.mjs --strict`)
- **Status**: Pendente reexecução logo após a gravação deste documento (ver saída do comando abaixo).

### 5. Execução Ao Vivo (Tauri 2 Desktop)
- **Status**: Processo `target/debug/piano-tect-tauri` e servidor Vite (`vite` porta `1430`) em execução contínua em `pts/1` com hot reload das alterações do Ciclo 2.
