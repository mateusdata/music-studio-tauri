import { useMemo } from "react";
import { FolderOpen, Music2 } from "lucide-react";
import { SearchableSelect } from "./SearchableSelect";
import type { MusicPackageSummary } from "../types/engine";

const SPLIT_POINTS: { value: string; label: string }[] = [
  { value: "41", label: "1º Fá (F2 • MIDI 41)" },
  { value: "48", label: "Dó 3 (C3 • MIDI 48)" },
  { value: "53", label: "2º Fá (F3 • MIDI 53)" },
  { value: "54", label: "2º Fá# (F#3 • Padrão)" },
  { value: "55", label: "Sol 3 (G3 • MIDI 55)" },
  { value: "59", label: "Si 3 (B3 • MIDI 59)" },
  { value: "60", label: "Dó 4 (C4 • MIDI 60)" },
  { value: "65", label: "Fá 4 (F4 • MIDI 65)" },
];

export interface VoicesSplitPanelProps {
  arranger: MusicPackageSummary;
  onVoiceChange: (
    slot: "upper1" | "upper2" | "lower" | "split",
    instIdx: number | null,
    splitKey?: number | null,
    splitEnabled?: boolean | null,
    chordScanLower?: boolean | null,
    chordScanUpper?: boolean | null,
    upperOctave?: number | null,
    transpose?: number | null,
    chordMode?: string | null,
  ) => void;
  onPickInstrumentDialog: () => void;
}

/**
 * Right arranger console column for Upper 1/Upper 2/Lower timbre assignment
 * (.Music, .SF2, and .Instrument), Split Divider, Chord Scan, and pitch steppers.
 */
