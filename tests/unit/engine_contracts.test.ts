import { afterEach, describe, expect, it } from "vitest";
import { engineClient } from "../../src/services/engineClient";
import type {
  BounceParams,
  EffectSlot,
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  InstrumentCategory,
  MidiConfig,
  MidiTransportState,
  NoteEditOffset,
  NoteFxParams,
  PedalState,
  PlayMode,
  PlayModeParams,
  TemperamentId,
  TuningParams,
} from "../../src/types/engine";

const TEMPERAMENT_IDS = [
  "Equal",
  "Pythagorean",
  "Zarlino",
  "MeanTone",
  "WellTempered",
  "WerckmeisterIII",
  "KirnbergerIII",
  "Vallotti",
] as const satisfies readonly TemperamentId[];

const INSTRUMENT_CATEGORIES = [
  "GrandPiano",
  "UprightPiano",
  "ElectricPiano",
  "Historical",
  "Mallet",
  "Plucked",
  "Prepared",
  "Synthesizer",
] as const satisfies readonly InstrumentCategory[];

const EFFECT_TYPES = [
  "Delay",
  "ParametricEq",
  "Chorus",
  "Tremolo",
  "Compressor",
] as const satisfies readonly EffectSlot["effect_type"][];

const KEY_RANGE_MODES = [
  "Full88",
  "Compact61",
  "Extended105",
] as const satisfies readonly MidiConfig["key_range_mode"][];

const PLAY_MODES = [
  "Standard",
  "DiatonicHarp",
  "Guitar",
] as const satisfies readonly PlayMode[];

const DIATONIC_PITCH_CLASS_TO_PEDAL_INDEX: Record<number, number> = {
  0: 0, // C
  2: 1, // D
  4: 2, // E
  5: 3, // F
  7: 4, // G
  9: 5, // A
  11: 6, // B
};

const KEYBOARD_RANGES: Record<
  MidiConfig["key_range_mode"],
  {
    min: number;
    max: number;
    totalKeys: number;
    whiteKeys: number;
    blackKeys: number;
    minLabel: string;
    maxLabel: string;
  }
> = {
  Full88: {
    min: 21,
    max: 108,
    totalKeys: 88,
    whiteKeys: 52,
    blackKeys: 36,
    minLabel: "A0",
    maxLabel: "C8",
  },
  Compact61: {
    min: 36,
    max: 96,
    totalKeys: 61,
    whiteKeys: 36,
    blackKeys: 25,
    minLabel: "C2",
    maxLabel: "C7",
  },
  Extended105: {
    min: 12,
    max: 116,
    totalKeys: 105,
    whiteKeys: 61,
    blackKeys: 44,
    minLabel: "C0",
    maxLabel: "G#8",
  },
};

const BLACK_SEMITONES = new Set([1, 3, 6, 8, 10]);
const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

function midiToNoteLabel(midi: number): string {
  const octave = Math.floor(midi / 12) - 1;
  return `${NOTE_NAMES[((midi % 12) + 12) % 12]}${octave}`;
}

function isBlackMidiKey(midi: number): boolean {
  return BLACK_SEMITONES.has(((midi % 12) + 12) % 12);
}

/**
 * Reference temperament ratio relative to 12-TET (A unchanged = 1.0),
 * matching `src-tauri/src/synthesis/mod.rs::temperament_ratio`.
 */
