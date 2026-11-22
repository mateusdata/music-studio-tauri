# Piano Tect Tauri

Software de piano virtual profissional baseado em modelagem física em Rust, Tauri 2, React e TypeScript, projetado para Linux (PipeWire/ALSA/JACK) com paridade funcional progressiva com o Pianoteq 9.3.1 através de implementação própria e verificável.

## Estrutura do Projeto

- `src/` — Interface desktop React + TypeScript (teclado virtual de 88/105 teclas, pedais, controles em tempo real, visualizações acústicas e telemetria).
- `src-tauri/src/contracts.rs` — Contratos de tipos compartilhados entre módulos e IPC.
- `src-tauri/src/synthesis/` — Motor de síntese por modelagem física (cordas rígidas inarmônicas, martelo dinâmico por velocidade, perfil espectral de 8 parciais, ressonância simpática/duplex, ruído mecânico, temperamentos).
- `src-tauri/src/instruments/` — Modelos acústicos e famílias de instrumentos.
- `src-tauri/src/effects/` — Cadeia DSP pós-síntese (palco binaural/microfones, EQ, Delay/Chorus/Tremolo, Reverb, Limiter).
- `src-tauri/src/audio/` — Engine de áudio de baixa latência (`cpal`) e exportação WAV (`hound`).
- `src-tauri/src/midi/` — Sistema MIDI (`midir`) com curva de velocidade, mapeamento CC, transposição e filtros.
- `src-tauri/src/presets/` — Gerenciador de presets, morphing A/B e persistência em disco.
- `docs/` — Matriz mestre de funcionalidades (`features.json`, `FUNCTIONALITY_MATRIX.md`), pesquisa REA, decisões arquiteturais e protocolo de continuidade (`HANDOFF.md`, `CURRENT_STATE.md`, `NEXT_TASK.md`).
- `tests/` — Testes de UI, unidade, áudio, integração e desempenho.

## Como Executar em Desenvolvimento

O projeto inclui um sysroot de desenvolvimento não-root em `.sysroot/` para Linux (`scripts/setup-dev-sysroot.sh` e `scripts/with-sysroot.sh`).

```bash
# 1. Caso .sysroot ainda não exista:
./scripts/setup-dev-sysroot.sh

# 2. Executar o aplicativo Tauri 2 com hot-reload:
./scripts/with-sysroot.sh npm run tauri dev

# 3. Rodar testes Rust e Frontend:
./scripts/with-sysroot.sh cargo test --manifest-path src-tauri/Cargo.toml
npm test
```

## Protocolo de Retomada para Novos Agentes

1. Leia `README.md`, `docs/HANDOFF.md`, `docs/CURRENT_STATE.md` e `docs/NEXT_TASK.md`.
2. Consulte `docs/features.json` e `docs/FUNCTIONALITY_MATRIX.md`.
3. Execute `git status` e os testes existentes antes de iniciar novas implementações.
# music-studio-tauri
