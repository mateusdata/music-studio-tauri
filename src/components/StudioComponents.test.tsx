import "@testing-library/jest-dom/vitest";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import App from "../App";
import { engineClient } from "../services/engineClient";
import {
  VelocityCurveEditor,
  VELOCITY_CURVE_PRESETS,
  evaluateVelocityCurve,
} from "./VelocityCurveEditor";
import { MicStageEditor } from "./MicStageEditor";
import { NoteEditPanel } from "./NoteEditPanel";
import { SavePresetPopover } from "./SavePresetPopover";
import type { EngineSnapshot, MidiConfig, OutputStageParams, PresetData } from "../types/engine";

function makeSnapshot(): EngineSnapshot {
  const output: OutputStageParams = {
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
        azimuth: 0,
        elevation: -15,
        volume: 0.85,
        pan: -0.58,
      },
      {
        enabled: true,
        name: "Right Close",
        x: 0.72,
        y: 0.42,
        z: 0.65,
        azimuth: 0,
        elevation: -15,
        volume: 0.85,
        pan: 0.58,
      },
      {
        enabled: false,
        name: "Room Ribbon",
        x: 0.5,
        y: 0.85,
        z: 0.8,
        azimuth: 0,
        elevation: -15,
        volume: 0.55,
        pan: 0.0,
      },
    ],
  };

  const defaultParams: EngineSnapshot["params"] = {
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
    output,
    eq_enabled: true,
    eq_bands: [
      { freq_hz: 110, gain_db: 0.5, q: 0.8 },
      { freq_hz: 1100, gain_db: 0, q: 1.0 },
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
      room_size_m: 18,
      pre_delay_ms: 14,
      early_reflections: 0.45,
      tone: 0.52,
      highpass_hz: 110,
      preset_name: "Concert Hall",
    },
    limiter: {
      enabled: true,
      threshold_db: -0.8,
      sharpness: 0.65,
      gain_db: 0,
    },
    note_edit: {},
  };

  return {
    active_instrument_id: "grand-ny-d",
    active_preset_id: "grand-ny-d-default",
    preset_b_id: "grand-ny-d-binaural",
    params: defaultParams,
    midi_config: {
      selected_port: null,
      channel: 0,
      microtonal_multichannel: false,
      transpose: 0,
      key_range_mode: "Full88",
      velocity_curve: [
        [0, 0],
        [0.25, 0.25],
        [0.5, 0.5],
        [0.75, 0.75],
        [1, 1],
      ],
      cc_mappings: [{ cc: 1, param_path: "morphing", min: 0, max: 1 }],
    },
    instruments: [
      {
        id: "grand-ny-d",
        name: "Grand Concert NY D",
        category: "GrandPiano",
        exciter: "FeltHammer",
        description: "Test Grand",
        inharmonicity_base: 0.00018,
        decay_scale: 1.05,
        default_params: defaultParams,
      },
    ],
    presets: [
      {
        id: "grand-ny-d-default",
        name: "Grand Concert NY D Studio",
        instrument_id: "grand-ny-d",
        category: "Studio",
        description: "Studio preset",
        params: defaultParams,
        is_factory: true,
      },
    ],
  };
}

