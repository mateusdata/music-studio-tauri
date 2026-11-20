# Next Task (`NEXT_TASK.md`)

1. Concluir validação de baseline (`npm run build`, `npm test`, `cargo test`) e manter `npm run tauri dev` executando em sessão contínua.
2. Criar os 7 git worktrees / branches isoladas e despachar os subagentes especializados em paralelo:
   - `feature/rea-analysis` (Subagente 01: Análise & REA)
   - `feature/audio-engine` (Subagente 02: Motor de Áudio & Síntese Física)
   - `feature/ui` (Subagente 03: Interface Tauri React/TS)
   - `feature/midi` (Subagente 04: Sistema MIDI)
   - `feature/dsp-effects` (Subagente 05: Efeitos & Presets)
   - `feature/instruments-acoustics` (Subagente 07: Instrumentos & Acústica)
   - `test/validation` (Subagente 06: Testes & QA)
3. Integrar as entregas no `main`, validar com regressão completa e atualizar `docs/features.json` e `docs/FUNCTIONALITY_MATRIX.md`.
