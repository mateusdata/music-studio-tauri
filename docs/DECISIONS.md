# Architectural Decisions Log (`DECISIONS.md`)

## ADR-001: Tauri 2 + Rust Audio Engine + React/TypeScript UI
- **Data**: 2026-11-20
- **Decisão**: Utilizar Tauri 2 com backend DSP integralmente em Rust (`cpal`, `midir`, `hound`, `parking_lot`) e frontend React 18 + TypeScript + Vite.
- **Justificativa**: Garante baixa latência nativa no Linux (PipeWire/ALSA/JACK) sem overhead de Electron/JVM e mantém separação limpa entre DSP real-time e UI webview.

## ADR-002: Sysroot de Desenvolvimento Linux Não-Root (`.sysroot/`)
- **Data**: 2026-11-20
- **Decisão**: Como `sudo` não é interativo no ambiente de execução e `libwebkit2gtk-4.1-dev` / `libgtk-3-dev` não estavam instalados globalmente (apenas os runtimes `.so.0`), os pacotes `-dev` são baixados via `apt-get download` e extraídos em `.sysroot/` via `scripts/setup-dev-sysroot.sh`, sendo injetados via `scripts/with-sysroot.sh`.
- **Justificativa**: Permite compilar e executar `npm run tauri dev` integralmente sem privilégios root e sem modificar arquivos do sistema host.

## ADR-003: Contratos de Dados Centralizados em `contracts.rs`
- **Data**: 2026-11-20
- **Decisão**: Todos os subagentes operam sobre diretórios exclusivos mantendo as assinaturas definidas em `src-tauri/src/contracts.rs` e `src/types/engine.ts`.
- **Justificativa**: Permite execução paralela de 7 subagentes em branches/worktrees isolados sem conflitos de merge.

## ADR-004: Implementação Própria de Síntese Física (Zero Reaproveitamento Proprietário)
- **Data**: 2026-11-20
- **Decisão**: Nenhuma biblioteca proprietária ou código extraído do binário de referência é linkado ou copiado; toda a síntese, temperamentos e DSP são implementados do zero em Rust usando física acústica padrão (corda rígida inarmônica, filtro de martelo, ressonância modal e reverberação algorítmica).