function temperamentRatio(temp: TemperamentId, pitchClass: number): number {
  const pc = ((pitchClass % 12) + 12) % 12;
  switch (temp) {
    case "Equal":
      return 1.0;
    case "Pythagorean": {
      const delta = 1.5 / Math.pow(2, 7 / 12);
      const exps = [-3, -8, -1, -6, 1, -4, 3, -2, -7, 0, -5, 2];
      return Math.pow(delta, exps[pc]);
    }
    case "Zarlino": {
      const ratio2c = [
        1.0,
        25 / 24,
        9 / 8,
        32 / 27,
        5 / 4,
        4 / 3,
        45 / 32,
        3 / 2,
        25 / 16,
        5 / 3,
        16 / 9,
        15 / 8,
      ];
      const aNorm = ratio2c[9] / Math.pow(2, 9 / 12);
      return ratio2c[pc] / Math.pow(2, pc / 12) / aNorm;
    }
    case "MeanTone": {
      const q = Math.pow(5, 0.25);
      const ratio2c = [
        1.0,
        (5 * Math.pow(q, 3)) / 16,
        (q * q) / 2,
        (4 * q) / 5,
        5 / 4,
        2 / q,
        (5 * q * q) / 8,
        q,
        25 / 16,
        Math.pow(q, 3) / 2,
        (4 * q * q) / 5,
        (5 * q) / 4,
      ];
      const aNorm = ratio2c[9] / Math.pow(2, 9 / 12);
      return ratio2c[pc] / Math.pow(2, pc / 12) / aNorm;
    }
    case "WellTempered": {
      const ratio2c = [
        1.0, 1.053498, 1.118034, 1.185185, 1.253134, 1.333333, 1.404664, 1.495349, 1.580247,
        1.671851, 1.777778, 1.879701,
      ];
      const aNorm = ratio2c[9] / Math.pow(2, 9 / 12);
      return ratio2c[pc] / Math.pow(2, pc / 12) / aNorm;
    }
    case "WerckmeisterIII": {
      const ratio2c = [
        1.0,
        256 / 243,
        1.1174,
        32 / 27,
        1.2528,
        4 / 3,
        1024 / 729,
        1.4949,
        128 / 81,
        1.6704,
        16 / 9,
        1.8792,
      ];
      const aNorm = ratio2c[9] / Math.pow(2, 9 / 12);
      return ratio2c[pc] / Math.pow(2, pc / 12) / aNorm;
    }
    case "KirnbergerIII": {
      const cents = [
        0.0, 90.225, 193.157, 294.135, 386.314, 498.045, 590.224, 696.578, 792.18, 889.735,
        996.09, 1088.269,
      ];
      const aDev = cents[9] - 900.0;
      return Math.pow(2, (cents[pc] - pc * 100.0 - aDev) / 1200.0);
    }
    case "Vallotti": {
      const cents = [
        0.0, 94.135, 196.09, 298.045, 392.18, 501.955, 592.18, 698.045, 796.09, 894.135, 1000.0,
        1090.225,
      ];
      const aDev = cents[9] - 900.0;
      return Math.pow(2, (cents[pc] - pc * 100.0 - aDev) / 1200.0);
    }
  }
}

/**
 * Reference fundamental frequency calculation in Hz,
 * matching `src-tauri/src/synthesis/mod.rs::compute_note_frequency`.
 */
function computeNoteFrequency(
  midiNote: number,
  tuning: TuningParams,
  noteEdit?: NoteEditOffset,
): number {
  const baseSemi = midiNote - 69;
  const stretchedSemi =
    baseSemi +
    (tuning.octave_stretching - 1.0) * (baseSemi / 12.0) * 0.045 * Math.sqrt(Math.abs(baseSemi));
  let freq = tuning.diapason_hz * Math.pow(2, (stretchedSemi + tuning.pitch_bend_semitones) / 12);
  freq *= temperamentRatio(tuning.temperament, midiNote % 12);
  if (noteEdit) {
    freq *= Math.pow(2, noteEdit.detune_cents / 1200);
  }
  return Math.max(12.0, Math.min(16000.0, freq));
}

function clampMidiVelocity(velocity: number, allowZero = true): number {
  if (!Number.isFinite(velocity)) return allowZero ? 0 : 1;
  const min = allowZero ? 0 : 1;
  return Math.max(min, Math.min(127, Math.round(velocity)));
}

/**
 * Matches `VirtualKeyboard.tsx::computeVelocity`.
 */
function computeClickVelocity(relativeY: number): number {
  const relY = Math.max(0.15, Math.min(1, relativeY));
  return Math.round(28 + relY * 96);
}

/**
 * Matches `src-tauri/src/midi/mod.rs::apply_velocity_curve`.
 */
function applyVelocityCurve(rawVel: number, curve: [number, number][]): number {
  if (rawVel <= 0) return 0;
  const clampedRaw = Math.min(127, Math.round(rawVel));
  if (curve.length === 0) return clampedRaw;

  const x = Math.max(0, Math.min(1, clampedRaw / 127));
  const pts = [...curve].sort((a, b) => a[0] - b[0]);

  if (x <= pts[0][0]) {
    return Math.round(Math.max(0.01, Math.min(1.0, pts[0][1])) * 127);
  }
  for (let i = 0; i < pts.length - 1; i++) {
    const [x0, y0] = pts[i];
    const [x1, y1] = pts[i + 1];
    if (x <= x1) {
      const span = Math.max(1e-5, x1 - x0);
      const t = (x - x0) / span;
      const y = y0 + (y1 - y0) * t;
      return Math.round(Math.max(0.01, Math.min(1.0, y)) * 127);
    }
  }
  const lastY = pts[pts.length - 1][1];
  return Math.round(Math.max(0.01, Math.min(1.0, lastY)) * 127);
}

