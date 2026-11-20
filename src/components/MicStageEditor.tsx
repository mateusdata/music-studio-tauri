import React, { useRef, useState } from "react";
import { Headphones, Mic, MicOff, Radio, RotateCcw } from "lucide-react";
import type { MicConfig, OutputStageParams } from "../types/engine";
import { ParameterSlider } from "./ParameterSlider";

export interface MicStageEditorProps {
  output: OutputStageParams;
  onChange: (nextOutput: OutputStageParams) => void;
}

const STAGE_W = 380;
const STAGE_H = 290;
const STAGE_PAD = { left: 26, right: 26, top: 22, bottom: 26 };
const PLOT_W = STAGE_W - STAGE_PAD.left - STAGE_PAD.right;
const PLOT_H = STAGE_H - STAGE_PAD.top - STAGE_PAD.bottom;

const MIC_COLORS = ["#48b8d0", "#e29d38", "#44c082"];

type DragTarget =
  | { kind: "mic"; index: number }
  | { kind: "head" }
  | null;

function normToSvgX(x: number): number {
  return STAGE_PAD.left + Math.max(0, Math.min(1, x)) * PLOT_W;
}

function normToSvgY(y: number): number {
  return STAGE_PAD.top + (1 - Math.max(0, Math.min(1, y))) * PLOT_H;
}

function clamp01(v: number): number {
  if (!Number.isFinite(v)) return 0;
  return Math.round(Math.max(0, Math.min(1, v)) * 100) / 100;
}

