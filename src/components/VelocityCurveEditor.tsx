import React, { useMemo, useRef, useState } from "react";
import { Plus, Trash2, Activity } from "lucide-react";
import { engineClient } from "../services/engineClient";
import type { MidiConfig } from "../types/engine";
import { ParameterSlider } from "./ParameterSlider";

export interface VelocityCurvePreset {
  id: string;
  label:
    | "Slow keyboard"
    | "Moderately slow keyboard"
    | "Normal keyboard"
    | "Moderately fast keyboard"
    | "Fast keyboard";
  points: [number, number][];
}

export const VELOCITY_CURVE_PRESETS: VelocityCurvePreset[] = [
  {
    id: "slow-keyboard",
    label: "Slow keyboard",
    points: [
      [0, 0],
      [0.25, 0.12],
      [0.5, 0.32],
      [0.75, 0.62],
      [1, 1],
    ],
  },
  {
    id: "mod-slow-keyboard",
    label: "Moderately slow keyboard",
    points: [
      [0, 0],
      [0.25, 0.18],
      [0.5, 0.4],
      [0.75, 0.68],
      [1, 1],
    ],
  },
  {
    id: "normal-keyboard",
    label: "Normal keyboard",
    points: [
      [0, 0],
      [0.25, 0.25],
      [0.5, 0.5],
      [0.75, 0.75],
      [1, 1],
    ],
  },
  {
    id: "mod-fast-keyboard",
    label: "Moderately fast keyboard",
    points: [
      [0, 0],
      [0.25, 0.34],
      [0.5, 0.62],
      [0.75, 0.84],
      [1, 1],
    ],
  },
  {
    id: "fast-keyboard",
    label: "Fast keyboard",
    points: [
      [0, 0.05],
      [0.25, 0.42],
      [0.5, 0.72],
      [0.75, 0.9],
      [1, 1],
    ],
  },
];

export function normalizeCurvePoints(curve: [number, number][]): [number, number][] {
  if (!curve || curve.length === 0) {
    return VELOCITY_CURVE_PRESETS[0].points.map(([x, y]) => [x, y]);
  }
  return [...curve]
    .map(([x, y]): [number, number] => [
      Math.max(0, Math.min(1, Number.isFinite(x) ? x : 0)),
      Math.max(0, Math.min(1, Number.isFinite(y) ? y : 0)),
    ])
    .sort((a, b) => a[0] - b[0]);
}

export function evaluateVelocityCurve(rawVelocity: number, curve: [number, number][]): number {
  if (rawVelocity <= 0) return 0;
  const pts = normalizeCurvePoints(curve);
  const x = Math.max(0, Math.min(1, rawVelocity / 127));
  if (x <= pts[0][0]) {
    return Math.round(Math.max(0.01, Math.min(1, pts[0][1])) * 127);
  }
  for (let i = 0; i < pts.length - 1; i++) {
    const [x0, y0] = pts[i];
    const [x1, y1] = pts[i + 1];
    if (x <= x1) {
      const span = Math.max(1e-5, x1 - x0);
      const t = (x - x0) / span;
      const y = y0 + (y1 - y0) * t;
      return Math.round(Math.max(0.01, Math.min(1, y)) * 127);
    }
  }
  const lastY = pts[pts.length - 1][1];
  return Math.round(Math.max(0.01, Math.min(1, lastY)) * 127);
}

export function detectVelocityPreset(curve: [number, number][]): string {
  const pts = normalizeCurvePoints(curve);
  for (const preset of VELOCITY_CURVE_PRESETS) {
    if (preset.points.length !== pts.length) continue;
    const matches = preset.points.every(
      ([px, py], i) =>
        Math.abs(px - pts[i][0]) < 0.015 && Math.abs(py - pts[i][1]) < 0.015,
    );
    if (matches) return preset.id;
  }
  return "custom";
}

export interface VelocityCurveEditorProps {
  midiConfig: MidiConfig;
  lastVelocity?: number | null;
  onUpdateMidiConfig?: (nextConfig: MidiConfig) => void;
}

const SVG_W = 280;
const SVG_H = 196;
const PAD = { left: 30, right: 14, top: 14, bottom: 24 };
const PLOT_W = SVG_W - PAD.left - PAD.right;
const PLOT_H = SVG_H - PAD.top - PAD.bottom;

function toSvgX(normX: number): number {
  return PAD.left + normX * PLOT_W;
}

