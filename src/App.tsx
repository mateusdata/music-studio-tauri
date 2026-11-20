import { useCallback, useEffect, useMemo, useState } from "react";
import { Volume2, Disc, Sliders, Activity, Cable, Play } from "lucide-react";
import { engineClient } from "./services/engineClient";
import { ParameterSlider } from "./components/ParameterSlider";
import { VirtualKeyboard } from "./components/VirtualKeyboard";
import { VelocityCurveEditor } from "./components/VelocityCurveEditor";
import { MicStageEditor } from "./components/MicStageEditor";
import { NoteEditPanel } from "./components/NoteEditPanel";
import { SavePresetPopover } from "./components/SavePresetPopover";
import {
  BounceAndNoteFxPanel,
  DEFAULT_BOUNCE_PARAMS,
  DEFAULT_NOTE_FX_PARAMS,
  DEFAULT_PLAY_MODE_PARAMS,
} from "./components/BounceAndNoteFxPanel";
import { MidiTransportBar } from "./components/MidiTransportBar";
import type {
  EffectSlot,
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiConfig,
  MidiPortInfo,
  PresetData,
  TemperamentId,
} from "./types/engine";

type PanelTab =
  | "voicing"
  | "tuning"
  | "design"
  | "stage"
  | "effects"
  | "midi"
  | "noteEdit";

type PedalId =
  | "soft"
  | "sostenuto"
  | "harmonic"
  | "sustain"
  | "celeste"
  | "buff_stop"
  | "rattle";

const TEMPERAMENTS: { id: TemperamentId; label: string }[] = [
  { id: "Equal", label: "Equal (12-TET)" },
  { id: "Pythagorean", label: "Pythagorean" },
  { id: "Zarlino", label: "Zarlino Just" },
  { id: "MeanTone", label: "1/4 Comma Meantone" },
  { id: "WellTempered", label: "Well-Tempered" },
  { id: "WerckmeisterIII", label: "Werckmeister III" },
  { id: "KirnbergerIII", label: "Kirnberger III" },
  { id: "Vallotti", label: "Vallotti" },
];

const PRIMARY_PEDALS: [PedalId, string][] = [
  ["soft", "Una Corda (Soft)"],
  ["sostenuto", "Sostenuto"],
  ["harmonic", "Harmonic"],
  ["sustain", "Sustain (Damper)"],
];

const HISTORICAL_PEDALS: [PedalId, string][] = [
  ["celeste", "Celeste"],
  ["buff_stop", "Buff Stop"],
  ["rattle", "Rattle"],
];

function createDefaultParams(): EngineParams {
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
      { freq_hz: 110.0, gain_db: 0.5, q: 0.8 },
      { freq_hz: 1100.0, gain_db: 0.0, q: 1.0 },
      { freq_hz: 5200.0, gain_db: 1.0, q: 0.9 },
    ],
    fx: [
      {
        enabled: false,
        effect_type: "Delay",
        mix: 0.25,
        param_1: 0.35,
        param_2: 0.4,
        param_3: 0.6,
        param_4: 0.5,
      },
      {
        enabled: false,
        effect_type: "Chorus",
        mix: 0.2,
        param_1: 0.35,
        param_2: 0.4,
        param_3: 0.6,
        param_4: 0.5,
      },
      {
        enabled: false,
        effect_type: "Compressor",
        mix: 0.5,
        param_1: 0.35,
        param_2: 0.4,
        param_3: 0.6,
        param_4: 0.5,
      },
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
    bounce: { ...DEFAULT_BOUNCE_PARAMS },
    note_fx: { ...DEFAULT_NOTE_FX_PARAMS },
    play_mode: {
      ...DEFAULT_PLAY_MODE_PARAMS,
      harp_pedals: [...DEFAULT_PLAY_MODE_PARAMS.harp_pedals],
    },
  };
}

