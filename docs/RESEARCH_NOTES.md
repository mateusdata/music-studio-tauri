# Research Notes — Pianoteq 9.3.1 Technical Reference Inventory

> Artefato estruturado associado: [`docs/REFERENCE_PARAMETERS.json`](REFERENCE_PARAMETERS.json)  
> Política epistemológica: Separação estrita entre **Evidências Observadas**, **Hipóteses Técnicas**, **Descobertas Confirmadas** e **Questões em Aberto**.

---

## 1. Evidências Observadas (REA, LV2, CLI & Manual Oficial)

### 1.1 Artefatos Binários Inspecionados via REA
- **`Pianoteq 9` (Standalone ELF64 amd64, build `9.3.1/20261118`)**:
  - Path: `/home/data/projects/pianoteq-eng-reverse2/Pianoteq-v931/x86-64bit/Pianoteq 9`
  - SHA-256: `1d0b7f62548ca311b0526b9b2f18f7f11fb7e81f63a82e3c91a9800b89fe385f`
  - Tamanho: `58,939,184` bytes
  - Evidence IDs REA: `ev_1dd73cd50e6511cbe09bccf7cfcbbef5e6dd6f1f57b8cca204080aa36c2cec68`, `ev_6960eaa80c9741c870ae6df79b36a5abef3d7df483902d4d1960daebd9dc291b`, `ev_eeef488bf478d9168109abd7f4b9b514ba91054bb28d29c5c188a03ea5f26c65`
  - Mitigações/Linkage (`inspect_binary_hardening` + `readelf`): PIE + NX + Full RELRO + Stack Protector + Fortify + Stripped; dependências dinâmicas exclusivas: `libGL.so.1`, `libasound.so.2`, `libEGL.so.1`, `libfreetype.so.6`, `libcurl.so.4`, `libdl.so.2`, `libfontconfig.so.1`, `libm.so.6`, `libgcc_s.so.1`, `libpthread.so.0`, `libc.so.6`.
  - Strings internas em `.rodata` encontram-se empacotadas/protegidas no binário final (verificado por ausência de símbolos literais de parâmetros como `blooming_energy` em `strings` direto).
- **`Pianoteq_9.so` (LV2 DSP + X11UI Shared Library ELF64 amd64)**:
  - Path: `/home/data/projects/pianoteq-eng-reverse2/Pianoteq-v931/x86-64bit/Pianoteq 9.lv2/Pianoteq_9.so`
  - SHA-256: `8312e07d64000b38c4b92f695f2ce2b3ebbbc82038289cf0204431e0144b4e8a`
  - Tamanho: `55,374,656` bytes
  - Evidence IDs REA: `ev_a116654a8945b4d1b33cf66d56666cc3e5fb62588bf58fa73bd243b0afb8ebb1`, `ev_61bcc0c2031feca2f79a8aaf33adfe3025a41c560c69609c8db9934fcb697424` (4 símbolos LV2 exportados).

### 1.2 Contrato Público LV2 (`manifest.ttl`, `dsp.ttl`, `ui.ttl`)
Inspeção integral de `Pianoteq 9.lv2/dsp.ttl` (3.039 linhas) observou:
- **URI do Plugin / UI**: `<https://www.modartt.com/lv2/Pianoteq9>` (`lv2:minorVersion 3`, `lv2:microVersion 1`, `doap:revision "9.3.1"`, `lv2:hardRTCapable`) e `<https://www.modartt.com/lv2/Pianoteq9:UI>` (`ui:X11UI` com `ui:scaleFactor` e `param:sampleRate`).
- **22 Portas LV2**:
  - Índice `0`: `audio_in_1` (`lv2:InputPort`, `lv2:AudioPort`, `pg:group plug:input_group_1`, `center`, `lv2:connectionOptional`) — entrada mono opcional (alimenta processamento de áudio externo).
  - Índices `1..16`: `audio_out_1..audio_out_16` divididos em **8 buses estéreo** (`output_group_1..8`), sendo `output_group_1` (`1..2`) obrigatório e `output_group_2..8` (`3..16`) opcionais.
  - Índices `17..18`: `in` (`atom:Sequence`, min `23672` bytes, aceita `midi:MidiEvent`, `patch:Message`, `time:Position`) e `out` (`atom:Sequence`, min `23672` bytes, `patch:Message`).
  - Índices `19..21`: `latency` (output frames), `freeWheeling` (input toggle `0.0`), `enabled` (input toggle `1.0`).
