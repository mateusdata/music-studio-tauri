import { afterEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import App from "../../src/App";
import { VirtualKeyboard } from "../../src/components/VirtualKeyboard";
import { engineClient } from "../../src/services/engineClient";
import type {
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MusicPackageEntry,
  MusicPackageSummary,
} from "../../src/types/engine";

function buildMockEngineParams(): EngineParams {
  return {
    volume_db: -4.0,
    dynamics_db: 40.0,
    velocity_offset: 0.0,
    condition: 0.04,
    post_effect_gain_db: 0.0,
    morphing: 0.0,
    tuning: {
      diapason_hz: 440.0,
      temperament: "Equal",
      unison_width_cents: 0.7,
      unison_balance: 0.5,
      octave_stretching: 1.0,
      direct_sound_duration: 0.5,
      pitch_bend_semitones: 0.0,
    },
    voicing: {
      hammer_hardness_piano: 0.38,
      hammer_hardness_mezzo: 0.58,
      hammer_hardness_forte: 0.82,
      spectrum_profile: [1.0, 0.92, 0.84, 0.72, 0.6, 0.48, 0.34, 0.28],
      hammer_noise: 0.35,
      hammer_tone: 0.5,
      strike_point: 0.125,
      soft_level: 0.65,
      pickup_symmetry: 0.5,
      pickup_distance: 0.5,
    },
    design: {
      soundboard_impedance: 0.52,
      impedance_cutoff: 0.5,
      impedance_slope: 0.5,
      string_length_m: 2.74,
      sympathetic_resonance: 0.45,
      duplex_scale_resonance: 0.35,
      blooming_energy: 0.28,
      blooming_inertia: 0.5,
      quadratic_effect: 0.22,
    },
    action: {
      damper_position: 0.5,
      damper_noise: 0.28,
      pedal_noise: 0.25,
      key_release_noise: 0.22,
      damping_duration: 0.5,
      mute: 0.0,
      keyboard_range_extended: false,
    },
    output: {
      lid_position: 1.0,
      stereo_width: 0.62,
      mic_level_compensation: true,
      mic_delay_compensation: false,
      binaural_mode: false,
      head_x: 0.5,
      head_y: 0.22,
      head_z: 0.65,
      head_diameter: 0.18,
      head_angle: 0.0,
      mics: [
        {
          enabled: true,
          name: "Left Close",
          x: 0.28,
          y: 0.42,
          z: 0.65,
          azimuth: 0.0,
          elevation: -15.0,
          volume: 0.85,
          pan: -0.58,
        },
      ],
    },
    eq_enabled: true,
    eq_bands: [
      { freq_hz: 110, gain_db: 0.5, q: 0.8 },
      { freq_hz: 1100, gain_db: 0.0, q: 1.0 },
      { freq_hz: 5200, gain_db: 1.0, q: 0.9 },
    ],
    fx: [
      { enabled: false, effect_type: "Delay", mix: 0.25, param_1: 0.35, param_2: 0.4, param_3: 0.6, param_4: 0.5 },
      { enabled: false, effect_type: "Chorus", mix: 0.2, param_1: 0.35, param_2: 0.4, param_3: 0.6, param_4: 0.5 },
      { enabled: false, effect_type: "Compressor", mix: 0.5, param_1: 0.35, param_2: 0.4, param_3: 0.6, param_4: 0.5 },
    ],
    reverb: {
      enabled: true,
      mix: 0.22,
      duration_s: 1.8,
      room_size_m: 18.0,
      pre_delay_ms: 14.0,
      early_reflections: 0.45,
      tone: 0.52,
      highpass_hz: 110.0,
      preset_name: "Concert Hall",
    },
    limiter: {
      enabled: true,
      threshold_db: -0.8,
      sharpness: 0.65,
      gain_db: 0.0,
    },
    note_edit: {},
    bounce: {
      enabled: false,
      delay_ms: 120.0,
      sync_tempo: false,
      velocity_sensitivity: 0.5,
      delay_loss: 0.15,
      velocity_loss: 0.35,
      humanization: 0.1,
      max_bounces: 6,
    },
    note_fx: {
      lfo_shape: 0.5,
      lfo_skew: 0.5,
      lfo_rate_hz: 5.2,
      lfo_phase_locked: false,
      onset_duration_ms: 80.0,
      vibrato_cents: 0.0,
      vibrato_offset_cents: 0.0,
      tremolo_depth_db: 0.0,
      tremolo_phase: 0.0,
      attack_envelope: 0.5,
      smear: 0.0,
    },
    play_mode: {
      mode: "Standard",
      harp_pedals: [0, 0, 0, 0, 0, 0, 0],
      guitar_fret: 0,
      guitar_legato: false,
      guitar_body: 0.5,
    },
  };
}

function buildMockArrangerState(overrides?: Partial<MusicPackageSummary>): MusicPackageSummary {
  return {
    loaded: true,
    file_name: "Josiran_Pisadinha_GR.Music",
    file_path: "/music/Josiran_Pisadinha_GR.Music",
    version: "1.4.4",
    source_format: "music",
    soundfont_name: "FluidR3_GM.sf2",
    instruments: [
      { index: 1, name: "Strings (Stereo)", zone_count: 7, has_samples: true, is_drumkit: false },
      { index: 2, name: "BTR JOSIRAN PISADINHA", zone_count: 33, has_samples: true, is_drumkit: true },
      { index: 7, name: "BANDONEON VA7 ST SOLO", zone_count: 12, has_samples: true, is_drumkit: false },
    ],
    styles: [
      {
        id: 1,
        name: "Josiran Pisadinha (GR)",
        category: "GABRIEL RODRIGUES GR",
        sections: ["Intro 1", "Variation 1", "Variation 2", "Fill 1", "Ending 1"],
      },
    ],
    active_style_index: 0,
    active_section: "Variation 1",
    pending_section: null,
    upper1_instrument: 7,
    upper2_instrument: null,
    lower_instrument: null,
    split_key: 54,
    split_enabled: true,
    chord_scan_lower: true,
    chord_scan_upper: false,
    upper_octave: 0,
    transpose: 0,
    bpm: 120,
    tempo_locked: false,
    rhythm_playing: false,
    sync_start: true,
    auto_fill: true,
    detected_chord_root: 0,
    detected_chord_quality: "Major",
    detected_chord: "C",
    chord_mode: "fingered_3_7",
    track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
    track_instruments: [null, null, null, null, null, null, null, null],
    ots_presets: [],
    beat_position: 0,
    sty_warnings: [],
    has_audio_style_chunks: false,
    uses_expansion_kit: false,
    sty_diagnostics_summary: null,
    ...overrides,
  };
}

function buildMockSnapshot(): EngineSnapshot {
  return {
    active_instrument_id: "default",
    active_preset_id: "default",
    preset_b_id: null,
    params: buildMockEngineParams(),
    midi_config: {
      selected_port: null,
      channel: 0,
      microtonal_multichannel: false,
      transpose: 0,
      key_range_mode: "Full88",
      velocity_curve: [
        [0, 0],
        [1, 1],
      ],
      cc_mappings: [],
    },
    instruments: [],
    presets: [],
  };
}

function buildMockTelemetry(): EngineTelemetry {
  return {
    sample_rate: 48000,
    buffer_size: 256,
    latency_ms: 5.3,
    active_voices: 4,
    max_polyphony: 128,
    cpu_load_pct: 3.1,
    peak_l: 0.3,
    peak_r: 0.3,
    rms_l: 0.15,
    rms_r: 0.15,
    audio_backend: "PipeWire",
    audio_active: true,
    midi_connected: false,
    midi_port_name: null,
    last_midi_note: 60,
    last_midi_velocity: 90,
    active_notes: [60],
    pedals: {
      sustain: 0,
      soft: 0,
      sostenuto: 0,
      harmonic: 0,
      celeste: 0,
      buff_stop: 0,
      rattle: 0,
      super_sostenuto: false,
      reversed_sustain: false,
    },
  };
}

function setupTauriStateMocks(pkgList: MusicPackageEntry[], initialArr: MusicPackageSummary) {
  let liveArr = initialArr;
  const telemetry = buildMockTelemetry();
  const snapshot = buildMockSnapshot();

  vi.spyOn(engineClient, "isTauri").mockReturnValue(true);
  vi.spyOn(engineClient, "listMusicPackages").mockResolvedValue(pkgList);
  vi.spyOn(engineClient, "getArrangerState").mockImplementation(async () => liveArr);
  vi.spyOn(engineClient, "getSnapshot").mockResolvedValue(snapshot);
  vi.spyOn(engineClient, "listMidiDevices").mockResolvedValue([]);
  vi.spyOn(engineClient, "getTelemetry").mockResolvedValue(telemetry);

  return {
    telemetry,
    getLive: () => liveArr,
    setLive: (next: MusicPackageSummary) => {
      liveArr = next;
      return liveArr;
    },
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Music Studio Tauri Arranger App UI", () => {
  it("edits velocity without touching other MIDI settings and saves after queued writes", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    const snapshot = buildMockSnapshot();
    snapshot.midi_config.selected_port = "Current MIDI";
    snapshot.midi_config.transpose = 3;
    vi.mocked(engineClient.getSnapshot).mockResolvedValue(snapshot);
    const update = vi.spyOn(engineClient, "updateMidiConfig").mockResolvedValue(undefined);
    const save = vi.spyOn(engineClient, "saveCurrentConfiguration").mockResolvedValue(undefined);
    render(<App />);
    fireEvent.click(screen.getByLabelText("Ferramentas"));
    const open = screen.getByRole("button", { name: "Curva de velocidade" });
    await waitFor(() => expect(open).not.toBeDisabled());
    fireEvent.click(open);
    const dialog = screen.getByRole("dialog", { name: "Curva de velocidade" });
    expect(update).not.toHaveBeenCalled();
    fireEvent.change(within(dialog).getByLabelText("Modo da curva de velocidade"), { target: { value: "fixed" } });
    fireEvent.change(within(dialog).getByRole("slider", { name: "Velocidade fixa" }), { target: { value: "70" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Fechar curva de velocidade" }));
    fireEvent.click(screen.getByRole("button", { name: "Salvar configuração atual" }));
    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    expect(update).toHaveBeenLastCalledWith({ ...snapshot.midi_config, velocity_curve: [[0, 1.4], [1, 1.4]] });
    expect(update.mock.invocationCallOrder[update.mock.invocationCallOrder.length - 1]).toBeLessThan(save.mock.invocationCallOrder[0]);
  });

  it("renders slim arranger header, searchable selectors, LCD screen, quick chords, and virtual keyboard in preview mode", () => {
    render(<App />);

    expect(screen.getByText("Music Studio Tauri")).toBeInTheDocument();
    expect(screen.getByLabelText("Music Package Selector")).toBeInTheDocument();
    expect(screen.getByLabelText("Style Selector")).toBeInTheDocument();
    expect(screen.getByLabelText("Upper 1 Instrument")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /START RITMO/i })).toBeInTheDocument();
    expect(screen.getByTestId("chord-display")).toHaveTextContent("C");

    // Ensure old Pianoteq clutter tabs are completely gone
    expect(screen.queryByRole("button", { name: "Voicing & Hammers" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Note Edit (PRO)" })).not.toBeInTheDocument();

    // Quick Chord switching updates preview LCD display
    fireEvent.click(screen.getByRole("button", { name: "Am" }));
    expect(screen.getByTestId("chord-display")).toHaveTextContent("Am");
  });

  it("renders 61 virtual keyboard keys with split chord highlighting and red split divider", () => {
    render(<App />);

    const keyboard = screen.getByTestId("virtual-keyboard");
    const keyButtons = within(keyboard).getAllByRole("button");
    expect(keyButtons).toHaveLength(61);

    expect(within(keyboard).getByRole("button", { name: "Key F3" })).toHaveClass("split-chord-zone");
    expect(within(keyboard).getByRole("button", { name: "Key F#3" })).toHaveClass("split-chord-zone");
    const keyG3 = within(keyboard).getByRole("button", { name: "Key G3" });
    expect(keyG3).not.toHaveClass("split-chord-zone");
    expect(keyG3).toHaveClass("split-point-edge");
    expect(within(keyboard).getByTestId("split-divider-line")).toBeInTheDocument();
    expect(screen.getByLabelText("Audio Buffer Size")).toHaveValue("256");
    for (const oct of [3, 4, 5, 6, 7, 8]) {
      expect(keyboard.querySelector(`.key-oct-${oct}`)).toBeInTheDocument();
    }
  });

  it("supports searchable filtering on Music Package Selector dropdown", () => {
    render(<App />);

    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    const searchInput = screen.getByLabelText("Music Package Selector Search");
    fireEvent.change(searchInput, { target: { value: "Arrasta" } });

    const popover = screen.getByTestId("music-package-selector-popover");
    expect(within(popover).getByRole("option", { name: /Arrasta_Pe_Forro_GR\.Music/i })).toBeInTheDocument();
    expect(within(popover).queryByRole("option", { name: /Josiran_Pisadinha_GR\.Music/i })).not.toBeInTheDocument();

    fireEvent.click(within(popover).getByRole("option", { name: /Arrasta_Pe_Forro_GR\.Music/i }));
    expect(screen.queryByTestId("music-package-selector-popover")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Music Package Selector Trigger")).toHaveTextContent("Arrasta_Pe_Forro_GR.Music");
  });

  it("wires Tauri arranger commands for package loading, chord switching, sections, transport, and sustain pedal", async () => {
    const pkgList: MusicPackageEntry[] = [
      {
        file_name: "Josiran_Pisadinha_GR.Music",
        file_path: "/music/Josiran_Pisadinha_GR.Music",
        size_mb: 23.4,
        version: "1.4.4",
      },
      {
        file_name: "Arrasta_Pe_Forro_GR.Music",
        file_path: "/music/Arrasta_Pe_Forro_GR.Music",
        size_mb: 38.8,
        version: "1.4.4",
      },
    ];
    const initialArr = buildMockArrangerState();
    const { telemetry, getLive, setLive } = setupTauriStateMocks(pkgList, initialArr);

    const chordSpy = vi
      .spyOn(engineClient, "setArrangerChord")
      .mockImplementation(async (rootPc, quality) =>
        setLive({
          ...getLive(),
          detected_chord: quality === "Minor" ? "Am" : "C",
          detected_chord_root: rootPc,
          detected_chord_quality: quality,
        }),
      );
    const transportSpy = vi
      .spyOn(engineClient, "setArrangerTransport")
      .mockImplementation(async (playing, syncStart, bpm, autoFill, tempoLocked = null) =>
        setLive({
          ...getLive(),
          rhythm_playing: playing,
          ...(syncStart !== null ? { sync_start: syncStart } : {}),
          ...(bpm !== null ? { bpm } : {}),
          ...(autoFill !== null ? { auto_fill: autoFill } : {}),
          ...(tempoLocked !== null ? { tempo_locked: tempoLocked } : {}),
        }),
      );
    const sectionSpy = vi
      .spyOn(engineClient, "setArrangerSection")
      .mockImplementation(async (sectionName) =>
        setLive({ ...getLive(), active_section: sectionName }),
      );
    const loadSpy = vi
      .spyOn(engineClient, "loadMusicPackage")
      .mockImplementation(async (path) =>
        setLive({
          ...getLive(),
          file_name: path.split("/").pop() ?? path,
          file_path: path,
        }),
      );
    const pickSpy = vi
      .spyOn(engineClient, "pickMusicPackageDialog")
      .mockImplementation(async () =>
        setLive({
          ...getLive(),
          file_name: "Picked.Music",
          file_path: "/home/data/Documents/Ritmos_Forro_Music_Studio/Picked.Music",
        }),
      );
    const pickInstSpy = vi
      .spyOn(engineClient, "pickInstrumentDialog")
      .mockImplementation(async () =>
        setLive({
          ...getLive(),
          upper1_instrument: 8,
          instruments: [
            ...getLive().instruments,
            {
              index: 8,
              name: "ACORDION (ACORDION_MT)",
              zone_count: 6,
              has_samples: true,
              is_drumkit: false,
            },
          ],
        }),
      );
    const voiceSpy = vi
      .spyOn(engineClient, "setArrangerKeyboardVoice")
      .mockImplementation(
        async (
          slot,
          instIdx,
          splitKey,
          splitEnabled,
          chordScanLower,
          chordScanUpper,
          upperOctave,
          transpose,
          chordMode,
        ) =>
          setLive({
            ...getLive(),
            upper1_instrument:
              slot === "upper1" && instIdx !== null ? instIdx : getLive().upper1_instrument,
            split_key: splitKey ?? getLive().split_key,
            split_enabled: splitEnabled ?? getLive().split_enabled,
            chord_scan_lower: chordScanLower ?? getLive().chord_scan_lower,
            chord_scan_upper: chordScanUpper ?? getLive().chord_scan_upper,
            upper_octave: upperOctave ?? getLive().upper_octave,
            transpose: transpose ?? getLive().transpose,
            chord_mode: chordMode ?? getLive().chord_mode,
          }),
      );
    const bufSpy = vi
      .spyOn(engineClient, "setAudioBufferSize")
      .mockResolvedValue({ ...telemetry, buffer_size: 128, latency_ms: 2.7 });
    const updateParamsSpy = vi.spyOn(engineClient, "updateParams").mockResolvedValue();
    const setPedalSpy = vi.spyOn(engineClient, "setPedal").mockResolvedValue();

    render(<App />);

    expect(await screen.findByText("Visor do Arranjador")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Caminho manual .Music" })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Am" }));
    await waitFor(() => {
      expect(chordSpy).toHaveBeenCalledWith(9, "Minor");
    });

    fireEvent.click(screen.getByRole("button", { name: /START RITMO/i }));
    await waitFor(() => {
      expect(transportSpy).toHaveBeenCalledWith(true, null, null, null);
    });

    fireEvent.click(screen.getByRole("button", { name: "Tempo Lock" }));
    await waitFor(() => {
      expect(transportSpy).toHaveBeenCalledWith(true, null, null, null, true);
    });

    fireEvent.change(screen.getByLabelText("Audio Buffer Size"), { target: { value: "128" } });
    await waitFor(() => {
      expect(bufSpy).toHaveBeenCalledWith(128);
    });

    fireEvent.click(screen.getByRole("button", { name: "Aumentar Volume" }));
    await waitFor(() => {
      expect(updateParamsSpy).toHaveBeenCalled();
    });

    fireEvent.click(screen.getByRole("button", { name: "Variation 2" }));
    await waitFor(() => {
      expect(sectionSpy).toHaveBeenCalledWith("Variation 2", false);
    });

    fireEvent.click(screen.getByRole("button", { name: "SCAN UP" }));
    await waitFor(() => {
      expect(voiceSpy).toHaveBeenCalledWith("split", null, null, null, null, true, null, null);
    });

    const chordModeSelect = screen.getByLabelText("Tipo de Acorde");
    expect(chordModeSelect).toHaveValue("fingered_3_7");
    fireEvent.change(chordModeSelect, { target: { value: "single_finger" } });
    await waitFor(() => {
      expect(voiceSpy).toHaveBeenCalledWith(
        "split",
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        "single_finger",
      );
    });

    fireEvent.click(screen.getByRole("button", { name: /Importar \.Music/i }));
    await waitFor(() => {
      expect(pickSpy).toHaveBeenCalledTimes(1);
    });

    fireEvent.click(screen.getByRole("button", { name: "Importar Timbre .Instrument" }));
    await waitFor(() => {
      expect(pickInstSpy).toHaveBeenCalledWith(true);
    });

    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    fireEvent.change(screen.getByLabelText("Music Package Selector Search"), {
      target: { value: "Arrasta" },
    });
    const pkgPopover = screen.getByTestId("music-package-selector-popover");
    fireEvent.click(within(pkgPopover).getByRole("option", { name: /Arrasta_Pe_Forro_GR\.Music/i }));
    await waitFor(() => {
      expect(loadSpy).toHaveBeenCalledWith("/music/Arrasta_Pe_Forro_GR.Music");
    });

    const sustainBtn = screen.getByRole("button", { name: "Sustain (Damper)" });
    fireEvent.click(sustainBtn);
    await waitFor(() => {
      expect(setPedalSpy).toHaveBeenCalledWith("sustain", 1.0);
    });
  });

  it("wires .STY file import, .STY folder import, and SoundFont .SF2 import in Tauri mode", async () => {
    const pkgList: MusicPackageEntry[] = [
      {
        file_name: "Josiran_Pisadinha_GR.Music",
        file_path: "/music/Josiran_Pisadinha_GR.Music",
        size_mb: 23.4,
        version: "1.4.4",
        format: "music",
      },
    ];
    const initialArr = buildMockArrangerState({ soundfont_name: null });
    const { getLive, setLive } = setupTauriStateMocks(pkgList, initialArr);

    const pickStySpy = vi.spyOn(engineClient, "pickStyDialog").mockImplementation(async () =>
      setLive({
        ...getLive(),
        file_name: "Forro_Classico.sty",
        file_path: "/styles/Forro_Classico.sty",
        source_format: "sty",
        styles: [
          {
            id: 10,
            name: "Forró Clássico",
            category: "Yamaha .STY",
            sections: ["Intro 1", "Variation 1", "Variation 2", "Fill 1", "Ending 1"],
          },
        ],
      }),
    );

    const pickStyFolderSpy = vi
      .spyOn(engineClient, "pickStyFolderDialog")
      .mockImplementation(async () =>
        setLive({
          ...getLive(),
          file_name: "Pasta_Ritmos (2)",
          file_path: "/styles",
          source_format: "sty_pack",
          styles: [
            {
              id: 10,
              name: "Forró Clássico",
              category: "Yamaha .STY",
              sections: ["Intro 1", "Variation 1", "Ending 1"],
            },
            {
              id: 11,
              name: "Xote Alegre",
              category: "Yamaha .STY",
              sections: ["Intro 1", "Variation 1", "Ending 1"],
            },
          ],
        }),
      );

    const pickSf2Spy = vi.spyOn(engineClient, "pickSoundfontDialog").mockImplementation(async () =>
      setLive({
        ...getLive(),
        soundfont_name: "GeneralUser_GS.sf2",
        instruments: [
          ...getLive().instruments,
          { index: 12, name: "Accordion SF2", zone_count: 16, has_samples: true, is_drumkit: false },
        ],
      }),
    );

    render(<App />);
    expect(await screen.findByText("Visor do Arranjador")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Importar .STY" }));
    await waitFor(() => {
      expect(pickStySpy).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByText("YAMAHA .STY")).toBeInTheDocument();

    // Imported .STY package should be merged into Music Package Selector
    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    const pkgPopover = screen.getByTestId("music-package-selector-popover");
    expect(within(pkgPopover).getByRole("option", { name: /Forro_Classico\.sty/i })).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));

    fireEvent.click(screen.getByRole("button", { name: "Pasta .STY" }));
    await waitFor(() => {
      expect(pickStyFolderSpy).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByText("MULTI .STY")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Importar SoundFont .SF2" }));
    await waitFor(() => {
      expect(pickSf2Spy).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findAllByText("GeneralUser_GS.sf2")).toHaveLength(2);
  });

  it("wires per-track instrument override and One Touch Setting (OTS) presets in Tauri mode", async () => {
    const initialArr = buildMockArrangerState({
      source_format: "sty",
      file_name: "Baião_Tradicional.sty",
      file_path: "/styles/Baião_Tradicional.sty",
      instruments: [
        { index: 1, name: "Strings (Stereo)", zone_count: 7, has_samples: true, is_drumkit: false },
        { index: 2, name: "BTR JOSIRAN PISADINHA", zone_count: 33, has_samples: true, is_drumkit: true },
        { index: 7, name: "BANDONEON VA7 ST SOLO", zone_count: 12, has_samples: true, is_drumkit: false },
        { index: 8, name: "Tenor Sax SF2", zone_count: 10, has_samples: true, is_drumkit: false },
      ],
      ots_presets: [
        {
          index: 0,
          name: "Sanfona Solo",
          upper1_instrument: 7,
          upper2_instrument: null,
          lower_instrument: null,
        },
        {
          index: 1,
          name: "Sax & Strings",
          upper1_instrument: 8,
          upper2_instrument: 1,
          lower_instrument: null,
        },
      ],
    });
    const { getLive, setLive } = setupTauriStateMocks([], initialArr);

    const trackInstSpy = vi
      .spyOn(engineClient, "setArrangerTrackInstrument")
      .mockImplementation(async (trackIndex, instrumentIndex) => {
        const nextInsts = [...(getLive().track_instruments ?? Array(8).fill(null))];
        nextInsts[trackIndex] = instrumentIndex;
        return setLive({ ...getLive(), track_instruments: nextInsts });
      });

    const otsSpy = vi
      .spyOn(engineClient, "applyArrangerOts")
      .mockImplementation(async (otsIndex) => {
        const preset = (getLive().ots_presets ?? []).find((p) => p.index === otsIndex);
        if (!preset) return getLive();
        return setLive({
          ...getLive(),
          upper1_instrument: preset.upper1_instrument,
          upper2_instrument: preset.upper2_instrument,
          lower_instrument: preset.lower_instrument,
        });
      });

    render(<App />);
    expect(await screen.findByText("Visor do Arranjador")).toBeInTheDocument();

    const drumSelect = screen.getByLabelText("Drum Instrument");
    fireEvent.change(drumSelect, { target: { value: "2" } });
    await waitFor(() => {
      expect(trackInstSpy).toHaveBeenCalledWith(0, 2);
    });
    expect(drumSelect).toHaveValue("2");

    fireEvent.change(drumSelect, { target: { value: "" } });
    await waitFor(() => {
      expect(trackInstSpy).toHaveBeenCalledWith(0, null);
    });
    expect(drumSelect).toHaveValue("");

    const ots2Btn = screen.getByRole("button", { name: "OTS 2: Sax & Strings" });
    expect(ots2Btn).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(ots2Btn);

    await waitFor(() => {
      expect(otsSpy).toHaveBeenCalledWith(1);
    });
    expect(screen.getByRole("button", { name: "OTS 2: Sax & Strings" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByLabelText("Upper 1 Instrument Trigger")).toHaveTextContent("Tenor Sax SF2");
    expect(screen.getByLabelText("Upper 2 Instrument Trigger")).toHaveTextContent("Strings (Stereo)");
  });

  it("filters Music Package Selector options with topbar format filter pills and loads .STY transparently", async () => {
    const pkgList: MusicPackageEntry[] = [
      {
        file_name: "Josiran_Pisadinha_GR.Music",
        file_path: "/music/Josiran_Pisadinha_GR.Music",
        size_mb: 23.4,
        version: "1.4.4",
        format: "music",
      },
      {
        file_name: "Arrasta_Pe_Forro_GR.Music",
        file_path: "/music/Arrasta_Pe_Forro_GR.Music",
        size_mb: 38.8,
        version: "1.4.4",
        format: "music",
      },
      {
        file_name: "Vaneira_Sul.sty",
        file_path: "/styles/Vaneira_Sul.sty",
        size_mb: 0.08,
        version: "STY",
        format: "sty",
      },
      {
        file_name: "Forro_Piseiro.sty",
        file_path: "/styles/Forro_Piseiro.sty",
        size_mb: 0.12,
        version: "STY",
      },
    ];
    const initialArr = buildMockArrangerState();
    const { getLive, setLive } = setupTauriStateMocks(pkgList, initialArr);

    const loadSpy = vi
      .spyOn(engineClient, "loadMusicPackage")
      .mockImplementation(async (path) =>
        setLive({
          ...getLive(),
          file_name: path.split("/").pop() ?? path,
          file_path: path,
          source_format: path.toLowerCase().endsWith(".sty") ? "sty" : "music",
        }),
      );

    render(<App />);
    expect(await screen.findByText("Visor do Arranjador")).toBeInTheDocument();

    const filterGroup = screen.getByRole("group", { name: "Filtro de formato da biblioteca" });
    const allPill = within(filterGroup).getByRole("button", { name: "Todos" });
    const musicPill = within(filterGroup).getByRole("button", { name: ".Music" });
    const styPill = within(filterGroup).getByRole("button", { name: ".STY" });

    expect(allPill).toHaveAttribute("aria-pressed", "true");

    // Filter to .STY only
    fireEvent.click(styPill);
    expect(styPill).toHaveAttribute("aria-pressed", "true");
    expect(allPill).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    const styPopover = screen.getByTestId("music-package-selector-popover");
    expect(within(styPopover).getByRole("option", { name: /Vaneira_Sul\.sty/i })).toBeInTheDocument();
    expect(within(styPopover).getByRole("option", { name: /Forro_Piseiro\.sty/i })).toBeInTheDocument();
    expect(within(styPopover).queryByRole("option", { name: /Josiran_Pisadinha_GR\.Music/i })).not.toBeInTheDocument();

    fireEvent.click(within(styPopover).getByRole("option", { name: /Vaneira_Sul\.sty/i }));
    await waitFor(() => {
      expect(loadSpy).toHaveBeenCalledWith("/styles/Vaneira_Sul.sty");
    });

    // Filter to .Music only
    fireEvent.click(musicPill);
    expect(musicPill).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    const musicPopover = screen.getByTestId("music-package-selector-popover");
    expect(within(musicPopover).getByRole("option", { name: /Josiran_Pisadinha_GR\.Music/i })).toBeInTheDocument();
    expect(within(musicPopover).getByRole("option", { name: /Arrasta_Pe_Forro_GR\.Music/i })).toBeInTheDocument();
    expect(within(musicPopover).queryByRole("option", { name: /Vaneira_Sul\.sty/i })).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));

    // Restore Todos
    fireEvent.click(allPill);
    fireEvent.click(screen.getByLabelText("Music Package Selector Trigger"));
    const allPopover = screen.getByTestId("music-package-selector-popover");
    expect(within(allPopover).getByRole("option", { name: /Josiran_Pisadinha_GR\.Music/i })).toBeInTheDocument();
    expect(within(allPopover).getByRole("option", { name: /Vaneira_Sul\.sty/i })).toBeInTheDocument();
  });

  it("supports track instrument override, disabled OTS placeholders, and preview dialog fallbacks without Tauri", () => {
    render(<App />);

    expect(screen.getAllByText("FluidR3_GM.sf2")).toHaveLength(2);
    for (const slot of [1, 2, 3, 4]) {
      expect(screen.getByRole("button", { name: `OTS ${slot}` })).toBeDisabled();
    }

    const bassSelect = screen.getByLabelText("Bass Instrument");
    fireEvent.change(bassSelect, { target: { value: "3" } });
    expect(bassSelect).toHaveValue("3");

    fireEvent.click(screen.getByRole("button", { name: "Importar .STY" }));
    expect(screen.queryByText(/Importador de ritmos \.STY disponível/i)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    expect(screen.getByText(/Importador de ritmos \.STY disponível no aplicativo desktop Tauri/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Fechar logs" }));
    fireEvent.click(screen.getByRole("button", { name: "Importar SoundFont .SF2" }));
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    expect(screen.getByText(/Importador de SoundFont \.SF2 disponível no aplicativo desktop Tauri/i)).toBeInTheDocument();
  });

  it("keeps STY warnings and import diagnostics in logs, off the main screen", async () => {
    const expWarning = "Expansion Kit voice bank detected (MSB 63) — remapped to GM/SF2";
    const audioWarning = "Live Audio Style chunks (AASM/AFil/AWav) detected — audio stems skipped";
    const diagnosticsSummary = "CFF2 | Expansion Kit MSB 63 | Audio chunks: AASM, AFil, AWav";

    const initialArr = buildMockArrangerState({
      source_format: "sty",
      file_name: "Forro_Audio_Live.sty",
      file_path: "/styles/Forro_Audio_Live.sty",
      sty_warnings: [expWarning, audioWarning],
      has_audio_style_chunks: true,
      uses_expansion_kit: true,
      sty_diagnostics_summary: diagnosticsSummary,
    });
    setupTauriStateMocks([], initialArr);

    render(<App />);
    expect(screen.queryByTestId("sty-warning-banner")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    const banner = await screen.findByRole("dialog", { name: "Logs" });
    await waitFor(() => expect(banner).toHaveTextContent(diagnosticsSummary));
    expect(banner).toHaveTextContent(expWarning);
    expect(banner).toHaveTextContent(audioWarning);
  });

  it("records fallback STY notices from boolean flags in logs", async () => {
    const flagOnlyArr = buildMockArrangerState({
      source_format: "sty",
      file_name: "Expansion_Only.sty",
      file_path: "/styles/Expansion_Only.sty",
      sty_warnings: [],
      has_audio_style_chunks: true,
      uses_expansion_kit: true,
      sty_diagnostics_summary: null,
    });
    setupTauriStateMocks([], flagOnlyArr);

    render(<App />);
    expect(screen.queryByTestId("sty-warning-banner")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    const fallbackBanner = await screen.findByRole("dialog", { name: "Logs" });
    await waitFor(() => expect(fallbackBanner).toHaveTextContent(/Expansion Kit/i));
    expect(fallbackBanner).toHaveTextContent(/AASM\/AFil\/AWav/i);

  });

  it("hides sty-warning-banner when sty_warnings is empty in .Music mode", async () => {
    const musicArr = buildMockArrangerState({
      source_format: "music",
      styles: [
        {
          id: 99,
          name: "Clean Music Style",
          category: "Forró",
          sections: ["Variation 1"],
        },
      ],
      sty_warnings: [],
      has_audio_style_chunks: false,
      uses_expansion_kit: false,
      sty_diagnostics_summary: null,
    });
    setupTauriStateMocks([], musicArr);

    render(<App />);
    await waitFor(() => {
      expect(screen.getByLabelText("Style Selector Trigger")).toHaveTextContent("Clean Music Style");
    });
    expect(screen.queryByTestId("sty-warning-banner")).not.toBeInTheDocument();
  });
});

describe("VirtualKeyboard component", () => {
  it("supports 105-key extended range (C0 to G#8) when extendedRange is true", () => {
    render(
      <VirtualKeyboard
        activeNotes={[]}
        extendedRange={true}
        onNoteOn={vi.fn()}
        onNoteOff={vi.fn()}
      />,
    );

    const keyboard = screen.getByTestId("virtual-keyboard");
    expect(within(keyboard).getAllByRole("button")).toHaveLength(105);
    expect(within(keyboard).getByRole("button", { name: "Key C0" })).toBeInTheDocument();
    expect(within(keyboard).getByRole("button", { name: "Key G#8" })).toBeInTheDocument();
    expect(keyboard.querySelectorAll(".white-key")).toHaveLength(61);
    expect(keyboard.querySelectorAll(".black-key")).toHaveLength(44);
  });

  it("highlights activeNotes and triggers mouse and computer-key note events", () => {
    const onNoteOn = vi.fn();
    const onNoteOff = vi.fn();

    render(
      <VirtualKeyboard
        activeNotes={[64]}
        extendedRange={false}
        onNoteOn={onNoteOn}
        onNoteOff={onNoteOff}
      />,
    );

    const keyE4 = screen.getByRole("button", { name: "Key E4" });
    const keyC4 = screen.getByRole("button", { name: "Key C4" });

    expect(keyE4).toHaveClass("active");
    expect(keyC4).not.toHaveClass("active");

    fireEvent.mouseDown(keyC4, { clientY: 50 });
    expect(onNoteOn).toHaveBeenCalledTimes(1);
    expect(onNoteOn.mock.calls[0][0]).toBe(60);
    expect(keyC4).toHaveClass("active");

    fireEvent.mouseUp(keyC4);
    expect(onNoteOff).toHaveBeenCalledWith(60);
    expect(keyC4).not.toHaveClass("active");

    fireEvent.keyDown(window, { key: "q" });
    expect(onNoteOn).toHaveBeenLastCalledWith(60, 88);
    expect(keyC4).toHaveClass("active");

    fireEvent.keyUp(window, { key: "q" });
    expect(onNoteOff).toHaveBeenLastCalledWith(60);
    expect(keyC4).not.toHaveClass("active");
  });
});

describe("Startup, audio negotiation, and maintenance", () => {
  it("offers every requested buffer size, defaults to 256, and avoids fake zero latency", () => {
    render(<App />);
    const buffer = screen.getByLabelText("Audio Buffer Size");
    expect(buffer).toHaveValue("256");
    expect(within(buffer).getAllByRole("option").map((option) => option.getAttribute("value")))
      .toEqual(["64", "128", "256", "512", "1024"]);
    expect(screen.getByText("— ms")).toBeInTheDocument();
    const footer = screen.getByRole("contentinfo");
    expect(within(footer).queryByText(/MIDI desconectado/)).not.toBeInTheDocument();
    expect(within(footer).queryByText(/Carregado/)).not.toBeInTheDocument();
    expect(within(footer).queryByRole("button", { name: "Sustain (Damper)" })).not.toBeInTheDocument();
    expect(screen.queryByText(/Teclado PC: Z–M/)).not.toBeInTheDocument();
    const splitControls = screen.getByRole("group", { name: "Split and Chord Scan Controls" });
    expect(within(splitControls).getByRole("button", { name: "Sustain (Damper)" })).toBeInTheDocument();
    const start = screen.getByRole("button", { name: "START RITMO" });
    expect(start.closest(".lcd-tempo-row")).toContainElement(screen.getByLabelText("Tempo BPM"));
    expect(screen.getByTestId("virtual-keyboard").firstElementChild).toHaveStyle({ minWidth: "1224px" });
  });

  it("initializes audio and arranger without waiting for a slow library and preserves imports", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    let resolveLibrary!: (value: MusicPackageEntry[]) => void;
    vi.mocked(engineClient.listMusicPackages).mockReturnValue(new Promise((resolve) => { resolveLibrary = resolve; }));
    let resolveInitial!: (value: MusicPackageSummary) => void;
    vi.mocked(engineClient.getArrangerState).mockReturnValueOnce(new Promise((resolve) => { resolveInitial = resolve; }));
    const imported = buildMockArrangerState({ file_name: "New.Music", file_path: "/music/New.Music" });
    vi.spyOn(engineClient, "pickMusicPackageDialog").mockResolvedValue(imported);
    render(<App />);
    await waitFor(() => expect(screen.getByText("5.3 ms")).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: /Importar .Music/ }));
    await waitFor(() => expect(screen.getByLabelText("Music Package Selector Trigger")).toHaveTextContent("New.Music"));
    await act(async () => {
      resolveInitial(buildMockArrangerState());
      resolveLibrary([]);
    });
    expect(screen.getByLabelText("Music Package Selector Trigger")).toHaveTextContent("New.Music");
    expect(screen.queryByText(/^Carregado Josiran/)).not.toBeInTheDocument();
  });

  it("keeps requested and observed callback sizes distinct and serializes changes", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    vi.mocked(engineClient.getTelemetry).mockResolvedValue({ ...buildMockTelemetry(), requested_buffer_size: 128, buffer_size: 256 });
    let resolveChange!: (value: EngineTelemetry) => void;
    const change = vi.spyOn(engineClient, "setAudioBufferSize").mockReturnValue(new Promise((resolve) => { resolveChange = resolve; }));
    render(<App />);
    const buffer = screen.getByLabelText("Audio Buffer Size");
    await waitFor(() => expect(buffer).toHaveValue("128"));
    expect(screen.getByText("5.3 ms")).toBeInTheDocument();
    fireEvent.change(buffer, { target: { value: "64" } });
    expect(buffer).toBeDisabled();
    fireEvent.change(buffer, { target: { value: "512" } });
    expect(change).toHaveBeenCalledTimes(1);
    await act(async () => resolveChange({ ...buildMockTelemetry(), requested_buffer_size: 64, buffer_size: 0, latency_ms: 0, audio_active: false }));
    expect(buffer).not.toBeDisabled();
    expect(buffer).toHaveValue("64");
    expect(screen.getByText("— ms")).toBeInTheDocument();
  });

  it("shows only measured block duration when driver timing is zero, with details in logs", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    vi.mocked(engineClient.getTelemetry).mockResolvedValue({ ...buildMockTelemetry(), output_latency_ms: 0, latency_ms: 99 });
    render(<App />);
    const readout = await screen.findByText("5.3 ms");
    expect(readout).toHaveAttribute("title", expect.stringContaining("256 frames ÷ 48000 Hz"));
    expect(screen.queryByText(/saída est\./)).not.toBeInTheDocument();
    expect(screen.queryByText(/callback ·/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    expect(screen.getByRole("dialog", { name: "Logs" })).toHaveTextContent("indisponível");
  });

  it("keeps rejected strict 128 selected, reports the error, then permits explicit 64", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    vi.mocked(engineClient.getTelemetry).mockResolvedValue({ ...buildMockTelemetry(), requested_buffer_size: 256 });
    const change = vi.spyOn(engineClient, "setAudioBufferSize")
      .mockRejectedValueOnce(new Error("Exact buffer rejected"))
      .mockResolvedValueOnce({ ...buildMockTelemetry(), requested_buffer_size: 64, buffer_size: 16, latency_ms: 1 / 3 });
    render(<App />);
    await screen.findByText("5.3 ms");
    const buffer = screen.getByLabelText("Audio Buffer Size");
    fireEvent.change(buffer, { target: { value: "128" } });
    const logs = await screen.findByRole("dialog", { name: "Logs" });
    expect(logs).toHaveTextContent("Exact buffer rejected");
    expect(buffer).toHaveValue("128");
    expect(change).toHaveBeenCalledTimes(1);
    expect(change).toHaveBeenNthCalledWith(1, 128);
    fireEvent.click(screen.getByRole("button", { name: "Fechar logs" }));
    fireEvent.change(buffer, { target: { value: "64" } });
    await waitFor(() => expect(buffer).not.toBeDisabled());
    expect(change).toHaveBeenNthCalledWith(2, 64);
    expect(buffer).toHaveValue("64");
  });

  it("keeps audio/configuration diagnostics only in closable logs and allows clearing", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    vi.mocked(engineClient.getTelemetry).mockResolvedValue({ ...buildMockTelemetry(), audio_error: "Output disconnected", configuration_error: "Saved configuration invalid" });
    render(<App />);
    await waitFor(() => expect(screen.getByText("5.3 ms")).toBeInTheDocument());
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.queryByText("Output disconnected")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    const dialog = await screen.findByRole("dialog", { name: "Logs" });
    expect(dialog).toHaveTextContent("Output disconnected");
    expect(dialog).toHaveTextContent("Saved configuration invalid");
    expect(dialog).toHaveTextContent("256 ÷ 48000 × 1000 = 5.33 ms");
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Logs" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Logs" }));
    expect(screen.getByRole("dialog", { name: "Logs" })).toHaveTextContent("Output disconnected");
    fireEvent.click(screen.getByRole("button", { name: "Limpar logs" }));
    expect(screen.queryByText("Output disconnected")).not.toBeInTheDocument();
    expect(screen.queryByText(/^Carregado /)).not.toBeInTheDocument();
  });

  it("resets instruments, saves configuration, and confirms saved-only removal", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    const reset = vi.spyOn(engineClient, "resetArrangerInstruments").mockResolvedValue(buildMockArrangerState());
    const save = vi.spyOn(engineClient, "saveCurrentConfiguration").mockResolvedValue();
    const clear = vi.spyOn(engineClient, "clearSavedConfiguration").mockResolvedValue();
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(<App />);
    fireEvent.click(screen.getByLabelText("Ferramentas"));
    fireEvent.click(screen.getByRole("button", { name: "Resetar timbres" }));
    await waitFor(() => expect(reset).toHaveBeenCalledOnce());
    await waitFor(() => expect(screen.getByRole("button", { name: "Salvar configuração atual" })).not.toBeDisabled());
    fireEvent.click(screen.getByRole("button", { name: "Salvar configuração atual" }));
    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    await waitFor(() => expect(screen.getByRole("button", { name: "Remover configuração salva" })).not.toBeDisabled());
    fireEvent.click(screen.getByRole("button", { name: "Remover configuração salva" }));
    expect(confirm).toHaveBeenCalledWith(expect.stringContaining("Os ajustes em uso não serão alterados"));
    expect(clear).not.toHaveBeenCalled();
    confirm.mockReturnValue(true);
    fireEvent.click(screen.getByRole("button", { name: "Remover configuração salva" }));
    await waitFor(() => expect(clear).toHaveBeenCalledOnce());
  });

  it("scans before confirmation, reports recovery errors and manifest, and restores quarantine", async () => {
    setupTauriStateMocks([], buildMockArrangerState());
    const scan = { root: "/styles", groups: [], scanned_files: 3, redundant_files: 1, reclaimable_bytes: 1048576 };
    const report = { manifest_path: "/quarantine/job/manifest.json", moved_files: 0, restored_files: 0, quarantined_bytes: 0, status: "recovery_required", errors: ["Restore conflict"] };
    const scanSpy = vi.spyOn(engineClient, "scanStyleDuplicates").mockResolvedValue(scan);
    const clean = vi.spyOn(engineClient, "cleanupStyleDuplicates").mockResolvedValue(report);
    const restore = vi.spyOn(engineClient, "restoreStyleDuplicates").mockResolvedValue({ ...report, status: "restored", errors: [], restored_files: 1 });
    render(<App />);
    fireEvent.click(screen.getByLabelText("Ferramentas"));
    fireEvent.click(screen.getByRole("button", { name: "Limpar ritmos duplicados" }));
    const dialog = await screen.findByRole("dialog", { name: "Confirmar limpeza de ritmos" });
    expect(scanSpy).toHaveBeenCalledOnce();
    expect(clean).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole("button", { name: "Confirmar quarentena" }));
    await waitFor(() => expect(clean).toHaveBeenCalledWith(scan));
    const result = await screen.findByRole("alert", { name: "Resultado da limpeza" });
    expect(result).toHaveTextContent("Recuperação necessária");
    expect(result).toHaveTextContent("Movidos: 0");
    expect(result).toHaveTextContent(report.manifest_path);
    expect(result).toHaveTextContent("Restore conflict");
    fireEvent.click(within(result).getByRole("button", { name: "Desfazer quarentena" }));
    await waitFor(() => expect(restore).toHaveBeenCalledWith(report.manifest_path));
    expect(await screen.findByText("Restauração concluída")).toBeInTheDocument();
  });

  it("maps the entire lower PC row below F#3 inclusive and preserves solo Q", () => {
    const noteOn = vi.fn();
    const noteOff = vi.fn();
    render(<VirtualKeyboard activeNotes={[]} extendedRange={false} splitEnabled splitKey={54} onNoteOn={noteOn} onNoteOff={noteOff} />);
    ["z", "s", "x", "d", "c", "v", "g", "b", "h", "n", "j", "m"].forEach((key, index) => {
      fireEvent.keyDown(window, { key });
      expect(noteOn).toHaveBeenLastCalledWith(36 + index, 88);
      fireEvent.keyUp(window, { key });
      expect(noteOff).toHaveBeenLastCalledWith(36 + index);
    });
    fireEvent.keyDown(window, { key: "q" });
    expect(noteOn).toHaveBeenLastCalledWith(60, 88);
    fireEvent.keyUp(window, { key: "q" });
    expect(screen.getByRole("button", { name: "Key F#3" })).toHaveClass("split-chord-zone");
    expect(screen.getByRole("button", { name: "Key G3" })).not.toHaveClass("split-chord-zone");
  });
});

describe("Late saved configuration bootstrap", () => {
  it("refreshes restored volume and MIDI, then preserves subsequent user edits", async () => {
    vi.useFakeTimers();
    try {
      setupTauriStateMocks([], buildMockArrangerState());
      vi.mocked(engineClient.listMidiDevices).mockResolvedValue([
        { index: 0, name: "Restored MIDI" }, { index: 1, name: "User MIDI" },
      ]);
      vi.spyOn(engineClient, "updateParams").mockResolvedValue();
      vi.spyOn(engineClient, "connectMidiDevice").mockResolvedValue("User MIDI");
      render(<App />);
      await act(async () => {});
      const volume = screen.getByRole("group", { name: "Master Volume Controls" });
      expect(volume).toHaveTextContent("-4.0 dB");
      expect(screen.getByLabelText("MIDI Input")).toHaveValue("");
      const restored = buildMockSnapshot();
      restored.params.volume_db = -11;
      restored.midi_config.selected_port = "Restored MIDI";
      vi.mocked(engineClient.getSnapshot).mockResolvedValue(restored);
      await act(async () => { await vi.advanceTimersByTimeAsync(2500); });
      expect(volume).toHaveTextContent("-11.0 dB");
      expect(screen.getByLabelText("MIDI Input")).toHaveValue("Restored MIDI");
      fireEvent.click(screen.getByRole("button", { name: "Aumentar Volume" }));
      fireEvent.change(screen.getByLabelText("MIDI Input"), { target: { value: "User MIDI" } });
      const stale = buildMockSnapshot();
      stale.params.volume_db = -19;
      stale.midi_config.selected_port = "Restored MIDI";
      vi.mocked(engineClient.getSnapshot).mockResolvedValue(stale);
      await act(async () => { await vi.advanceTimersByTimeAsync(2500); });
      expect(volume).toHaveTextContent("-9.5 dB");
      expect(screen.getByLabelText("MIDI Input")).toHaveValue("User MIDI");
    } finally {
      vi.useRealTimers();
    }
  });
});