/**
 * Matches `src-tauri/src/midi/mod.rs::apply_transpose_and_range`.
 */
function applyTransposeAndRange(
  note: number,
  transpose: number,
  keyRangeMode: MidiConfig["key_range_mode"],
): number | null {
  let shifted = note + transpose;
  if (keyRangeMode === "Compact61") {
    shifted = Math.max(36, Math.min(96, shifted));
  }
  return shifted >= 0 && shifted <= 127 ? shifted : null;
}

/**
 * 7-pedal diatonic harp pitch mapping:
 * `harpPedals` has 7 entries `[C, D, E, F, G, A, B]` where `-1` = flat, `0` = natural, `+1` = sharp.
 * Returns mapped MIDI note for diatonic keys in `[0..127]`, or `null` for non-diatonic keys / out-of-range notes.
 */
function mapDiatonicHarpPitch(midiNote: number, harpPedals: readonly number[]): number | null {
  if (harpPedals.length !== 7) {
    throw new Error(`Expected 7 harp pedals, got ${harpPedals.length}`);
  }
  const pc = ((midiNote % 12) + 12) % 12;
  const pedalIdx = DIATONIC_PITCH_CLASS_TO_PEDAL_INDEX[pc];
  if (pedalIdx === undefined) {
    return null;
  }
  const rawShift = Math.trunc(harpPedals[pedalIdx]);
  const clampedShift = Math.max(-1, Math.min(1, rawShift));
  const mapped = midiNote + clampedShift;
  return mapped >= 0 && mapped <= 127 ? mapped : null;
}

interface BounceHit {
  index: number;
  stepDelayMs: number;
  cumulativeTimeMs: number;
  velocity: number;
}

/**
 * Reference Mallet Bounce geometric decay scheduler:
 * - Inter-bounce delay decays geometrically by `(1 - delay_loss)^(k - 1)` from `delay_ms`
 *   (scaled by `velocity_sensitivity` around midpoint velocity 64).
 * - Bounce velocity decays geometrically by `(1 - velocity_loss)^k` from `initialVelocity`.
 */
function computeBounceSequence(initialVelocity: number, bounce: BounceParams): BounceHit[] {
  if (!bounce.enabled || initialVelocity <= 0 || bounce.max_bounces <= 0) {
    return [];
  }
  const delayLoss = Math.max(0, Math.min(0.95, bounce.delay_loss));
  const velLoss = Math.max(0, Math.min(0.99, bounce.velocity_loss));
  const velSens = Math.max(0, Math.min(1, bounce.velocity_sensitivity));
  const velNorm = Math.max(0, Math.min(1, initialVelocity / 127));
  const baseDelayMs = Math.max(1, bounce.delay_ms * (1 + velSens * (velNorm - 0.5)));

  const hits: BounceHit[] = [];
  let cumulativeTimeMs = 0;

  for (let k = 1; k <= bounce.max_bounces; k++) {
    const velocity = initialVelocity * Math.pow(1 - velLoss, k);
    if (velocity < 1.0) {
      break;
    }
    const stepDelayMs = baseDelayMs * Math.pow(1 - delayLoss, k - 1);
    cumulativeTimeMs += stepDelayMs;
    hits.push({
      index: k,
      stepDelayMs,
      cumulativeTimeMs,
      velocity,
    });
  }
  return hits;
}

function createDefaultBounceParams(): BounceParams {
  return {
    enabled: false,
    delay_ms: 120.0,
    sync_tempo: false,
    velocity_sensitivity: 0.5,
    delay_loss: 0.15,
    velocity_loss: 0.35,
    humanization: 0.1,
    max_bounces: 6,
  };
}

function createDefaultNoteFxParams(): NoteFxParams {
  return {
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
  };
}

function createDefaultPlayModeParams(): PlayModeParams {
  return {
    mode: "Standard",
    harp_pedals: [0, 0, 0, 0, 0, 0, 0],
    guitar_fret: 0,
    guitar_legato: false,
    guitar_body: 0.5,
  };
}

function createDefaultMidiTransportState(): MidiTransportState {
  return {
    is_playing: false,
    is_recording: false,
    playback_speed: 1.0,
    recorded_events_count: 0,
    current_track_name: null,
  };
}