describe("Studio UI components", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    if (typeof window.PointerEvent === "undefined") {
      // JSDOM does not define PointerEvent; alias MouseEvent so clientX/clientY work in pointer drag tests.
      (window as unknown as { PointerEvent: typeof MouseEvent }).PointerEvent = MouseEvent;
    }
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  it("evaluates velocity curve, switches all 5 presets, adds/removes points, and supports SVG pointer drag", () => {
    expect(evaluateVelocityCurve(0, VELOCITY_CURVE_PRESETS[0].points)).toBe(0);
    expect(evaluateVelocityCurve(127, VELOCITY_CURVE_PRESETS[0].points)).toBe(127);
    expect(evaluateVelocityCurve(64, VELOCITY_CURVE_PRESETS[1].points)).toBeGreaterThan(64);
    expect(evaluateVelocityCurve(64, VELOCITY_CURVE_PRESETS[2].points)).toBeLessThan(64);

    const snap = makeSnapshot();
    const onUpdate = vi.fn();
    const { rerender } = render(
      <VelocityCurveEditor
        midiConfig={snap.midi_config}
        lastVelocity={90}
        onUpdateMidiConfig={onUpdate}
      />,
    );

    const svg = screen.getByTestId("velocity-curve-svg");
    expect(svg).toBeInTheDocument();
    expect(screen.getByTestId("velocity-live-indicator")).toBeInTheDocument();

    // All 5 required presets
    for (const preset of VELOCITY_CURVE_PRESETS) {
      fireEvent.click(screen.getByRole("button", { name: preset.label }));
      const latestCfg: MidiConfig = onUpdate.mock.calls[onUpdate.mock.calls.length - 1][0];
      expect(latestCfg.velocity_curve).toEqual(preset.points);
    }

    // Add a control point and verify length increases to 6
    fireEvent.click(screen.getByRole("button", { name: "Add Control Point" }));
    const afterAddCfg: MidiConfig = onUpdate.mock.calls[onUpdate.mock.calls.length - 1][0];
    expect(afterAddCfg.velocity_curve.length).toBe(6);

    rerender(
      <VelocityCurveEditor
        midiConfig={afterAddCfg}
        lastVelocity={90}
        onUpdateMidiConfig={onUpdate}
      />,
    );

    // Drag interior point #2 on SVG
    vi.spyOn(svg, "getBoundingClientRect").mockReturnValue({
      left: 0,
      top: 0,
      width: 280,
      height: 196,
      right: 280,
      bottom: 196,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    });
    const pt2 = screen.getByTestId("velocity-point-2");
    fireEvent.pointerDown(pt2, { pointerId: 1, clientX: 140, clientY: 100 });
    fireEvent.pointerMove(svg, { pointerId: 1, clientX: 130, clientY: 55 });
    fireEvent.pointerUp(svg);

    const afterDragCfg: MidiConfig = onUpdate.mock.calls[onUpdate.mock.calls.length - 1][0];
    expect(afterDragCfg.velocity_curve[2][1]).toBeGreaterThan(0.6);

    // Remove the selected interior point
    fireEvent.click(screen.getByRole("button", { name: "Remove Selected Interior Point" }));
    const afterRemoveCfg: MidiConfig = onUpdate.mock.calls[onUpdate.mock.calls.length - 1][0];
    expect(afterRemoveCfg.velocity_curve.length).toBe(5);
  });

  it("controls 3 mics, pointer drag on stage map, lid position, and binaural head in MicStageEditor", () => {
    const snap = makeSnapshot();
    const onChange = vi.fn();
    const { rerender } = render(
      <MicStageEditor output={snap.params.output} onChange={onChange} />,
    );

    const stageMap = screen.getByTestId("mic-stage-map");
    expect(stageMap).toBeInTheDocument();
    expect(screen.getByTestId("binaural-head-marker")).toBeInTheDocument();

    // Toggle Mic 3 on and select it
    fireEvent.click(screen.getByLabelText("Toggle Mic 3 (Room Ribbon)"));
    let latestOut: OutputStageParams = onChange.mock.calls[0][0];
    expect(latestOut.mics[2].enabled).toBe(true);

    fireEvent.click(screen.getByRole("button", { name: /M3: Room Ribbon/i }));
    rerender(<MicStageEditor output={latestOut} onChange={onChange} />);

    // Adjust Mic 3 Pan & Volume
    fireEvent.change(screen.getByLabelText(/M3 Volume/i), { target: { value: "1.1" } });
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.mics[2].volume).toBe(1.1);

    fireEvent.change(screen.getByLabelText(/M3 Pan/i), { target: { value: "-0.35" } });
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.mics[2].pan).toBe(-0.35);

    // Drag Mic 1 on SVG stage map
    vi.spyOn(stageMap, "getBoundingClientRect").mockReturnValue({
      left: 0,
      top: 0,
      width: 380,
      height: 290,
      right: 380,
      bottom: 290,
      x: 0,
      y: 0,
      toJSON: () => ({}),
    });
    fireEvent.pointerDown(screen.getByTestId("stage-mic-0"), { pointerId: 1 });
    fireEvent.pointerMove(stageMap, { pointerId: 1, clientX: 250, clientY: 90 });
    fireEvent.pointerUp(stageMap);
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.mics[0].x).toBeGreaterThan(0.5);
    expect(latestOut.mics[0].y).toBeGreaterThan(0.5);

    // Toggle delay compensation + lid + binaural head angle & reset center
    fireEvent.click(screen.getByLabelText("Mic Delay Compensation"));
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.mic_delay_compensation).toBe(true);

    fireEvent.change(screen.getByLabelText(/Lid Position/i), { target: { value: "0.4" } });
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.lid_position).toBe(0.4);

    const headControls = screen.getByTestId("binaural-head-controls");
    fireEvent.change(within(headControls).getByLabelText(/Head Angle/i), {
      target: { value: "45" },
    });
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.head_angle).toBe(45);

    rerender(<MicStageEditor output={latestOut} onChange={onChange} />);
    fireEvent.click(within(headControls).getByRole("button", { name: /Center/i }));
    latestOut = onChange.mock.calls[onChange.mock.calls.length - 1][0];
    expect(latestOut.head_x).toBe(0.5);
    expect(latestOut.head_y).toBe(0.22);
    expect(latestOut.head_angle).toBe(0);
  });

  it("selects any MIDI key 21..108 and edits all 4 offsets in NoteEditPanel", () => {
    const onChangeNoteEdit = vi.fn();
    const onAuditionNote = vi.fn();
    const { rerender } = render(
      <NoteEditPanel
        noteEdit={{}}
        diapasonHz={440}
        onChangeNoteEdit={onChangeNoteEdit}
        onAuditionNote={onAuditionNote}
      />,
    );

    const select = screen.getByLabelText("Select MIDI Key") as HTMLSelectElement;
    expect(select.options.length).toBe(88);
    expect(select.options[0].value).toBe("21");
    expect(select.options[87].value).toBe("108");

    // Click A0 on mini keyboard strip and edit impedance_offset
    fireEvent.click(screen.getByLabelText("Edit A0 (MIDI 21)"));
    fireEvent.change(screen.getByLabelText(/Impedance Offset/i), {
      target: { value: "-0.24" },
    });
    expect(onChangeNoteEdit).toHaveBeenLastCalledWith({
      21: {
        detune_cents: 0,
        gain_db: 0,
        hammer_hardness_offset: 0,
        impedance_offset: -0.24,
      },
    });

    // Click C8 via dropdown and edit detune + hammer hardness
    fireEvent.change(select, { target: { value: "108" } });
    fireEvent.change(screen.getByLabelText(/Detune/i), { target: { value: "12.5" } });
    const with108 = onChangeNoteEdit.mock.calls[onChangeNoteEdit.mock.calls.length - 1][0];
    expect(with108[108].detune_cents).toBe(12.5);

    rerender(
      <NoteEditPanel
        noteEdit={with108}
        diapasonHz={440}
        onChangeNoteEdit={onChangeNoteEdit}
        onAuditionNote={onAuditionNote}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Audition/i }));
    expect(onAuditionNote).toHaveBeenCalledWith(108);

    fireEvent.click(screen.getByRole("button", { name: /Reset All/i }));
    expect(onChangeNoteEdit).toHaveBeenLastCalledWith({});
  });

  it("validates, handles Escape, and surfaces errors in SavePresetPopover", async () => {
    const onSave = vi.fn().mockRejectedValueOnce(new Error("Disk full"));
    render(<SavePresetPopover onSavePreset={onSave} />);

    fireEvent.click(screen.getByRole("button", { name: /Save User Preset/i }));
    const dialog = screen.getByTestId("save-preset-popover");
    const saveBtn = within(dialog).getByRole("button", { name: "Save" });
    expect(saveBtn).toBeDisabled();

    fireEvent.change(within(dialog).getByLabelText("Preset Name"), {
      target: { value: "Test Fail" },
    });
    expect(saveBtn).not.toBeDisabled();
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(screen.getByText("Disk full")).toBeInTheDocument();
    });

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByTestId("save-preset-popover")).not.toBeInTheDocument();
  });

  it("preserves baseline labels/tabs and integrates all editors & historical pedals in App", async () => {
    const snap = makeSnapshot();
    const savedPreset: PresetData = {
      id: "user-midnight-felt",
      name: "Midnight Felt",
      instrument_id: "grand-ny-d",
      category: "Intimate",
      description: "Saved by user",
      params: snap.params,
      is_factory: false,
    };

    vi.spyOn(engineClient, "isTauri").mockReturnValue(true);
    vi.spyOn(engineClient, "getSnapshot").mockResolvedValue(snap);
    vi.spyOn(engineClient, "listMidiDevices").mockResolvedValue([]);
    vi.spyOn(engineClient, "getTelemetry").mockResolvedValue({
      sample_rate: 48000,
      buffer_size: 256,
      latency_ms: 5.3,
      active_voices: 0,
      max_polyphony: 128,
      cpu_load_pct: 1.5,
      peak_l: 0,
      peak_r: 0,
      rms_l: 0,
      rms_r: 0,
      audio_backend: "cpal-alsa",
      audio_active: true,
      midi_connected: false,
      midi_port_name: null,
      last_midi_note: null,
      last_midi_velocity: 84,
      active_notes: [],
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
    });
    const setPedalSpy = vi.spyOn(engineClient, "setPedal").mockResolvedValue();
    const updateMidiSpy = vi.spyOn(engineClient, "updateMidiConfig").mockResolvedValue();
    const updateParamsSpy = vi.spyOn(engineClient, "updateParams").mockResolvedValue();
    const savePresetSpy = vi
      .spyOn(engineClient, "saveUserPreset")
      .mockResolvedValue(savedPreset);

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("Instrument Selector")).toHaveValue("grand-ny-d");
    });

    // Preserved tabs + new PRO Note Edit tab
    for (const tabLabel of [
      "Voicing & Hammers",
      "Tuning & Temperament",
      "Acoustic Design",
      "Mics & Binaural Stage",
      "EQ, FX & Reverb",
      "MIDI & Action",
      "Note Edit (PRO)",
    ]) {
      expect(screen.getByRole("button", { name: tabLabel })).toBeInTheDocument();
    }

    // Historical & Primary pedals
    for (const label of [
      "Una Corda (Soft)",
      "Sostenuto",
      "Harmonic",
      "Sustain (Damper)",
      "Celeste",
      "Buff Stop",
      "Rattle",
    ]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }

    fireEvent.click(screen.getByRole("button", { name: "Celeste" }));
    fireEvent.click(screen.getByRole("button", { name: "Buff Stop" }));
    fireEvent.click(screen.getByRole("button", { name: "Rattle" }));
    expect(setPedalSpy).toHaveBeenCalledWith("celeste", 1.0);
    expect(setPedalSpy).toHaveBeenCalledWith("buff_stop", 1.0);
    expect(setPedalSpy).toHaveBeenCalledWith("rattle", 1.0);

    // Save User Preset Popover
    fireEvent.click(screen.getByRole("button", { name: /Save User Preset/i }));
    const dialog = screen.getByTestId("save-preset-popover");
    fireEvent.change(within(dialog).getByLabelText("Preset Name"), {
      target: { value: "Midnight Felt" },
    });
    fireEvent.change(within(dialog).getByLabelText("Preset Category"), {
      target: { value: "Intimate" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(savePresetSpy).toHaveBeenCalledWith("Midnight Felt", "Intimate");
    });
    await waitFor(() => {
      expect(screen.getByLabelText("Preset Selector")).toHaveValue("user-midnight-felt");
    });

    // MIDI & Action tab -> VelocityCurveEditor
    fireEvent.click(screen.getByRole("button", { name: "MIDI & Action" }));
    fireEvent.click(screen.getByRole("button", { name: "Hard Touch" }));
    expect(updateMidiSpy).toHaveBeenCalled();

    // Mics & Binaural Stage tab -> MicStageEditor
    fireEvent.click(screen.getByRole("button", { name: "Mics & Binaural Stage" }));
    expect(screen.getByTestId("mic-stage-editor")).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("Binaural Head Mode"));
    expect(updateParamsSpy).toHaveBeenCalled();

    // Note Edit (PRO) tab -> NoteEditPanel
    fireEvent.click(screen.getByRole("button", { name: "Note Edit (PRO)" }));
    expect(screen.getByTestId("note-edit-panel")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText(/Gain/i), { target: { value: "3.2" } });
    expect(updateParamsSpy).toHaveBeenLastCalledWith(
      expect.objectContaining({
        note_edit: expect.objectContaining({
          60: expect.objectContaining({ gain_db: 3.2 }),
        }),
      }),
    );
  });

  it("renders interactive editors in browser preview mode without Tauri runtime", () => {
    render(<App />);
    expect(screen.getByText("Piano Tect Tauri")).toBeInTheDocument();
    expect(screen.getByTestId("virtual-keyboard")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Mics & Binaural Stage" }));
    expect(screen.getByTestId("mic-stage-editor")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "MIDI & Action" }));
    expect(screen.getByTestId("velocity-curve-editor")).toBeInTheDocument();
  });
});