- **215 Parâmetros `lv2:Parameter` (`atom:Float`, intervalo normalizado `[0.0, 1.0]`)** em **15 `pg:Group`**, todos idênticos em `patch:writable` e `patch:readable` (porém ordenados diferentemente entre a declaração de definição e a lista `patch:writable`/`patch:readable`):
  1. `paramgroup_main` (`Main`, 9): `condition`, `dynamics`, `volume`, `velocity_offset`, `post_effect_gain`, `morphing`, `equalizer_switch`, `aftertouch`, `diapason`.
  2. `paramgroup_tuning` (`Tuning`, 5): `pitch_bend`, `unison`, `octave_stretching`, `unison_balance`, `direct_sound`.
  3. `paramgroup_output` (`Output`, 78): `stereo_width`, `bus_1_volume`, `bus_1_pan`, `lid_position`, `mic_level_compensation`, `mic_delay_compensation`, cabeça binaural (`head_x/y/z/diameter/angle`), 8 microfones (`mic_1..8` com `mic_switch`, `x`, `y`, `z`, `azimuth`, `elevation`, `bus_1_volume`, `bus_1_pan`), `sound_speed`, `wall_distance`, `lid`.
  4. `paramgroup_voicing` (`Voicing`, 21): `hammer_noise`, `hammer_hardness`, `hammer_hardness_piano`, `hammer_hardness_mezzo`, `hammer_hardness_forte`, `profil_1..8`, `hammer_tone`, `strike_point`, `pinch_harmonic_point`, `pickup_symmetry`, `pickup_distance`, `soft_level`, `clavinet_low_mic`, `clavinet_high_mic`.
  5. `paramgroup_pedals` (`Pedals`, 16): `sustain_pedal`, `soft_pedal`, `sostenuto_pedal`, `harmonic_pedal`, `rattle_pedal`, `lute_stop_pedal` (`Buff Stop Pedal`), `celeste_pedal`, `mozart_rail`, `super_sostenuto`, `pinch_harmonic_pedal`, `glissando_pedal`, `thunder_pedal`, `harpsichord_register_1..3`, `reversed_sustain`.
  6. `paramgroup_design` (`Design`, 12): `impedance`, `impedance_cutoff`, `q_factor` (`Impedance Slope`), `string_length`, `sympathetic_resonance`, `duplex_scale_resonance`, `quadratic_effect`, `hammer_tine_noise`, `blooming_energy`, `blooming_inertia`, `external_audio_mix`, `external_audio_tone`.
  7. `paramgroup_bounce` (`Mallet bounce`, 8): `bounce_switch`, `bounce_delay`, `bounce_sync`, `bounce_sync_speed`, `bounce_velocity_sensitivity`, `bounce_delay_loss`, `bounce_velocity_loss`, `bounce_humanization`.
  8. `paramgroup_fx1` (`Effect 1`, 9): `effect_1_switch`, `effect_1_param_1..8` (default rotulado como Delay: `DelMix`, `DelTime`, `DelFb`, `DelTone`, `DelPol`, `DelPing`, `Param 7`, `Param 8`).
  9. `paramgroup_fx2` (`Effect 2`, 9): `effect_2_switch`, `effect_2_param_1..8` (default rotulado como Parametric EQ 2 bandas: `PEQ1F/G/Q/T`, `PEQ2F/G/Q/T`).
  10. `paramgroup_fx3` (`Effect 3`, 9): `effect_3_switch`, `effect_3_param_1..8` (default rotulado como Parametric EQ 2 bandas: `PEQ1F/G/Q/T`, `PEQ2F/G/Q/T`).
  11. `paramgroup_reverb` (`Reverb`, 10): `reverb_switch`, `reverb_duration`, `reverb_mix`, `room_dimensions`, `reverb_pre_delay`, `reverb_early_reflections`, `reverb_tone`, `reverb_envelope`, `reverb_resampling`, `reverb_highpass_cutoff`.
  12. `paramgroup_limiter` (`Limiter`, 4): `limiter_switch`, `limiter_sharpness`, `limiter_threshold`, `limiter_gain`.
  13. `paramgroup_action` (`Action`, 8): `mute`, `damper_noise`, `damper_position`, `last_damper`, `pedal_noise`, `key_release_noise`, `damping_duration`, `keyboard_range_switch`.
  14. `paramgroup_notefx` (`Note effects`, 12): `nfx_lfo_shape`, `nfx_lfo_skew`, `nfx_lfo_rate`, `nfx_lfo_phase`, `nfx_lfo_phase_locked`, `nfx_onset_duration`, `nfx_vibrato`, `nfx_vibrato_offset`, `nfx_tremolo_depth`, `nfx_tremolo_phase`, `attack_envelope`, `smear`.
  15. `paramgroup_guitar` (`Guitar`, 5): `virtuosity`, `fret`, `guitar_legato`, `guitar_easy_fingering`, `guitar_body`.