function createDefaultTuning(): TuningParams {
  return {
    diapason_hz: 440.0,
    temperament: "Equal",
    unison_width_cents: 0.7,
    unison_balance: 0.5,
    octave_stretching: 1.0,
    direct_sound_duration: 0.5,
    pitch_bend_semitones: 0.0,
  };
}

function createDefaultPedalState(): PedalState {
  return {
    sustain: 0.0,
    soft: 0.0,
    sostenuto: 0.0,
    harmonic: 0.0,
    celeste: 0.0,
    buff_stop: 0.0,
    rattle: 0.0,
    super_sostenuto: false,
    reversed_sustain: false,
  };
}

function createDefaultEngineParams(): EngineParams {
  return {
    volume_db: -4.0,
    dynamics_db: 40.0,
    velocity_offset: 0.0,
    condition: 0.04,
    post_effect_gain_db: 0.0,
    morphing: 0.0,
    tuning: createDefaultTuning(),
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
      { freq_hz: 110.0, gain_db: 0.5, q: 0.8 },
      { freq_hz: 1100.0, gain_db: 0.0, q: 1.0 },
      { freq_hz: 5200.0, gain_db: 1.0, q: 0.9 },
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
    bounce: createDefaultBounceParams(),
    note_fx: createDefaultNoteFxParams(),
    play_mode: createDefaultPlayModeParams(),
  };
}

afterEach(() => {
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
});

describe("Engine Type & Contract Structures", () => {
  it("matches Rust enum variants for temperaments, categories, effects, key ranges, and play modes", () => {
    expect(TEMPERAMENT_IDS).toHaveLength(8);
    expect(INSTRUMENT_CATEGORIES).toHaveLength(8);
    expect(EFFECT_TYPES).toHaveLength(5);
    expect(KEY_RANGE_MODES).toEqual(["Full88", "Compact61", "Extended105"]);
    expect(PLAY_MODES).toEqual(["Standard", "DiatonicHarp", "Guitar"]);
  });

  it("validates EngineParams, EngineTelemetry, and EngineSnapshot structure parity", () => {
    const params = createDefaultEngineParams();
    const pedals = createDefaultPedalState();
    const midiConfig: MidiConfig = {
      selected_port: null,
      channel: 0,
      microtonal_multichannel: false,
      transpose: 0,
      key_range_mode: "Full88",
      velocity_curve: [
        [0.0, 0.0],
        [0.25, 0.22],
        [0.5, 0.5],
        [0.75, 0.78],
        [1.0, 1.0],
      ],
      cc_mappings: [
        { cc: 1, param_path: "morphing", min: 0.0, max: 1.0 },
        { cc: 7, param_path: "volume_db", min: -48.0, max: 6.0 },
      ],
    };

    const telemetry: EngineTelemetry = {
      sample_rate: 48000,
      buffer_size: 256,
      latency_ms: 5.33,
      active_voices: 0,
      max_polyphony: 128,
      cpu_load_pct: 0.0,
      peak_l: 0.0,
      peak_r: 0.0,
      rms_l: 0.0,
      rms_r: 0.0,
      audio_backend: "Headless",
      audio_active: false,
      midi_connected: false,
      midi_port_name: null,
      last_midi_note: null,
      last_midi_velocity: null,
      active_notes: [],
      pedals,
      midi_transport: createDefaultMidiTransportState(),
      midi_learn_target: null,
    };

    const snapshot: EngineSnapshot = {
      active_instrument_id: "steinway_d",
      active_preset_id: "default",
      preset_b_id: null,
      params,
      midi_config: midiConfig,
      instruments: [
        {
          id: "steinway_d",
          name: "Model D Concert Grand",
          category: "GrandPiano",
          exciter: "FeltHammer",
          description: "Reference grand piano",
          inharmonicity_base: 0.0001,
          decay_scale: 1.0,
          default_params: params,
        },
      ],
      presets: [
        {
          id: "default",
          name: "Concert Hall",
          instrument_id: "steinway_d",
          category: "Acoustic",
          description: "Factory default",
          params,
          is_factory: true,
        },
      ],
    };

    expect(snapshot.params.voicing.spectrum_profile).toHaveLength(8);
    expect(snapshot.params.output.mics).toHaveLength(3);
    expect(snapshot.params.eq_bands).toHaveLength(3);
    expect(snapshot.params.fx).toHaveLength(3);
    expect(snapshot.params.bounce).toEqual(createDefaultBounceParams());
    expect(snapshot.params.note_fx).toEqual(createDefaultNoteFxParams());
    expect(snapshot.params.play_mode?.harp_pedals).toHaveLength(7);
    expect(snapshot.params.play_mode?.mode).toBe("Standard");
    expect(telemetry.max_polyphony).toBe(128);
    expect(Object.keys(telemetry.pedals)).toHaveLength(9);
    expect(telemetry.midi_transport).toEqual(createDefaultMidiTransportState());
    expect(telemetry.midi_learn_target).toBeNull();
  });

  it("validates Cycle 2 BounceParams, NoteFxParams, PlayModeParams, and MidiTransportState defaults", () => {
    const bounce = createDefaultBounceParams();
    expect(bounce.enabled).toBe(false);
    expect(bounce.delay_ms).toBe(120.0);
    expect(bounce.delay_loss).toBe(0.15);
    expect(bounce.velocity_loss).toBe(0.35);
    expect(bounce.max_bounces).toBe(6);

    const noteFx = createDefaultNoteFxParams();
    expect(noteFx.lfo_rate_hz).toBe(5.2);
    expect(noteFx.onset_duration_ms).toBe(80.0);
    expect(noteFx.vibrato_cents).toBe(0.0);
    expect(noteFx.tremolo_depth_db).toBe(0.0);

    const playMode = createDefaultPlayModeParams();
    expect(playMode.mode).toBe("Standard");
    expect(playMode.harp_pedals).toEqual([0, 0, 0, 0, 0, 0, 0]);
    expect(playMode.guitar_fret).toBe(0);
    expect(playMode.guitar_legato).toBe(false);
    expect(playMode.guitar_body).toBe(0.5);

    const transport = createDefaultMidiTransportState();
    expect(transport.is_playing).toBe(false);
    expect(transport.is_recording).toBe(false);
    expect(transport.playback_speed).toBe(1.0);
    expect(transport.recorded_events_count).toBe(0);
    expect(transport.current_track_name).toBeNull();
  });

  it("exposes expected engineClient IPC facade including Cycle 2 transport and CC learn methods", () => {
    const expectedMethods = [
      "isTauri",
      "getSnapshot",
      "getTelemetry",
      "noteOn",
      "noteOff",
      "allNotesOff",
      "setPedal",
      "updateParams",
      "selectInstrument",
      "selectPreset",
      "morphPresets",
      "saveUserPreset",
      "listMidiDevices",
      "connectMidiDevice",
      "updateMidiConfig",
      "playTestChord",
      "exportDemoWav",
      "playDemoMidi",
      "stopMidiTransport",
      "toggleMidiRecord",
      "playRecordedMidi",
      "setMidiPlaybackSpeed",
      "armMidiCcLearn",
    ] as const;

    for (const method of expectedMethods) {
      expect(typeof engineClient[method]).toBe("function");
    }

    expect(engineClient.isTauri()).toBe(false);
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    expect(engineClient.isTauri()).toBe(true);
  });
});

