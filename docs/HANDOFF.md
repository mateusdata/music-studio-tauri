# Agent Handoff Document (`HANDOFF.md`)

## 1. Objetivo Geral
Desenvolver o **Piano Tect Tauri**, um software de piano virtual profissional para Linux baseado em modelagem física (Rust + Tauri 2 + React + TypeScript), buscando paridade funcional com o Pianoteq 9.3.1 com código próprio e testes objetivos.

## 2. Arquitetura Atual
- **Sysroot Linux**: `.sysroot/` gerado por `scripts/setup-dev-sysroot.sh`. Sempre executar comandos Rust/Tauri com `./scripts/with-sysroot.sh ...`.
- **Backend Rust (`src-tauri/src/`)**:
  - `contracts.rs`: contratos compartilhados (`EngineParams`, `InstrumentModel`, `PresetData`, `MidiConfig`, `EngineTelemetry`).
  - `synthesis/mod.rs`: motor físico polifônico (128 vozes, 8 parciais inarmônicos, martelo por velocity, strike point, 8 temperamentos, pedais sustain/sostenuto/soft/harmonic/rattle, PRO note edit, ruídos mecânicos).
  - `instruments/mod.rs`: 32 modelos fisicamente calibrados em 8 categorias (96 presets de fábrica).
  - `effects/mod.rs`: palco virtual 3-mic (compensação nível/delay), lid, largura estéreo, modo binaural, EQ 3 bandas, FX1..3 (Delay, PEQ, Chorus, Tremolo, Compressor), Reverb (pre-delay, early reflections, room size, tone, highpass), Limiter brickwall.
  - `audio/mod.rs`: stream `cpal` (PipeWire/ALSA/JACK + fallback headless) e exportação WAV `hound`.
  - `midi/mod.rs`: parser MIDI, `midir`, curva de velocity multi-ponto, transposição, 61/88/105 teclas, microtonalidade 16ch, CC mapping, sequência demo e diff hot-plug.
  - `presets/mod.rs`: 96 presets de fábrica, morphing linear A/B e persistência JSON XDG.
- **Frontend (`src/`)**: `App.tsx`, `VirtualKeyboard.tsx`, `ParameterSlider.tsx`, `VelocityCurveEditor.tsx`, `MicStageEditor.tsx`, `NoteEditPanel.tsx`, `SavePresetPopover.tsx`.

## 3. Resumo do Progresso
- Ciclo 1 (7 subagentes em worktrees isolados) concluído e integrado no `main`.
- **13/17 funcionalidades `APPROVED`**, **3 `TESTING`**, **1 `IMPLEMENTED`** (ver `docs/features.json`).
- **39/39 testes Rust** + **23/23 testes Vitest** + `npm run build` + `validate-features.mjs --strict` aprovados.

## 4. Decisões Importantes
- Ver `docs/DECISIONS.md` (ADR-001 a ADR-004).
- Worktrees isolados em `.worktrees/` podem ser limpos ou atualizados com `git merge main` antes do próximo ciclo paralelo.

## 5. Dependências Instaladas
- Rust: `tauri 2`, `serde`, `serde_json`, `cpal 0.15`, `midir 0.10`, `hound 3.5`, `parking_lot 0.12`.
- Node: `react 18`, `vite 6`, `@tauri-apps/api 2`, `@tauri-apps/cli 2`, `vitest 3`, `@testing-library/react 16`, `lucide-react`.

## 6. Comandos de Execução
- Dev app ao vivo (porta `1430`): `DISPLAY=:0 WAYLAND_DISPLAY=wayland-0 ./scripts/with-sysroot.sh npm run tauri dev` (já ativo em `pts/1`).
- Testes Rust: `./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`
- Testes Frontend: `npm test`
- Build Frontend: `npm run build`
- Validação de Matriz: `node scripts/validate-features.mjs --strict`

## 7. Estado dos Testes
- Ver `docs/TEST_RESULTS.md` (39 Rust + 23 Vitest + build + validador aprovados).

## 8. Tarefa Interrompida
- Nenhuma (Ciclo 1 finalizado de forma limpa).

## 9. Próxima Ação Exata
- Ler `docs/NEXT_TASK.md` e executar o Ciclo 2 (fechar testes dos 4 itens restantes `SYNTH-002`, `SYNTH-003`, `MIDI-001`, `FX-001` e adicionar Mallet Bounce / Note Effects / MIDI Player).

## 10. Cuidados para Evitar Regressões
- Nunca iniciar uma segunda instância de `npm run tauri dev` se uma já estiver rodando na porta `1430`.
- Sempre sincronizar os worktrees (`.worktrees/*`) com `main` antes de disparar novos subagentes.
- Nunca promover item para `APPROVED` em `docs/features.json` sem teste real executado após a última edição.