function createPreviewSnapshot(): EngineSnapshot {
  const defaultParams = createDefaultParams();
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
    },
    instruments: [
      {
        id: "grand-ny-d",
        name: "Grand Concert NY D",
        category: "GrandPiano",
        exciter: "FeltHammer",
        description: "9-foot New York concert grand preview model.",
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
        description: "Balanced studio voicing preview.",
        params: defaultParams,
        is_factory: true,
      },
      {
        id: "grand-ny-d-binaural",
        name: "Grand Concert NY D Binaural Player",
        instrument_id: "grand-ny-d",
        category: "Binaural",
        description: "Player-perspective 3D binaural stage.",
        params: {
          ...defaultParams,
          output: { ...defaultParams.output, binaural_mode: true, stereo_width: 0.8 },
        },
        is_factory: true,
      },
    ],
  };
}

export function App() {
  const [snapshot, setSnapshot] = useState<EngineSnapshot | null>(() =>
    engineClient.isTauri() ? null : createPreviewSnapshot(),
  );
  const [telemetry, setTelemetry] = useState<EngineTelemetry | null>(null);
  const [midiPorts, setMidiPorts] = useState<MidiPortInfo[]>([]);
  const [activeTab, setActiveTab] = useState<PanelTab>("voicing");
  const [statusMsg, setStatusMsg] = useState<string>(() =>
    engineClient.isTauri()
      ? "Ready"
      : "Browser preview mode (run via Tauri dev for native audio)",
  );
  const [pedals, setPedals] = useState<Record<PedalId, boolean>>({
    sustain: false,
    soft: false,
    sostenuto: false,
    harmonic: false,
    celeste: false,
    buff_stop: false,
    rattle: false,
  });

  useEffect(() => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Browser preview mode (run via Tauri dev for native audio)");
      return;
    }
    engineClient
      .getSnapshot()
      .then(setSnapshot)
      .catch((err) => setStatusMsg(String(err)));
    engineClient.listMidiDevices().then(setMidiPorts).catch(() => {});

    const timer = setInterval(() => {
      engineClient.getTelemetry().then(setTelemetry).catch(() => {});
    }, 90);
    return () => clearInterval(timer);
  }, []);

  const updateParams = useCallback(
    (mutator: (draft: EngineParams) => void) => {
      setSnapshot((prev) => {
        if (!prev) return prev;
        const cloned: EngineParams = JSON.parse(JSON.stringify(prev.params));
        mutator(cloned);
        if (engineClient.isTauri()) {
          engineClient.updateParams(cloned).catch(() => {});
        }
        return { ...prev, params: cloned };
      });
    },
    [],
  );

  const handleUpdateMidiConfig = useCallback((nextConfig: MidiConfig) => {
    setSnapshot((prev) => (prev ? { ...prev, midi_config: nextConfig } : prev));
    if (engineClient.isTauri()) {
      engineClient.updateMidiConfig(nextConfig).catch((err) => setStatusMsg(String(err)));
    }
  }, []);

  const handleInstrumentChange = async (id: string) => {
    if (!engineClient.isTauri()) {
      setSnapshot((prev) => {
        if (!prev) return prev;
        const inst = prev.instruments.find((i) => i.id === id);
        if (!inst) return prev;
        return {
          ...prev,
          active_instrument_id: inst.id,
          params: JSON.parse(JSON.stringify(inst.default_params)),
        };
      });
      setStatusMsg(`Preview instrument: ${id}`);
      return;
    }
    try {
      const next = await engineClient.selectInstrument(id);
      setSnapshot(next);
      setStatusMsg(`Loaded instrument: ${id}`);
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const handlePresetChange = async (id: string) => {
    if (!engineClient.isTauri()) {
      setSnapshot((prev) => {
        if (!prev) return prev;
        const found = prev.presets.find((p) => p.id === id);
        if (!found) return prev;
        return {
          ...prev,
          active_preset_id: found.id,
          params: JSON.parse(JSON.stringify(found.params)),
        };
      });
      setStatusMsg(`Preview preset: ${id}`);
      return;
    }
    try {
      const next = await engineClient.selectPreset(id);
      setSnapshot(next);
      setStatusMsg(`Loaded preset: ${id}`);
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const handleSaveUserPreset = async (name: string, category: string) => {
    if (!engineClient.isTauri()) {
      setSnapshot((prev) => {
        if (!prev) return prev;
        const slug = name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/^-|-$/g, "");
        const created: PresetData = {
          id: `user-${slug || Date.now()}`,
          name,
          instrument_id: prev.active_instrument_id,
          category,
          description: "Preview user preset",
          params: JSON.parse(JSON.stringify(prev.params)),
          is_factory: false,
        };
        const remaining = prev.presets.filter((p) => p.id !== created.id);
        return {
          ...prev,
          active_preset_id: created.id,
          presets: [...remaining, created],
        };
      });
      setStatusMsg(`Saved preview preset: ${name}`);
      return;
    }
    const saved = await engineClient.saveUserPreset(name, category);
    setSnapshot((prev) => {
      if (!prev) return prev;
      const remaining = prev.presets.filter((p) => p.id !== saved.id);
      return {
        ...prev,
        active_preset_id: saved.id,
        presets: [...remaining, saved],
      };
    });
    setStatusMsg(`Saved preset: ${saved.name} (${saved.category})`);
  };

  const handleMorphChange = async (t: number) => {
    if (!snapshot) return;
    if (!engineClient.isTauri()) {
      updateParams((d) => {
        d.morphing = t;
      });
      return;
    }
    try {
      const bId =
        snapshot.preset_b_id ?? snapshot.presets[1]?.id ?? snapshot.active_preset_id;
      const nextParams = await engineClient.morphPresets(
        snapshot.active_preset_id,
        bId,
        t,
      );
      setSnapshot((prev) => (prev ? { ...prev, params: nextParams } : prev));
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const togglePedal = (name: PedalId) => {
    const nextVal = !pedals[name];
    setPedals((p) => ({ ...p, [name]: nextVal }));
    if (engineClient.isTauri()) {
      engineClient.setPedal(name, nextVal ? 1.0 : 0.0).catch(() => {});
    }
  };

  const handleExportWav = async () => {
    if (!engineClient.isTauri()) return;
    try {
      const path = await engineClient.exportDemoWav();
      setStatusMsg(`Exported WAV: ${path}`);
    } catch (err) {
      setStatusMsg(String(err));
    }
  };

  const handleAuditionNote = useCallback((note: number) => {
    if (!engineClient.isTauri()) return;
    engineClient.noteOn(note, 84).catch(() => {});
    setTimeout(() => {
      engineClient.noteOff(note).catch(() => {});
    }, 350);
  }, []);

  const filteredPresets = useMemo(() => {
    if (!snapshot) return [];
    const byInst = snapshot.presets.filter(
      (p) => p.instrument_id === snapshot.active_instrument_id,
    );
    return byInst.length > 0 ? byInst : snapshot.presets;
  }, [snapshot]);

  const activePreset = useMemo(
    () => snapshot?.presets.find((p) => p.id === snapshot.active_preset_id),
    [snapshot],
  );

  const params = snapshot?.params;
  const peakPct = Math.min(
    100,
    Math.round(Math.max(telemetry?.peak_l ?? 0, telemetry?.peak_r ?? 0) * 100),
  );

  return (
    <div className="studio-shell">
      <header className="topbar">
        <div className="brand-block">
          <div className="brand-badge">PT</div>
          <div>
            <div className="brand-title">Piano Tect Tauri</div>
            <div className="brand-sub">
              {telemetry?.audio_backend ?? "Physical Modeling Studio"}
            </div>
          </div>
        </div>

        <div className="selector-row">
          <select
            aria-label="Instrument Selector"
            className="studio-select"
            value={snapshot?.active_instrument_id ?? ""}
            onChange={(e) => handleInstrumentChange(e.target.value)}
          >
            {snapshot?.instruments.map((inst) => (
              <option key={inst.id} value={inst.id}>
                {inst.name}
              </option>
            ))}
          </select>

          <select
            aria-label="Preset Selector"
            className="studio-select"
            value={snapshot?.active_preset_id ?? ""}
            onChange={(e) => handlePresetChange(e.target.value)}
          >
            {filteredPresets.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>

          <SavePresetPopover
            defaultName={activePreset ? `${activePreset.name} Custom` : ""}
            onSavePreset={handleSaveUserPreset}
          />

          <button
            type="button"
            className="btn btn-primary"
            onClick={() => {
              if (engineClient.isTauri()) {
                engineClient.playTestChord().catch(() => {});
              }
            }}
          >
            <Play size={13} /> Test Chord
          </button>
          <button type="button" className="btn" onClick={handleExportWav}>
            <Disc size={13} /> Export WAV
          </button>
        </div>

        <div className="telemetry-strip">
          <div className="meter-pill">
            <Activity size={13} color="#e29d38" />
            <span>
              {telemetry?.active_voices ?? 0}/{telemetry?.max_polyphony ?? 128} voices
            </span>
          </div>
          <div className="meter-pill">
            <span>CPU {(telemetry?.cpu_load_pct ?? 0).toFixed(1)}%</span>
            <span>{(telemetry?.latency_ms ?? 5.3).toFixed(1)} ms</span>
          </div>
          <div className="meter-pill">
            <Volume2 size={13} />
            <div className="vu-bar-wrap">
              <div className="vu-bar-fill" style={{ width: `${peakPct}%` }} />
            </div>
          </div>
        </div>
      </header>

      <MidiTransportBar telemetry={telemetry} onStatusMessage={setStatusMsg} />

      <main className="workspace-body">
        <nav className="tabs-nav" aria-label="Parameter Sections">
          {(
            [
              ["voicing", "Voicing & Hammers"],
              ["tuning", "Tuning & Temperament"],
              ["design", "Acoustic Design"],
              ["stage", "Mics & Binaural Stage"],
              ["effects", "EQ, FX & Reverb"],
              ["midi", "MIDI & Action"],
              ["noteEdit", "Note Edit (PRO)"],
            ] as [PanelTab, string][]
          ).map(([id, label]) => (
            <button
              key={id}
              type="button"
              className={`tab-btn ${activeTab === id ? "active" : ""}`}
              onClick={() => setActiveTab(id)}
            >
              {label}
            </button>
          ))}
        </nav>

        {params && (
          <div className="panels-grid">
            {/* Persistent Master Card */}
            <section className="panel-card">
              <div className="panel-title">
                <span>Master & Morphing</span>
                <Sliders size={14} />
              </div>
              <ParameterSlider
                label="Master Volume"
                value={params.volume_db}
                min={-36}
                max={6}
                step={0.5}
                unit="dB"
                onChange={(v) => updateParams((d) => { d.volume_db = v; })}
              />
              <ParameterSlider
                label="Dynamics"
                value={params.dynamics_db}
                min={10}
                max={80}
                step={1}
                unit="dB"
                onChange={(v) => updateParams((d) => { d.dynamics_db = v; })}
              />
              <ParameterSlider
                label="Instrument Condition"
                value={params.condition}
                min={0}
                max={1}
                step={0.01}
                onChange={(v) => updateParams((d) => { d.condition = v; })}
              />
              <ParameterSlider
                label="A/B Preset Morph"
                value={params.morphing}
                min={0}
                max={1}
                step={0.01}
                onChange={handleMorphChange}
              />
              <ParameterSlider
                label="Velocity Offset"
                value={params.velocity_offset}
                min={-1}
                max={1}
                step={0.02}
                onChange={(v) => updateParams((d) => { d.velocity_offset = v; })}
              />
            </section>

            {activeTab === "voicing" && (
              <>
                <section className="panel-card">
                  <div className="panel-title">Hammer Felt Hardness</div>
                  <ParameterSlider
                    label="Piano (pp)"
                    value={params.voicing.hammer_hardness_piano}
                    min={0.05}
                    max={1.2}
                    onChange={(v) => updateParams((d) => { d.voicing.hammer_hardness_piano = v; })}
                  />
                  <ParameterSlider
                    label="Mezzo (mf)"
                    value={params.voicing.hammer_hardness_mezzo}
                    min={0.05}
                    max={1.2}
                    onChange={(v) => updateParams((d) => { d.voicing.hammer_hardness_mezzo = v; })}
                  />
                  <ParameterSlider
                    label="Forte (ff)"
                    value={params.voicing.hammer_hardness_forte}
                    min={0.05}
                    max={1.2}
                    onChange={(v) => updateParams((d) => { d.voicing.hammer_hardness_forte = v; })}
                  />
                  <ParameterSlider
                    label="Hammer Noise"
                    value={params.voicing.hammer_noise}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.voicing.hammer_noise = v; })}
                  />
                  <ParameterSlider
                    label="Strike Point"
                    value={params.voicing.strike_point}
                    min={0.04}
                    max={0.45}
                    onChange={(v) => updateParams((d) => { d.voicing.strike_point = v; })}
                  />
                </section>

                <section className="panel-card">
                  <div className="panel-title">8-Partial Spectrum Profile</div>
                  <div className="spectrum-bars">
                    {params.voicing.spectrum_profile.map((val, idx) => (
                      <div key={idx} className="spectrum-col">
                        <input
                          type="range"
                          min={0}
                          max={1.5}
                          step={0.02}
                          value={val}
                          aria-label={`Overtone ${idx + 1}`}
                          onChange={(e) => {
                            const nextVal = parseFloat(e.target.value);
                            updateParams((d) => {
                              d.voicing.spectrum_profile[idx] = nextVal;
                            });
                          }}
                        />
                        <span style={{ fontSize: 10, color: "#98a4b8" }}>H{idx + 1}</span>
                      </div>
                    ))}
                  </div>
                  <ParameterSlider
                    label="Hammer Tone"
                    value={params.voicing.hammer_tone}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.voicing.hammer_tone = v; })}
                  />
                  <ParameterSlider
                    label="Soft Pedal Level"
                    value={params.voicing.soft_level}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.voicing.soft_level = v; })}
                  />
                </section>
              </>
            )}

            {activeTab === "tuning" && (
              <section className="panel-card">
                <div className="panel-title">Tuning & Temperament</div>
                <label className="control-row">
                  <span className="control-label">Temperament</span>
                  <select
                    className="studio-select"
                    value={params.tuning.temperament}
                    onChange={(e) =>
                      updateParams((d) => {
                        d.tuning.temperament = e.target.value as TemperamentId;
                      })
                    }
                  >
                    {TEMPERAMENTS.map((t) => (
                      <option key={t.id} value={t.id}>
                        {t.label}
                      </option>
                    ))}
                  </select>
                </label>
                <ParameterSlider
                  label="Diapason (A4)"
                  value={params.tuning.diapason_hz}
                  min={415}
                  max={466}
                  step={0.5}
                  unit="Hz"
                  onChange={(v) => updateParams((d) => { d.tuning.diapason_hz = v; })}
                />
                <ParameterSlider
                  label="Unison Width"
                  value={params.tuning.unison_width_cents}
                  min={0}
                  max={25}
                  step={0.1}
                  unit="ct"
                  onChange={(v) => updateParams((d) => { d.tuning.unison_width_cents = v; })}
                />
                <ParameterSlider
                  label="Unison Balance"
                  value={params.tuning.unison_balance}
                  min={0}
                  max={1}
                  step={0.01}
                  onChange={(v) => updateParams((d) => { d.tuning.unison_balance = v; })}
                />
                <ParameterSlider
                  label="Octave Stretching"
                  value={params.tuning.octave_stretching}
                  min={0.5}
                  max={1.6}
                  step={0.01}
                  onChange={(v) => updateParams((d) => { d.tuning.octave_stretching = v; })}
                />
                <ParameterSlider
                  label="Direct Sound Duration"
                  value={params.tuning.direct_sound_duration}
                  min={0.1}
                  max={1.5}
                  step={0.01}
                  onChange={(v) => updateParams((d) => { d.tuning.direct_sound_duration = v; })}
                />
              </section>
            )}

            {activeTab === "design" && (
              <>
                <section className="panel-card">
                  <div className="panel-title">Soundboard & Strings</div>
                  <ParameterSlider
                    label="String Length"
                    value={params.design.string_length_m}
                    min={0.65}
                    max={3.5}
                    step={0.05}
                    unit="m"
                    onChange={(v) => updateParams((d) => { d.design.string_length_m = v; })}
                  />
                  <ParameterSlider
                    label="Soundboard Impedance"
                    value={params.design.soundboard_impedance}
                    min={0.1}
                    max={1.0}
                    onChange={(v) => updateParams((d) => { d.design.soundboard_impedance = v; })}
                  />
                  <ParameterSlider
                    label="Impedance Cutoff"
                    value={params.design.impedance_cutoff}
                    min={0.1}
                    max={1.0}
                    onChange={(v) => updateParams((d) => { d.design.impedance_cutoff = v; })}
                  />
                  <ParameterSlider
                    label="Impedance Slope"
                    value={params.design.impedance_slope}
                    min={0.1}
                    max={1.0}
                    onChange={(v) => updateParams((d) => { d.design.impedance_slope = v; })}
                  />
                </section>

                <section className="panel-card">
                  <div className="panel-title">Resonance & Energy</div>
                  <ParameterSlider
                    label="Sympathetic Resonance"
                    value={params.design.sympathetic_resonance}
                    min={0}
                    max={1.2}
                    onChange={(v) => updateParams((d) => { d.design.sympathetic_resonance = v; })}
                  />
                  <ParameterSlider
                    label="Duplex Scale Resonance"
                    value={params.design.duplex_scale_resonance}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.design.duplex_scale_resonance = v; })}
                  />
                  <ParameterSlider
                    label="Blooming Energy"
                    value={params.design.blooming_energy}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.design.blooming_energy = v; })}
                  />
                  <ParameterSlider
                    label="Quadratic Effect"
                    value={params.design.quadratic_effect}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.design.quadratic_effect = v; })}
                  />
                </section>

                <BounceAndNoteFxPanel
                  params={params}
                  onUpdateParams={updateParams}
                />
              </>
            )}

            {activeTab === "stage" && (
              <MicStageEditor
                output={params.output}
                onChange={(nextOutput) =>
                  updateParams((d) => {
                    d.output = nextOutput;
                  })
                }
              />
            )}

            {activeTab === "effects" && (
              <>
                <section className="panel-card">
                  <div className="panel-title">
                    <span>Reverb & Limiter</span>
                    <label className="check-inline">
                      <input
                        type="checkbox"
                        checked={params.reverb.enabled}
                        onChange={(e) =>
                          updateParams((d) => {
                            d.reverb.enabled = e.target.checked;
                          })
                        }
                      />
                      <span>Reverb</span>
                    </label>
                  </div>
                  <ParameterSlider
                    label="Reverb Mix"
                    value={params.reverb.mix}
                    min={0}
                    max={0.8}
                    onChange={(v) => updateParams((d) => { d.reverb.mix = v; })}
                  />
                  <ParameterSlider
                    label="Reverb Duration"
                    value={params.reverb.duration_s}
                    min={0.3}
                    max={5.0}
                    step={0.1}
                    unit="s"
                    onChange={(v) => updateParams((d) => { d.reverb.duration_s = v; })}
                  />
                  <ParameterSlider
                    label="Reverb Tone"
                    value={params.reverb.tone}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.reverb.tone = v; })}
                  />
                  <ParameterSlider
                    label="Limiter Threshold"
                    value={params.limiter.threshold_db}
                    min={-12}
                    max={0}
                    step={0.2}
                    unit="dB"
                    onChange={(v) => updateParams((d) => { d.limiter.threshold_db = v; })}
                  />
                </section>
                <section className="panel-card">
                  <div className="panel-title">
                    <span>3-Band Parametric EQ</span>
                    <label className="check-inline">
                      <input
                        type="checkbox"
                        checked={params.eq_enabled}
                        onChange={(e) =>
                          updateParams((d) => {
                            d.eq_enabled = e.target.checked;
                          })
                        }
                      />
                      <span>Enabled</span>
                    </label>
                  </div>
                  {params.eq_bands.map((band, idx) => (
                    <ParameterSlider
                      key={idx}
                      label={`Band ${idx + 1} (${Math.round(band.freq_hz)} Hz)`}
                      value={band.gain_db}
                      min={-12}
                      max={12}
                      step={0.5}
                      unit="dB"
                      onChange={(v) =>
                        updateParams((d) => {
                          d.eq_bands[idx].gain_db = v;
                        })
                      }
                    />
                  ))}
                </section>
                <section className="panel-card">
                  <div className="panel-title">FX Chain Slots</div>
                  {params.fx.map((slot, idx) => (
                    <div key={idx} className="fx-slot-row">
                      <div className="fx-slot-head">
                        <label className="check-inline">
                          <input
                            type="checkbox"
                            checked={slot.enabled}
                            onChange={(e) =>
                              updateParams((d) => {
                                d.fx[idx].enabled = e.target.checked;
                              })
                            }
                          />
                          <span>Slot {idx + 1}</span>
                        </label>
                        <select
                          className="studio-select studio-select-sm"
                          aria-label={`FX Slot ${idx + 1} Type`}
                          value={slot.effect_type}
                          onChange={(e) =>
                            updateParams((d) => {
                              d.fx[idx].effect_type = e.target
                                .value as EffectSlot["effect_type"];
                            })
                          }
                        >
                          <option value="Delay">Delay</option>
                          <option value="Chorus">Chorus</option>
                          <option value="Tremolo">Tremolo</option>
                          <option value="Compressor">Compressor</option>
                          <option value="ParametricEq">Parametric EQ</option>
                        </select>
                      </div>
                      <ParameterSlider
                        label="Mix"
                        value={slot.mix}
                        min={0}
                        max={1}
                        step={0.01}
                        onChange={(v) =>
                          updateParams((d) => {
                            d.fx[idx].mix = v;
                          })
                        }
                      />
                    </div>
                  ))}
                </section>
              </>
            )}

            {activeTab === "midi" && snapshot && (
              <>
                <section className="panel-card">
                  <div className="panel-title">
                    <span>MIDI Ports & Action</span>
                    <Cable size={14} />
                  </div>
                  <label className="control-row">
                    <span className="control-label">MIDI Input Device</span>
                    <select
                      className="studio-select"
                      value={snapshot.midi_config.selected_port ?? ""}
                      onChange={(e) => {
                        if (engineClient.isTauri() && e.target.value) {
                          engineClient
                            .connectMidiDevice(e.target.value)
                            .then((name) => setStatusMsg(`Connected MIDI: ${name}`))
                            .catch((err) => setStatusMsg(String(err)));
                        }
                      }}
                    >
                      <option value="">-- Select MIDI Port --</option>
                      {midiPorts.map((p) => (
                        <option key={p.index} value={p.name}>
                          {p.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <div className="two-col-mini">
                    <label className="control-row">
                      <span className="control-label">Key Range</span>
                      <select
                        className="studio-select"
                        value={snapshot.midi_config.key_range_mode}
                        onChange={(e) =>
                          handleUpdateMidiConfig({
                            ...snapshot.midi_config,
                            key_range_mode: e.target.value as MidiConfig["key_range_mode"],
                          })
                        }
                      >
                        <option value="Full88">Full 88</option>
                        <option value="Compact61">Compact 61</option>
                        <option value="Extended105">Extended 105</option>
                      </select>
                    </label>
                    <ParameterSlider
                      label="Transpose"
                      value={snapshot.midi_config.transpose}
                      min={-24}
                      max={24}
                      step={1}
                      unit="st"
                      onChange={(v) =>
                        handleUpdateMidiConfig({
                          ...snapshot.midi_config,
                          transpose: Math.round(v),
                        })
                      }
                    />
                  </div>
                  <ParameterSlider
                    label="Damper Noise"
                    value={params.action.damper_noise}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.action.damper_noise = v; })}
                  />
                  <ParameterSlider
                    label="Pedal Noise"
                    value={params.action.pedal_noise}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.action.pedal_noise = v; })}
                  />
                  <ParameterSlider
                    label="Key Release Noise"
                    value={params.action.key_release_noise}
                    min={0}
                    max={1}
                    onChange={(v) => updateParams((d) => { d.action.key_release_noise = v; })}
                  />
                  <label className="check-pill">
                    <input
                      type="checkbox"
                      checked={params.action.keyboard_range_extended}
                      onChange={(e) =>
                        updateParams((d) => {
                          d.action.keyboard_range_extended = e.target.checked;
                        })
                      }
                    />
                    <span>Extended 105-Key Range</span>
                  </label>
                </section>

                <VelocityCurveEditor
                  midiConfig={snapshot.midi_config}
                  lastVelocity={telemetry?.last_midi_velocity}
                  onUpdateMidiConfig={handleUpdateMidiConfig}
                />
              </>
            )}

            {activeTab === "noteEdit" && (
              <NoteEditPanel
                noteEdit={params.note_edit ?? {}}
                diapasonHz={params.tuning.diapason_hz}
                onChangeNoteEdit={(nextNoteEdit) =>
                  updateParams((d) => {
                    d.note_edit = nextNoteEdit;
                  })
                }
                onAuditionNote={handleAuditionNote}
              />
            )}
          </div>
        )}
      </main>

      <footer className="keyboard-dock">
        <div className="pedals-bar">
          <div className="pedals-cluster">
            <div className="pedal-group" role="group" aria-label="Primary Pedals">
              {PRIMARY_PEDALS.map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={pedals[id]}
                  className={`pedal-btn ${pedals[id] ? "active" : ""}`}
                  onClick={() => togglePedal(id)}
                >
                  {label}
                </button>
              ))}
            </div>
            <div className="pedal-divider" aria-hidden="true" />
            <div
              className="pedal-group pedal-group-historical"
              role="group"
              aria-label="Historical and Special Pedals"
            >
              <span className="pedal-group-tag">Historical</span>
              {HISTORICAL_PEDALS.map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={pedals[id]}
                  className={`pedal-btn pedal-btn-special ${pedals[id] ? "active" : ""}`}
                  onClick={() => togglePedal(id)}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>
          <div className="status-banner">
            {statusMsg} • Keys: Z-M (C3-B3) / Q-P (C4-E5)
          </div>
        </div>

        <VirtualKeyboard
          activeNotes={telemetry?.active_notes ?? []}
          extendedRange={params?.action.keyboard_range_extended ?? false}
          onNoteOn={(note, vel) => {
            if (engineClient.isTauri()) {
              engineClient.noteOn(note, vel).catch(() => {});
            }
          }}
          onNoteOff={(note) => {
            if (engineClient.isTauri()) {
              engineClient.noteOff(note).catch(() => {});
            }
          }}
        />
      </footer>
    </div>
  );
}

export default App;
