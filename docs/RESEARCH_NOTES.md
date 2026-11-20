# Research Notes — Pianoteq 9.3.1 Technical Reference Inventory

## 1. Evidências Observadas (REA & Artefatos Oficiais)

### 1.1 Artefatos Binários Inspecionados via REA
- **`Pianoteq 9` (Standalone ELF64 amd64)**:
  - Path: `/home/data/projects/pianoteq-eng-reverse2/Pianoteq-v931/x86-64bit/Pianoteq 9`
  - SHA-256: `1d0b7f62548ca311b0526b9b2f18f7f11fb7e81f63a82e3c91a9800b89fe385f`
  - Tamanho: `58,939,184` bytes
  - Evidence IDs REA: `ev_1dd73cd50e6511cbe09bccf7cfcbbef5e6dd6f1f57b8cca204080aa36c2cec68`, `ev_6960eaa80c9741c870ae6df79b36a5abef3d7df483902d4d1960daebd9dc291b`, `ev_eeef488bf478d9168109abd7f4b9b514ba91054bb28d29c5c188a03ea5f26c65`
  - Mitigações/Linkage (`inspect_binary_hardening` + `readelf`): PIE + NX + Full RELRO + Stack Protector + Fortify + Stripped; depende dinamicamente apenas de `libGL.so.1`, `libasound.so.2`, `libEGL.so.1`, `libfreetype.so.6`, `libcurl.so.4`, `libdl.so.2`, `libfontconfig.so.1`, `libm.so.6`, `libgcc_s.so.1`, `libpthread.so.0`, `libc.so.6`.
- **`Pianoteq_9.so` (LV2 DSP Shared Library ELF64 amd64)**:
  - Path: `/home/data/projects/pianoteq-eng-reverse2/Pianoteq-v931/x86-64bit/Pianoteq 9.lv2/Pianoteq_9.so`
  - SHA-256: `8312e07d64000b38c4b92f695f2ce2b3ebbbc82038289cf0204431e0144b4e8a`
  - Tamanho: `55,374,656` bytes
  - Evidence IDs REA: `ev_a116654a8945b4d1b33cf66d56666cc3e5fb62588bf58fa73bd243b0afb8ebb1`, `ev_61bcc0c2031feca2f79a8aaf33adfe3025a41c560c69609c8db9934fcb697424` (4 símbolos LV2 exportados).

### 1.2 Contrato Público de Parâmetros LV2 (`dsp.ttl`)
Inspeção direta de `Pianoteq 9.lv2/dsp.ttl` confirmou **215 parâmetros normalizados [0..1]** organizados em **15 grupos**:
1. `paramgroup_main` (9): `condition`, `dynamics`, `volume`, `velocity_offset`, `post_effect_gain`, `morphing`, `equalizer_switch`, `aftertouch`, `diapason`.
2. `paramgroup_tuning` (5): `pitch_bend`, `unison`, `octave_stretching`, `unison_balance`, `direct_sound`.
3. `paramgroup_output` (78): `stereo_width`, `bus_1_volume`, `bus_1_pan`, `lid_position`, `mic_level_compensation`, `mic_delay_compensation`, binaural head (`head_x/y/z/diameter/angle`), 8 canais de microfone (`mic_1..8` com switch, x, y, z, azimuth, elevation, bus volume, pan), `sound_speed`, `wall_distance`, `lid`.
4. `paramgroup_voicing` (21): `hammer_noise`, `hammer_hardness` (global + piano/mezzo/forte), `profil_1..8` (Spectrum Profile 1..8), `hammer_tone`, `strike_point`, `pinch_harmonic_point`, `pickup_symmetry`, `pickup_distance`, `soft_level`, `clavinet_low_mic`, `clavinet_high_mic`.
5. `paramgroup_pedals` (16): `sustain_pedal`, `soft_pedal`, `sostenuto_pedal`, `harmonic_pedal`, `rattle_pedal`, `lute_stop_pedal` (Buff Stop), `celeste_pedal`, `mozart_rail`, `super_sostenuto`, `pinch_harmonic_pedal`, `glissando_pedal`, `thunder_pedal`, `harpsichord_register_1..3`, `reversed_sustain`.
6. `paramgroup_design` (12): `impedance`, `impedance_cutoff`, `q_factor` (Impedance Slope), `string_length`, `sympathetic_resonance`, `duplex_scale_resonance`, `quadratic_effect`, `hammer_tine_noise`, `blooming_energy`, `blooming_inertia`, `external_audio_mix`, `external_audio_tone`.
7. `paramgroup_bounce` (8): `bounce_switch`, `delay`, `sync`, `sync_speed`, `velocity_sensitivity`, `delay_loss`, `velocity_loss`, `humanization`.
8. `paramgroup_fx1/2/3` (27): Delay (Mix/Time/Fb/Tone/Pol/Ping), Parametric EQ/Chorus/Tremolo/Comp slots.
9. `paramgroup_reverb` (10): Switch, Duration, Mix, Room Dimensions, Pre-delay, Early Reflections, Tone, Envelope, Resampling, Highpass Cutoff.
10. `paramgroup_limiter` (4): Switch, Sharpness, Threshold, Gain.
11. `paramgroup_action` (8): Mute, Damper Noise, Damper Position, Last Damper, Pedal Noise, Key Release Noise, Damping Duration, Keyboard Range Switch.
12. `paramgroup_notefx` (12): LFO Shape/Skew/Rate/Phase/Locked, Onset Duration, Vibrato, Vibrato Offset, Tremolo Depth/Phase, Attack Envelope, Smear.
13. `paramgroup_guitar` (5): Virtuosity, Fret, Guitar Legato, Guitar easy fingering, Guitar Body.

