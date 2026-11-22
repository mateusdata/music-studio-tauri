import { useMemo, useState } from "react";
import {
  Activity,
  Disc,
  FileMusic,
  FolderOpen,
  Minus,
  Plus,
  Settings2,
  Volume2,
} from "lucide-react";
import { SearchableSelect } from "./SearchableSelect";
import type {
  EngineTelemetry,
  MidiPortInfo,
  MusicPackageEntry,
  MusicStyleSummary,
} from "../types/engine";

export type LibraryFormatFilter = "all" | "music" | "sty";

export interface ArrangerTopbarProps {
  packages: MusicPackageEntry[];
  activePackagePath: string;
  styles: MusicStyleSummary[];
  activeStyleIndex: number;
  midiPorts: MidiPortInfo[];
  selectedMidiPort: string;
  masterVolumeDb: number;
  bufferSize: number;
  telemetry: EngineTelemetry | null;
  onLoadPackage: (filePath: string) => void;
  onSelectStyle: (styleIndex: number) => void;
  onPickMusicDialog: () => void;
  onPickStyDialog: () => void;
  onPickStyFolderDialog: () => void;
  onMidiConnect: (portName: string) => void;
  onExportWav: () => void;
  onOpenLogs: () => void;
  onVelocitySettings: () => void;
  velocitySettingsReady: boolean;
  onResetInstruments: () => void;
  onSaveConfiguration: () => void;
  onClearConfiguration: () => void;
  onScanDuplicates: () => void;
  toolsBusy: boolean;
  bufferBusy: boolean;
  onMasterVolumeChange: (nextDb: number) => void;
  onBufferSizeChange: (nextSize: number) => void;
}

const FORMAT_FILTER_OPTIONS: { id: LibraryFormatFilter; label: string }[] = [
  { id: "all", label: "Todos" },
  { id: "music", label: ".Music" },
  { id: "sty", label: ".STY" },
];

function getPackageFormat(pkg: MusicPackageEntry): "music" | "sty" {
  if (pkg.format) {
    return pkg.format.toLowerCase().includes("sty") ? "sty" : "music";
  }
  return pkg.file_name.toLowerCase().endsWith(".sty") ? "sty" : "music";
}

function formatPackageSublabel(pkg: MusicPackageEntry): string {
  const isSty = getPackageFormat(pkg) === "sty";
  const sizeStr =
    pkg.size_mb > 0
      ? pkg.size_mb < 0.1
        ? `${Math.max(1, Math.round(pkg.size_mb * 1024))} KB`
        : `${pkg.size_mb.toFixed(1)} MB`
      : "";

  if (isSty) {
    return sizeStr ? `[STY] • ${sizeStr}` : "[STY]";
  }
  const versionTag = pkg.version ? `[Music v${pkg.version}]` : "[Music]";
  return sizeStr ? `${versionTag} • ${sizeStr}` : versionTag;
}

/**
 * Compact single-line arranger header with library format filters, package/style
 * selectors, .Music / .STY import triggers, MIDI/WAV controls, and audio telemetry.
 */
