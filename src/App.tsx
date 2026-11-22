import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Activity,
  Cable,
  Disc,
  FolderOpen,
  Music2,
  Play,
  Sliders,
  Square,
  Volume2,
} from "lucide-react";
import { engineClient } from "./services/engineClient";
import { SearchableSelect } from "./components/SearchableSelect";
import { ParameterSlider } from "./components/ParameterSlider";
import { VirtualKeyboard } from "./components/VirtualKeyboard";
import type {
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiPortInfo,
  MusicPackageEntry,
  MusicPackageSummary,
} from "./types/engine";

const SECTION_ROWS: { label: string; items: string[] }[] = [
  { label: "Intro", items: ["Intro 1", "Intro 2", "Intro 3"] },
  { label: "Variation", items: ["Variation 1", "Variation 2", "Variation 3", "Variation 4"] },
  { label: "Fill / Break", items: ["Fill 1", "Fill 2", "Fill 3", "Fill 4", "Break"] },
  { label: "Ending", items: ["Ending 1", "Ending 2", "Ending 3"] },
];

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

const TRACK_LABELS = ["Drum", "Perc", "Bass", "Acc 1", "Acc 2", "Acc 3", "Acc 4", "Acc 5"];

const SPLIT_POINTS: { value: string; label: string }[] = [
  { value: "41", label: "1º Fá (F2 • MIDI 41)" },
  { value: "48", label: "Dó 3 (C3 • MIDI 48)" },
  { value: "53", label: "2º Fá (F3 • Padrão)" },
  { value: "55", label: "Sol 3 (G3 • MIDI 55)" },
  { value: "59", label: "Si 3 (B3 • MIDI 59)" },
  { value: "60", label: "Dó 4 (C4 • MIDI 60)" },
  { value: "65", label: "Fá 4 (F4 • MIDI 65)" },
];

function createPreviewArrangerState(): MusicPackageSummary {
  return {
    loaded: true,
    file_name: "Josiran_Pisadinha_GR.Music",
    file_path: "preview://Josiran_Pisadinha_GR.Music",
    version: "1.4.4",
    instruments: [
      { index: 0, name: "Metronome / Internal FX", zone_count: 23, has_samples: true, is_drumkit: true },
      { index: 1, name: "Strings (Stereo)", zone_count: 7, has_samples: true, is_drumkit: false },
      { index: 2, name: "BTR JOSIRAN PISADINHA", zone_count: 33, has_samples: true, is_drumkit: true },
      { index: 3, name: "BAIXO SLAP (TM)22", zone_count: 17, has_samples: true, is_drumkit: false },
      { index: 4, name: "Gtr Vintange Pa50", zone_count: 18, has_samples: true, is_drumkit: false },
      { index: 7, name: "BANDONEON VA7 ST SOLO", zone_count: 12, has_samples: true, is_drumkit: false },
      { index: 8, name: "LEAD SUPER (TM)", zone_count: 3, has_samples: true, is_drumkit: false },
    ],
    styles: [
      {
        id: 1,
        name: "Josiran Pisadinha (GR)",
        category: "GABRIEL RODRIGUES GR",
        sections: [
          "Intro 1",
          "Intro 2",
          "Intro 3",
          "Variation 1",
          "Variation 2",
          "Variation 3",
          "Variation 4",
          "Fill 1",
          "Fill 2",
          "Fill 3",
          "Fill 4",
          "Ending 1",
          "Ending 2",
        ],
      },
    ],
    active_style_index: 0,
    active_section: "Variation 1",
    pending_section: null,
    upper1_instrument: 7,
    upper2_instrument: null,
    lower_instrument: null,
    split_key: 53,
    split_enabled: true,
    chord_scan_lower: true,
    chord_scan_upper: false,
    upper_octave: 0,
    transpose: 0,
    bpm: 85,
    rhythm_playing: false,
    sync_start: true,
    auto_fill: true,
    detected_chord_root: 0,
    detected_chord_quality: "Major",
    detected_chord: "C",
    track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
    beat_position: 0,
  };
}

