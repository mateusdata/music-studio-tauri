import { useState } from "react";
import type { MidiConfig } from "../types/engine";

export const DEFAULT_VELOCITY_CURVE: [number, number][] = [[0, 0], [0.25, 0.45], [0.5, 0.75], [0.75, 0.92], [1, 1]];
export const MAX_VELOCITY_PERCENT = 130;
export const MAX_VELOCITY_SCALE = MAX_VELOCITY_PERCENT / 100;
type CurveMode = "default" | "custom" | "fixed";

export function velocityCurveFor(mode: CurveMode, nodes: number[], fixed: number): [number, number][] {
  if (mode === "default") return DEFAULT_VELOCITY_CURVE.map(([x, y]) => [x, y]);
  const output = (percent: number) => Math.max(1 / 127, Math.min(MAX_VELOCITY_SCALE, percent / 100));
  if (mode === "fixed") return [[0, output(fixed)], [1, output(fixed)]];
  return [[0, output(nodes[0])], [0.5, output(nodes[1])], [1, output(nodes[2])]];
}

const clampPercent = (y: number) => Math.max(0, Math.min(MAX_VELOCITY_PERCENT, Math.round(y * 100)));
const svgX = (x: number) => 34 + x * 266;
const svgY = (y: number) => 160 - (Math.max(0, Math.min(MAX_VELOCITY_SCALE, y)) / MAX_VELOCITY_SCALE) * 140;

export function VelocitySettings({ config, onChange, onClose }: {
  config: MidiConfig;
  onChange: (curve: [number, number][]) => void;
  onClose: () => void;
}) {
  const isDefault = JSON.stringify(config.velocity_curve) === JSON.stringify(DEFAULT_VELOCITY_CURVE);
  const isFixed = config.velocity_curve.length === 2 && config.velocity_curve[0][1] === config.velocity_curve[1][1];
  const [mode, setMode] = useState<CurveMode>(isDefault ? "default" : isFixed ? "fixed" : "custom");
  const [nodes, setNodes] = useState(() => config.velocity_curve.length === 3 ? config.velocity_curve.map(([, y]) => clampPercent(y)) : [50, 50, 50]);
  const [fixed, setFixed] = useState(isFixed ? clampPercent(config.velocity_curve[0][1]) : 50);
  const curve = config.velocity_curve;
  const points = curve.map(([x, y]) => `${svgX(x)},${svgY(y).toFixed(1)}`).join(" ");
  const y100 = svgY(1);
  const y50 = svgY(0.5);
  const selectMode = (next: CurveMode) => {
    setMode(next);
    onChange(velocityCurveFor(next, nodes, fixed));
  };
  return (
    <div className="velocity-settings-backdrop" onKeyDown={(event) => { if (event.key === "Escape") onClose(); }}>
      <section className="velocity-settings" role="dialog" aria-modal="true" aria-label="Curva de velocidade">
        <div className="panel-title"><span>Curva de velocidade</span><button autoFocus type="button" className="btn btn-xs" onClick={onClose} aria-label="Fechar curva de velocidade">Fechar</button></div>
        <p>Lo / Mid / Hi representam toque suave, médio e forte — não são graves, médios e agudos de EQ.</p>
        <label className="velocity-mode-label">Modo<select aria-label="Modo da curva de velocidade" className="studio-select" value={mode} onChange={(event) => selectMode(event.target.value as CurveMode)}>
          <option value="default">Padrão MIDI reforçado</option><option value="custom">Personalizada</option><option value="fixed">Fixa</option>
        </select></label>
        <svg className="velocity-settings-graph" viewBox="0 0 330 190" role="img" aria-label={`Curva de velocidade: ${mode === "default" ? "Padrão MIDI reforçado" : mode === "fixed" ? "Fixa" : "Personalizada"}`}>
          <path d={`M34 20V160H300 M34 ${y50.toFixed(1)}H300 M167 20V160`} fill="none" stroke="#40516b" />
          <line x1="34" y1={y100.toFixed(1)} x2="300" y2={y100.toFixed(1)} stroke="#60a5fa" strokeOpacity="0.45" strokeDasharray="4 3" />
          <polyline data-testid="velocity-settings-line" points={points} fill="none" stroke="#60a5fa" strokeWidth="3" />
          {curve.map(([x, y], index) => <circle key={index} cx={svgX(x)} cy={svgY(y).toFixed(1)} r="4" fill="#93c5fd" />)}
          <text x="34" y="180">Suave (Lo)</text><text x="142" y="180">Médio (Mid)</text><text x="245" y="180">Forte (Hi)</text>
          <text x="2" y="24">130%</text><text x="2" y={y100 + 3}>100%</text><text x="12" y="163">0%</text>
        </svg>
        {mode === "custom" && ["Lo — toque suave", "Mid — toque médio", "Hi — toque forte"].map((label, index) => (
          <label className="velocity-output-control" key={label}><span>{label}<output>{nodes[index]}%</output></span><input aria-label={label} type="range" min="0" max={MAX_VELOCITY_PERCENT} step="1" value={nodes[index]} onChange={(event) => {
            const next = nodes.map((value, i) => i === index ? Number(event.target.value) : value);
            setNodes(next); onChange(velocityCurveFor("custom", next, fixed));
          }} /></label>
        ))}
        {mode === "fixed" && <label className="velocity-output-control"><span>Saída fixa<output>{fixed}%</output></span><input aria-label="Velocidade fixa" type="range" min="0" max={MAX_VELOCITY_PERCENT} step="1" value={fixed} onChange={(event) => { const next = Number(event.target.value); setFixed(next); onChange(velocityCurveFor("fixed", nodes, next)); }} /></label>}
        <p>{mode === "default" ? "Padrão MIDI mais sensível: toque médio soa mais forte, preservando dinâmica. Abrir o painel não altera a curva em uso." : "Saída em % (0% a 130%). Até 100% ajusta o velocity MIDI (1–127; 50% = 64). Acima de 100% aplica até +30% de reforço de volume nas teclas tocadas; velocity 0 continua soltando a nota."}</p>
        <div className="velocity-settings-actions"><button type="button" className="btn" onClick={() => selectMode("default")}>Restaurar padrão MIDI</button><span>Para persistir, use Opções → Salvar configuração atual.</span></div>
      </section>
    </div>
  );
}