describe("Note Frequency Math Reference", () => {
  it("computes standard 12-TET piano frequencies accurately", () => {
    const tuning = createDefaultTuning();

    expect(computeNoteFrequency(69, tuning)).toBeCloseTo(440.0, 6);
    expect(computeNoteFrequency(21, tuning)).toBeCloseTo(27.5, 6);
    expect(computeNoteFrequency(60, tuning)).toBeCloseTo(261.625565, 4);
    expect(computeNoteFrequency(108, tuning)).toBeCloseTo(4186.009045, 3);

    // Exact octave doubling across the 88-key range
    for (let midi = 21; midi <= 96; midi += 12) {
      const fLow = computeNoteFrequency(midi, tuning);
      const fHigh = computeNoteFrequency(midi + 12, tuning);
      expect(fHigh / fLow).toBeCloseTo(2.0, 6);
    }
  });

  it("applies Diapason, Octave Stretching, Pitch Bend, and PRO NoteEdit detune", () => {
    const baroqueTuning: TuningParams = { ...createDefaultTuning(), diapason_hz: 415.0 };
    expect(computeNoteFrequency(69, baroqueTuning)).toBeCloseTo(415.0, 6);

    const stretchedTuning: TuningParams = { ...createDefaultTuning(), octave_stretching: 1.3 };
    expect(computeNoteFrequency(69, stretchedTuning)).toBeCloseTo(440.0, 6);
    expect(computeNoteFrequency(108, stretchedTuning)).toBeGreaterThan(
      computeNoteFrequency(108, createDefaultTuning()),
    );
    expect(computeNoteFrequency(21, stretchedTuning)).toBeLessThan(
      computeNoteFrequency(21, createDefaultTuning()),
    );

    const bentTuning: TuningParams = { ...createDefaultTuning(), pitch_bend_semitones: 2.0 };
    expect(computeNoteFrequency(69, bentTuning)).toBeCloseTo(440.0 * Math.pow(2, 2 / 12), 5);

    const sharpNote = computeNoteFrequency(60, createDefaultTuning(), {
      detune_cents: 100,
      gain_db: 0,
      hammer_hardness_offset: 0,
      impedance_offset: 0,
    });
    expect(sharpNote).toBeCloseTo(computeNoteFrequency(61, createDefaultTuning()), 4);
  });

  it("clamps extreme frequencies to [12.0, 16000.0] Hz and anchors A4 across temperaments", () => {
    const lowTuning: TuningParams = { ...createDefaultTuning(), pitch_bend_semitones: -48 };
    const highTuning: TuningParams = { ...createDefaultTuning(), pitch_bend_semitones: 48 };
    expect(computeNoteFrequency(0, lowTuning)).toBe(12.0);
    expect(computeNoteFrequency(127, highTuning)).toBe(16000.0);

    for (const temp of TEMPERAMENT_IDS) {
      const t: TuningParams = { ...createDefaultTuning(), temperament: temp };
      // Pitch class 9 (A) is normalized to ratio 1.0 in every temperament
      expect(temperamentRatio(temp, 9)).toBeCloseTo(1.0, 5);
      expect(computeNoteFrequency(69, t)).toBeCloseTo(440.0, 4);
    }

    // Zarlino Just Intonation C4->E4 major third equals 5/4
    const zarlino: TuningParams = { ...createDefaultTuning(), temperament: "Zarlino" };
    const c4 = computeNoteFrequency(60, zarlino);
    const e4 = computeNoteFrequency(64, zarlino);
    expect(e4 / c4).toBeCloseTo(5 / 4, 5);

    // Pythagorean C4->G4 fifth equals 3/2
    const pyth: TuningParams = { ...createDefaultTuning(), temperament: "Pythagorean" };
    const pc4 = computeNoteFrequency(60, pyth);
    const pg4 = computeNoteFrequency(67, pyth);
    expect(pg4 / pc4).toBeCloseTo(3 / 2, 5);
  });
});