- **29 parâmetros enumerados (`lv2:enumeration` + `lv2:scalePoint`)**: 28 chaves binárias `Off`/`On` (`0`/`1`) e `bounce_sync_speed` com **30 subdivisões rítmicas** (`16d`, `16`, `16t`, `8d`, `8`, `8t`, `4d`, `4`, `4t`, `2d`, `2`, `2t`, `1d`, `1`, `1t`, `1/2d`, `1/2`, `1/2t`, `1/4d`, `1/4`, `1/4t`, `1/8d`, `1/8`, `1/8t`, `1/16d`, `1/16`, `1/16t`, `1/32d`, `1/32`, `1/32t` mapeadas uniformemente em `k/29` de `0` a `1`).

### 1.3 Catálogo CLI (`--list-instruments`, `--list-presets`, `--help`) & Manual HTML
- **`--list-instruments`** retornou exatamente **56 entradas** (53 strings únicas, com 3 nomes exibidos em duplicata: `Vintage Reeds` nos índices 19–20, `Spacedrum` nos índices 35–36 e `Hand Pan` nos índices 37–38).
- **`--list-presets`** retornou exatamente **820 presets** (incluindo prefixos distintos `W2 ...` [37] e `W1 ...` [16] para os dois modelos de `Vintage Reeds`, além de variantes `hand played` e `mallets` para `Spacedrum` e `Hand Pan`).
- **`--help`** confirmou flags oficiais de headless rendering/export (`--headless`, `--midi`, `--wav`, `--flac`, `--mp3`, `--rate`, `--bit-depth`, `--dither`, `--normalize`, `--normalize-db`, `--mono`, `--export-lv2-presets`, `--serve [IP:]PORT`).
- **Manual oficial (`Documentation/pianoteq-english.html`, 1.458 linhas)** forneceu as especificações textuais das seções `1.3`, `4.1–4.3`, `5`, `6.1–6.7`, `7.1–7.4`, `8.1–8.3`, `9.1–9.2`, `10`, `11`, `12.1–12.7`, `13.1–13.6`, `14.1–14.6`, `15.1–15.3`, `16.1–16.12` e `17` (código MATLAB para `pythagore`, `zarlino`, `mesotonic`, `welltempered`, `werck`).

---

## 2. Hipóteses Técnicas (A Validar via DSP / Medição / Análise Adicional)

1. **Equação de Inarmonicidade & Comprimento de Corda**:
   - A inarmonicidade acústica descrita nas seções 8.1.2 e 15.2 é consistente com a relação clássica de corda rígida $f_n = n f_0 \sqrt{1 + B n^2}$, onde o coeficiente $B$ decai rapidamente com o aumento do comprimento efetivo da corda ($B \propto L^{-2}$ para seção/tensão constantes).
2. **Escala dos Parâmetros Normalizados `[0, 1]`**:
   - Enquanto `dsp.ttl` expõe todos os 215 parâmetros em `[0.0, 1.0]`, assume-se por convenção de interface que `0.5` representa posição neutra/central em parâmetros bipolares (e.g., `profil_1..8` = `0 dB`, `hammer_tone` = neutro, `velocity_offset` = `0`, `unison_balance` = `0`, `reverb_tone` = neutro) e escalonamentos logarítmicos para frequências (`impedance_cutoff`, `reverb_highpass_cutoff`, `nfx_lfo_rate`), mas a lei exata de mapeamento normalizado $\rightarrow$ unidade física não consta em `dsp.ttl`.
