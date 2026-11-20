# Roadmap — Piano Tect Tauri

## Fase A — Reconhecimento & Sysroot (Concluída)
- Inspeção do ambiente Linux (Rust 1.98.1, Node 24, PipeWire 1.6.8, Wayland/X11).
- Análise REA (`inspect_native_exports`, `inspect_binary_hardening`) e extração de contrato LV2 (`dsp.ttl`) e manual HTML do Pianoteq 9.3.1.
- Configuração de sysroot `.sysroot/` não-root e verificação de linkagem WebKit2GTK 4.1 + ALSA + JACK.

## Fase B & C — Fundação Tauri 2 & Execução Visual Imediata (Em execução agora)
- Scaffold completo `src-tauri/` + `src/` com contratos compartilhados (`contracts.rs`).
- Motor físico inicial polifônico (128 vozes), 9 instrumentos iniciais, presets, efeitos, entrada MIDI e teclado virtual de 88 teclas.
- Inicialização contínua do app em `npm run tauri dev`.

## Fase D — Expansão Paralela Multiagente (7 Subagentes em Worktrees Isolados)
1. **Subagente 01 (Análise & REA)**: Extrair especificação integral dos 215 parâmetros LV2 e recursos avançados (Syngular, Guitar, Harp, Bounce, Note Edit) para `docs/REFERENCE_PARAMETERS.json` e `docs/RESEARCH_NOTES.md`.
2. **Subagente 02 (Motor de Áudio & Síntese)**: Aprimorar modelagem física (ressonância simpática por parciais acoplados, duplex scale, blooming dinâmico, ruído de soltura de tecla, pedais históricos celeste/mozart/rattle/buff, precisão de afinação).
3. **Subagente 03 (Interface Tauri React/TS)**: Interface completa estilo estúdio Pianoteq 9 (canvas de curva de velocity editável, palco 2D/3D interativo de microfones, editor Note Edit por tecla, visualizador de espectro, painel MIDI CC Learn e navegação profissional).
4. **Subagente 04 (Sistema MIDI)**: Hot-plug de portas MIDI, MIDI CC Learn dinâmico, modo microtonal 16 canais, curvas de velocidade multi-ponto, suporte 61/88/105 teclas, player/gravador de sequências MIDI integrado.
5. **Subagente 05 (Efeitos & Presets)**: Simulação espacial de até 3 microfones com atraso/nível por distância física, EQ paramétrico/gráfico completo, FX1/FX2/FX3 (Delay Ping-Pong, EQ, Chorus, Tremolo, Compressor), Reverb completo com Early Reflections, presets e morphing A/B.
6. **Subagente 07 (Instrumentos & Acústica)**: Expansão do catálogo para cobrir todas as famílias de referência (Grands, Uprights, Rhodes/Wurlitzer/Clavinet/Pianet, Harpsichord/Pianoforte históricos, Mallets, Steelpans, Guitar, Harp, Prepared, Syngular) com perfis acústicos calibrados.
7. **Subagente 06 (QA & Testes)**: Suíte abrangente de testes unitários, áudio (frequência A4, inarmonicidade, clipping, polifonia, NaN-free), MIDI, UI (Vitest/RTL), desempenho/CPU e script validador de `docs/features.json`.

## Fase E — Integração Central, Regressão e Hardening Final
- Merge ordenado das branches no `main` pelo Coordenador.
- Validação de regressão completa e atualização da Matriz Mestre (`APPROVED` somente após execução real).
