import { useMemo } from "react";
import { Lock, Play, Square, Unlock } from "lucide-react";
import type { MusicPackageSummary, OtsPresetSummary } from "../types/engine";

const QUICK_CHORDS: { label: string; root: number; quality: string }[] = [
  { label: "C", root: 0, quality: "Major" },
  { label: "Cm", root: 0, quality: "Minor" },
  { label: "D", root: 2, quality: "Major" },
  { label: "Dm", root: 2, quality: "Minor" },
  { label: "E", root: 4, quality: "Major" },
  { label: "Em", root: 4, quality: "Minor" },
  { label: "F", root: 5, quality: "Major" },
  { label: "Fm", root: 5, quality: "Minor" },
  { label: "G", root: 7, quality: "Major" },
  { label: "G7", root: 7, quality: "Dom7" },
  { label: "Am", root: 9, quality: "Minor" },
  { label: "A", root: 9, quality: "Major" },
  { label: "Bb", root: 10, quality: "Major" },
  { label: "B", root: 11, quality: "Major" },
];

export interface LcdPanelProps {
  arranger: MusicPackageSummary;
  onTogglePlay: () => void;
  onBpmChange: (bpm: number) => void;
  onTempoLockToggle: () => void;
  onApplyOts: (otsIndex: number) => void;
  onChordClick: (root: number, quality: string, label: string) => void;
}

function getFormatPillInfo(arranger: MusicPackageSummary): {
  label: string;
  isSty: boolean;
} {
  if (!arranger.loaded) {
    return { label: "Sem pacote", isSty: false };
  }
  const fmt = arranger.source_format?.toLowerCase();
  if (fmt === "sty_pack") {
    return { label: "MULTI .STY", isSty: true };
  }
  if (fmt === "sty" || (!fmt && arranger.file_name.toLowerCase().endsWith(".sty"))) {
    return { label: "YAMAHA .STY", isSty: true };
  }
  return {
    label: arranger.version ? `MUSIC STUDIO v${arranger.version}` : "MUSIC STUDIO",
    isSty: false,
  };
}

function buildOtsTooltip(
  slotNumber: number,
  preset: OtsPresetSummary,
  instrumentByIndex: Map<number, string>,
): string {
  const parts = [`OTS ${slotNumber}: ${preset.name}`];
  const u1 = instrumentByIndex.get(preset.upper1_instrument);
  if (u1) parts.push(`U1: ${u1}`);
  if (preset.upper2_instrument !== null) {
    const u2 = instrumentByIndex.get(preset.upper2_instrument);
    if (u2) parts.push(`U2: ${u2}`);
  }
  if (preset.lower_instrument !== null) {
    const low = instrumentByIndex.get(preset.lower_instrument);
    if (low) parts.push(`Low: ${low}`);
  }
  return parts.join(" • ");
}

/**
 * Resolves user-facing STY expansion kit / Live Audio Style warnings, falling back
 * to flag-derived notices when the backend sets boolean flags without explicit lines.
 */
export function resolveStyWarnings(arranger: MusicPackageSummary): string[] {
  const explicitWarnings = (arranger.sty_warnings ?? []).filter(
    (warning) => warning.trim().length > 0,
  );
  if (explicitWarnings.length > 0) {
    return explicitWarnings;
  }

  const fallback: string[] = [];
  if (arranger.uses_expansion_kit) {
    fallback.push(
      "Expansion Kit / MegaVoice detected in .STY (mapped to standard GM/SF2 fallback)",
    );
  }
  if (arranger.has_audio_style_chunks) {
    fallback.push(
      "Live Audio Style chunks (AASM/AFil/AWav) present — audio stems are not played",
    );
  }
  return fallback;
}

/**
 * Center hardware LCD readout displaying active style/section, detected chord,
 * beat progress bar, tempo controls, One Touch Setting (OTS) bar, and quick chords.
 */