3. **Filtro Não-Linear de Dureza do Martelo**:
   - Os três pontos de controle `hammer_hardness_piano` (MIDI ~41), `hammer_hardness_mezzo` (MIDI ~70) e `hammer_hardness_forte` (MIDI ~98) provavelmente interpolam uma curva contínua dependente de velocidade que controla largura do pulso de contato martelo-corda / corte espectral passa-baixa.
4. **Identificação das Duplicatas em `--list-instruments`**:
   - Os pares duplicados `Vintage Reeds` (#19/#20) correspondem aos modelos W2 e W1 (`W2 ...` e `W1 ...` em `--list-presets`, citados na Sec. 16.5), e os pares `Spacedrum` (#35/#36) e `Hand Pan` (#37/#38) correspondem às variantes de execução com mão vs baquetas (`hand played` vs `mallets`, Sec. 16.11) ou escalas distintas empacotadas sob o mesmo rótulo base.
5. **Reverberação Paramétrica sobre Convolução**:
   - A combinação de parâmetros de convolução (`room_dimensions`, `reverb_early_reflections`, `reverb_duration`, `reverb_resampling`, `reverb_envelope`) indica síntese ou reamostragem/envelope dinâmico aplicado a respostas ao impulso (IRs) internas ou carregadas via WAV.

---

## 3. Descobertas Confirmadas (Por Subsistema Solicitado)

### 3.1 Voicing (`paramgroup_voicing` — 21 parâmetros, Manual Sec. 7.1–7.4, 14.2, 14.5)
- **Instrumentos Acústicos (Sec. 7.1)**:
  - **Hammer Hardness**: Controle global (`hammer_hardness`) mais 3 bandas ancoradas por velocidade MIDI: `hammer_hardness_piano` (~vel `41`), `hammer_hardness_mezzo` (~vel `70`) e `hammer_hardness_forte` (~vel `98`). Valores maiores aumentam o brilho; na prática pianística típica cresce do *piano* ao *forte*.
  - **Spectrum Profile (`profil_1`..`profil_8`)**: 8 sliders que ajustam individualmente a intensidade dos **8 primeiros sobretons** (sendo `profil_1` a fundamental) para cada nota (exemplos no manual até `+6 dB`, `+15 dB`). Diferencia-se do Equalizador por atuar em razão harmônica relativa por nota, não por frequência absoluta em Hz.
  - **Hammer Noise (`hammer_noise`)**: Peso do transiente percussivo de impacto do feltro, independente da dureza do martelo (permite martelo macio com impacto audível ou martelo duro com knock atenuado).
  - **Hammer Tone (`hammer_tone`)**: Equilibra o impacto grave amadeirado vs o contato agudo com a corda no ataque.
  - **Soft Pedal (`soft_level`)**: Grau de suavização do pedal *una corda* (modela golpe na região mais macia do feltro).
- **Instrumentos Eletroacústicos (Sec. 7.2)**:
  - Compartilham `hammer_hardness`, `profil_1..8` e `hammer_noise`.
  - **`pickup_symmetry`**: Move o captador da posição assimétrica de repouso para simétrica em relação ao ressoador (tine/reed/bar); na simetria exata, o tom salta **uma oitava acima**.
  - **`pickup_distance`**: Distância entre fonte sonora e captador; aproximar o captador eleva distorção e contraste tímbrico entre *soft* e *loud*.
- **Clavinet D6 (Sec. 7.3)**:
  - Suporta os 4 ajustes padrão de dois captadores (inferior/superior): `A-C` (lower), `B-C` (upper), `A-D` (ambos em fase), `B-D` (ambos fora de fase), com mistura contínua via `clavinet_low_mic` e `clavinet_high_mic` e chaves originais de timbre (`Brilliant/Bright`, `Treble`, `Medium`, `Soft`).
- **Syngular (Sec. 7.4)**:
  - Substitui na interface principal os controles de `Hammer noise`, `Hammer tone` e `Soft pedal` pelos controles `Rate` e `Depth` de Vibrato e Tremolo conectados diretamente ao motor `Note Effects`.

### 3.2 Design (`paramgroup_design` — 12 parâmetros, Manual Sec. 8.1–8.3, 14.3, 14.6)
- **Tampo Harmônico / Soundboard (Sec. 8.1.1, 14.3)**:
  - **`impedance`**: Impedância mecânica ($Z = F / v$). Maior impedância desacelera a transferência de energia corda $\rightarrow$ tampo, gerando notas mais longas (em pianos reais, à custa de menor volume acústico; no Pianoteq o ganho é compensado livremente).
  - **`impedance_cutoff`**: Frequência de corte abaixo da qual a impedância permanece alta e acima da qual começa a cair.
  - **`q_factor` (label `Impedance Slope`)**: Inclinação da queda da impedância acima da frequência de corte; controla a taxa de decaimento dos sobretons agudos.
- **Comprimento de Corda (`string_length`, Sec. 8.1.2)**:
  - Define a escala de comprimentos de corda (até **10 m** virtuais, com inarmonicidade próxima de zero; exemplo de referência: piano de **2,7 m** possui cordas de ~**2 m** nos graves a ~**5 cm** nos agudos). Reduzir o comprimento eleva a inarmonicidade e aproxima o timbre de sinos.
- **Ponto de Percussão (`strike_point`, Sec. 8.1.3)**:
  - Posição onde o martelo/baqueta atinge a corda ou barra; possui opção **Humanize** (variação aleatória bilateral por nota, destacada para percussão cromática).
- **Ressonâncias (`sympathetic_resonance`, `duplex_scale_resonance`, Sec. 8.1.4–8.1.5)**:
  - Ressonância simpática depende individualmente da posição de cada abafador (e do pedal sustain, suportando teclas pressionadas silenciosamente com excitação em *staccato* ou transferência pós-liberação).
  - *Duplex scale* modela porções não abafadas entre cravelhas/estrutura (*front scale*) e cavalete/estrutura (*rear scale*, patente Steinway/Helmholtz 1872).
- **Blooming (`blooming_energy`, `blooming_inertia`, Sec. 8.1.6, 8.2)**:
  - Originado em chapas finas de aço dos *steel drums*: transferência não-linear de energia durante o ataque dos parciais graves para os mais agudos. `blooming_energy` controla a quantidade transferida; `blooming_inertia` controla o tempo/inércia da transferência. Também aplicável a eletroacústicos (e.g., preset `MKI Blooming Grit`).
- **Eletroacústicos & Áudio Externo (Sec. 8.2 & `dsp.ttl`)**:
  - Ruído de martelo e abafador no *tine* (`hammer_tine_noise` em `dsp.ttl`; UI de Vintage Tines MKI/MKII separa *Tine noise hammer* e *Tine noise damper*).
  - `quadratic_effect` no grupo Design, além de `external_audio_mix` e `external_audio_tone` conectados à porta `audio_in_1`.
- **Syngular Design (Sec. 8.3, 14.6)**:
  - Mapeia parâmetros mecânicos para envelope estilo sintetizador: **Attack** (doma ataques rápidos à esquerda, alonga para pads/sopros à direita), **Decay** (pulso breve até minutos), **Decay Cutoff** e **Decay Slope** (equivalentes à impedância), **Release** (conectado diretamente a `Action -> Damping Duration` + `Damper Position`), e **Harmonicity** (inverso de inarmonicidade). Não há parâmetro `Sustain` fixo pois cada parcial decai fisicamente conforme o modelo acoplado de 3 cordas por nota.

### 3.3 Mallet Bounce (`paramgroup_bounce` — 8 parâmetros + `attack_envelope`, Manual Sec. 13.2 & 9.2)
- Projetado especialmente para percussão cromática e cimbalom; atua gerando repetições automáticas em ricochete da baqueta:
  - `bounce_switch`: Habilita/desabilita (`Off`/`On`).
  - `bounce_delay`: Atraso inicial até o primeiro rebote em modo livre.
  - `bounce_sync` + `bounce_sync_speed`: Sincroniza ao andamento host com **30 valores discretos** de `16d` até `1/32t` (observados via `lv2:scalePoint` em `dsp.ttl`).
  - `bounce_velocity_sensitivity`: Quanto maior, maior o intervalo dos primeiros rebotes em velocidades altas (simulando maior altura de queda da baqueta).
  - `bounce_delay_loss`: Redução progressiva do intervalo entre rebotes sucessivos.
  - `bounce_velocity_loss`: Atenuação progressiva de velocidade/amplitude a cada rebote.
  - `bounce_humanization`: Variação estocástica nos parâmetros de atraso e velocidade dos rebotes.
  - `attack_envelope` (compartilhado em `paramgroup_notefx`): Na seção Mallet Bounce ajusta apenas o envelope de ataque das **notas rebatidas**, mantendo o ataque inicial sob controle de Note Effects.
- **Abafamento em Mallet Instruments (Sec. 9.2)**: Opção em `Velocity panel -> Sustained notes damping` para abafar notas sustentadas pelo pedal via *Low-velocity note-on* ou *Aftertouch*.

### 3.4 Note Effects (`paramgroup_notefx` — 12 parâmetros, Manual Sec. 13.4 & 7.4)
- Atua **por nota individual** diretamente sobre o modelo físico (não é efeito pós-mix):
  - **LFO por nota**: `nfx_lfo_shape`, `nfx_lfo_skew`, `nfx_lfo_rate`, `nfx_lfo_phase`, `nfx_lfo_phase_locked` (`Off`/`On`) e `nfx_onset_duration` (rampa de entrada antes da modulação aparecer).
  - **Vibrato**: `nfx_vibrato` (profundidade em **cents**) e `nfx_vibrato_offset` (assimetria positiva vs negativa da modulação de frequência).
  - **Tremolo**: `nfx_tremolo_depth` (profundidade em **dB**) e `nfx_tremolo_phase` (fase relativa ao vibrato).
  - **Envelope & Suavização**: `attack_envelope` (modificação do transiente de ataque) e `smear` (suavização/desfocagem de fase entre ondas/parciais, muito audível em Syngular nos graves e uníssono estreito).
  - **Modo Monofônico (Manual Sec. 13.4)**: Modo `Mono` com opções `Legato` (suprime o ataque de notas ligadas) e `Portamento` (glissando entre notas em taxa definida) — presentes na UI/FXP, **não** expostos como parâmetros LV2 separados em `dsp.ttl`.

### 3.5 Modos Guitar e Harp (`paramgroup_guitar` + `paramgroup_pedals` + Manual Sec. 4.2, 4.3, 16.3, 16.4)
- **Modo Diatônico para Harpa (Sec. 4.2 & 16.4)**:
  - Modelos incluídos: *Concert Harp* (Salvi, 7 pedais Érard), *Celtic Harp* (Aoyama, cordas de nylon com alavancas) e *Triple Harp* (barroca italiana com 3 fileiras de cordas: duas em uníssono e fileira central para acidentes).
  - No modo Diatônico (`D` no teclado), apenas **teclas brancas** produzem som; os **7 pedais** (`D, C, B, E, F, G, A` da esquerda para direita) são controlados por grupos de **3 teclas pretas** (esquerda = bemol `♭`, centro = natural, direita = sustenido `#`).
  - Suporta flageolets (`pinch_harmonic_pedal`) e glissandos (`glissando_pedal`).
- **Modo Guitarra (`Classical Guitar` & `paramgroup_guitar`, Sec. 4.3 & 16.3)**:
  - Baseado em violões clássicos de nylon (incluindo concert guitar C9 de Jean-Marie Fouilleul). Disponível em qualquer instrumento clicando no seletor de modo à esquerda do teclado.
  - **Parâmetros LV2 dedicados (5 + pedais)**: `virtuosity` (slider *Casual to Pro* — em *Pro*, zera ruídos adicionais de traste/deslizamento/corpo), `fret` (posição no braço), `guitar_legato` (*hammer-on* / *pull-off* até terça menor enquanto a primeira nota permanece pressionada), `guitar_easy_fingering` (`Off`/`On` — modo automático vs `Adv.` manual), `guitar_body` (*hollow* vs *solid* body); somam-se `pinch_harmonic_pedal`, `pinch_harmonic_point`, `glissando_pedal` e `lute_stop_pedal` (*Buff Stop* atuando como *palm mute*).
  - **Mecanismo de 6 cordas & técnicas**:
    - Seletor automático de cordas resolve conflitos de notas simultâneas (segundas/terças menores) na posição atual da mão esquerda; *key switches* amarelos à esquerda permitem forçar corda específica, corda solta (*Open strings*), alternância *gate* vs *toggle*, toque com polegar (*thumb*), *rasgueado* e repetições/batidas *strum up/down* de acordes anteriores.
    - **Glissando**: Disparado quando a nota inicial é interrompida pela nota final; a velocidade da nota final define a velocidade do slide; glissando de acorde calcula o deslocamento relativo à nota do baixo.
    - **Roteamento MIDI**: Permite canal MIDI dedicado por corda (ideal para guitarras MIDI) e canal MIDI separado para *key switches*, além de afinações alternativas.

### 3.6 PRO Note Edit (`Manual Sec. 1.3, 6.2, 9.1, 15.1–15.3`)
- **Diferenciação de Edições (Sec. 1.3)**:
  - *Stage*: até **48 kHz**, sem edição física ou Note Edit.
  - *Standard*: até **48 kHz**, Note Edit restrito a **Volume**, **Detune** e **Attack Envelope**.
  - *PRO*: até **192 kHz** de taxa interna + Note Edit completo de parâmetros físicos (Tuning/Detune, Unison Width, Direct Sound Duration, Velocity Offset, Hammer Hardness, Strike Point, String Length, Spectrum Profile [milhares de sobretons], Soundboard Impedance, Damper Position).
- **Ferramentas Gerais de Curva (Sec. 15.1)**:
  - Trilho de **Control Notes** abaixo do gráfico (duplo clique adiciona/remove âncoras); arrasto com `Shift` para ajuste fino; operadores globais de forma: `Random`, `Smooth`, `Rescale` e `Reset`.
- **Octave Stretching Note Edit (Sec. 15.2)**:
  - Painel *Stretch Points*: escolhe o **Tuned interval** (*Octave*, *Fifth*, *12th*, *15th*, *19th*, *22th*, *Major Third*) e a razão exata de casamento entre parciais para cada nota base (Oitavas: `2:1`, `4:2`, `8:4`, `10:5`, `12:6`; Quintas: `3:2`, `6:4`, `9:6`, `12:8`, `15:10`).
  - Inclui slider **Roughness** para suavizar transições entre *Stretch Points* adjacentes e interage diretamente com `String Length` (inarmonicidade). Também suporta exportar o esticamento resultante para *Detune Note Edit*.
- **Spectrum Profile Note Edit (Sec. 15.3)**:
  - Possui conjunto próprio e independente de *Control Notes*, com **interpolação linear** para notas intermediárias.
  - Modos de desenho: `Simple draw`, `Haircut`, `Fill only`.
  - Combinações harmônicas de seleção: `Individual`, `Harmonic`, `Comb`, `Major`, `Octave`.

### 3.7 Equalizer (`equalizer_switch` + `effect_2/3` EQ, Manual Sec. 13.3, 13.5, 14.5)
- **Equalizador Principal (`Main` / `equalizer_switch`)**:
  - Opera em **pré-processamento** (alterando as amplitudes dos parciais dentro do modelo físico antes da síntese de áudio), diferente de um EQ pós-áudio.
  - Vantagem confirmada: **compensação automática de volume**, preservando o equilíbrio natural de nível entre graves, médios e agudos ao longo do teclado, além de suportar pontos de controle arbitrários (clique adiciona, duplo clique remove).
  - Controles adicionais na mesma área: **`Res Eq`** (equaliza especificamente as ressonâncias simpáticas) e **`Res Dur`** (modifica a duração das ressonâncias simpáticas).
- **Slot de Efeito `EQ` (`Effects` panel, Sec. 13.5)**:
  - Equalizador paramétrico padrão de **5 bandas** em **pós-processamento** (`Freq`, `Gain`, `Q` via `Shift+drag`, `Bypass` por banda e seleção de tipo de filtro como *low-pass*, *band-pass*, etc.). Em `dsp.ttl`, os slots default `fx2` e `fx3` expõem parâmetros rotulados `PEQ1F/G/Q/T` e `PEQ2F/G/Q/T`.
- **Cadeia Serial de 3 Slots FX (Sec. 13.5)**:
  - 3 slots permutáveis entre: `Tremolo`, `Wah` (com Auto-Wah sensível à amplitude), `Chorus`, `Flanger`, `Phaser`, `Fuzz`, `Delay`, `Amp` (`Mix`, `Drive`, `Bass`, `Midrange`, `Treble`), `Comp` (`Ratio`, `Attack`, `Release`, `Threshold`, `Gain`) e `EQ`, seguidos de `post_effect_gain`.

### 3.8 Reverb (`paramgroup_reverb` — 10 parâmetros, Manual Sec. 13.6)
- Motor de **reverberação por convolução** com modelos de salas (Studio até Cathedral / Taj Mahal), `Plate`, `Spring`, `Speakers`, `Broceliande` e carregamento de arquivos **WAV impulse externos** (em Standard e PRO).
- Controles documentados e mapeados em `dsp.ttl`:
  - `reverb_switch`: Liga/desliga (`Off`/`On`).
  - `reverb_mix`: Nível relativo em **dB** entre sinal reverberante e sinal direto.
  - `reverb_duration`: Duração da cauda reverberante.
  - `room_dimensions`: Dimensão da sala (**5 m a 50 m**).
  - `reverb_pre_delay`: Atraso entre som direto e início da reverberação.
  - `reverb_early_reflections`: Nível relativo em **dB** entre reflexões iniciais (*early reflections*) e cauda (*tail*).
  - `reverb_tone`: Equalizador tipo *tilt* aplicado apenas ao sinal refletido (mais escuro à esquerda, mais brilhante à direita).
  - `reverb_highpass_cutoff` (`Low cut`): Filtro passa-alta aplicado somente ao sinal *wet*.
  - `reverb_envelope` e `reverb_resampling`: Parâmetros adicionais presentes no contrato público LV2 (`dsp.ttl`).

### 3.9 Ambiente & Arquitetura Geral (Confirmados)
- Processamento interno em ponto flutuante 32-bit, suporte multi-canal MIDI (16×128 = **2.048 notas** para microtonalidade como 31-EDO / 53-EDO, Sec. 6.3), opção de *retuning* por **String tension** (altera timbre/inarmonicidade) vs **Full rebuild** (mantém timbre por altura, Sec. 6.2), Morphing físico vs Layering misto (Sec. 5), e faixa dinâmica acústica medida de ~50 dB (Sec. 14.4.1, mantendo nível de *forte* constante ao alterar `Dynamics`).
- Máquina local Linux possui PipeWire 1.6.8 (interfaces PulseAudio, ALSA, JACK) e sessão gráfica Wayland/X11; `README_LINUX.txt` recomenda buffers múltiplos de 64 samples para ALSA/JACK.

---

## 4. Questões em Aberto (Para Próximas Etapas / Outros Subagentes)

1. **Curvas Exatas de Desnormalização `[0.0, 1.0]` $\rightarrow$ Unidades Físicas**:
   - `dsp.ttl` não inclui `units:unit` nem faixas físicas reais (Hz, ms, dB, metros, cents) para os parâmetros contínuos (exceto `room_dimensions` 5–50 m no manual e `bounce_sync_speed` via `scalePoint`). Seria necessário inspecionar respostas JSON-RPC (`--serve`) ou presets LV2 exportados/UI sem violar proteções caso o motor Rust precise replicar exatamente os mostradores em dB/Hz/ms.
2. **Mapeamento Dinâmico dos Slots `effect_1..3_param_1..8`**:
   - Como os 3 slots de efeito compartilham 8 portas genéricas cada, os rótulos em `dsp.ttl` refletem apenas o preset default (`Delay` no slot 1, `PEQ` nos slots 2 e 3). Falta tabular o mapeamento exato de `param_1..8` para os demais 7 algoritmos (`Tremolo`, `Wah`, `Chorus`, `Flanger`, `Phaser`, `Fuzz`, `Amp`, `Comp`).
3. **Curvas Per-Note e Parâmetros Não-LV2 (FXP / Estado Interno)**:
   - Controles como curvas per-note do PRO Note Edit, pontos arbitrários do Equalizador Principal + `Res Eq`/`Res Dur`, modo `Mono`/`Legato`/`Portamento` em Note Effects, seletores de modelo de Microfone (U87, DPA, C414, etc.) e seletores de Impulse Response de Reverb residem em estado de preset/chunk LV2 (`state:interface`), não em parâmetros `lv2:Parameter` individuais.
4. **Desambiguação Exata dos 3 Pares Duplicados em `--list-instruments`**:
   - Confirmar via inspeção de presets/JSON-RPC oficial se os índices #19/#20 (`Vintage Reeds`) correspondem exatamente à ordem W2 / W1 e se #35/#36 (`Spacedrum`) e #37/#38 (`Hand Pan`) correspondem respectivamente às variantes *hand played* vs *mallets* ou a layouts distintos de notas.