export function VoicesSplitPanel({
  arranger,
  onVoiceChange,
  onPickInstrumentDialog,
}: VoicesSplitPanelProps) {
  const playableInstrumentOptions = useMemo(
    () =>
      arranger.instruments
        .filter((inst) => inst.has_samples || inst.zone_count > 0)
        .map((inst) => ({
          value: String(inst.index),
          label: inst.name,
          sublabel: `${inst.is_drumkit ? "Bateria/Kit" : "Timbre"} • ${inst.zone_count} zonas`,
        })),
    [arranger.instruments],
  );

  const optionalVoiceOptions = useMemo(
    () => [{ value: "", label: "-- Só Acorde / Mudo --" }, ...playableInstrumentOptions],
    [playableInstrumentOptions],
  );

  return (
    <section className="panel-card arranger-voices-card">
      <div className="panel-title">
        <span>Timbres, Split Divider &amp; Chord Scan</span>
        <div className="panel-title-actions">
          <button
            type="button"
            className="btn btn-xs btn-primary"
            aria-label="Importar Timbre .Instrument"
            onClick={onPickInstrumentDialog}
            title="Importar arquivo .Instrument para usar em Upper 1 / Upper 2 / Lower"
          >
            <FolderOpen size={12} /> + .Instrument
          </button>
          <Music2 size={14} />
        </div>
      </div>

      <div className="voice-slots-grid">
        <div className="voice-slot-box">
          <span className="voice-tag voice-tag-u1">UPPER 1 (Solo)</span>
          <SearchableSelect
            ariaLabel="Upper 1 Instrument"
            value={String(arranger.upper1_instrument)}
            onChange={(val) => onVoiceChange("upper1", Number(val))}
            searchPlaceholder="Buscar timbre Upper 1..."
            options={playableInstrumentOptions}
          />
        </div>

        <div className="voice-slot-box">
          <span className="voice-tag voice-tag-u2">UPPER 2 (Camada)</span>
          <SearchableSelect
            ariaLabel="Upper 2 Instrument"
            value={arranger.upper2_instrument !== null ? String(arranger.upper2_instrument) : ""}
            onChange={(val) => onVoiceChange("upper2", val === "" ? null : Number(val))}
            searchPlaceholder="Buscar timbre Upper 2..."
            options={optionalVoiceOptions}
          />
        </div>

        <div className="voice-slot-box">
          <span className="voice-tag voice-tag-low">LOWER (Esq.)</span>
          <SearchableSelect
            ariaLabel="Lower Instrument"
            value={arranger.lower_instrument !== null ? String(arranger.lower_instrument) : ""}
            onChange={(val) => onVoiceChange("lower", val === "" ? null : Number(val))}
            searchPlaceholder="Mudo = Só Acorde"
            options={optionalVoiceOptions}
          />
        </div>
      </div>

      <div className="hardware-toggle-row" role="group" aria-label="Split and Chord Scan Controls">
        <button
          type="button"
          aria-pressed={arranger.split_enabled}
          className={`btn btn-compact btn-split-divider ${
            arranger.split_enabled ? "active" : ""
          }`}
          onClick={() => onVoiceChange("split", null, null, !arranger.split_enabled)}
        >
          <span className={`led-dot ${arranger.split_enabled ? "led-red" : ""}`} />
          SPLIT DIVIDER
        </button>

        <button
          type="button"
          aria-pressed={arranger.chord_scan_lower}
          className={`btn btn-compact ${arranger.chord_scan_lower ? "btn-flag-on" : ""}`}
          onClick={() =>
            onVoiceChange("split", null, null, null, !arranger.chord_scan_lower)
          }
        >
          CHORD LOWER
        </button>

        <button
          type="button"
          aria-pressed={arranger.chord_scan_upper}
          className={`btn btn-compact ${arranger.chord_scan_upper ? "btn-flag-on" : ""}`}
          onClick={() =>
            onVoiceChange("split", null, null, null, null, !arranger.chord_scan_upper)
          }
        >
          SCAN UP
        </button>
      </div>

      <div className="split-pitch-row">
        <select
          aria-label="Split Point"
          className="studio-select studio-select-sm"
          disabled={!arranger.split_enabled}
          value={String(arranger.split_key)}
          onChange={(e) => onVoiceChange("split", null, Number(e.target.value), null)}
        >
          {SPLIT_POINTS.map((sp) => (
            <option key={sp.value} value={sp.value}>
              Corte: {sp.label}
            </option>
          ))}
        </select>

        <select
          aria-label="Tipo de Acorde"
          className="studio-select studio-select-sm"
          value={arranger.chord_mode ?? "fingered_3_7"}
          onChange={(e) =>
            onVoiceChange("split", null, null, null, null, null, null, null, e.target.value)
          }
          title="Define se acordes exigem 3 a 7 teclas (7ª, 9ª, 7/9...) ou aceitam 1 tecla"
        >
          <option value="fingered_3_7">Tipo de Acorde: 3 a 7 Teclas (Padrão)</option>
          <option value="single_finger">Tipo de Acorde: 1 Tecla (Automático)</option>
        </select>

        <div className="pitch-stepper-group">
          <span className="pitch-tag">Oitava Upper</span>
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Upper Octave Down"
            onClick={() =>
              onVoiceChange("split", null, null, null, null, null, arranger.upper_octave - 1)
            }
          >
            -
          </button>
          <strong className="pitch-val">
            {arranger.upper_octave > 0 ? `+${arranger.upper_octave}` : arranger.upper_octave}
          </strong>
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Upper Octave Up"
            onClick={() =>
              onVoiceChange("split", null, null, null, null, null, arranger.upper_octave + 1)
            }
          >
            +
          </button>
        </div>

        <div className="pitch-stepper-group">
          <span className="pitch-tag">Transpose</span>
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Transpose Down"
            onClick={() =>
              onVoiceChange("split", null, null, null, null, null, null, arranger.transpose - 1)
            }
          >
            -
          </button>
          <strong className="pitch-val">
            {arranger.transpose > 0 ? `+${arranger.transpose}` : arranger.transpose}
          </strong>
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Transpose Up"
            onClick={() =>
              onVoiceChange("split", null, null, null, null, null, null, arranger.transpose + 1)
            }
          >
            +
          </button>
        </div>
      </div>
    </section>
  );
}
