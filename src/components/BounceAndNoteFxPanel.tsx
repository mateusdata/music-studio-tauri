import React from "react";
import { Sparkles, Music2, Sliders } from "lucide-react";
import type {
  BounceParams,
  EngineParams,
  NoteFxParams,
  PlayMode,
  PlayModeParams,
} from "../types/engine";
import { ParameterSlider } from "./ParameterSlider";

export const DEFAULT_BOUNCE_PARAMS: BounceParams = {
  enabled: false,
  delay_ms: 120.0,
  sync_tempo: false,
  velocity_sensitivity: 0.5,
  delay_loss: 0.15,
  velocity_loss: 0.35,
  humanization: 0.1,
  max_bounces: 6,
};

export const DEFAULT_NOTE_FX_PARAMS: NoteFxParams = {
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

export const DEFAULT_PLAY_MODE_PARAMS: PlayModeParams = {
  mode: "Standard",
  harp_pedals: [0, 0, 0, 0, 0, 0, 0],
  guitar_fret: 0,
  guitar_legato: false,
  guitar_body: 0.5,
};

const HARP_NOTE_NAMES = ["C", "D", "E", "F", "G", "A", "B"] as const;

function pedalSymbol(val: number): string {
  if (val < 0) return "♭";
  if (val > 0) return "♯";
  return "♮";
}

function pedalStateWord(val: number): string {
  if (val < 0) return "Flat (-1)";
  if (val > 0) return "Sharp (+1)";
  return "Natural (0)";
}

function nextPedalValue(val: number): number {
  if (val === -1) return 0;
  if (val === 0) return 1;
  return -1;
}

export interface BounceAndNoteFxPanelProps {
  params: EngineParams;
  onUpdateParams: (mutator: (draft: EngineParams) => void) => void;
}

export const BounceAndNoteFxPanel: React.FC<BounceAndNoteFxPanelProps> = ({
  params,
  onUpdateParams,
}) => {
  const bounce: BounceParams = {
    ...DEFAULT_BOUNCE_PARAMS,
    ...(params.bounce ?? {}),
  };
  const noteFx: NoteFxParams = {
    ...DEFAULT_NOTE_FX_PARAMS,
    ...(params.note_fx ?? {}),
  };
  const rawPlayMode = params.play_mode ?? DEFAULT_PLAY_MODE_PARAMS;
  const harpPedals =
    rawPlayMode.harp_pedals && rawPlayMode.harp_pedals.length === 7
      ? rawPlayMode.harp_pedals
      : [0, 0, 0, 0, 0, 0, 0];
  const playMode: PlayModeParams = {
    ...DEFAULT_PLAY_MODE_PARAMS,
    ...rawPlayMode,
    harp_pedals: harpPedals,
  };

  const mutateBounce = (patch: (b: BounceParams) => void) => {
    onUpdateParams((draft) => {
      draft.bounce = { ...DEFAULT_BOUNCE_PARAMS, ...(draft.bounce ?? {}) };
      patch(draft.bounce);
    });
  };

  const mutateNoteFx = (patch: (nf: NoteFxParams) => void) => {
    onUpdateParams((draft) => {
      draft.note_fx = { ...DEFAULT_NOTE_FX_PARAMS, ...(draft.note_fx ?? {}) };
      patch(draft.note_fx);
    });
  };

  const mutatePlayMode = (patch: (pm: PlayModeParams) => void) => {
    onUpdateParams((draft) => {
      const existing = draft.play_mode ?? DEFAULT_PLAY_MODE_PARAMS;
      const existingPedals =
        existing.harp_pedals && existing.harp_pedals.length === 7
          ? [...existing.harp_pedals]
          : [0, 0, 0, 0, 0, 0, 0];
      draft.play_mode = {
        ...DEFAULT_PLAY_MODE_PARAMS,
        ...existing,
        harp_pedals: existingPedals,
      };
      patch(draft.play_mode);
    });
  };

  return (
    <section
      className="panel-card panel-card-wide"
      data-testid="bounce-notefx-panel"
    >
      <div className="panel-title">
        <span>
          <Sparkles size={14} style={{ verticalAlign: "text-bottom", marginRight: 6 }} />
          Mallet Bounce, Per-Note FX & Play Mode
        </span>
        <span className="badge-subtle">
          Mode: {playMode.mode}
          {bounce.enabled ? " • Bounce ON" : ""}
        </span>
      </div>

      <div className="bounce-notefx-grid">
        {/* 1. Mallet Bounce */}
        <div className="stage-subcard" data-testid="bounce-section">
          <div className="subcard-title">
            <span>Mallet Bounce</span>
            <div className="toggles-row">
              <label className="check-inline">
                <input
                  type="checkbox"
                  aria-label="Enable Mallet Bounce"
                  checked={bounce.enabled}
                  onChange={(e) =>
                    mutateBounce((b) => {
                      b.enabled = e.target.checked;
                    })
                  }
                />
                <span>Enabled</span>
              </label>
              <label className="check-inline">
                <input
                  type="checkbox"
                  aria-label="Bounce Sync Tempo"
                  checked={bounce.sync_tempo}
                  onChange={(e) =>
                    mutateBounce((b) => {
                      b.sync_tempo = e.target.checked;
                    })
                  }
                />
                <span>Sync</span>
              </label>
            </div>
          </div>

          <div className="mic-detail-grid">
            <ParameterSlider
              label="Bounce Delay"
              value={bounce.delay_ms}
              min={20}
              max={600}
              step={1}
              unit="ms"
              onChange={(v) => mutateBounce((b) => { b.delay_ms = v; })}
            />
            <ParameterSlider
              label="Velocity Sensitivity"
              value={bounce.velocity_sensitivity}
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => mutateBounce((b) => { b.velocity_sensitivity = v; })}
            />
            <ParameterSlider
              label="Delay Loss"
              value={bounce.delay_loss}
              min={0}
              max={0.9}
              step={0.01}
              onChange={(v) => mutateBounce((b) => { b.delay_loss = v; })}
            />
            <ParameterSlider
              label="Velocity Loss"
              value={bounce.velocity_loss}
              min={0}
              max={0.95}
              step={0.01}
              onChange={(v) => mutateBounce((b) => { b.velocity_loss = v; })}
            />
            <ParameterSlider
              label="Humanization"
              value={bounce.humanization}
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => mutateBounce((b) => { b.humanization = v; })}
            />
            <ParameterSlider
              label="Max Bounces"
              value={bounce.max_bounces}
              min={1}
              max={16}
              step={1}
              onChange={(v) => mutateBounce((b) => { b.max_bounces = Math.round(v); })}
            />
          </div>
        </div>

        {/* 2. Per-Note Effects */}
        <div className="stage-subcard" data-testid="note-fx-section">
          <div className="subcard-title">
            <span>
              <Sliders size={12} style={{ verticalAlign: "text-bottom", marginRight: 5 }} />
              Per-Note Effects
            </span>
            <label className="check-inline">
              <input
                type="checkbox"
                aria-label="LFO Phase Locked"
                checked={noteFx.lfo_phase_locked}
                onChange={(e) =>
                  mutateNoteFx((nf) => {
                    nf.lfo_phase_locked = e.target.checked;
                  })
                }
              />
              <span>Phase Lock</span>
            </label>
          </div>

          <div className="mic-detail-grid">
            <ParameterSlider
              label="Vibrato Depth"
              value={noteFx.vibrato_cents}
              min={0}
              max={100}
              step={1}
              unit="ct"
              onChange={(v) => mutateNoteFx((nf) => { nf.vibrato_cents = v; })}
            />
            <ParameterSlider
              label="LFO Rate"
              value={noteFx.lfo_rate_hz}
              min={0.2}
              max={20}
              step={0.1}
              unit="Hz"
              onChange={(v) => mutateNoteFx((nf) => { nf.lfo_rate_hz = v; })}
            />
            <ParameterSlider
              label="Tremolo Depth"
              value={noteFx.tremolo_depth_db}
              min={0}
              max={24}
              step={0.5}
              unit="dB"
              onChange={(v) => mutateNoteFx((nf) => { nf.tremolo_depth_db = v; })}
            />
            <ParameterSlider
              label="Onset Duration"
              value={noteFx.onset_duration_ms}
              min={0}
              max={1000}
              step={5}
              unit="ms"
              onChange={(v) => mutateNoteFx((nf) => { nf.onset_duration_ms = v; })}
            />
            <ParameterSlider
              label="Attack Envelope"
              value={noteFx.attack_envelope}
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => mutateNoteFx((nf) => { nf.attack_envelope = v; })}
            />
            <ParameterSlider
              label="Smear"
              value={noteFx.smear}
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => mutateNoteFx((nf) => { nf.smear = v; })}
            />
          </div>
        </div>

        {/* 3. Play Mode (Standard, DiatonicHarp, Guitar) */}
        <div className="stage-subcard play-mode-card" data-testid="play-mode-section">
          <div className="subcard-title">
            <span>
              <Music2 size={12} style={{ verticalAlign: "text-bottom", marginRight: 5 }} />
              Play Mode
            </span>
            <div className="mode-segmented" role="group" aria-label="Play Mode Selector">
              {(["Standard", "DiatonicHarp", "Guitar"] as PlayMode[]).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  className={`chip-btn ${playMode.mode === mode ? "active" : ""}`}
                  aria-pressed={playMode.mode === mode}
                  onClick={() => mutatePlayMode((pm) => { pm.mode = mode; })}
                >
                  {mode}
                </button>
              ))}
            </div>
          </div>

          {playMode.mode === "Standard" && (
            <div className="play-mode-hint">
              Standard 12-tone keyboard articulation active. Select DiatonicHarp for 7-pedal tuning or Guitar for fretboard/legato modeling.
            </div>
          )}

          {playMode.mode === "DiatonicHarp" && (
            <div className="harp-controls-wrap" data-testid="harp-pedals-panel">
              <div className="harp-header-row">
                <span className="control-label">
                  Diatonic Harp Pedals (click to cycle ♭ -1 / ♮ 0 / ♯ +1)
                </span>
                <button
                  type="button"
                  className="btn btn-compact"
                  onClick={() =>
                    mutatePlayMode((pm) => {
                      pm.harp_pedals = [0, 0, 0, 0, 0, 0, 0];
                    })
                  }
                >
                  Reset Pedals
                </button>
              </div>
              <div className="harp-pedals-row" role="group" aria-label="Harp Pedals">
                {HARP_NOTE_NAMES.map((noteName, idx) => {
                  const val = playMode.harp_pedals[idx] ?? 0;
                  const stateCls =
                    val < 0 ? "flat" : val > 0 ? "sharp" : "natural";
                  return (
                    <button
                      key={noteName}
                      type="button"
                      className={`harp-pedal-btn ${stateCls}`}
                      data-pedal-value={val}
                      aria-label={`Harp Pedal ${noteName}: ${pedalStateWord(val)}`}
                      onClick={() =>
                        mutatePlayMode((pm) => {
                          pm.harp_pedals[idx] = nextPedalValue(val);
                        })
                      }
                    >
                      <span className="harp-note-letter">{noteName}</span>
                      <span className="harp-pedal-symbol">{pedalSymbol(val)}</span>
                      <span className="harp-pedal-num">
                        {val > 0 ? "+1" : String(val)}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {playMode.mode === "Guitar" && (
            <div className="guitar-controls-wrap" data-testid="guitar-mode-panel">
              <div className="mic-detail-grid">
                <ParameterSlider
                  label="Guitar Fret"
                  value={playMode.guitar_fret}
                  min={0}
                  max={24}
                  step={1}
                  onChange={(v) =>
                    mutatePlayMode((pm) => {
                      pm.guitar_fret = Math.round(v);
                    })
                  }
                />
                <ParameterSlider
                  label="Guitar Body"
                  value={playMode.guitar_body}
                  min={0}
                  max={1}
                  step={0.01}
                  onChange={(v) =>
                    mutatePlayMode((pm) => {
                      pm.guitar_body = v;
                    })
                  }
                />
              </div>
              <label className="check-pill">
                <input
                  type="checkbox"
                  checked={playMode.guitar_legato}
                  onChange={(e) =>
                    mutatePlayMode((pm) => {
                      pm.guitar_legato = e.target.checked;
                    })
                  }
                />
                <span>Guitar Legato (Hammer-On / Pull-Off)</span>
              </label>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};
