import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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
    split_key: 53,
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
    track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
    beat_position: 0,
    ...overrides,
  };
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Music Studio Tauri Arranger App UI", () => {
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
    expect(within(keyboard).getByRole("button", { name: "Key G3" })).toHaveClass("split-point-edge");
    expect(within(keyboard).getByTestId("split-divider-line")).toBeInTheDocument();
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
    const params = buildMockEngineParams();
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
    const snapshot: EngineSnapshot = {
      active_instrument_id: "default",
      active_preset_id: "default",
      preset_b_id: null,
      params,
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
    const telemetry: EngineTelemetry = {
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

    vi.spyOn(engineClient, "isTauri").mockReturnValue(true);
    vi.spyOn(engineClient, "listMusicPackages").mockResolvedValue(pkgList);
    vi.spyOn(engineClient, "getArrangerState").mockResolvedValue(initialArr);
    vi.spyOn(engineClient, "getSnapshot").mockResolvedValue(snapshot);
    vi.spyOn(engineClient, "listMidiDevices").mockResolvedValue([]);
    vi.spyOn(engineClient, "getTelemetry").mockResolvedValue(telemetry);

    const chordSpy = vi
      .spyOn(engineClient, "setArrangerChord")
      .mockResolvedValue(buildMockArrangerState({ detected_chord: "Am", detected_chord_root: 9, detected_chord_quality: "Minor" }));
    const transportSpy = vi
      .spyOn(engineClient, "setArrangerTransport")
      .mockResolvedValue(buildMockArrangerState({ rhythm_playing: true }));
    const sectionSpy = vi
      .spyOn(engineClient, "setArrangerSection")
      .mockResolvedValue(buildMockArrangerState({ active_section: "Variation 2" }));
    const loadSpy = vi
      .spyOn(engineClient, "loadMusicPackage")
      .mockResolvedValue(buildMockArrangerState({ file_name: "Arrasta_Pe_Forro_GR.Music", file_path: "/music/Arrasta_Pe_Forro_GR.Music" }));
    const pickSpy = vi
      .spyOn(engineClient, "pickMusicPackageDialog")
      .mockResolvedValue(buildMockArrangerState({ file_name: "Picked.Music", file_path: "/home/data/Documents/Ritmos_Forro_Music_Studio/Picked.Music" }));
    const pickInstSpy = vi
      .spyOn(engineClient, "pickInstrumentDialog")
      .mockResolvedValue(
        buildMockArrangerState({
          upper1_instrument: 8,
          instruments: [
            ...initialArr.instruments,
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
      .mockResolvedValue(buildMockArrangerState({ chord_scan_upper: true, split_enabled: false }));
    const bufSpy = vi
      .spyOn(engineClient, "setAudioBufferSize")
      .mockResolvedValue({ ...telemetry, buffer_size: 128, latency_ms: 2.7 });
    const updateParamsSpy = vi.spyOn(engineClient, "updateParams").mockResolvedValue();
    const setPedalSpy = vi.spyOn(engineClient, "setPedal").mockResolvedValue();

    render(<App />);

    expect(await screen.findByText("Visor do Arranjador (.Music)")).toBeInTheDocument();
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
