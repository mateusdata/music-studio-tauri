import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import App from "../../src/App";
import { VirtualKeyboard } from "../../src/components/VirtualKeyboard";
import { engineClient } from "../../src/services/engineClient";
import type { EngineParams, EngineSnapshot, EngineTelemetry } from "../../src/types/engine";

const PEDAL_LABELS = [
  "Una Corda (Soft)",
  "Sostenuto",
  "Harmonic",
  "Sustain (Damper)",
] as const;

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
        {
          enabled: true,
          name: "Right Close",
          x: 0.72,
          y: 0.42,
          z: 0.65,
          azimuth: 0.0,
          elevation: -15.0,
          volume: 0.85,
          pan: 0.58,
        },
        {
          enabled: false,
          name: "Room Ribbon",
          x: 0.5,
          y: 0.85,
          z: 0.8,
          azimuth: 0.0,
          elevation: -15.0,
          volume: 0.55,
          pan: 0.0,
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

afterEach(() => {
  vi.restoreAllMocks();
});

describe("Piano Tect Tauri App UI", () => {
  it("renders title, preview status, telemetry strip, tabs, and action buttons", () => {
    render(<App />);

    expect(screen.getByText("Piano Tect Tauri")).toBeInTheDocument();
    expect(screen.getByText("Physical Modeling Studio")).toBeInTheDocument();
    expect(screen.getByLabelText("Instrument Selector")).toBeInTheDocument();
    expect(screen.getByLabelText("Preset Selector")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Test Chord/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Export WAV/i })).toBeInTheDocument();
    expect(screen.getByText("0/128 voices")).toBeInTheDocument();
    expect(screen.getByText(/Browser preview mode/i)).toBeInTheDocument();

    const nav = screen.getByRole("navigation", { name: "Parameter Sections" });
    expect(within(nav).getAllByRole("button")).toHaveLength(7);
    expect(screen.getByRole("button", { name: "Voicing & Hammers" })).toHaveClass("active");
    expect(screen.getByRole("button", { name: "Note Edit (PRO)" })).toBeInTheDocument();
  });

  it("renders 88 virtual keyboard keys by default with expected note aria-labels", () => {
    render(<App />);

    const keyboard = screen.getByTestId("virtual-keyboard");
    const keyButtons = within(keyboard).getAllByRole("button");
    expect(keyButtons).toHaveLength(88);

    expect(within(keyboard).getByRole("button", { name: "Key A0" })).toBeInTheDocument();
    expect(within(keyboard).getByRole("button", { name: "Key C4" })).toBeInTheDocument();
    expect(within(keyboard).getByRole("button", { name: "Key A4" })).toBeInTheDocument();
    expect(within(keyboard).getByRole("button", { name: "Key C8" })).toBeInTheDocument();

    expect(within(keyboard).queryByRole("button", { name: "Key C0" })).not.toBeInTheDocument();
    expect(within(keyboard).queryByRole("button", { name: "Key G#8" })).not.toBeInTheDocument();

    expect(keyboard.querySelectorAll(".white-key")).toHaveLength(52);
    expect(keyboard.querySelectorAll(".black-key")).toHaveLength(36);
  });

  it("renders all four pedal buttons and toggles active state on click", () => {
    render(<App />);

    for (const label of PEDAL_LABELS) {
      const btn = screen.getByRole("button", { name: label });
      expect(btn).toBeInTheDocument();
      expect(btn).not.toHaveClass("active");

      fireEvent.click(btn);
      expect(btn).toHaveClass("active");
    }

    // All four pedals can remain active concurrently; clicking again deactivates individually
    for (const label of PEDAL_LABELS) {
      const btn = screen.getByRole("button", { name: label });
      expect(btn).toHaveClass("active");
      fireEvent.click(btn);
      expect(btn).not.toHaveClass("active");
    }
  });

  it("switches active parameter navigation tabs when clicked", () => {
    render(<App />);

    const voicingTab = screen.getByRole("button", { name: "Voicing & Hammers" });
    const tuningTab = screen.getByRole("button", { name: "Tuning & Temperament" });
    const midiTab = screen.getByRole("button", { name: "MIDI & Action" });

    expect(voicingTab).toHaveClass("active");
    expect(tuningTab).not.toHaveClass("active");

    fireEvent.click(tuningTab);
    expect(tuningTab).toHaveClass("active");
    expect(voicingTab).not.toHaveClass("active");

    fireEvent.click(midiTab);
    expect(midiTab).toHaveClass("active");
    expect(tuningTab).not.toHaveClass("active");
  });

  it("loads snapshot in Tauri mode and forwards pedal toggles to engineClient", async () => {
    const params = buildMockEngineParams();
    const snapshot: EngineSnapshot = {
      active_instrument_id: "steinway_d",
      active_preset_id: "concert_grand_warm",
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
      instruments: [
        {
          id: "steinway_d",
          name: "Model D Concert Grand",
          category: "GrandPiano",
          exciter: "FeltHammer",
          description: "Concert grand piano",
          inharmonicity_base: 0.0001,
          decay_scale: 1.0,
          default_params: params,
        },
      ],
      presets: [
        {
          id: "concert_grand_warm",
          name: "Concert Hall Warm",
          instrument_id: "steinway_d",
          category: "Acoustic",
          description: "Default concert preset",
          params,
          is_factory: true,
        },
      ],
    };
    const telemetry: EngineTelemetry = {
      sample_rate: 48000,
      buffer_size: 256,
      latency_ms: 5.3,
      active_voices: 3,
      max_polyphony: 128,
      cpu_load_pct: 4.2,
      peak_l: 0.4,
      peak_r: 0.35,
      rms_l: 0.2,
      rms_r: 0.18,
      audio_backend: "PipeWire",
      audio_active: true,
      midi_connected: false,
      midi_port_name: null,
      last_midi_note: 60,
      last_midi_velocity: 90,
      active_notes: [60, 64, 67],
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
      midi_transport: {
        is_playing: false,
        is_recording: false,
        playback_speed: 1.0,
        recorded_events_count: 0,
        current_track_name: null,
      },
      midi_learn_target: null,
    };

    vi.spyOn(engineClient, "isTauri").mockReturnValue(true);
    vi.spyOn(engineClient, "getSnapshot").mockResolvedValue(snapshot);
    vi.spyOn(engineClient, "listMidiDevices").mockResolvedValue([]);
    vi.spyOn(engineClient, "getTelemetry").mockResolvedValue(telemetry);
    const setPedalSpy = vi.spyOn(engineClient, "setPedal").mockResolvedValue();

    render(<App />);

    expect(await screen.findByText("Master & Morphing")).toBeInTheDocument();
    expect(screen.getByText("Hammer Felt Hardness")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Tuning & Temperament" }));
    expect(screen.getByText("Diapason (A4)")).toBeInTheDocument();

    const sustainBtn = screen.getByRole("button", { name: "Sustain (Damper)" });
    fireEvent.click(sustainBtn);
    expect(sustainBtn).toHaveClass("active");
    await waitFor(() => {
      expect(setPedalSpy).toHaveBeenCalledWith("sustain", 1.0);
    });

    fireEvent.click(sustainBtn);
    expect(sustainBtn).not.toHaveClass("active");
    await waitFor(() => {
      expect(setPedalSpy).toHaveBeenCalledWith("sustain", 0.0);
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
    expect(onNoteOn.mock.calls[0][1]).toBeGreaterThanOrEqual(42);
    expect(onNoteOn.mock.calls[0][1]).toBeLessThanOrEqual(124);
    expect(keyC4).toHaveClass("active");

    fireEvent.mouseUp(keyC4);
    expect(onNoteOff).toHaveBeenCalledWith(60);
    expect(keyC4).not.toHaveClass("active");

    // Computer keyboard 'q' maps to MIDI 60 (C4) at velocity 88
    fireEvent.keyDown(window, { key: "q" });
    expect(onNoteOn).toHaveBeenLastCalledWith(60, 88);
    expect(keyC4).toHaveClass("active");

    fireEvent.keyUp(window, { key: "q" });
    expect(onNoteOff).toHaveBeenLastCalledWith(60);
    expect(keyC4).not.toHaveClass("active");
  });
});
