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
import { BounceAndNoteFxPanel } from "./BounceAndNoteFxPanel";
import { MidiTransportBar } from "./MidiTransportBar";
import { InstrumentPackBrowser, PIANOTEQ_PACKS } from "./InstrumentPackBrowser";
import type {
  EngineParams,
  EngineSnapshot,
  InstrumentCategory,
  MidiConfig,
  MidiTransportState,
  OutputStageParams,
  PresetData,
} from "../types/engine";

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
    expect(screen.getByTestId("midi-transport-bar")).toBeInTheDocument();

    const nav = screen.getByRole("navigation", { name: "Parameter Sections" });
    expect(within(nav).getAllByRole("button")).toHaveLength(7);

    fireEvent.click(screen.getByRole("button", { name: "Acoustic Design" }));
    expect(screen.getByTestId("bounce-notefx-panel")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Mics & Binaural Stage" }));
    expect(screen.getByTestId("mic-stage-editor")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "MIDI & Action" }));
    expect(screen.getByTestId("velocity-curve-editor")).toBeInTheDocument();
  });

  it("controls Mallet Bounce, NoteFX sliders, DiatonicHarp 7 pedals, and Guitar mode in BounceAndNoteFxPanel", () => {
    let currentParams: EngineParams = JSON.parse(JSON.stringify(makeSnapshot().params));
    const applyMutator = (mutator: (draft: EngineParams) => void) => {
      const cloned: EngineParams = JSON.parse(JSON.stringify(currentParams));
      mutator(cloned);
      currentParams = cloned;
    };

    const { rerender } = render(
      <BounceAndNoteFxPanel params={currentParams} onUpdateParams={applyMutator} />,
    );

    // Toggle Mallet Bounce & adjust bounce sliders
    fireEvent.click(screen.getByLabelText("Enable Mallet Bounce"));
    expect(currentParams.bounce?.enabled).toBe(true);
    fireEvent.change(screen.getByLabelText(/Bounce Delay/i), { target: { value: "240" } });
    expect(currentParams.bounce?.delay_ms).toBe(240);
    fireEvent.change(screen.getByLabelText(/Max Bounces/i), { target: { value: "9" } });
    expect(currentParams.bounce?.max_bounces).toBe(9);

    // Adjust Per-Note FX sliders
    fireEvent.change(screen.getByLabelText(/Vibrato Depth/i), { target: { value: "35" } });
    expect(currentParams.note_fx?.vibrato_cents).toBe(35);
    fireEvent.change(screen.getByLabelText(/LFO Rate/i), { target: { value: "7.5" } });
    expect(currentParams.note_fx?.lfo_rate_hz).toBe(7.5);
    fireEvent.change(screen.getByLabelText(/Tremolo Depth/i), { target: { value: "6" } });
    expect(currentParams.note_fx?.tremolo_depth_db).toBe(6);
    fireEvent.change(screen.getByLabelText(/Onset Duration/i), { target: { value: "150" } });
    expect(currentParams.note_fx?.onset_duration_ms).toBe(150);
    fireEvent.change(screen.getByLabelText(/Attack Envelope/i), { target: { value: "0.75" } });
    expect(currentParams.note_fx?.attack_envelope).toBe(0.75);
    fireEvent.change(screen.getByLabelText(/Smear/i), { target: { value: "0.42" } });
    expect(currentParams.note_fx?.smear).toBe(0.42);

    // Switch to DiatonicHarp and verify 7 pedals cycle 0 -> +1 -> -1 -> 0
    fireEvent.click(screen.getByRole("button", { name: "DiatonicHarp" }));
    expect(currentParams.play_mode?.mode).toBe("DiatonicHarp");

    rerender(
      <BounceAndNoteFxPanel params={currentParams} onUpdateParams={applyMutator} />,
    );
    const harpGroup = screen.getByRole("group", { name: "Harp Pedals" });
    expect(within(harpGroup).getAllByRole("button")).toHaveLength(7);

    // Cycle C pedal: 0 -> +1 -> -1 -> 0
    fireEvent.click(screen.getByLabelText(/Harp Pedal C:/i));
    expect(currentParams.play_mode?.harp_pedals[0]).toBe(1);
    rerender(
      <BounceAndNoteFxPanel params={currentParams} onUpdateParams={applyMutator} />,
    );
    fireEvent.click(screen.getByLabelText(/Harp Pedal C:/i));
    expect(currentParams.play_mode?.harp_pedals[0]).toBe(-1);
    rerender(
      <BounceAndNoteFxPanel params={currentParams} onUpdateParams={applyMutator} />,
    );
    fireEvent.click(screen.getByLabelText(/Harp Pedal C:/i));
    expect(currentParams.play_mode?.harp_pedals[0]).toBe(0);

    // Switch to Guitar mode
    fireEvent.click(screen.getByRole("button", { name: "Guitar" }));
    expect(currentParams.play_mode?.mode).toBe("Guitar");
    rerender(
      <BounceAndNoteFxPanel params={currentParams} onUpdateParams={applyMutator} />,
    );
    fireEvent.change(screen.getByLabelText(/Guitar Fret/i), { target: { value: "5" } });
    expect(currentParams.play_mode?.guitar_fret).toBe(5);
    fireEvent.click(screen.getByLabelText(/Guitar Legato/i));
    expect(currentParams.play_mode?.guitar_legato).toBe(true);
    fireEvent.change(screen.getByLabelText(/Guitar Body/i), { target: { value: "0.8" } });
    expect(currentParams.play_mode?.guitar_body).toBe(0.8);
  });

  it("wires MidiTransportBar buttons, playback speed, and MIDI CC Learn to engineClient", async () => {
    const baseState: MidiTransportState = {
      is_playing: false,
      is_recording: false,
      playback_speed: 1.0,
      recorded_events_count: 0,
      current_track_name: null,
    };

    const playDemoSpy = vi
      .spyOn(engineClient, "playDemoMidi")
      .mockResolvedValue({ ...baseState, is_playing: true, current_track_name: "Demo Prelude" });
    const stopSpy = vi
      .spyOn(engineClient, "stopMidiTransport")
      .mockResolvedValue({ ...baseState, is_playing: false });
    const recSpy = vi
      .spyOn(engineClient, "toggleMidiRecord")
      .mockResolvedValue({ ...baseState, is_recording: true });
    const playRecSpy = vi
      .spyOn(engineClient, "playRecordedMidi")
      .mockResolvedValue({ ...baseState, is_playing: true, recorded_events_count: 12 });
    const speedSpy = vi
      .spyOn(engineClient, "setMidiPlaybackSpeed")
      .mockResolvedValue({ ...baseState, playback_speed: 1.5 });
    const learnSpy = vi
      .spyOn(engineClient, "armMidiCcLearn")
      .mockResolvedValueOnce("morphing")
      .mockResolvedValueOnce(null);

    const onStatus = vi.fn();
    render(<MidiTransportBar onStatusMessage={onStatus} />);

    fireEvent.click(screen.getByRole("button", { name: /Play Demo/i }));
    await waitFor(() => expect(playDemoSpy).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: /Stop/i }));
    await waitFor(() => expect(stopSpy).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: /Record/i }));
    await waitFor(() => expect(recSpy).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: /Play Rec/i }));
    await waitFor(() => expect(playRecSpy).toHaveBeenCalledTimes(1));

    fireEvent.change(screen.getByLabelText("MIDI Playback Speed"), {
      target: { value: "1.5" },
    });
    await waitFor(() => expect(speedSpy).toHaveBeenCalledWith(1.5));

    fireEvent.change(screen.getByLabelText("MIDI CC Learn Target"), {
      target: { value: "morphing" },
    });
    fireEvent.click(screen.getByRole("button", { name: /MIDI CC Learn/i }));
    await waitFor(() => expect(learnSpy).toHaveBeenCalledWith("morphing"));

    fireEvent.click(screen.getByRole("button", { name: /MIDI CC Learn/i }));
    await waitFor(() => expect(learnSpy).toHaveBeenLastCalledWith(null));
  });

  it("groups instruments by category with optgroup, exposes all 10 FX processors with dynamic labels, and exposes 10 reverb environments", async () => {
    const snap = makeSnapshot();
    const categories: [InstrumentCategory, string, string][] = [
      ["GrandPiano", "grand-ny-d", "Grand Concert NY D"],
      ["UprightPiano", "upright-u4", "Studio Upright U4"],
      ["ElectricPiano", "vintage-tines-mk2", "Vintage Tines MKII"],
      ["Historical", "ruckers-harpsichord", "Flemish Harpsichord II"],
      ["Mallet", "vibraphone-vb", "Concert Vibraphone V-B"],
      ["Plucked", "concert-harp", "Grand Concert Harp"],
      ["Prepared", "prepared-grand", "Prepared Grand"],
      ["Synthesizer", "synth-pad", "FM Hybrid Piano"],
    ];
    snap.instruments = categories.map(([category, id, name]) => ({
      id,
      name,
      category,
      exciter: "FeltHammer",
      description: name,
      inharmonicity_base: 0.00018,
      decay_scale: 1.0,
      default_params: snap.params,
    }));

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
      audio_backend: "PipeWire",
      audio_active: true,
      midi_connected: false,
      midi_port_name: null,
      last_midi_note: null,
      last_midi_velocity: null,
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
    const updateParamsSpy = vi.spyOn(engineClient, "updateParams").mockResolvedValue();

    render(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("Instrument Selector")).toHaveValue("grand-ny-d");
    });

    // Verify all 8 category optgroups
    const instSelect = screen.getByLabelText("Instrument Selector");
    const optgroups = Array.from(instSelect.querySelectorAll("optgroup")).map((g) => g.label);
    expect(optgroups).toEqual([
      "Grand Pianos",
      "Upright Pianos",
      "Electric Pianos",
      "Historical Collection",
      "Mallets & Percussion",
      "Plucked (Guitar & Harps)",
      "Prepared & Experimental",
      "Synthesizer",
    ]);

    // Switch to EQ, FX & Reverb tab
    fireEvent.click(screen.getByRole("button", { name: "EQ, FX & Reverb" }));

    // Verify all 10 Reverb environments + parameters
    const reverbSelect = screen.getByLabelText("Reverb Environment") as HTMLSelectElement;
    const reverbEnvs = Array.from(reverbSelect.options).map((o) => o.value);
    expect(reverbEnvs).toEqual([
      "Concert Hall",
      "Cathedral",
      "Studio",
      "Chamber",
      "Taj Mahal",
      "Plate",
      "Spring",
      "Speakers",
      "Broceliande",
      "Club",
    ]);

    fireEvent.change(reverbSelect, { target: { value: "Taj Mahal" } });
    expect(updateParamsSpy).toHaveBeenLastCalledWith(
      expect.objectContaining({
        reverb: expect.objectContaining({ preset_name: "Taj Mahal" }),
      }),
    );

    const reverbPanel = screen.getByTestId("reverb-panel");
    fireEvent.change(within(reverbPanel).getByLabelText(/Room Size/i), {
      target: { value: "42" },
    });
    expect(updateParamsSpy).toHaveBeenLastCalledWith(
      expect.objectContaining({
        reverb: expect.objectContaining({ room_size_m: 42 }),
      }),
    );
    fireEvent.click(within(reverbPanel).getByLabelText("Reverb Enabled"));
    expect(updateParamsSpy).toHaveBeenLastCalledWith(
      expect.objectContaining({
        reverb: expect.objectContaining({ enabled: false }),
      }),
    );

    // Verify all 10 FX types exist in Slot 1 & check dynamic labels across Wah, Phaser, Flanger, Fuzz, Amp
    const slot1Type = screen.getByLabelText("FX Slot 1 Type") as HTMLSelectElement;
    const fxOptions = Array.from(slot1Type.options).map((o) => o.value);
    expect(fxOptions).toEqual([
      "Delay",
      "Chorus",
      "Flanger",
      "Phaser",
      "Tremolo",
      "Wah",
      "Fuzz",
      "Amp",
      "Compressor",
      "ParametricEq",
    ]);

    const slot0 = screen.getByTestId("fx-slot-0");

    fireEvent.change(slot1Type, { target: { value: "Wah" } });
    expect(within(slot0).getByText("Wah Pedal")).toBeInTheDocument();
    expect(within(slot0).getByText("Auto-Wah Sens")).toBeInTheDocument();
    expect(within(slot0).getByText("Sweep Depth")).toBeInTheDocument();
    expect(within(slot0).getByText("Q")).toBeInTheDocument();
    fireEvent.change(within(slot0).getByLabelText(/Wah Pedal/i), { target: { value: "0.82" } });
    expect(updateParamsSpy).toHaveBeenLastCalledWith(
      expect.objectContaining({
        fx: expect.arrayContaining([
          expect.objectContaining({ effect_type: "Wah", param_1: 0.82 }),
        ]),
      }),
    );

    fireEvent.change(slot1Type, { target: { value: "Phaser" } });
    expect(within(slot0).getByText("Intensity")).toBeInTheDocument();
    expect(within(slot0).getByText("Center")).toBeInTheDocument();

    fireEvent.change(slot1Type, { target: { value: "Flanger" } });
    expect(within(slot0).getByText("Depth")).toBeInTheDocument();
    expect(within(slot0).getByText("Stereo")).toBeInTheDocument();

    fireEvent.change(slot1Type, { target: { value: "Fuzz" } });
    expect(within(slot0).getByText("Drive")).toBeInTheDocument();
    expect(within(slot0).getByText("Grit")).toBeInTheDocument();

    fireEvent.change(slot1Type, { target: { value: "Amp" } });
    expect(within(slot0).getByText("Preamp Drive")).toBeInTheDocument();
    expect(within(slot0).getByText("Tone Stack")).toBeInTheDocument();
    expect(within(slot0).getByText("Cabinet")).toBeInTheDocument();
    expect(within(slot0).getByText("Presence")).toBeInTheDocument();
  });

  it("opens All Packs & Presets 3-column mega-menu, filters packs, selects instrument/preset, and closes", async () => {
    const snap = makeSnapshot();
    snap.instruments = [
      ...snap.instruments,
      {
        id: "upright-u4",
        name: "Studio Upright U4",
        category: "UprightPiano",
        exciter: "FeltHammer",
        description: "Studio upright",
        inharmonicity_base: 0.00042,
        decay_scale: 0.78,
        default_params: snap.params,
      },
      {
        id: "prepared-grand",
        name: "Prepared Experimental Grand",
        category: "Prepared",
        exciter: "PreparedBolt",
        description: "Prepared piano",
        inharmonicity_base: 0.0003,
        decay_scale: 0.9,
        default_params: snap.params,
      },
    ];
    snap.presets = [
      ...snap.presets,
      {
        id: "upright-u4-default",
        name: "Studio Upright U4 Studio",
        instrument_id: "upright-u4",
        category: "Studio",
        description: "Upright studio preset",
        params: snap.params,
        is_factory: true,
      },
      {
        id: "upright-u4-felt",
        name: "Studio Upright U4 Intimate Felt",
        instrument_id: "upright-u4",
        category: "Intimate",
        description: "Felt upright preset",
        params: snap.params,
        is_factory: true,
      },
    ];

    const onSelectInstrument = vi.fn();
    const onSelectPreset = vi.fn();
    const onMorphPresets = vi.fn();

    render(
      <InstrumentPackBrowser
        instruments={snap.instruments}
        presets={snap.presets}
        activeInstrumentId={snap.active_instrument_id}
        activePresetId={snap.active_preset_id}
        presetBId="upright-u4-felt"
        morphing={0.25}
        recentPresetIds={["grand-ny-d-default"]}
        onSelectInstrument={onSelectInstrument}
        onSelectPreset={onSelectPreset}
        onMorphPresets={onMorphPresets}
      />,
    );

    // Open mega-menu
    fireEvent.click(screen.getByRole("button", { name: "All Packs & Presets" }));
    const dialog = screen.getByRole("dialog", { name: "Instrument Packs Browser" });
    expect(dialog).toBeInTheDocument();

    // All 28 Pianoteq pack headers rendered across 3 columns
    expect(PIANOTEQ_PACKS).toHaveLength(28);
    expect(within(dialog).getByTestId("packs-col-1")).toBeInTheDocument();
    expect(within(dialog).getByTestId("packs-col-2")).toBeInTheDocument();
    expect(within(dialog).getByTestId("packs-col-3")).toBeInTheDocument();

    for (const expectedHeader of [
      "Steinway D Instrument Pack",
      "U4 Instrument Pack",
      "Electric Instrument Pack",
      "Hohner Instrument Pack",
      "Syngular Instrument Pack",
      "Morphing/Layers",
      "KIViR Instruments & Bells",
    ]) {
      expect(within(dialog).getByText(expectedHeader)).toBeInTheDocument();
    }

    // Recent presets strip visible
    const recentStrip = within(dialog).getByTestId("recent-presets-strip");
    expect(within(recentStrip).getByRole("button", { name: "Grand Concert NY D Studio" })).toBeInTheDocument();

    // Morphing/Layers A/B controls work
    fireEvent.change(within(dialog).getByLabelText(/Morph A ↔ B/i), { target: { value: "0.7" } });
    expect(onMorphPresets).toHaveBeenCalledWith("grand-ny-d-default", "upright-u4-felt", 0.7);

    // Inspect U4 presets via ▸ submenu button & select a preset
    fireEvent.click(within(dialog).getByLabelText("Presets for Studio Upright U4"));
    const submenu = within(dialog).getByTestId("pack-presets-submenu");
    fireEvent.click(within(submenu).getByRole("button", { name: /Studio Upright U4 Intimate Felt/i }));
    await waitFor(() => expect(onSelectPreset).toHaveBeenCalledWith("upright-u4-felt"));
    expect(screen.queryByRole("dialog", { name: "Instrument Packs Browser" })).not.toBeInTheDocument();

    // Reopen, filter via search input, click instrument directly, and verify Escape/backdrop close
    fireEvent.click(screen.getByRole("button", { name: "All Packs & Presets" }));
    const reopened = screen.getByRole("dialog", { name: "Instrument Packs Browser" });
    fireEvent.change(within(reopened).getByLabelText("Search Instruments and Presets"), {
      target: { value: "Prepared" },
    });
    expect(within(reopened).getByText("Prepared Experimental Grand")).toBeInTheDocument();
    fireEvent.click(within(reopened).getByRole("button", { name: "Prepared Experimental Grand" }));
    await waitFor(() => expect(onSelectInstrument).toHaveBeenCalledWith("prepared-grand"));
    expect(screen.queryByRole("dialog", { name: "Instrument Packs Browser" })).not.toBeInTheDocument();

    // Reopen and close via Escape
    fireEvent.click(screen.getByRole("button", { name: "All Packs & Presets" }));
    expect(screen.getByRole("dialog", { name: "Instrument Packs Browser" })).toBeInTheDocument();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Instrument Packs Browser" })).not.toBeInTheDocument();

    // Reopen and close via backdrop click
    fireEvent.click(screen.getByRole("button", { name: "All Packs & Presets" }));
    fireEvent.click(screen.getByTestId("pack-browser-backdrop"));
    expect(screen.queryByRole("dialog", { name: "Instrument Packs Browser" })).not.toBeInTheDocument();
  });
});