export const MicStageEditor: React.FC<MicStageEditorProps> = ({
  output,
  onChange,
}) => {
  const [selectedMicIdx, setSelectedMicIdx] = useState<number>(0);
  const [dragTarget, setDragTarget] = useState<DragTarget>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const mics = output.mics ?? [];
  const safeMicIdx = Math.min(selectedMicIdx, Math.max(0, mics.length - 1));
  const activeMic = mics[safeMicIdx];

  const updateOutput = (mutator: (draft: OutputStageParams) => void) => {
    const cloned: OutputStageParams = JSON.parse(JSON.stringify(output));
    mutator(cloned);
    onChange(cloned);
  };

  const updateMic = (idx: number, mutator: (mic: MicConfig) => void) => {
    updateOutput((draft) => {
      if (draft.mics[idx]) {
        mutator(draft.mics[idx]);
      }
    });
  };

  const handleStagePointerMove = (e: React.PointerEvent<SVGSVGElement>) => {
    if (!dragTarget || !svgRef.current) return;
    if (!Number.isFinite(e.clientX) || !Number.isFinite(e.clientY)) return;
    const rect = svgRef.current.getBoundingClientRect();
    const w = Math.max(1, rect.width);
    const h = Math.max(1, rect.height);
    const viewX = ((e.clientX - rect.left) / w) * STAGE_W;
    const viewY = ((e.clientY - rect.top) / h) * STAGE_H;
    const nx = clamp01((viewX - STAGE_PAD.left) / PLOT_W);
    const ny = clamp01(1 - (viewY - STAGE_PAD.top) / PLOT_H);

    if (dragTarget.kind === "mic") {
      updateMic(dragTarget.index, (m) => {
        m.x = nx;
        m.y = ny;
      });
    } else if (dragTarget.kind === "head") {
      updateOutput((d) => {
        d.head_x = nx;
        d.head_y = ny;
      });
    }
  };

  // Piano outline mapped inside stage space:
  // Keyboard front at y ~ 0.30, tail up to y ~ 0.78, width x ~ 0.25..0.75
  const pxLeft = normToSvgX(0.24);
  const pxRight = normToSvgX(0.76);
  const pxMidRight = normToSvgX(0.58);
  const pyKeyFront = normToSvgY(0.29);
  const pyKeyBack = normToSvgY(0.35);
  const pyTail = normToSvgY(0.79);
  const pyTrebleEnd = normToSvgY(0.56);

  const pianoBodyPath = [
    `M ${pxLeft} ${pyKeyFront}`,
    `L ${pxRight} ${pyKeyFront}`,
    `L ${pxRight} ${pyTrebleEnd}`,
    `C ${pxRight} ${normToSvgY(0.65)}, ${pxMidRight + 24} ${normToSvgY(0.63)}, ${pxMidRight + 10} ${normToSvgY(0.73)}`,
    `C ${pxMidRight} ${pyTail}, ${pxLeft + 18} ${pyTail}, ${pxLeft} ${normToSvgY(0.73)}`,
    `Z`,
  ].join(" ");

  const lidRatio = Math.max(0, Math.min(1, output.lid_position));
  const lidStatusLabel =
    lidRatio < 0.08 ? "Closed" : lidRatio < 0.6 ? "Half Stick" : "Full Open";

  const headSvgX = normToSvgX(output.head_x);
  const headSvgY = normToSvgY(output.head_y);

  return (
    <section className="panel-card panel-card-wide" data-testid="mic-stage-editor">
      <div className="panel-title">
        <span>
          <Radio size={14} style={{ verticalAlign: "text-bottom", marginRight: 6 }} />
          Acoustic Stage, 3-Mic Array & Binaural Head
        </span>
        <span className="badge-subtle">
          Lid: {Math.round(lidRatio * 100)}% ({lidStatusLabel})
        </span>
      </div>

      <div className="stage-layout">
        {/* Left: Interactive 2D Stage Map */}
        <div className="stage-map-col">
          <svg
            ref={svgRef}
            viewBox={`0 0 ${STAGE_W} ${STAGE_H}`}
            className="stage-map-svg"
            data-testid="mic-stage-map"
            role="img"
            aria-label="2D Soundboard and Microphone Stage Map"
            onPointerMove={handleStagePointerMove}
            onPointerUp={() => setDragTarget(null)}
            onPointerLeave={() => setDragTarget(null)}
          >
            <defs>
              <radialGradient id="stageHallGlow" cx="50%" cy="46%" r="54%">
                <stop offset="0%" stopColor="#262f3f" stopOpacity="0.65" />
                <stop offset="100%" stopColor="#0a0d13" stopOpacity="0.98" />
              </radialGradient>
              <linearGradient id="soundboardWood" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stopColor="#3c2818" />
                <stop offset="55%" stopColor="#583a22" />
                <stop offset="100%" stopColor="#2c1d11" />
              </linearGradient>
            </defs>

            <rect
              x="0"
              y="0"
              width={STAGE_W}
              height={STAGE_H}
              rx="8"
              fill="url(#stageHallGlow)"
              stroke="#293242"
            />

            {/* Spatial reference grid */}
            {[0.25, 0.5, 0.75].map((t) => (
              <g key={t}>
                <line
                  x1={normToSvgX(t)}
                  y1={STAGE_PAD.top}
                  x2={normToSvgX(t)}
                  y2={STAGE_H - STAGE_PAD.bottom}
                  stroke="#1c2330"
                  strokeDasharray="2 4"
                />
                <line
                  x1={STAGE_PAD.left}
                  y1={normToSvgY(t)}
                  x2={STAGE_W - STAGE_PAD.right}
                  y2={normToSvgY(t)}
                  stroke="#1c2330"
                  strokeDasharray="2 4"
                />
              </g>
            ))}

            {/* Acoustic lid radiation arcs */}
            {lidRatio > 0.05 && (
              <g opacity={0.18 + lidRatio * 0.35}>
                <path
                  d={`M ${pxRight - 4} ${pyKeyBack - 8} Q ${pxRight + 32 * lidRatio} ${normToSvgY(0.52)} ${pxMidRight + 16} ${pyTail - 16}`}
                  fill="none"
                  stroke="#e29d38"
                  strokeWidth="1.4"
                  strokeDasharray="4 3"
                />
                <path
                  d={`M ${pxRight + 8} ${pyKeyBack - 4} Q ${pxRight + 54 * lidRatio} ${normToSvgY(0.54)} ${pxMidRight + 32} ${pyTail - 24}`}
                  fill="none"
                  stroke="#e29d38"
                  strokeWidth="1"
                  strokeDasharray="3 4"
                />
              </g>
            )}

            {/* Grand Piano Outline */}
            <path
              d={pianoBodyPath}
              fill="url(#soundboardWood)"
              stroke="#8c6943"
              strokeWidth="2"
            />

            {/* Strings fan over soundboard */}
            {Array.from({ length: 10 }).map((_, idx) => {
              const frac = idx / 9;
              const sx1 = pxLeft + 10 + frac * (pxRight - pxLeft - 20);
              const sx2 = pxLeft + 12 + frac * (pxMidRight + 8 - pxLeft);
              const sy2 = pyTail + 14 + frac * (pyTrebleEnd - pyTail - 18);
              return (
                <line
                  key={idx}
                  x1={sx1}
                  y1={pyKeyBack}
                  x2={sx2}
                  y2={sy2}
                  stroke="#c8a96e"
                  strokeOpacity={0.35 + lidRatio * 0.35}
                  strokeWidth="0.9"
                />
              );
            })}

            {/* Lid cover overlay: fades/slides in when lid closes */}
            <path
              d={pianoBodyPath}
              fill="#11151d"
              fillOpacity={(1 - lidRatio) * 0.78}
              stroke="none"
            />

            {/* Keyboard bed */}
            <rect
              x={pxLeft}
              y={pyKeyBack}
              width={pxRight - pxLeft}
              height={pyKeyFront - pyKeyBack}
              fill="#e9e4d8"
              stroke="#5c4732"
              strokeWidth="1.2"
              rx="1.5"
            />
            {Array.from({ length: 16 }).map((_, i) => (
              <rect
                key={i}
                x={pxLeft + 6 + i * ((pxRight - pxLeft - 14) / 16)}
                y={pyKeyBack + 1}
                width="3"
                height={(pyKeyFront - pyKeyBack) * 0.56}
                fill="#1b1e24"
              />
            ))}

            {/* Lid prop line on right bentside */}
            <line
              x1={pxLeft + 6}
              y1={normToSvgY(0.52)}
              x2={pxLeft + 6 + (pxRight - pxLeft) * (0.2 + lidRatio * 0.85)}
              y2={normToSvgY(0.52 - lidRatio * 0.06)}
              stroke="#e29d38"
              strokeWidth="2.2"
              strokeLinecap="round"
            />

            {/* Stage Labels */}
            <text x={STAGE_W / 2} y={14} textAnchor="middle" className="svg-axis-txt">
              Hall / Room Far Field
            </text>
            <text x={STAGE_W / 2} y={STAGE_H - 8} textAnchor="middle" className="svg-axis-txt">
              Player Position (Bench)
            </text>

            {/* Binaural Head Marker */}
            <g
              transform={`translate(${headSvgX}, ${headSvgY}) rotate(${output.head_angle})`}
              data-testid="binaural-head-marker"
              style={{ cursor: "grab" }}
              opacity={output.binaural_mode ? 1 : 0.42}
              onPointerDown={(e) => {
                e.stopPropagation();
                setDragTarget({ kind: "head" });
                if (typeof e.currentTarget.setPointerCapture === "function") {
                  try {
                    e.currentTarget.setPointerCapture(e.pointerId);
                  } catch {
                    // JSDOM may not implement pointer capture
                  }
                }
              }}
            >
              <circle
                r="14"
                fill={output.binaural_mode ? "rgba(72, 184, 208, 0.16)" : "transparent"}
                stroke="#48b8d0"
                strokeDasharray={output.binaural_mode ? undefined : "3 3"}
                strokeWidth="1.5"
              />
              {/* Ears */}
              <rect x="-13" y="-3.5" width="3" height="7" rx="1.5" fill="#48b8d0" />
              <rect x="10" y="-3.5" width="3" height="7" rx="1.5" fill="#48b8d0" />
              {/* Head + Nose facing direction */}
              <circle r="9.5" fill="#17202c" stroke="#48b8d0" strokeWidth="1.6" />
              <polygon points="0,-13 -4,-8 4,-8" fill="#48b8d0" />
            </g>

            {/* 3 Microphones */}
            {mics.slice(0, 3).map((mic, idx) => {
              const mx = normToSvgX(mic.x);
              const my = normToSvgY(mic.y);
              const color = MIC_COLORS[idx % MIC_COLORS.length];
              const isSelected = idx === safeMicIdx;
              const aimRad = ((mic.azimuth - 90) * Math.PI) / 180;
              return (
                <g
                  key={idx}
                  transform={`translate(${mx}, ${my})`}
                  data-testid={`stage-mic-${idx}`}
                  style={{ cursor: "grab" }}
                  opacity={mic.enabled ? 1 : 0.4}
                  onPointerDown={(e) => {
                    e.stopPropagation();
                    setSelectedMicIdx(idx);
                    setDragTarget({ kind: "mic", index: idx });
                    if (typeof e.currentTarget.setPointerCapture === "function") {
                      try {
                        e.currentTarget.setPointerCapture(e.pointerId);
                      } catch {
                        // JSDOM may not implement pointer capture
                      }
                    }
                  }}
                >
                  {isSelected && (
                    <circle
                      r="15"
                      fill="none"
                      stroke={color}
                      strokeOpacity="0.5"
                      strokeWidth="1.5"
                    />
                  )}
                  <line
                    x1="0"
                    y1="0"
                    x2={Math.cos(aimRad) * 14}
                    y2={Math.sin(aimRad) * 14}
                    stroke={color}
                    strokeWidth="1.6"
                  />
                  <circle
                    r="9"
                    fill="#121720"
                    stroke={color}
                    strokeWidth={isSelected ? "2.4" : "1.6"}
                    strokeDasharray={mic.enabled ? undefined : "2 2"}
                  />
                  <text
                    y="3"
                    textAnchor="middle"
                    fontSize="8.5"
                    fontWeight="700"
                    fill={color}
                  >
                    M{idx + 1}
                  </text>
                </g>
              );
            })}
          </svg>
        </div>

        {/* Right: Stage Controls + 3 Mic Toggle Cards + Binaural Head Controls */}
        <div className="stage-controls-col">
          <div className="stage-subcard">
            <div className="subcard-title">Stage Acoustics & Compensation</div>
            <div className="two-col-mini">
              <ParameterSlider
                label="Lid Position"
                value={output.lid_position}
                min={0}
                max={1}
                step={0.01}
                onChange={(v) => updateOutput((d) => { d.lid_position = v; })}
              />
              <ParameterSlider
                label="Stereo Width"
                value={output.stereo_width}
                min={0}
                max={1}
                step={0.01}
                onChange={(v) => updateOutput((d) => { d.stereo_width = v; })}
              />
            </div>

            <div className="toggles-row">
              <label className="check-pill">
                <input
                  type="checkbox"
                  checked={output.mic_level_compensation}
                  onChange={(e) =>
                    updateOutput((d) => {
                      d.mic_level_compensation = e.target.checked;
                    })
                  }
                />
                <span>Mic Level Compensation</span>
              </label>
              <label className="check-pill">
                <input
                  type="checkbox"
                  checked={output.mic_delay_compensation}
                  onChange={(e) =>
                    updateOutput((d) => {
                      d.mic_delay_compensation = e.target.checked;
                    })
                  }
                />
                <span>Mic Delay Compensation</span>
              </label>
              <label className="check-pill">
                <input
                  type="checkbox"
                  checked={output.binaural_mode}
                  onChange={(e) =>
                    updateOutput((d) => {
                      d.binaural_mode = e.target.checked;
                    })
                  }
                />
                <span>Binaural Head Mode</span>
              </label>
            </div>
          </div>

          {/* 3 Toggleable Microphones */}
          <div className="stage-subcard">
            <div className="subcard-title">Microphones (3-Channel Array)</div>
            <div className="mic-selector-strip">
              {mics.slice(0, 3).map((mic, idx) => {
                const color = MIC_COLORS[idx % MIC_COLORS.length];
                const isSelected = idx === safeMicIdx;
                return (
                  <div
                    key={idx}
                    className={`mic-tab-pill ${isSelected ? "selected" : ""}`}
                    style={{ borderColor: isSelected ? color : undefined }}
                    onClick={() => setSelectedMicIdx(idx)}
                  >
                    <label
                      className="mic-toggle-label"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <input
                        type="checkbox"
                        aria-label={`Toggle Mic ${idx + 1} (${mic.name})`}
                        checked={mic.enabled}
                        onChange={(e) =>
                          updateMic(idx, (m) => {
                            m.enabled = e.target.checked;
                          })
                        }
                      />
                      {mic.enabled ? (
                        <Mic size={12} color={color} />
                      ) : (
                        <MicOff size={12} color="#697589" />
                      )}
                    </label>
                    <button
                      type="button"
                      className="mic-name-btn"
                      onClick={() => setSelectedMicIdx(idx)}
                    >
                      M{idx + 1}: {mic.name}
                    </button>
                  </div>
                );
              })}
            </div>

            {activeMic && (
              <div className="mic-detail-grid" data-testid="active-mic-controls">
                <ParameterSlider
                  label={`M${safeMicIdx + 1} X Position`}
                  value={activeMic.x}
                  min={0}
                  max={1}
                  step={0.01}
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.x = v; })}
                />
                <ParameterSlider
                  label={`M${safeMicIdx + 1} Y Position`}
                  value={activeMic.y}
                  min={0}
                  max={1}
                  step={0.01}
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.y = v; })}
                />
                <ParameterSlider
                  label={`M${safeMicIdx + 1} Volume`}
                  value={activeMic.volume}
                  min={0}
                  max={1.5}
                  step={0.01}
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.volume = v; })}
                />
                <ParameterSlider
                  label={`M${safeMicIdx + 1} Pan`}
                  value={activeMic.pan}
                  min={-1}
                  max={1}
                  step={0.01}
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.pan = v; })}
                />
                <ParameterSlider
                  label={`M${safeMicIdx + 1} Z Height`}
                  value={activeMic.z}
                  min={0}
                  max={1}
                  step={0.01}
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.z = v; })}
                />
                <ParameterSlider
                  label={`M${safeMicIdx + 1} Azimuth`}
                  value={activeMic.azimuth}
                  min={-180}
                  max={180}
                  step={1}
                  unit="°"
                  onChange={(v) => updateMic(safeMicIdx, (m) => { m.azimuth = v; })}
                />
              </div>
            )}
          </div>

          {/* Binaural Head Coordinates */}
          <div className="stage-subcard" data-testid="binaural-head-controls">
            <div className="subcard-title">
              <span>
                <Headphones size={12} style={{ verticalAlign: "text-bottom", marginRight: 5 }} />
                Binaural Head Position & Orientation
              </span>
              <button
                type="button"
                className="btn btn-compact"
                onClick={() =>
                  updateOutput((d) => {
                    d.head_x = 0.5;
                    d.head_y = 0.22;
                    d.head_angle = 0;
                  })
                }
                title="Reset Binaural Head"
              >
                <RotateCcw size={11} /> Center
              </button>
            </div>
            <div className="mic-detail-grid">
              <ParameterSlider
                label="Head X"
                value={output.head_x}
                min={0}
                max={1}
                step={0.01}
                onChange={(v) => updateOutput((d) => { d.head_x = v; })}
              />
              <ParameterSlider
                label="Head Y"
                value={output.head_y}
                min={0}
                max={1}
                step={0.01}
                onChange={(v) => updateOutput((d) => { d.head_y = v; })}
              />
              <ParameterSlider
                label="Head Angle"
                value={output.head_angle}
                min={-180}
                max={180}
                step={1}
                unit="°"
                onChange={(v) => updateOutput((d) => { d.head_angle = v; })}
              />
              <ParameterSlider
                label="Head Diameter"
                value={output.head_diameter}
                min={0.12}
                max={0.28}
                step={0.01}
                unit="m"
                onChange={(v) => updateOutput((d) => { d.head_diameter = v; })}
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