describe("Velocity Clamping & Curve Remapping", () => {
  it("clamps MIDI velocity bytes and UI click velocities within valid bounds", () => {
    expect(clampMidiVelocity(-10)).toBe(0);
    expect(clampMidiVelocity(0)).toBe(0);
    expect(clampMidiVelocity(0, false)).toBe(1);
    expect(clampMidiVelocity(64.4)).toBe(64);
    expect(clampMidiVelocity(200)).toBe(127);
    expect(clampMidiVelocity(Number.NaN, false)).toBe(1);

    // VirtualKeyboard click velocity clamps relY in [0.15..1.0] -> [42..124]
    expect(computeClickVelocity(-0.5)).toBe(42);
    expect(computeClickVelocity(0.15)).toBe(42);
    expect(computeClickVelocity(0.5)).toBe(76);
    expect(computeClickVelocity(1.0)).toBe(124);
    expect(computeClickVelocity(2.0)).toBe(124);
  });

  it("remaps piecewise-linear velocity curves and preserves NoteOff (0)", () => {
    const linearCurve: [number, number][] = [
      [0.0, 0.0],
      [1.0, 1.0],
    ];
    expect(applyVelocityCurve(0, linearCurve)).toBe(0);
    expect(applyVelocityCurve(64, linearCurve)).toBe(64);
    expect(applyVelocityCurve(127, linearCurve)).toBe(127);
    expect(applyVelocityCurve(95, [])).toBe(95);

    // Out-of-bounds curve Y values clamp into [1..127]
    const extremeCurve: [number, number][] = [
      [0.0, -0.5],
      [1.0, 1.8],
    ];
    expect(applyVelocityCurve(1, extremeCurve)).toBeGreaterThanOrEqual(1);
    expect(applyVelocityCurve(127, extremeCurve)).toBe(127);

    // Soft curve lowers mid velocity compared to raw input
    const softCurve: [number, number][] = [
      [0.0, 0.0],
      [0.5, 0.25],
      [1.0, 1.0],
    ];
    expect(applyVelocityCurve(64, softCurve)).toBeLessThan(40);
  });
});