export function ArrangerTopbar({
  packages,
  activePackagePath,
  styles,
  activeStyleIndex,
  midiPorts,
  selectedMidiPort,
  masterVolumeDb,
  bufferSize,
  telemetry,
  onLoadPackage,
  onSelectStyle,
  onPickMusicDialog,
  onPickStyDialog,
  onPickStyFolderDialog,
  onMidiConnect,
  onExportWav,
  onOpenLogs,
  onVelocitySettings,
  velocitySettingsReady,
  onResetInstruments,
  onSaveConfiguration,
  onClearConfiguration,
  onScanDuplicates,
  toolsBusy,
  bufferBusy,
  onMasterVolumeChange,
  onBufferSizeChange,
}: ArrangerTopbarProps) {
  const [formatFilter, setFormatFilter] = useState<LibraryFormatFilter>("all");

  const packageOptions = useMemo(() => {
    return packages
      .filter((pkg) => formatFilter === "all" || getPackageFormat(pkg) === formatFilter)
      .map((pkg) => ({
        value: pkg.file_path,
        label: pkg.file_name,
        sublabel: formatPackageSublabel(pkg),
      }));
  }, [packages, formatFilter]);

  const styleOptions = useMemo(
    () =>
      styles.map((st, idx) => ({
        value: String(idx),
        label: st.name,
        sublabel: st.category,
      })),
    [styles],
  );

  const peakPct = Math.min(
    100,
    Math.round(Math.max(telemetry?.peak_l ?? 0, telemetry?.peak_r ?? 0) * 100),
  );

  return (
    <header className="topbar">
      <div className="brand-block">
        <div className="brand-badge">MS</div>
        <div>
          <div className="brand-title">Music Studio Tauri</div>
          <div className="brand-sub">
            {telemetry?.audio_backend ?? ".Music Arranger Engine"}
          </div>
        </div>
      </div>

      <div className="selector-row">
        <div
          className="format-filter-pills"
          role="group"
          aria-label="Filtro de formato da biblioteca"
        >
          {FORMAT_FILTER_OPTIONS.map((opt) => (
            <button
              key={opt.id}
              type="button"
              aria-pressed={formatFilter === opt.id}
              className={`format-filter-pill ${formatFilter === opt.id ? "active" : ""}`}
              onClick={() => setFormatFilter(opt.id)}
            >
              {opt.label}
            </button>
          ))}
        </div>

        <SearchableSelect
          ariaLabel="Music Package Selector"
          value={activePackagePath}
          onChange={onLoadPackage}
          searchPlaceholder="Pesquisar pacote .Music..."
          options={packageOptions}
        />

        <SearchableSelect
          ariaLabel="Style Selector"
          value={String(activeStyleIndex)}
          onChange={(val) => {
            const idx = Number(val);
            if (Number.isFinite(idx)) {
              onSelectStyle(idx);
            }
          }}
          searchPlaceholder="Pesquisar ritmo..."
          options={styleOptions}
        />

        <button
          type="button"
          className="btn btn-primary"
          onClick={onPickMusicDialog}
          title="Abrir pasta do Linux para importar .Music"
        >
          <FolderOpen size={13} /> Importar .Music
        </button>

        <button
          type="button"
          className="btn btn-sty"
          aria-label="Importar .STY"
          onClick={onPickStyDialog}
          title="Importar um ou mais arquivos .STY (Yamaha)"
        >
          <FileMusic size={13} /> Importar .STY
        </button>

        <button
          type="button"
          className="btn btn-sty"
          aria-label="Pasta .STY"
          onClick={onPickStyFolderDialog}
          title="Carregar pasta inteira de ritmos .STY"
        >
          <FolderOpen size={13} /> Pasta .STY
        </button>

        <select
          aria-label="MIDI Input"
          className="studio-select studio-select-sm topbar-midi-select"
          value={selectedMidiPort}
          onChange={(e) => onMidiConnect(e.target.value)}
        >
          <option value="">-- MIDI In --</option>
          {midiPorts.map((p) => (
            <option key={p.index} value={p.name}>
              {p.name}
            </option>
          ))}
        </select>

        <details className="tools-menu">
          <summary className="btn" aria-label="Ferramentas"><Settings2 size={14} /> Opções</summary>
          <div className="tools-menu-content">
            <button type="button" className="btn" onClick={onOpenLogs}>Logs</button>
            <button type="button" className="btn" disabled={!velocitySettingsReady} onClick={onVelocitySettings}>Curva de velocidade</button>
            <button type="button" className="btn" disabled={toolsBusy} onClick={onResetInstruments}>Resetar timbres</button>
            <button type="button" className="btn" disabled={toolsBusy} onClick={onSaveConfiguration}>Salvar configuração atual</button>
            <button type="button" className="btn" disabled={toolsBusy} onClick={onClearConfiguration}>Remover configuração salva</button>
            <button type="button" className="btn" disabled={toolsBusy} onClick={onScanDuplicates}>Limpar ritmos duplicados</button>
            <button type="button" className="btn" onClick={onExportWav}><Disc size={13} /> Exportar WAV</button>
          </div>
        </details>
      </div>

      <div className="telemetry-strip">
        <div className="meter-pill volume-pill" role="group" aria-label="Master Volume Controls">
          <Volume2 size={13} color="#60a5fa" />
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Diminuir Volume"
            onClick={() => onMasterVolumeChange(masterVolumeDb - 1.5)}
            title="Diminuir Volume Master"
          >
            <Minus size={11} />
          </button>
          <span className="vol-readout-pill">
            {masterVolumeDb > 0 ? `+${masterVolumeDb.toFixed(1)}` : masterVolumeDb.toFixed(1)} dB
          </span>
          <button
            type="button"
            className="btn btn-xs"
            aria-label="Aumentar Volume"
            onClick={() => onMasterVolumeChange(masterVolumeDb + 1.5)}
            title="Aumentar Volume Master"
          >
            <Plus size={11} />
          </button>
          <div className="vu-bar-wrap">
            <div className="vu-bar-fill" style={{ width: `${peakPct}%` }} />
          </div>
        </div>

        <div className="meter-pill latency-pill">
          <select
            aria-label="Audio Buffer Size"
            className="studio-select studio-select-sm latency-select"
            disabled={bufferBusy}
            value={String(bufferSize)}
            onChange={(e) => onBufferSizeChange(Number(e.target.value))}
            title="Buffer estrito em frames: sem dobrar, arredondar ou tentar outro tamanho. Se rejeitado, a saída fica parada com erro nos Logs."
          >
            <option value="64">64 buf</option>
            <option value="128">128 buf</option>
            <option value="256">256 buf</option>
            <option value="512">512 buf</option>
            <option value="1024">1024 buf</option>
          </select>
          <span className="latency-readout" title={`Duração do callback observado: ${telemetry?.buffer_size ?? 0} frames ÷ ${telemetry?.sample_rate ?? 0} Hz × 1000. Não mede latência total tecla → som. Detalhes em Opções → Logs.`}>
            {telemetry?.audio_active && telemetry.buffer_size > 0 && telemetry.sample_rate > 0 ? `${(telemetry.buffer_size / telemetry.sample_rate * 1000).toFixed(1)} ms` : "— ms"}
          </span>
        </div>

        <div className="meter-pill voices-pill">
          <Activity size={13} color="#48b8d0" />
          <span>{telemetry?.active_voices ?? 0} voices</span>
        </div>
      </div>
    </header>
  );
}