function toSvgY(normY: number): number {
  return PAD.top + (1 - normY) * PLOT_H;
}

export const VelocityCurveEditor: React.FC<VelocityCurveEditorProps> = ({
  midiConfig,
  lastVelocity = null,
  onUpdateMidiConfig,
}) => {
  const points = useMemo(
    () => normalizeCurvePoints(midiConfig.velocity_curve),
    [midiConfig.velocity_curve],
  );
  const [selectedIdx, setSelectedIdx] = useState<number>(2);
  const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const safeSelectedIdx = Math.min(selectedIdx, Math.max(0, points.length - 1));
  const selectedPt = points[safeSelectedIdx] ?? [0.5, 0.5];
  const activePresetId = detectVelocityPreset(points);

  const commitCurve = (nextPoints: [number, number][]) => {
    const normalized = normalizeCurvePoints(nextPoints);
    const nextConfig: MidiConfig = {
      ...midiConfig,
      velocity_curve: normalized,
    };
    if (onUpdateMidiConfig) {
      onUpdateMidiConfig(nextConfig);
    } else {
      engineClient.updateMidiConfig(nextConfig).catch(() => {});
    }
  };

  const applyPreset = (preset: VelocityCurvePreset) => {
    commitCurve(preset.points.map(([x, y]) => [x, y]));
    setSelectedIdx(Math.min(2, preset.points.length - 1));
  };

  const updatePoint = (idx: number, rawX: number, rawY: number) => {
    const clampedY = Math.round(Math.max(0, Math.min(1, rawY)) * 100) / 100;
    let clampedX = points[idx][0];
    if (idx > 0 && idx < points.length - 1) {
      const minX = points[idx - 1][0] + 0.02;
      const maxX = points[idx + 1][0] - 0.02;
      clampedX = Math.round(Math.max(minX, Math.min(maxX, rawX)) * 100) / 100;
    }
    const next = points.map((pt, i): [number, number] =>
      i === idx ? [clampedX, clampedY] : [pt[0], pt[1]],
    );
    commitCurve(next);
  };

  const pointerToNorm = (e: React.PointerEvent<SVGSVGElement>): [number, number] | null => {
    if (!Number.isFinite(e.clientX) || !Number.isFinite(e.clientY)) return null;
    const svg = svgRef.current;
    if (!svg) return null;
    const rect = svg.getBoundingClientRect();
    const w = Math.max(1, rect.width);
    const h = Math.max(1, rect.height);
    const viewX = ((e.clientX - rect.left) / w) * SVG_W;
    const viewY = ((e.clientY - rect.top) / h) * SVG_H;
    const normX = Math.max(0, Math.min(1, (viewX - PAD.left) / PLOT_W));
    const normY = Math.max(0, Math.min(1, 1 - (viewY - PAD.top) / PLOT_H));
    if (!Number.isFinite(normX) || !Number.isFinite(normY)) return null;
    return [normX, normY];
  };

  const handleSvgPointerMove = (e: React.PointerEvent<SVGSVGElement>) => {
    if (draggingIdx === null) return;
    const norm = pointerToNorm(e);
    if (!norm) return;
    updatePoint(draggingIdx, norm[0], norm[1]);
  };

  const handleAddPoint = () => {
    if (points.length >= 9) return;
    const leftIdx = Math.min(safeSelectedIdx, points.length - 2);
    const [x0, y0] = points[leftIdx];
    const [x1, y1] = points[leftIdx + 1];
    if (x1 - x0 < 0.05) return;
    const newPt: [number, number] = [
      Math.round(((x0 + x1) / 2) * 100) / 100,
      Math.round(((y0 + y1) / 2) * 100) / 100,
    ];
    const next = [...points.slice(0, leftIdx + 1), newPt, ...points.slice(leftIdx + 1)];
    commitCurve(next);
    setSelectedIdx(leftIdx + 1);
  };

  const handleRemovePoint = () => {
    if (points.length <= 2) return;
    if (safeSelectedIdx === 0 || safeSelectedIdx === points.length - 1) return;
    const next = points.filter((_, i) => i !== safeSelectedIdx);
    commitCurve(next);
    setSelectedIdx(Math.max(0, safeSelectedIdx - 1));
  };

  const polylineStr = points.map(([x, y]) => `${toSvgX(x)},${toSvgY(y)}`).join(" ");
  const areaStr = [
    `${toSvgX(points[0][0])},${toSvgY(0)}`,
    ...points.map(([x, y]) => `${toSvgX(x)},${toSvgY(y)}`),
    `${toSvgX(points[points.length - 1][0])},${toSvgY(0)}`,
  ].join(" ");

  const liveOut =
    lastVelocity && lastVelocity > 0
      ? evaluateVelocityCurve(lastVelocity, points)
      : null;

  const isInterior = safeSelectedIdx > 0 && safeSelectedIdx < points.length - 1;

  return (
    <section className="panel-card" data-testid="velocity-curve-editor">
      <div className="panel-title">
        <span>Velocity Curve</span>
        <span className="badge-subtle">
          <Activity size={12} />{" "}
          {activePresetId === "custom"
            ? `Custom (${points.length} pts)`
            : VELOCITY_CURVE_PRESETS.find((p) => p.id === activePresetId)?.label}
        </span>
      </div>

      <div className="preset-chips-row" role="group" aria-label="Velocity Curve Presets">
        {VELOCITY_CURVE_PRESETS.map((preset) => (
          <button
            key={preset.id}
            type="button"
            className={`chip-btn ${activePresetId === preset.id ? "active" : ""}`}
            aria-pressed={activePresetId === preset.id}
            onClick={() => applyPreset(preset)}
          >
            {preset.label}
          </button>
        ))}
      </div>

      <div className="curve-canvas-wrap">
        <svg
          ref={svgRef}
          viewBox={`0 0 ${SVG_W} ${SVG_H}`}
          className="velocity-curve-svg"
          data-testid="velocity-curve-svg"
          role="img"
          aria-label="MIDI Velocity Curve Graph"
          onPointerMove={handleSvgPointerMove}
          onPointerUp={() => setDraggingIdx(null)}
          onPointerLeave={() => setDraggingIdx(null)}
          onDoubleClick={(e) => {
            if (points.length >= 9) return;
            const rect = e.currentTarget.getBoundingClientRect();
            const w = Math.max(1, rect.width);
            const h = Math.max(1, rect.height);
            const vx = ((e.clientX - rect.left) / w) * SVG_W;
            const vy = ((e.clientY - rect.top) / h) * SVG_H;
            const nx = Math.round(Math.max(0.03, Math.min(0.97, (vx - PAD.left) / PLOT_W)) * 100) / 100;
            const ny = Math.round(Math.max(0, Math.min(1, 1 - (vy - PAD.top) / PLOT_H)) * 100) / 100;
            const tooClose = points.some(([px]) => Math.abs(px - nx) < 0.03);
            if (!tooClose) {
              const next = normalizeCurvePoints([...points, [nx, ny]]);
              commitCurve(next);
              const found = next.findIndex(([px]) => Math.abs(px - nx) < 0.005);
              if (found >= 0) setSelectedIdx(found);
            }
          }}
        >
          <defs>
            <linearGradient id="velCurveFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#e29d38" stopOpacity="0.34" />
              <stop offset="100%" stopColor="#e29d38" stopOpacity="0.02" />
            </linearGradient>
          </defs>

          <rect
            x={PAD.left}
            y={PAD.top}
            width={PLOT_W}
            height={PLOT_H}
            fill="#0b0e14"
            stroke="#2a3342"
            rx="4"
          />

          {[0.25, 0.5, 0.75].map((t) => (
            <g key={t}>
              <line
                x1={toSvgX(t)}
                y1={PAD.top}
                x2={toSvgX(t)}
                y2={PAD.top + PLOT_H}
                stroke="#1d2431"
                strokeDasharray="2 3"
              />
              <line
                x1={PAD.left}
                y1={toSvgY(t)}
                x2={PAD.left + PLOT_W}
                y2={toSvgY(t)}
                stroke="#1d2431"
                strokeDasharray="2 3"
              />
            </g>
          ))}

          {/* Linear 1:1 Reference */}
          <line
            x1={toSvgX(0)}
            y1={toSvgY(0)}
            x2={toSvgX(1)}
            y2={toSvgY(1)}
            stroke="#364256"
            strokeWidth="1"
            strokeDasharray="4 4"
          />

          {/* Axis Labels */}
          <text x={PAD.left - 5} y={toSvgY(1) + 4} textAnchor="end" className="svg-axis-txt">
            127
          </text>
          <text x={PAD.left - 5} y={toSvgY(0.5) + 3} textAnchor="end" className="svg-axis-txt">
            64
          </text>
          <text x={PAD.left - 5} y={toSvgY(0) + 2} textAnchor="end" className="svg-axis-txt">
            0
          </text>
          <text x={toSvgX(0)} y={SVG_H - 7} textAnchor="middle" className="svg-axis-txt">
            0
          </text>
          <text x={toSvgX(0.5)} y={SVG_H - 7} textAnchor="middle" className="svg-axis-txt">
            In Vel (64)
          </text>
          <text x={toSvgX(1)} y={SVG_H - 7} textAnchor="middle" className="svg-axis-txt">
            127
          </text>

          <polygon points={areaStr} fill="url(#velCurveFill)" />
          <polyline
            points={polylineStr}
            fill="none"
            stroke="#e29d38"
            strokeWidth="2.2"
            strokeLinejoin="round"
          />

          {lastVelocity && lastVelocity > 0 && liveOut !== null && (
            <g data-testid="velocity-live-indicator">
              <line
                x1={toSvgX(lastVelocity / 127)}
                y1={PAD.top + PLOT_H}
                x2={toSvgX(lastVelocity / 127)}
                y2={toSvgY(liveOut / 127)}
                stroke="#48b8d0"
                strokeDasharray="3 2"
              />
              <circle
                cx={toSvgX(lastVelocity / 127)}
                cy={toSvgY(liveOut / 127)}
                r="4.5"
                fill="#48b8d0"
                stroke="#0b0e14"
                strokeWidth="1.5"
              />
            </g>
          )}

          {points.map(([x, y], idx) => {
            const isSelected = idx === safeSelectedIdx;
            return (
              <circle
                key={idx}
                cx={toSvgX(x)}
                cy={toSvgY(y)}
                r={isSelected ? 6 : 4.5}
                fill={isSelected ? "#ffd188" : "#e29d38"}
                stroke="#0b0e14"
                strokeWidth="1.6"
                style={{ cursor: "grab" }}
                data-testid={`velocity-point-${idx}`}
                onPointerDown={(e) => {
                  e.stopPropagation();
                  setSelectedIdx(idx);
                  setDraggingIdx(idx);
                  if (typeof e.currentTarget.setPointerCapture === "function") {
                    try {
                      e.currentTarget.setPointerCapture(e.pointerId);
                    } catch {
                      // JSDOM may not implement pointer capture
                    }
                  }
                }}
              />
            );
          })}
        </svg>
      </div>

      <div className="curve-point-bar">
        <span className="curve-point-readout">
          Point #{safeSelectedIdx + 1}: In {Math.round(selectedPt[0] * 127)} → Out{" "}
          {Math.round(selectedPt[1] * 127)}
          {liveOut !== null && lastVelocity
            ? ` • Live: ${lastVelocity} → ${liveOut}`
            : ""}
        </span>
        <div className="curve-point-actions">
          <button
            type="button"
            className="btn btn-compact"
            onClick={handleAddPoint}
            disabled={points.length >= 9}
            aria-label="Add Control Point"
            title="Add Control Point"
          >
            <Plus size={12} /> Point
          </button>
          <button
            type="button"
            className="btn btn-compact"
            onClick={handleRemovePoint}
            disabled={points.length <= 2 || !isInterior}
            aria-label="Remove Selected Interior Point"
            title="Remove Selected Interior Point"
          >
            <Trash2 size={12} />
          </button>
        </div>
      </div>

      <div className="two-col-mini">
        <ParameterSlider
          label={`Point ${safeSelectedIdx + 1} Input`}
          value={selectedPt[0]}
          min={isInterior ? points[safeSelectedIdx - 1][0] + 0.02 : selectedPt[0]}
          max={isInterior ? points[safeSelectedIdx + 1][0] - 0.02 : selectedPt[0]}
          step={0.01}
          onChange={(vx) => {
            if (isInterior) updatePoint(safeSelectedIdx, vx, selectedPt[1]);
          }}
        />
        <ParameterSlider
          label={`Point ${safeSelectedIdx + 1} Output`}
          value={selectedPt[1]}
          min={0}
          max={1}
          step={0.01}
          onChange={(vy) => updatePoint(safeSelectedIdx, selectedPt[0], vy)}
        />
      </div>
    </section>
  );
};