describe("Keyboard MIDI Note Ranges", () => {
  it("verifies key counts, white/black splits, and endpoint labels for 61/88/105 ranges", () => {
    for (const mode of KEY_RANGE_MODES) {
      const spec = KEYBOARD_RANGES[mode];
      let total = 0;
      let whites = 0;
      let blacks = 0;

      for (let midi = spec.min; midi <= spec.max; midi++) {
        total++;
        if (isBlackMidiKey(midi)) {
          blacks++;
        } else {
          whites++;
        }
      }

      expect(total).toBe(spec.totalKeys);
      expect(whites).toBe(spec.whiteKeys);
      expect(blacks).toBe(spec.blackKeys);
      expect(midiToNoteLabel(spec.min)).toBe(spec.minLabel);
      expect(midiToNoteLabel(spec.max)).toBe(spec.maxLabel);
    }

    expect(midiToNoteLabel(60)).toBe("C4");
    expect(midiToNoteLabel(69)).toBe("A4");
  });

  it("applies transpose and Compact61 clamping correctly", () => {
    expect(applyTransposeAndRange(60, 12, "Full88")).toBe(72);
    expect(applyTransposeAndRange(10, -12, "Full88")).toBeNull();
    expect(applyTransposeAndRange(120, 12, "Extended105")).toBeNull();

    expect(applyTransposeAndRange(21, 0, "Compact61")).toBe(36);
    expect(applyTransposeAndRange(108, 0, "Compact61")).toBe(96);
    expect(applyTransposeAndRange(60, -30, "Compact61")).toBe(36);
  });
});

describe("7-Pedal Diatonic Harp Pitch Mapping Math", () => {
  it("maps natural C-D-E-F-G-A-B notes across -1, 0, +1 pedal states and gates black keys", () => {
    const cOctave4 = [60, 62, 64, 65, 67, 69, 71]; // C4, D4, E4, F4, G4, A4, B4

    // Natural [0,0,0,0,0,0,0]: identity mapping for all 7 white keys
    expect(cOctave4.map((n) => mapDiatonicHarpPitch(n, [0, 0, 0, 0, 0, 0, 0]))).toEqual(cOctave4);

    // All flat [-1,-1,-1,-1,-1,-1,-1] (Cb major): -1 semitone per string
    expect(
      cOctave4.map((n) => mapDiatonicHarpPitch(n, [-1, -1, -1, -1, -1, -1, -1])),
    ).toEqual([59, 61, 63, 64, 66, 68, 70]);

    // All sharp [+1,+1,+1,+1,+1,+1,+1] (C# major): +1 semitone per string
    expect(
      cOctave4.map((n) => mapDiatonicHarpPitch(n, [1, 1, 1, 1, 1, 1, 1])),
    ).toEqual([61, 63, 65, 66, 68, 70, 72]);

    // G major [C=0, D=0, E=0, F=+1, G=0, A=0, B=0] raises only F strings (F4 65 -> F#4 66)
    const gMajor = [0, 0, 0, 1, 0, 0, 0];
    expect(mapDiatonicHarpPitch(65, gMajor)).toBe(66);
    expect(mapDiatonicHarpPitch(53, gMajor)).toBe(54); // F3 -> F#3
    expect(mapDiatonicHarpPitch(60, gMajor)).toBe(60); // C4 unchanged

    // Enharmonic equivalence: C# (C=+1) == Db (D=-1) == MIDI 61
    expect(mapDiatonicHarpPitch(60, [1, 0, 0, 0, 0, 0, 0])).toBe(
      mapDiatonicHarpPitch(62, [0, -1, 0, 0, 0, 0, 0]),
    );

    // Non-diatonic chromatic black keys return null in diatonic harp mapping
    for (const blackKey of [61, 63, 66, 68, 70]) {
      expect(mapDiatonicHarpPitch(blackKey, [0, 0, 0, 0, 0, 0, 0])).toBeNull();
    }
  });

  it("clamps out-of-range pedal values, guards MIDI [0..127], and shifts frequency by 1 semitone", () => {
    // Out-of-range pedal offsets clamp to [-1, +1]
    expect(mapDiatonicHarpPitch(60, [-4, 0, 0, 0, 0, 0, 5])).toBe(59);
    expect(mapDiatonicHarpPitch(71, [-4, 0, 0, 0, 0, 0, 5])).toBe(72);

    // MIDI lower/upper boundary guard (C-1 = MIDI 0 flattened -> out of range -> null)
    expect(mapDiatonicHarpPitch(0, [-1, 0, 0, 0, 0, 0, 0])).toBeNull();
    expect(mapDiatonicHarpPitch(0, [1, 0, 0, 0, 0, 0, 0])).toBe(1);

    // Frequency ratio checks for A4 under flat / natural / sharp
    const tuning = createDefaultTuning();
    const aFlatMidi = mapDiatonicHarpPitch(69, [0, 0, 0, 0, 0, -1, 0])!;
    const aNatMidi = mapDiatonicHarpPitch(69, [0, 0, 0, 0, 0, 0, 0])!;
    const aSharpMidi = mapDiatonicHarpPitch(69, [0, 0, 0, 0, 0, 1, 0])!;

    expect(computeNoteFrequency(aFlatMidi, tuning)).toBeCloseTo(440 * Math.pow(2, -1 / 12), 4);
    expect(computeNoteFrequency(aNatMidi, tuning)).toBeCloseTo(440.0, 6);
    expect(computeNoteFrequency(aSharpMidi, tuning)).toBeCloseTo(440 * Math.pow(2, 1 / 12), 4);
  });
});

