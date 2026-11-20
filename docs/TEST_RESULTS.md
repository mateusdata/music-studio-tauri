# Test Results (`TEST_RESULTS.md`)

Registro objetivo de execuções reais após integração no `main` (`1be1299`).

## Execução Integrada — 2026-11-20 (Pós-Merge das 7 Branches)

### 1. Rust Backend & DSP (`./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`)
- **Status**: **PASSED (39 passed, 0 failed, 0 ignored)** em `1.79s`
- **Cobertura observada**:
  - `synthesis::tests` (12 testes): `test_diapason_and_a4_tuning_accuracy`, `test_all_8_temperaments_a4_and_intervals`, `test_velocity_dynamics_monotonicity`, `test_soft_pedal_reduces_level_and_brightness`, `test_key_release_mechanical_noise`, `test_pro_note_edit_per_key_offsets`, `test_sample_rate_and_dynamic_buffer_sizes`, `test_rattle_pedal_buzz_and_threshold`, `test_harmonic_pedal_damps_fundamental_sustains_overtones`, `test_sostenuto_pedal_selective_hold`, `test_sustain_and_reversed_sustain_pedals`, `test_64_voice_polyphony_stress`.
  - `audio::tests` (4 testes): `test_render_dsp_slice_64_voice_polyphony_and_dynamic_buffers`, `test_audio_manager_pedals_and_preset_morph`, `test_offline_wav_export_via_hound`, `test_offline_wav_export_with_selected_instrument`.
  - `instruments::tests` (4 testes): `factory_instrument_count_and_unique_metadata` (32 modelos), `all_eight_instrument_categories_covered`, `physical_parameter_sanity_bounds`, `acoustic_differentiation_across_families`.
  - `effects::tests` (7 testes): `test_mic_spatializer_level_and_delay_compensation`, `test_delay_echo_tail`, `test_brickwall_limiter_and_zero_nan_inf`, `test_compressor_gain_reduction`, `test_eq_boost_and_cut_spectral_rms`, `test_chorus_and_tremolo_modulation`, `test_reverb_tail_predelay_and_early_reflections`.
  - `presets::tests` (2 testes): `test_preset_morph_ab_linearity`, `test_factory_presets_and_json_roundtrip`.
  - `midi::tests` (10 testes): `parse_note_on_and_vel_zero_note_off`, `parse_control_change_pedals_and_channel_messages`, `parse_invalid_or_truncated_midi_returns_none`, `velocity_curve_interpolation_and_monotonicity`, `transpose_and_key_range_clamping`, `microtonal_16_channel_offset_and_bijection`, `cc_mapping_evaluation`, `demo_midi_sequence_is_well_formed`, `port_hotplug_diff_detection`, `midi_port_discovery_and_missing_port_connect_error`.

### 2. Frontend React/TS & Contratos (`npm test` via Vitest v3.2.7)
- **Status**: **PASSED (23 passed, 0 failed)** em `1.45s`
- **Arquivos**:
  - `tests/unit/engine_contracts.test.ts`: 10/10 aprovados.
  - `tests/ui/app.test.tsx`: 7/7 aprovados.
  - `src/components/StudioComponents.test.tsx`: 6/6 aprovados.

### 3. Build de Produção (`npm run build` via `tsc && vite build`)
- **Status**: **PASSED** (`exit 0`, 1597 módulos transformados em `860ms`).

### 4. Validador da Matriz (`node scripts/validate-features.mjs --strict`)
- **Status**: **PASSED** (`17 features checked, 0 warnings, 0 schema errors`).

### 5. Execução Ao Vivo (Tauri 2 Desktop)
- **Status**: Processo `target/debug/piano-tect-tauri` (PID `318486`) e servidor Vite (`vite` porta `1430`, PID `275777`) recompilados e ativos após o merge em `pts/1`.
- **Pendências de Teste (Explícitas)**:
  - Teste manual/hardware de entrada MIDI com controlador físico ou porta virtual ALSA (`MIDI-001` em `TESTING`).
  - Testes unitários isolados de ressonância simpática/duplex (`SYNTH-002`), ruído de martelo/abafador (`SYNTH-003`) e atenuação de tampo/binaural (`FX-001`).