function createPreviewPackages(): MusicPackageEntry[] {
  return [
    {
      file_name: "Josiran_Pisadinha_GR.Music",
      file_path: "preview://Josiran_Pisadinha_GR.Music",
      size_mb: 23.4,
      version: "1.4.4",
    },
    {
      file_name: "Arrasta_Pe_Forro_GR.Music",
      file_path: "preview://Arrasta_Pe_Forro_GR.Music",
      size_mb: 38.8,
      version: "1.4.4",
    },
    {
      file_name: "Bonde_do_Brasil_TM.Music",
      file_path: "preview://Bonde_do_Brasil_TM.Music",
      size_mb: 72.5,
      version: "1.3.0",
    },
  ];
}

export function App() {
  const [packages, setPackages] = useState<MusicPackageEntry[]>(() =>
    engineClient.isTauri() ? [] : createPreviewPackages(),
  );
  const [arranger, setArranger] = useState<MusicPackageSummary>(() =>
    engineClient.isTauri()
      ? {
          loaded: false,
          file_name: "",
          file_path: "",
          version: "",
          instruments: [],
          styles: [],
          active_style_index: 0,
          active_section: "Variation 1",
          pending_section: null,
          upper1_instrument: 0,
          upper2_instrument: null,
          lower_instrument: null,
          split_key: 53,
          split_enabled: true,
          chord_scan_lower: true,
          chord_scan_upper: false,
          upper_octave: 0,
          transpose: 0,
          bpm: 120,
          rhythm_playing: false,
          sync_start: true,
          auto_fill: true,
          detected_chord_root: 0,
          detected_chord_quality: "Major",
          detected_chord: "C",
          track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
          beat_position: 0,
        }
      : createPreviewArrangerState(),
  );

  const [snapshot, setSnapshot] = useState<EngineSnapshot | null>(null);
  const [telemetry, setTelemetry] = useState<EngineTelemetry | null>(null);
  const [midiPorts, setMidiPorts] = useState<MidiPortInfo[]>([]);
  const [selectedMidiPort, setSelectedMidiPort] = useState<string>("");
  const [sustainOn, setSustainOn] = useState(false);
  const [showCustomImport, setShowCustomImport] = useState(false);
  const [customPathInput, setCustomPathInput] = useState("");
  const [statusMsg, setStatusMsg] = useState<string>(() =>
    engineClient.isTauri()
      ? "Pronto"
      : "Modo preview (execute com Tauri para carregar .Music e áudio nativo)",
  );

  useEffect(() => {
    if (!engineClient.isTauri()) return;
    let alive = true;

    Promise.all([
      engineClient.listMusicPackages(),
      engineClient.getArrangerState(),
      engineClient.getSnapshot(),
      engineClient.listMidiDevices(),
    ])
      .then(([pkgs, arrState, snap, ports]) => {
        if (!alive) return;
        setPackages(pkgs);
        setArranger(arrState);
        setSnapshot(snap);
        setMidiPorts(ports);
        setSelectedMidiPort(snap.midi_config.selected_port ?? "");
        if (arrState.loaded) {
          setStatusMsg(
            `Carregado ${arrState.file_name} (${arrState.instruments.length} timbres, v${arrState.version})`,
          );
        }
      })
      .catch((err) => {
        if (alive) setStatusMsg(`Erro ao inicializar: ${String(err)}`);
      });

    const interval = setInterval(() => {
      engineClient.getTelemetry().then((t) => {
        if (alive) setTelemetry(t);
      }).catch(() => {});
      engineClient.getArrangerState().then((s) => {
        if (alive && s.loaded) setArranger(s);
      }).catch(() => {});
    }, 100);

    const midiPoll = setInterval(() => {
      engineClient.listMidiDevices().then((p) => {
        if (alive) setMidiPorts(p);
      }).catch(() => {});
    }, 2500);

    return () => {
      alive = false;
      clearInterval(interval);
      clearInterval(midiPoll);
    };
  }, []);

  const params = snapshot?.params ?? null;

  const updateParams = useCallback(
    (mutator: (draft: EngineParams) => void) => {
      if (!snapshot) return;
      const next: EngineParams = structuredClone(snapshot.params);
      mutator(next);
      setSnapshot({ ...snapshot, params: next });
      if (engineClient.isTauri()) {
        engineClient.updateParams(next).catch(() => {});
      }
    },
    [snapshot],
  );

  const handleLoadPackage = useCallback(async (filePath: string) => {
    if (!filePath) return;
    if (!engineClient.isTauri()) {
      const name = filePath.split("/").pop() ?? filePath;
      setArranger((prev) => ({ ...prev, file_name: name, file_path: filePath }));
      setStatusMsg(`Preview selecionado: ${name}`);
      return;
    }
    setStatusMsg("Importando pacote .Music...");
    try {
      const nextState = await engineClient.loadMusicPackage(filePath);
      setArranger(nextState);
      setStatusMsg(
        `Carregado ${nextState.file_name}: ${nextState.instruments.length} timbres e ${nextState.styles.length} estilo(s)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao carregar .Music: ${String(err)}`);
    }
  }, []);

  const handlePickFileDialog = async () => {
    if (!engineClient.isTauri()) {
      setShowCustomImport((v) => !v);
      return;
    }
    setStatusMsg("Abrindo explorador de arquivos...");
    try {
      const nextState = await engineClient.pickMusicPackageDialog();
      if (!nextState) {
        setStatusMsg("Importação cancelada.");
        return;
      }
      setArranger(nextState);
      setPackages((prev) => {
        if (prev.some((p) => p.file_path === nextState.file_path)) return prev;
        return [
          {
            file_name: nextState.file_name,
            file_path: nextState.file_path,
            size_mb: 0,
            version: nextState.version,
          },
          ...prev,
        ];
      });
      setStatusMsg(
        `Carregado ${nextState.file_name}: ${nextState.instruments.length} timbres e ${nextState.styles.length} estilo(s)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao abrir arquivo .Music: ${String(err)}`);
    }
  };

  const handleCustomImportSubmit = async () => {
    const trimmed = customPathInput.trim();
    if (!trimmed) return;
    await handleLoadPackage(trimmed);
    setShowCustomImport(false);
  };

  const handleSelectStyle = useCallback(async (val: string) => {
    const idx = Number(val);
    if (!Number.isFinite(idx)) return;
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, active_style_index: idx }));
      return;
    }
    const nextState = await engineClient.setArrangerStyle(idx);
    setArranger(nextState);
  }, []);

  const handleSectionClick = useCallback(async (sectionName: string) => {
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, active_section: sectionName, pending_section: null }));
      return;
    }
    const nextState = await engineClient.setArrangerSection(sectionName, false);
    setArranger(nextState);
  }, []);

  const handleTogglePlay = useCallback(async () => {
    const nextPlay = !arranger.rhythm_playing;
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, rhythm_playing: nextPlay }));
      return;
    }
    const nextState = await engineClient.setArrangerTransport(nextPlay, null, null, null);
    setArranger(nextState);
  }, [arranger.rhythm_playing]);

  const handleTransportFlag = useCallback(
    async (field: "sync_start" | "auto_fill", val: boolean) => {
      if (!engineClient.isTauri()) {
        setArranger((prev) => ({ ...prev, [field]: val }));
        return;
      }
      const nextState = await engineClient.setArrangerTransport(
        arranger.rhythm_playing,
        field === "sync_start" ? val : null,
        null,
        field === "auto_fill" ? val : null,
      );
      setArranger(nextState);
    },
    [arranger.rhythm_playing],
  );

  const handleBpmChange = useCallback(
    async (bpmVal: number) => {
      const clamped = Math.max(40, Math.min(240, Math.round(bpmVal)));
      if (!engineClient.isTauri()) {
        setArranger((prev) => ({ ...prev, bpm: clamped }));
        return;
      }
      const nextState = await engineClient.setArrangerTransport(
        arranger.rhythm_playing,
        null,
        clamped,
        null,
      );
      setArranger(nextState);
    },
    [arranger.rhythm_playing],
  );

  const handleVoiceChange = useCallback(
    async (
      slot: "upper1" | "upper2" | "lower" | "split",
      instIdx: number | null,
      splitKey: number | null = null,
      splitEnabled: boolean | null = null,
      chordScanLower: boolean | null = null,
      chordScanUpper: boolean | null = null,
      upperOctave: number | null = null,
      transpose: number | null = null,
    ) => {
      if (!engineClient.isTauri()) {
        setArranger((prev) => ({
          ...prev,
          upper1_instrument: slot === "upper1" && instIdx !== null ? instIdx : prev.upper1_instrument,
          upper2_instrument: slot === "upper2" ? instIdx : prev.upper2_instrument,
          lower_instrument: slot === "lower" ? instIdx : prev.lower_instrument,
          split_key: splitKey ?? prev.split_key,
          split_enabled: splitEnabled ?? prev.split_enabled,
          chord_scan_lower: chordScanLower ?? prev.chord_scan_lower,
          chord_scan_upper: chordScanUpper ?? prev.chord_scan_upper,
          upper_octave: upperOctave ?? prev.upper_octave,
          transpose: transpose ?? prev.transpose,
        }));
        return;
      }
      const nextState = await engineClient.setArrangerKeyboardVoice(
        slot,
        instIdx,
        splitKey,
        splitEnabled,
        chordScanLower,
        chordScanUpper,
        upperOctave,
        transpose,
      );
      setArranger(nextState);
    },
    [],
  );

  const handleChordClick = useCallback(async (root: number, quality: string, label: string) => {
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({
        ...prev,
        detected_chord_root: root,
        detected_chord_quality: quality,
        detected_chord: label,
      }));
      return;
    }
    const nextState = await engineClient.setArrangerChord(root, quality);
    setArranger(nextState);
  }, []);

  const handleTrackVolume = useCallback(async (slot: number, vol: number) => {
    if (!engineClient.isTauri()) {
      setArranger((prev) => {
        const nextVols = [...prev.track_volumes];
        nextVols[slot] = vol;
        return { ...prev, track_volumes: nextVols };
      });
      return;
    }
    const nextState = await engineClient.setArrangerTrackVolume(slot, vol);
    setArranger(nextState);
  }, []);

  const handleMidiConnect = async (portName: string) => {
    setSelectedMidiPort(portName);
    if (!portName || !engineClient.isTauri()) return;
    try {
      const msg = await engineClient.connectMidiDevice(portName);
      setStatusMsg(`MIDI conectado: ${msg}`);
    } catch (err) {
      setStatusMsg(`Erro MIDI: ${String(err)}`);
    }
  };

  const handleToggleSustain = () => {
    const next = !sustainOn;
    setSustainOn(next);
    if (engineClient.isTauri()) {
      engineClient.setPedal("sustain", next ? 1.0 : 0.0).catch(() => {});
    }
  };

  const handlePanic = () => {
    if (engineClient.isTauri()) {
      engineClient.allNotesOff().catch(() => {});
    }
  };

  const handleExportWav = async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Exportação WAV disponível no app Tauri");
      return;
    }
    try {
      const outPath = await engineClient.exportDemoWav();
      setStatusMsg(`WAV exportado em ${outPath}`);
    } catch (err) {
      setStatusMsg(`Erro ao exportar WAV: ${String(err)}`);
    }
  };

  const activeStyle = arranger.styles[arranger.active_style_index] ?? arranger.styles[0] ?? null;
  const availableSections = useMemo(
    () => new Set(activeStyle?.sections ?? []),
    [activeStyle],
  );

  const playableInstrumentOptions = useMemo(() => {
    return arranger.instruments
      .filter((inst) => inst.has_samples || inst.zone_count > 0)
      .map((inst) => ({
        value: String(inst.index),
        label: inst.name,
        sublabel: `${inst.is_drumkit ? "Bateria/Kit" : "Timbre"} • ${inst.zone_count} zonas`,
      }));
  }, [arranger.instruments]);

  const optionalVoiceOptions = useMemo(
    () => [{ value: "", label: "-- Só Acorde / Mudo --" }, ...playableInstrumentOptions],
    [playableInstrumentOptions],
  );

  const peakPct = Math.min(
    100,
    Math.round(Math.max(telemetry?.peak_l ?? 0, telemetry?.peak_r ?? 0) * 100),
  );

  return (
    <div className="studio-shell">
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
          <SearchableSelect
            ariaLabel="Music Package Selector"
            value={arranger.file_path}
            onChange={handleLoadPackage}
            searchPlaceholder="Pesquisar pacote .Music..."
            options={packages.map((p) => ({
              value: p.file_path,
              label: p.file_name,
              sublabel: `v${p.version} • ${p.size_mb.toFixed(1)} MB`,
            }))}
          />

          <SearchableSelect
            ariaLabel="Style Selector"
            value={String(arranger.active_style_index)}
            onChange={handleSelectStyle}
            searchPlaceholder="Pesquisar ritmo..."
            options={arranger.styles.map((st, idx) => ({
              value: String(idx),
              label: st.name,
              sublabel: st.category,
            }))}
          />

          <button
            type="button"
            className="btn btn-primary"
            onClick={handlePickFileDialog}
            title="Abrir pasta do Linux para importar .Music"
          >
            <FolderOpen size={13} /> Importar .Music
          </button>

          <button
            type="button"
            className="btn btn-compact"
            aria-label="Caminho manual .Music"
            onClick={() => setShowCustomImport((v) => !v)}
            title="Digitar caminho manualmente"
          >
            Caminho
          </button>

          <select
            aria-label="MIDI Input"
            className="studio-select studio-select-sm"
            value={selectedMidiPort}
            onChange={(e) => handleMidiConnect(e.target.value)}
          >
            <option value="">-- MIDI In --</option>
            {midiPorts.map((p) => (
              <option key={p.index} value={p.name}>
                {p.name}
              </option>
            ))}
          </select>

          <button type="button" className="btn" onClick={handleExportWav}>
            <Disc size={13} /> WAV
          </button>
        </div>

        <div className="telemetry-strip">
          <div className="meter-pill">
            <Activity size={13} color="#e29d38" />
            <span>{telemetry?.active_voices ?? 0} voices</span>
          </div>
          <div className="meter-pill">
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

      {showCustomImport && (
        <div className="custom-import-bar">
          <input
            type="text"
            aria-label="Custom .Music File Path"
            className="studio-input custom-import-input"
            placeholder="/caminho/para/seu_arquivo.Music"
            value={customPathInput}
            onChange={(e) => setCustomPathInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleCustomImportSubmit();
            }}
          />
          <button type="button" className="btn btn-primary" onClick={handleCustomImportSubmit}>
            Carregar
          </button>
        </div>
      )}

      <main className="arranger-workspace">
        {/* ROW 1: 3-Column Hardware Arranger Console (Mixer | LCD & Chords | Voices & Split/Scan) */}
        <div className="arranger-console-grid">
          <section className="panel-card arranger-mixer-card">
            <div className="panel-title">
              <span>Mixer de Trilhas & Master</span>
              <Sliders size={14} />
            </div>

            {params && (
              <ParameterSlider
                label="Master Volume"
                value={params.volume_db}
                min={-36}
                max={6}
                step={0.5}
                unit="dB"
                onChange={(v) => updateParams((d) => { d.volume_db = v; })}
              />
            )}

            <div className="track-faders-grid">
              {TRACK_LABELS.map((label, slot) => (
                <label key={label} className="track-fader-col">
                  <span className="track-fader-val">
                    {Math.round((arranger.track_volumes[slot] ?? 1) * 100)}%
                  </span>
                  <input
                    type="range"
                    aria-label={`${label} Volume`}
                    className="studio-range"
                    min={0}
                    max={1.25}
                    step={0.02}
                    value={arranger.track_volumes[slot] ?? 1}
                    onChange={(e) => handleTrackVolume(slot, Number(e.target.value))}
                  />
                  <span className="track-fader-name">{label}</span>
                </label>
              ))}
            </div>
          </section>

          <section className="panel-card arranger-lcd-card">
            <div className="panel-title">
              <span>Visor do Arranjador (.Music)</span>
              <span className="badge-subtle">
                {arranger.loaded ? `v${arranger.version}` : "Sem pacote"}
              </span>
            </div>

            <div className="lcd-screen">
              <div className="lcd-main-row">
                <div className="lcd-info-col">
                  <div className="lcd-label">RITMO ATIVO</div>
                  <div className="lcd-style-name">{activeStyle?.name ?? arranger.file_name}</div>
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
                  style={{ width: `${Math.round(arranger.beat_position * 100)}%` }}
                />
              </div>

              <div className="lcd-tempo-row">
                <span className="lcd-label">TEMPO</span>
                <div className="tempo-stepper">
                  <button
                    type="button"
                    className="btn btn-xs"
                    aria-label="Decrease Tempo"
                    onClick={() => handleBpmChange(arranger.bpm - 2)}
                  >
                    -
                  </button>
                  <strong className="tempo-readout">{Math.round(arranger.bpm)} BPM</strong>
                  <button
                    type="button"
                    className="btn btn-xs"
                    aria-label="Increase Tempo"
                    onClick={() => handleBpmChange(arranger.bpm + 2)}
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
                  onChange={(e) => handleBpmChange(Number(e.target.value))}
                />
              </div>
            </div>

            <div className="chord-pads-section">
              <div className="chord-pads-label">Acorde Rápido (ou toque na esquerda / Chord Scan):</div>
              <div className="chord-pads-grid" role="group" aria-label="Quick Chords">
                {QUICK_CHORDS.map((c) => (
                  <button
                    key={c.label}
                    type="button"
                    className={`chord-pad-btn ${
                      arranger.detected_chord === c.label ? "active" : ""
                    }`}
                    onClick={() => handleChordClick(c.root, c.quality, c.label)}
                  >
                    {c.label}
                  </button>
                ))}
              </div>
            </div>
          </section>

          <section className="panel-card arranger-voices-card">
            <div className="panel-title">
              <span>Timbres, Split Divider & Chord Scan</span>
              <Music2 size={14} />
            </div>

            <div className="voice-slots-grid">
              <div className="voice-slot-box">
                <span className="voice-tag voice-tag-u1">UPPER 1 (Solo)</span>
                <SearchableSelect
                  ariaLabel="Upper 1 Instrument"
                  value={String(arranger.upper1_instrument)}
                  onChange={(val) => handleVoiceChange("upper1", Number(val))}
                  searchPlaceholder="Buscar timbre Upper 1..."
                  options={playableInstrumentOptions}
                />
              </div>

              <div className="voice-slot-box">
                <span className="voice-tag voice-tag-u2">UPPER 2 (Camada)</span>
                <SearchableSelect
                  ariaLabel="Upper 2 Instrument"
                  value={arranger.upper2_instrument !== null ? String(arranger.upper2_instrument) : ""}
                  onChange={(val) =>
                    handleVoiceChange("upper2", val === "" ? null : Number(val))
                  }
                  searchPlaceholder="Buscar timbre Upper 2..."
                  options={optionalVoiceOptions}
                />
              </div>

              <div className="voice-slot-box">
                <span className="voice-tag voice-tag-low">LOWER (Esq.)</span>
                <SearchableSelect
                  ariaLabel="Lower Instrument"
                  value={arranger.lower_instrument !== null ? String(arranger.lower_instrument) : ""}
                  onChange={(val) =>
                    handleVoiceChange("lower", val === "" ? null : Number(val))
                  }
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
                onClick={() =>
                  handleVoiceChange("split", null, null, !arranger.split_enabled)
                }
              >
                <span className={`led-dot ${arranger.split_enabled ? "led-red" : ""}`} />
                SPLIT DIVIDER
              </button>

              <button
                type="button"
                aria-pressed={arranger.chord_scan_lower}
                className={`btn btn-compact ${arranger.chord_scan_lower ? "btn-flag-on" : ""}`}
                onClick={() =>
                  handleVoiceChange(
                    "split",
                    null,
                    null,
                    null,
                    !arranger.chord_scan_lower,
                  )
                }
              >
                CHORD LOWER
              </button>

              <button
                type="button"
                aria-pressed={arranger.chord_scan_upper}
                className={`btn btn-compact ${arranger.chord_scan_upper ? "btn-flag-on" : ""}`}
                onClick={() =>
                  handleVoiceChange(
                    "split",
                    null,
                    null,
                    null,
                    null,
                    !arranger.chord_scan_upper,
                  )
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
                onChange={(e) =>
                  handleVoiceChange("split", null, Number(e.target.value), null)
                }
              >
                {SPLIT_POINTS.map((sp) => (
                  <option key={sp.value} value={sp.value}>
                    Corte: {sp.label}
                  </option>
                ))}
              </select>

              <div className="pitch-stepper-group">
                <span className="pitch-tag">Oitava Upper</span>
                <button
                  type="button"
                  className="btn btn-xs"
                  aria-label="Upper Octave Down"
                  onClick={() =>
                    handleVoiceChange("split", null, null, null, null, null, arranger.upper_octave - 1)
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
                    handleVoiceChange("split", null, null, null, null, null, arranger.upper_octave + 1)
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
                    handleVoiceChange("split", null, null, null, null, null, null, arranger.transpose - 1)
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
                    handleVoiceChange("split", null, null, null, null, null, null, arranger.transpose + 1)
                  }
                >
                  +
                </button>
              </div>
            </div>
          </section>
        </div>

        {/* ROW 2: Hardware Intro / Variation / Fill / Ending & Start/Stop Strip */}
        <section className="panel-card arranger-rhythm-strip">
          <div className="rhythm-strip-top">
            <div className="arranger-transport-strip">
              <button
                type="button"
                className={`btn btn-start-stop ${arranger.rhythm_playing ? "playing" : ""}`}
                onClick={handleTogglePlay}
              >
                {arranger.rhythm_playing ? <Square size={14} /> : <Play size={14} />}
                <span>{arranger.rhythm_playing ? "STOP RITMO" : "START RITMO"}</span>
              </button>

              <button
                type="button"
                aria-pressed={arranger.sync_start}
                className={`btn ${arranger.sync_start ? "btn-flag-on" : ""}`}
                onClick={() => handleTransportFlag("sync_start", !arranger.sync_start)}
              >
                Synchro Start
              </button>

              <button
                type="button"
                aria-pressed={arranger.auto_fill}
                className={`btn ${arranger.auto_fill ? "btn-flag-on" : ""}`}
                onClick={() => handleTransportFlag("auto_fill", !arranger.auto_fill)}
              >
                Auto Fill
              </button>

              <button type="button" className="btn" onClick={handlePanic}>
                All Notes Off
              </button>
            </div>

            <div className="sections-matrix-inline">
              {SECTION_ROWS.map((row) => (
                <div key={row.label} className="section-group-inline">
                  <span className="section-row-tag">{row.label}</span>
                  <div className="section-buttons-row">
                    {row.items.map((secName) => {
                      const exists = availableSections.has(secName);
                      const isActive = arranger.active_section === secName;
                      const isPending = arranger.pending_section === secName;
                      return (
                        <button
                          key={secName}
                          type="button"
                          disabled={!exists && arranger.loaded}
                          className={`section-btn ${isActive ? "active" : ""} ${
                            isPending ? "pending" : ""
                          }`}
                          onClick={() => handleSectionClick(secName)}
                        >
                          {secName}
                        </button>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>
      </main>

      <footer className="keyboard-dock">
        <div className="pedals-bar">
          <div className="pedals-cluster">
            <button
              type="button"
              aria-pressed={sustainOn}
              className={`pedal-btn ${sustainOn ? "active" : ""}`}
              onClick={handleToggleSustain}
            >
              Sustain (Damper)
            </button>
            <div className="meter-pill">
              <Cable size={12} color={telemetry?.midi_connected ? "#3ac982" : "#9ba3b4"} />
              <span>
                {telemetry?.midi_connected && telemetry.midi_port_name
                  ? telemetry.midi_port_name
                  : "MIDI desconectado"}
              </span>
            </div>
          </div>

          <div className="status-banner">
            {statusMsg} • Teclado PC: Z–M (Acordes) / Q–P (Solo)
          </div>
        </div>

        <VirtualKeyboard
          activeNotes={telemetry?.active_notes ?? []}
          extendedRange={false}
          splitKey={arranger.split_key}
          splitEnabled={arranger.split_enabled}
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
