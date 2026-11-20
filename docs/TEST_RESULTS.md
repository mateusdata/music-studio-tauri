# Test Results (`TEST_RESULTS.md`)

Registro objetivo de execuções reais após o upgrade de paridade acústica, 10 FX do Pianoteq 9, 62 instrumentos (incl. KIViR & Bells) e navegador por Packs.

## Execução Integrada — 2026-11-20

### 1. Rust Backend & DSP (`./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`)
- **Status**: **PASSED (55 passed, 0 failed, 0 ignored)** em `8.43s`
- **Cobertura observada por módulo**:
  - `synthesis::tests` (18 testes): inclui `test_32_partial_trichord_and_soundboard_body_richness` (32 parciais inarmônicos, tricórdio 1/2/3 cordas, decaimento em dois estágios *prompt + aftersound*, tampo harmônico de 4 modos), além dos testes de diapasão, 8 temperamentos, velocidade, pedais, ruídos mecânicos, ressonância simpática/duplex, *Mallet Bounce*, *NoteFx*, *Harpa Diatônica / Guitarra*, *PRO Note Edit* e polifonia 64 vozes.
  - `effects::tests` (13 testes): inclui `test_flanger_and_phaser_modulation`, `test_wah_fuzz_and_amp_spectral_shaping`, `test_reverb_environment_presets_differentiation` (10 ambientes de reverb), `test_lid_position_spectral_attenuation`, `test_stereo_width_mono_to_wide_separation`, `test_binaural_mode_interaural_crossfeed_and_head_angle`, além de EQ, Delay, Chorus, Tremolo, Compressor e Limiter.
  - `instruments::tests` (4 testes): **62 modelos** em 8 categorias e 8 excitadores (incluindo todos os pacotes do Pianoteq 9 + KIViR Instruments & Bells).
  - `presets::tests` (2 testes): **310 presets de fábrica** (5 por instrumento, incluindo presets FX dedicados) + morphing A/B + round-trip JSON.
  - `midi::tests` (13 testes): SMF `.mid` Formato 0/1, dispatch de stream, hot-plug, curva de velocity, microtonalidade 16ch e CC mapping.
  - `audio::tests` (5 testes): transporte MIDI, gravador, CC Learn, renderização em tempo real e exportação WAV `hound`.

### 2. Frontend React/TS & Contratos (`npm test` via Vitest v3.2.7)
- **Status**: **PASSED (33 passed, 0 failed)** em `1.77s`
- **Arquivos**:
  - `tests/unit/engine_contracts.test.ts`: 16/16 aprovados.
  - `tests/ui/app.test.tsx`: 7/7 aprovados.
  - `src/components/StudioComponents.test.tsx`: 10/10 aprovados (incluindo `InstrumentPackBrowser` multi-coluna por Packs, seletor dos 10 FX + parâmetros dinâmicos e 10 salas de Reverb).

### 3. Build de Produção (`npm run build` via `tsc && vite build`)
- **Status**: **PASSED** (`exit 0`, 1600 módulos transformados em `916ms`; CSS `19.65 kB`, JS `229.21 kB`).

### 4. Validador da Matriz (`node scripts/validate-features.mjs --strict`)
- **Status**: **PASSED** (`21/21` funcionalidades `APPROVED`, 0 avisos, 0 erros).