### 1.3 Catálogo de Instrumentos & CLI (`--list-instruments`, `--help`)
- **56 entradas de instrumentos** reportadas via `--list-instruments` (Grands Steinway NY/HB/B, Shigeru Kawai SK-EX, YFX, Bösendorfer 280VC, Bechstein DG, Ant. Petrof, Petrof Mistral, Steingraeber, Grotrian, Blüthner, K2, YC5, Upright U4, Vintage Tines MKI/MKII, Vintage Reeds, Clavinet D6, Pianet N/T, Electra-Piano, Syngular, Vibraphones V-B/V-M, Celesta, Glockenspiel, Toy Piano, Kalimba, Marimba, Xylophone, Steelpans, Classical Guitar, Concert/Celtic/Triple Harp, Hans Ruckers II Harpsichord e coleção KIViR histórica).
- **Fórmulas matemáticas de temperamento confirmadas no manual (Seção 17)**:
  - `pythagore`, `zarlino`, `mesotonic`, `welltempered`, `werck` (normalizadas para deixar Lá4 inalterado).
- **Diretrizes Linux (`README_LINUX.txt`)**:
  - Buffers múltiplos de 64 samples recomendados para ALSA/JACK; ALSA latência ~ 3x buffer.
  - Preferências em XDG (`~/.config/` e `~/.local/share/`).

## 2. Hipóteses Técnicas de Síntese
- A inarmonicidade acústica observada na documentação (Seção 15.2) segue a equação clássica de corda rígida $f_n = n f_0 \sqrt{1 + B n^2}$, onde $B \propto L^{-2}$.
- O perfil espectral de 8 barras (`profil_1..8`) pondera diretamente a amplitude dos 8 primeiros parciais de cada nota, enquanto o `strike_point` $\alpha \in (0, 0.5)$ atenua parciais segundo $\lvert\sin(n\pi\alpha)\rvert$.
- A dureza do martelo (`hammer_hardness_piano`, `mezzo`, `forte`) atua como um filtro espectral não-linear dependente da velocidade MIDI (41 / 70 / 98).

## 3. Descobertas Confirmadas
- O binário standalone usa processamento interno em ponto flutuante 32-bit, arquitetura separada entre áudio em tempo real e UI, e suporta 16 canais MIDI para microtonalidade (>128 notas).
- A máquina local possui PipeWire 1.6.8 com interfaces PulseAudio, ALSA e JACK ativas e sessão Wayland/X11 disponível (`DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`).

## 4. Questões em Aberto (Para Subagente 01 - Análise & REA)
- Extrair a lista completa e limites/defaults dos 215 parâmetros de `dsp.ttl` para `docs/REFERENCE_PARAMETERS.json`.
- Mapear detalhadamente os parâmetros diferenciais de `Syngular`, `Mallet Bounce`, `Guitar`, `Harp` e `Note Edit` descritos nas seções 4, 7.4, 8.3, 13.2, 13.4 e 15 do manual HTML.
