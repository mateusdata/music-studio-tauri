import { useMemo } from "react";
import { FolderOpen, Sliders } from "lucide-react";
import { ParameterSlider } from "./ParameterSlider";
import type { MusicInstrumentSummary } from "../types/engine";

const TRACK_LABELS = ["Drum", "Perc", "Bass", "Acc 1", "Acc 2", "Acc 3", "Acc 4", "Acc 5"];

export interface MixerPanelProps {
  masterVolumeDb: number;
  trackVolumes: number[];
  trackInstruments?: (number | null)[];
  instruments: MusicInstrumentSummary[];
  soundfontName?: string | null;
  onMasterVolumeChange: (nextDb: number) => void;
  onTrackVolumeChange: (trackIndex: number, volume: number) => void;
  onTrackInstrumentChange: (trackIndex: number, instrumentIndex: number | null) => void;
  onPickSoundfont: () => void;
}

/**
 * 8-track arranger mixer with SoundFont (.sf2) loader badge and per-track
 * volume faders + instrument override selectors.
 */
export function MixerPanel({
  masterVolumeDb,
  trackVolumes,
  trackInstruments,
  instruments,
  soundfontName,
  onMasterVolumeChange,
  onTrackVolumeChange,
  onTrackInstrumentChange,
  onPickSoundfont,
}: MixerPanelProps) {
  const playableInstruments = useMemo(
    () => instruments.filter((inst) => inst.has_samples || inst.zone_count > 0),
    [instruments],
  );

  // Place drumkits first on Drum/Perc tracks and melodic timbres first on Bass/Acc tracks.
  const drumOrderedInstruments = useMemo(
    () =>
      [...playableInstruments].sort(
        (a, b) => Number(b.is_drumkit) - Number(a.is_drumkit),
      ),
    [playableInstruments],
  );

  const melodicOrderedInstruments = useMemo(
    () =>
      [...playableInstruments].sort(
        (a, b) => Number(a.is_drumkit) - Number(b.is_drumkit),
      ),
    [playableInstruments],
  );

  return (
    <section className="panel-card arranger-mixer-card">
      <div className="panel-title">
        <span>Mixer de Trilhas &amp; SoundFont</span>
        <div className="panel-title-actions">
          {soundfontName && (
            <span
              className="badge-subtle sf2-badge"
              title={`SoundFont ativo: ${soundfontName}`}
            >
              {soundfontName}
            </span>
          )}
          <button
            type="button"
            className="btn btn-xs btn-sty-soft"
            aria-label="Importar SoundFont .SF2"
            onClick={onPickSoundfont}
            title="Carregar SoundFont (.sf2) para ritmos .STY e timbres"
          >
            <FolderOpen size={12} /> + .SF2
          </button>
          <Sliders size={14} />
        </div>
      </div>

      <ParameterSlider
        label="Master Volume"
        value={masterVolumeDb}
        min={-36}
        max={6}
        step={0.5}
        unit="dB"
        onChange={onMasterVolumeChange}
      />

      <div className="track-faders-grid">
        {TRACK_LABELS.map((label, slot) => {
          const vol = trackVolumes[slot] ?? 1;
          const selectedInst = trackInstruments?.[slot] ?? null;
          const trackOptions = slot <= 1 ? drumOrderedInstruments : melodicOrderedInstruments;

          return (
            <div key={label} className="track-fader-col">
              <span className="track-fader-val">{Math.round(vol * 100)}%</span>
              <input
                type="range"
                aria-label={`${label} Volume`}
                className="studio-range"
                min={0}
                max={1.25}
                step={0.02}
                value={vol}
                onChange={(e) => onTrackVolumeChange(slot, Number(e.target.value))}
              />
              <span className="track-fader-name">{label}</span>
              <select
                aria-label={`${label} Instrument`}
                className="studio-select studio-select-xs track-inst-select"
                value={selectedInst !== null ? String(selectedInst) : ""}
                onChange={(e) => {
                  const raw = e.target.value;
                  onTrackInstrumentChange(slot, raw === "" ? null : Number(raw));
                }}
                title={`Selecionar timbre para ${label}`}
              >
                <option value="">Padrão do Ritmo</option>
                {trackOptions.map((inst) => (
                  <option key={inst.index} value={String(inst.index)}>
                    {inst.is_drumkit ? `[Kit] ${inst.name}` : inst.name}
                  </option>
                ))}
              </select>
            </div>
          );
        })}
      </div>
    </section>
  );
}
