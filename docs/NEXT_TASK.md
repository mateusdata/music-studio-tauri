# Next Task (`NEXT_TASK.md`)

## Tarefa Imediata (Ciclo 2 — Fechamento de Cobertura + Novos Recursos de Referência)

1. **Fechar validação dos 4 itens restantes da matriz (`SYNTH-002`, `SYNTH-003`, `MIDI-001`, `FX-001`)**:
   - Criar testes unitários isolados em `src-tauri/src/synthesis/mod.rs` para `sympathetic_resonance` / `duplex_scale_resonance` (`SYNTH-002`) e `hammer_noise` / `damper_noise` / `pedal_noise` (`SYNTH-003`).
   - Criar testes unitários em `src-tauri/src/effects/mod.rs` verificando atenuação espectral de `lid_position` e crossfeed `binaural_mode` (`FX-001`).
   - Testar conexão MIDI ao vivo via porta virtual ALSA (`snd-virmidi` ou `aconnect` loopback) para promover `MIDI-001` a `APPROVED`.

2. **Expandir Paridade com Pianoteq 9.3.1 (conforme `docs/REFERENCE_PARAMETERS.json`)**:
   - Expor e implementar `paramgroup_bounce` (Mallet Bounce), `paramgroup_notefx` (Vibrato, Tremolo por nota, Smear, Onset Duration) e modo `Guitar` / `Harp` diatônico.
   - Conectar o gerador/player MIDI (`generate_demo_midi_sequence` e arquivos `.mid`) a comandos Tauri + controles de transporte na barra superior.
   - Conectar o avaliador CC (`evaluate_cc_mapping`) ao roteador MIDI em `audio/mod.rs` e adicionar UI de MIDI Learn.
