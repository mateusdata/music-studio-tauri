# Agent Handoff Document (`HANDOFF.md`)

## 1. Objetivo Geral
Desenvolver o **Piano Tect Tauri**, um software de piano virtual profissional para Linux baseado em modelagem física (Rust + Tauri 2 + React + TypeScript), buscando máxima paridade funcional com o Pianoteq 9.3.1 mediante implementação própria, código aberto/autorizado e testes objetivos.

## 2. Arquitetura Atual
- **Sysroot Linux**: `.sysroot/` gerado por `scripts/setup-dev-sysroot.sh` (contém headers/pkgconfig GTK3, WebKit2GTK 4.1, libsoup-3.0, librsvg2, ayatana-appindicator3). Todo comando `cargo` ou `tauri` deve usar `./scripts/with-sysroot.sh ...`.
- **Backend Rust (`src-tauri/src/`)**:
  - `contracts.rs`: contratos canônicos compartilhados.
  - `synthesis/mod.rs`: motor modal/corda rígida de 128 vozes.
  - `instruments/mod.rs`: perfis físicos por instrumento.
  - `effects/mod.rs`: cadeia estéreo/binaural, EQ, FX slots, reverb e limiter.
  - `audio/mod.rs`: supervisor `cpal` (PipeWire/ALSA/JACK + fallback headless) e exportação WAV.
  - `midi/mod.rs`: parser MIDI, `midir` input, curvas de velocity, transposição.
  - `presets/mod.rs`: presets de fábrica, morphing A/B e persistência XDG.
  - `commands/mod.rs`: comandos Tauri IPC.
- **Frontend (`src/`)**: React 18 + Vite + TypeScript + `lucide-react`.

## 3. Resumo do Progresso
- Ambiente verificado, evidências REA coletadas, sysroot funcional, projeto Tauri 2 estruturado e compilando.

## 4. Decisões Importantes
- Ver `docs/DECISIONS.md` (ADR-001 a ADR-004). Não modificar `contracts.rs` sem coordenação central.

## 5. Dependências Instaladas
- Rust: `tauri 2`, `serde`, `serde_json`, `cpal 0.15`, `midir 0.10`, `hound 3.5`, `parking_lot 0.12`.
- Node: `react 18`, `vite 6`, `@tauri-apps/api 2`, `@tauri-apps/cli 2`, `vitest 3`, `@testing-library/react 16`, `lucide-react`.

## 6. Comandos de Execução
- Dev app ao vivo: `./scripts/with-sysroot.sh npm run tauri dev`
- Testes Rust: `./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml`
- Testes UI: `npm test`
- Build Frontend: `npm run build`

## 7. Estado dos Testes
- Ver `docs/TEST_RESULTS.md`.

## 8. Tarefa Atual / Interrompida
- Preparação de baseline e disparo dos 7 subagentes em worktrees isolados.

## 9. Próxima Ação Exata
- Ler `docs/NEXT_TASK.md` e executar a etapa pendente ou revisar entregas dos subagentes.

## 10. Cuidados para Evitar Regressões
- Nunca executar operações bloqueantes ou alocações dinâmicas no callback `cpal`.
- Sempre envolver comandos Rust/Tauri em `./scripts/with-sysroot.sh`.
- Manter separação de arquivos por subagente e apenas marcar `APPROVED` em `features.json` após teste real executado.
