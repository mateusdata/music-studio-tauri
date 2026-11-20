import { useCallback, useEffect, useMemo, useState } from "react";
import { Volume2, Disc, Sliders, Radio, Activity, Cable, Play } from "lucide-react";
import { engineClient } from "./services/engineClient";
import { ParameterSlider } from "./components/ParameterSlider";
import { VirtualKeyboard } from "./components/VirtualKeyboard";
import type {
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiPortInfo,
  TemperamentId,
} from "./types/engine";

type PanelTab = "tuning" | "voicing" | "design" | "stage" | "effects" | "midi";

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

export function App() {
  const [snapshot, setSnapshot] = useState<EngineSnapshot | null>(null);
  const [telemetry, setTelemetry] = useState<EngineTelemetry | null>(null);
  const [midiPorts, setMidiPorts] = useState<MidiPortInfo[]>([]);
  const [activeTab, setActiveTab] = useState<PanelTab>("voicing");
  const [statusMsg, setStatusMsg] = useState<string>("Ready");
  const [pedals, setPedals] = useState({
    sustain: false,
    soft: false,
    sostenuto: false,
    harmonic: false,
  });

  useEffect(() => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Browser preview mode (run via Tauri dev for native audio)");
      return;
    }
    engineClient.getSnapshot().then(setSnapshot).catch((err) => setStatusMsg(String(err)));
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

  const handleInstrumentChange = async (id: string) => {
    if (!engineClient.isTauri()) return;
    const next = await engineClient.selectInstrument(id);
    setSnapshot(next);
    setStatusMsg(`Loaded instrument: ${id}`);
  };

  const handlePresetChange = async (id: string) => {
    if (!engineClient.isTauri()) return;
    const next = await engineClient.selectPreset(id);
    setSnapshot(next);
    setStatusMsg(`Loaded preset: ${id}`);
  };

  const handleMorphChange = async (t: number) => {
    if (!snapshot || !engineClient.isTauri()) return;
    const bId = snapshot.preset_b_id ?? snapshot.presets[1]?.id ?? snapshot.active_preset_id;
    const nextParams = await engineClient.morphPresets(snapshot.active_preset_id, bId, t);
    setSnapshot((prev) => (prev ? { ...prev, params: nextParams } : prev));
  };

  const togglePedal = (name: keyof typeof pedals) => {
    const nextVal = !pedals[name];
    setPedals((p) => ({ ...p, [name]: nextVal }));
    if (engineClient.isTauri()) {
      engineClient.setPedal(name, nextVal ? 1.0 : 0.0).catch(() => {});
    }
  };

  const handleExportWav = async () => {
    if (!engineClient.isTauri()) return;
    const path = await engineClient.exportDemoWav();
    setStatusMsg(`Exported WAV: ${path}`);
  };

  const filteredPresets = useMemo(() => {
    if (!snapshot) return [];
    const byInst = snapshot.presets.filter(
      (p) => p.instrument_id === snapshot.active_instrument_id,
    );
    return byInst.length > 0 ? byInst : snapshot.presets;
  }, [snapshot]);

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

          <button
            type="button"
            className="btn btn-primary"
            onClick={() => engineClient.isTauri() && engineClient.playTestChord()}
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
                  label="Octave Stretching"
                  value={params.tuning.octave_stretching}
                  min={0.5}
                  max={1.6}
                  step={0.01}
                  onChange={(v) => updateParams((d) => { d.tuning.octave_stretching = v; })}
                />
              </section>
            )}

            {activeTab === "design" && (
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
              </section>
            )}

            {activeTab === "stage" && (
              <section className="panel-card">
                <div className="panel-title">
                  <span>Acoustic Stage & Binaural</span>
                  <Radio size={14} />
                </div>
                <ParameterSlider
                  label="Lid Position"
                  value={params.output.lid_position}
                  min={0}
                  max={1}
                  onChange={(v) => updateParams((d) => { d.output.lid_position = v; })}
                />
                <ParameterSlider
                  label="Stereo Width"
                  value={params.output.stereo_width}
                  min={0}
                  max={1}
                  onChange={(v) => updateParams((d) => { d.output.stereo_width = v; })}
                />
                <label className="control-label" style={{ cursor: "pointer" }}>
                  <span>Binaural Head Mode</span>
                  <input
                    type="checkbox"
                    checked={params.output.binaural_mode}
                    onChange={(e) =>
                      updateParams((d) => {
                        d.output.binaural_mode = e.target.checked;
                      })
                    }
                  />
                </label>
              </section>
            )}

            {activeTab === "effects" && (
              <>
                <section className="panel-card">
                  <div className="panel-title">Reverb & Limiter</div>
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
                  <div className="panel-title">3-Band Parametric EQ</div>
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
              </>
            )}

            {activeTab === "midi" && (
              <section className="panel-card">
                <div className="panel-title">
                  <span>MIDI Ports & Action</span>
                  <Cable size={14} />
                </div>
                <label className="control-row">
                  <span className="control-label">MIDI Input Device</span>
                  <select
                    className="studio-select"
                    value={snapshot?.midi_config.selected_port ?? ""}
                    onChange={(e) => {
                      if (engineClient.isTauri() && e.target.value) {
                        engineClient.connectMidiDevice(e.target.value).catch(() => {});
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
              </section>
            )}
          </div>
        )}
      </main>

      <footer className="keyboard-dock">
        <div className="pedals-bar">
          <div className="pedal-group">
            {(
              [
                ["soft", "Una Corda (Soft)"],
                ["sostenuto", "Sostenuto"],
                ["harmonic", "Harmonic"],
                ["sustain", "Sustain (Damper)"],
              ] as [keyof typeof pedals, string][]
            ).map(([id, label]) => (
              <button
                key={id}
                type="button"
                className={`pedal-btn ${pedals[id] ? "active" : ""}`}
                onClick={() => togglePedal(id)}
              >
                {label}
              </button>
            ))}
          </div>
          <div className="status-banner">
            {statusMsg} • Keys: Z-M (C3-B3) / Q-P (C4-E5)
          </div>
        </div>

        <VirtualKeyboard
          activeNotes={telemetry?.active_notes ?? []}
          extendedRange={params?.action.keyboard_range_extended ?? false}
          onNoteOn={(note, vel) => engineClient.isTauri() && engineClient.noteOn(note, vel)}
          onNoteOff={(note) => engineClient.isTauri() && engineClient.noteOff(note)}
        />
      </footer>
    </div>
  );
}

export default App;