describe("Mallet Bounce Geometric Decay Formula", () => {
  it("returns no bounces when disabled, max_bounces is 0, or velocity is 0", () => {
    const disabled = createDefaultBounceParams();
    expect(computeBounceSequence(100, disabled)).toEqual([]);

    expect(computeBounceSequence(0, { ...disabled, enabled: true })).toEqual([]);
    expect(computeBounceSequence(100, { ...disabled, enabled: true, max_bounces: 0 })).toEqual([]);
  });

  it("computes geometric delay and velocity decay matching BounceParams closed-form series", () => {
    const bounce: BounceParams = {
      ...createDefaultBounceParams(),
      enabled: true,
      delay_ms: 120.0,
      velocity_sensitivity: 0.0,
      delay_loss: 0.15,
      velocity_loss: 0.35,
      max_bounces: 6,
    };

    const initialVel = 100;
    const hits = computeBounceSequence(initialVel, bounce);
    expect(hits).toHaveLength(6);

    for (let i = 0; i < hits.length; i++) {
      const k = i + 1;
      expect(hits[i].stepDelayMs).toBeCloseTo(120.0 * Math.pow(0.85, k - 1), 6);
      expect(hits[i].velocity).toBeCloseTo(initialVel * Math.pow(0.65, k), 6);
      if (i > 0) {
        expect(hits[i].stepDelayMs / hits[i - 1].stepDelayMs).toBeCloseTo(0.85, 6);
        expect(hits[i].velocity / hits[i - 1].velocity).toBeCloseTo(0.65, 6);
      }
    }

    // Closed-form finite geometric series sum: d0 * (1 - r^n) / (1 - r)
    const expectedTotalTime = 120.0 * ((1 - Math.pow(0.85, 6)) / 0.15);
    expect(hits[hits.length - 1].cumulativeTimeMs).toBeCloseTo(expectedTotalTime, 5);
  });

  it("handles velocity sensitivity, zero-loss edge cases, and sub-threshold termination", () => {
    const sensitive: BounceParams = {
      ...createDefaultBounceParams(),
      enabled: true,
      delay_ms: 100.0,
      velocity_sensitivity: 0.5,
      delay_loss: 0.0,
      velocity_loss: 0.0,
      max_bounces: 4,
    };

    // High velocity increases initial bounce interval vs low velocity when velocity_sensitivity > 0
    const hardHits = computeBounceSequence(127, sensitive);
    const softHits = computeBounceSequence(1, sensitive);
    expect(hardHits).toHaveLength(4);
    expect(softHits).toHaveLength(4);
    expect(hardHits[0].stepDelayMs).toBeGreaterThan(softHits[0].stepDelayMs);

    // Zero loss preserves constant delay and velocity
    expect(hardHits[3].stepDelayMs).toBeCloseTo(hardHits[0].stepDelayMs, 6);
    expect(hardHits[3].velocity).toBeCloseTo(127, 6);

    // Steep velocity_loss terminates early once bounced velocity drops below 1.0
    const steepDecay: BounceParams = {
      ...createDefaultBounceParams(),
      enabled: true,
      velocity_loss: 0.8,
      max_bounces: 10,
    };
    const shortHits = computeBounceSequence(10, steepDecay);
    // k=1 -> 2.0 (>=1), k=2 -> 0.4 (<1, stops)
    expect(shortHits).toHaveLength(1);
    expect(shortHits[0].velocity).toBeCloseTo(2.0, 6);
  });
});