export function LcdPanel({
  arranger,
  onTogglePlay,
  onBpmChange,
  onTempoLockToggle,
  onApplyOts,
  onChordClick,
}: LcdPanelProps) {
  const activeStyle =
    arranger.styles[arranger.active_style_index] ?? arranger.styles[0] ?? null;
  const formatPill = getFormatPillInfo(arranger);
  const otsPresets = arranger.ots_presets ?? [];

  const instrumentNameMap = useMemo(() => {
    const map = new Map<number, string>();
    for (const inst of arranger.instruments) {
      map.set(inst.index, inst.name);
    }
    return map;
  }, [arranger.instruments]);

  return (
    <section className="panel-card arranger-lcd-card">
      <div className="panel-title">
        <span>Visor do Arranjador</span>
        <div className="panel-title-actions">
          {arranger.soundfont_name && (
            <span
              className="badge-subtle lcd-sf2-hint"
              title={`SoundFont ativo: ${arranger.soundfont_name}`}
            >
              {arranger.soundfont_name}
            </span>
          )}
          <span className={`badge-subtle ${formatPill.isSty ? "badge-sty" : ""}`}>
            {formatPill.label}
          </span>
        </div>
      </div>

      <div className="lcd-screen">
        <div className="lcd-main-row">
          <div className="lcd-info-col">
            <div className="lcd-label">RITMO ATIVO</div>
            <div className="lcd-style-name">
              {activeStyle?.name ?? (arranger.file_name || "Nenhum ritmo carregado")}
            </div>
            <div className="lcd-sub-meta">
              Seção: <strong>{arranger.active_section}</strong>
              {arranger.pending_section && (
                <span className="lcd-pending"> → {arranger.pending_section}</span>
              )}
            </div>
          </div>

          <div className="lcd-chord-box" data-testid="chord-display">
            <div className="lcd-label">ACORDE</div>
            <div className="lcd-chord-value">{arranger.detected_chord || "C"}</div>
          </div>
        </div>

        <div className="lcd-beat-bar">
          <div
            className="lcd-beat-fill"
            style={{ width: `${Math.round((arranger.beat_position ?? 0) * 100)}%` }}
          />
        </div>

        <div className="lcd-tempo-row lcd-playback-row">
          <button
            type="button"
            className={`btn btn-start-stop ${arranger.rhythm_playing ? "playing" : ""}`}
            onClick={onTogglePlay}
            aria-pressed={arranger.rhythm_playing}
          >
            {arranger.rhythm_playing ? <Square size={14} /> : <Play size={14} />}
            <span>{arranger.rhythm_playing ? "STOP RITMO" : "START RITMO"}</span>
          </button>
          <span className="lcd-label">TEMPO</span>
          <div className="tempo-stepper">
            <button
              type="button"
              className="btn btn-xs"
              aria-label="Decrease Tempo"
              onClick={() => onBpmChange(arranger.bpm - 2)}
            >
              -
            </button>
            <strong className="tempo-readout">{Math.round(arranger.bpm)} BPM</strong>
            <button
              type="button"
              className="btn btn-xs"
              aria-label="Increase Tempo"
              onClick={() => onBpmChange(arranger.bpm + 2)}
            >
              +
            </button>
          </div>
          <input
            type="range"
            aria-label="Tempo BPM"
            className="studio-range tempo-slider"
            min={40}
            max={240}
            step={1}
            value={Math.round(arranger.bpm)}
            onChange={(e) => onBpmChange(Number(e.target.value))}
          />
          <button
            type="button"
            aria-label="Tempo Lock"
            aria-pressed={Boolean(arranger.tempo_locked)}
            className={`btn btn-xs ${arranger.tempo_locked ? "btn-flag-on" : ""}`}
            onClick={onTempoLockToggle}
            title="Travar BPM ao trocar de estilo/ritmo"
          >
            {arranger.tempo_locked ? <Lock size={11} /> : <Unlock size={11} />}
            <span>Lock</span>
          </button>
        </div>

        <div className="lcd-ots-bar" role="group" aria-label="One Touch Setting">
          <span className="lcd-label ots-bar-label">OTS</span>
          <div className="ots-buttons-row">
            {otsPresets.length > 0
              ? otsPresets.map((preset, pos) => {
                  const slotNumber = pos + 1;
                  const isActive =
                    preset.upper1_instrument === arranger.upper1_instrument &&
                    preset.upper2_instrument === arranger.upper2_instrument &&
                    preset.lower_instrument === arranger.lower_instrument;
                  const tooltip = buildOtsTooltip(slotNumber, preset, instrumentNameMap);

                  return (
                    <button
                      key={preset.index}
                      type="button"
                      aria-label={`OTS ${slotNumber}: ${preset.name}`}
                      aria-pressed={isActive}
                      className={`btn btn-xs ots-btn ${isActive ? "active" : ""}`}
                      onClick={() => onApplyOts(preset.index)}
                      title={tooltip}
                    >
                      <span className="ots-num">OTS {slotNumber}</span>
                      {preset.name && <span className="ots-name">{preset.name}</span>}
                    </button>
                  );
                })
              : [1, 2, 3, 4].map((slotNumber) => (
                  <button
                    key={slotNumber}
                    type="button"
                    disabled
                    aria-label={`OTS ${slotNumber}`}
                    className="btn btn-xs ots-btn ots-btn-disabled"
                    title="One Touch Setting indisponível neste ritmo"
                  >
                    <span className="ots-num">OTS {slotNumber}</span>
                  </button>
                ))}
          </div>
        </div>


      </div>

      <div className="chord-pads-section">
        <div className="chord-pads-label">
          Acorde Rápido (ou toque na esquerda / Chord Scan):
        </div>
        <div className="chord-pads-grid" role="group" aria-label="Quick Chords">
          {QUICK_CHORDS.map((c) => (
            <button
              key={c.label}
              type="button"
              className={`chord-pad-btn ${
                arranger.detected_chord === c.label ? "active" : ""
              }`}
              onClick={() => onChordClick(c.root, c.quality, c.label)}
            >
              {c.label}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
