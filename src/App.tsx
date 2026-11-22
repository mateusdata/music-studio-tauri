import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { engineClient } from "./services/engineClient";
import { ArrangerTopbar } from "./components/ArrangerTopbar";
import { MixerPanel } from "./components/MixerPanel";
import { LogsDialog, type StudioLogEntry } from "./components/LogsDialog";
import { LcdPanel, resolveStyWarnings } from "./components/LcdPanel";
import { VoicesSplitPanel } from "./components/VoicesSplitPanel";
import { DEFAULT_VELOCITY_CURVE, VelocitySettings } from "./components/VelocitySettings";
import { VirtualKeyboard } from "./components/VirtualKeyboard";
import type {
  CleanupReport,
  DuplicateScan,
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiConfig,
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

function createPreviewArrangerState(): MusicPackageSummary {
  return {
    loaded: true,
    file_name: "Josiran_Pisadinha_GR.Music",
    file_path: "preview://Josiran_Pisadinha_GR.Music",
    version: "1.4.4",
    source_format: "music",
    soundfont_name: "FluidR3_GM.sf2",
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
    split_key: 54,
    split_enabled: true,
    chord_scan_lower: true,
    chord_scan_upper: false,
    upper_octave: 0,
    transpose: 0,
    bpm: 85,
    tempo_locked: false,
    rhythm_playing: false,
    sync_start: true,
    auto_fill: true,
    detected_chord_root: 0,
    detected_chord_quality: "Major",
    detected_chord: "C",
    chord_mode: "fingered_3_7",
    track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
    track_instruments: [null, null, null, null, null, null, null, null],
    ots_presets: [],
    beat_position: 0,
  };
}

function createInitialTauriArrangerState(): MusicPackageSummary {
  return {
    loaded: false,
    file_name: "",
    file_path: "",
    version: "",
    source_format: "music",
    soundfont_name: null,
    instruments: [],
    styles: [],
    active_style_index: 0,
    active_section: "Variation 1",
    pending_section: null,
    upper1_instrument: 0,
    upper2_instrument: null,
    lower_instrument: null,
    split_key: 54,
    split_enabled: true,
    chord_scan_lower: true,
    chord_scan_upper: false,
    upper_octave: 0,
    transpose: 0,
    bpm: 120,
    tempo_locked: false,
    rhythm_playing: false,
    sync_start: true,
    auto_fill: true,
    detected_chord_root: 0,
    detected_chord_quality: "Major",
    detected_chord: "C",
    chord_mode: "fingered_3_7",
    track_volumes: [1, 1, 1, 1, 1, 1, 1, 1],
    track_instruments: [null, null, null, null, null, null, null, null],
    ots_presets: [],
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
      format: "music",
    },
    {
      file_name: "Arrasta_Pe_Forro_GR.Music",
      file_path: "preview://Arrasta_Pe_Forro_GR.Music",
      size_mb: 38.8,
      version: "1.4.4",
      format: "music",
    },
    {
      file_name: "Bonde_do_Brasil_TM.Music",
      file_path: "preview://Bonde_do_Brasil_TM.Music",
      size_mb: 72.5,
      version: "1.3.0",
      format: "music",
    },
  ];
}

function mergePackageEntry(
  prev: MusicPackageEntry[],
  summary: MusicPackageSummary,
  format: "music" | "sty",
): MusicPackageEntry[] {
  if (!summary.file_path || prev.some((p) => p.file_path === summary.file_path)) {
    return prev;
  }
  return [
    {
      file_name: summary.file_name,
      file_path: summary.file_path,
      size_mb: 0,
      version: summary.version || (format === "sty" ? "STY" : ""),
      format,
    },
    ...prev,
  ];
}

export function App() {
  const [packages, setPackages] = useState<MusicPackageEntry[]>(() =>
    engineClient.isTauri() ? [] : createPreviewPackages(),
  );
  const [arranger, setArranger] = useState<MusicPackageSummary>(() =>
    engineClient.isTauri()
      ? createInitialTauriArrangerState()
      : createPreviewArrangerState(),
  );

  const [snapshot, setSnapshot] = useState<EngineSnapshot | null>(null);
  const [telemetry, setTelemetry] = useState<EngineTelemetry | null>(null);
  const [midiPorts, setMidiPorts] = useState<MidiPortInfo[]>([]);
  const [selectedMidiPort, setSelectedMidiPort] = useState<string>("");
  const [velocityOpen, setVelocityOpen] = useState(false);
  const [midiConfig, setMidiConfig] = useState<MidiConfig>({
    selected_port: null, channel: 0, microtonal_multichannel: false, transpose: 0,
    key_range_mode: "Full88", velocity_curve: DEFAULT_VELOCITY_CURVE.map(([x, y]) => [x, y]), cc_mappings: [],
  });
  const midiConfigRef = useRef(midiConfig);
  const midiWriteQueue = useRef<Promise<void>>(Promise.resolve());
  const [midiConfigReady, setMidiConfigReady] = useState(() => !engineClient.isTauri());
  const [sustainOn, setSustainOn] = useState(false);
  const [masterVolumeDb, setMasterVolumeDb] = useState<number>(-4.0);
  const [bufferSize, setBufferSize] = useState<number>(256);
  const [logsOpen, setLogsOpen] = useState(false);
  const [logs, setLogs] = useState<StudioLogEntry[]>([]);
  const logSequence = useRef(0);
  const appendLog = useCallback((message: string, source = "Aplicativo") => {
    if (!message.trim()) return;
    const entry: StudioLogEntry = {
      id: ++logSequence.current, time: new Date().toLocaleTimeString(), source, message,
      level: /erro|falha|invalid|disconnected|failed/i.test(message) ? "error" : "info",
    };
    setLogs((current) => [...current.slice(-199), entry]);
  }, []);
  const setStatusMsg = useCallback((message: string) => appendLog(message), [appendLog]);

  const [bufferBusy, setBufferBusy] = useState(false);
  const bufferPending = useRef(false);
  const bufferRevision = useRef(0);
  const arrangerRevision = useRef(0);
  const paramsEdited = useRef(false);
  const midiEdited = useRef(false);
  const [toolsBusy, setToolsBusy] = useState(false);
  const toolsPending = useRef(false);
  const [duplicateScan, setDuplicateScan] = useState<DuplicateScan | null>(null);
  const [cleanupReport, setCleanupReport] = useState<CleanupReport | null>(null);

  useEffect(() => {
    if (!engineClient.isTauri()) return;
    let alive = true;
    let telemetryPending = false;
    let arrangerPending = false;
    let snapshotPending = false;
    const startupError = (label: string, err: unknown) => {
      if (alive) setStatusMsg(`Erro ao inicializar ${label}: ${String(err)}`);
    };
    // Library discovery must never block transport, audio, MIDI, or the first frame.
    engineClient.listMusicPackages().then((pkgs) => {
      if (alive) setPackages((current) => [
        ...current, ...pkgs.filter((pkg) => !current.some((p) => p.file_path === pkg.file_path)),
      ]);
    }).catch((err) => startupError("biblioteca", err));
    const refreshSnapshot = () => {
      if (snapshotPending) return;
      snapshotPending = true;
      engineClient.getSnapshot().then((snap) => {
        if (!alive) return;
        setSnapshot((current) => paramsEdited.current && current ? current : snap);
        if (!paramsEdited.current) setMasterVolumeDb(snap.params.volume_db);
        if (!midiEdited.current) {
          midiConfigRef.current = snap.midi_config;
          setMidiConfig(snap.midi_config);
          setSelectedMidiPort(snap.midi_config.selected_port ?? "");
        }
        setMidiConfigReady(true);
      }).catch((err) => startupError("configuração", err))
        .finally(() => { snapshotPending = false; });
    };
    const refreshTelemetry = () => {
      if (telemetryPending || bufferPending.current) return;
      telemetryPending = true;
      const revision = bufferRevision.current;
      engineClient.getTelemetry().then((t) => {
        if (!alive || revision !== bufferRevision.current) return;
        setTelemetry(t);
        if (t.requested_buffer_size !== undefined) setBufferSize(t.requested_buffer_size);
      }).catch((err) => startupError("áudio", err))
        .finally(() => { telemetryPending = false; });
    };
    const refreshArranger = () => {
      if (arrangerPending) return;
      arrangerPending = true;
      const revision = arrangerRevision.current;
      engineClient.getArrangerState().then((s) => {
        if (alive && revision === arrangerRevision.current) setArranger(s);
      }).catch((err) => startupError("arranjador", err))
        .finally(() => { arrangerPending = false; });
    };
    const refreshMidi = () => engineClient.listMidiDevices().then((ports) => {
      if (alive) setMidiPorts(ports);
    }).catch((err) => startupError("MIDI", err));
    refreshSnapshot();
    refreshTelemetry();
    refreshArranger();
    refreshMidi();
    const interval = setInterval(() => { refreshTelemetry(); refreshArranger(); }, 100);
    // Background configuration/bootstrap can finish after the initial queries.
    const bootstrapPoll = setInterval(() => { refreshMidi(); refreshSnapshot(); }, 2500);
    return () => {
      alive = false;
      clearInterval(interval);
      clearInterval(bootstrapPoll);
    };
  }, []);

  useEffect(() => {
    if (telemetry?.audio_error) appendLog(telemetry.audio_error, "Áudio");
  }, [telemetry?.audio_error, appendLog]);
  useEffect(() => {
    if (telemetry?.configuration_error) appendLog(telemetry.configuration_error, "Configuração");
  }, [telemetry?.configuration_error, appendLog]);
  const importDiagnostics = [...resolveStyWarnings(arranger), arranger.sty_diagnostics_summary].filter(Boolean).join("\n");
  useEffect(() => {
    if (importDiagnostics) appendLog(`${arranger.file_name}\n${importDiagnostics}`, "Importação");
  }, [arranger.file_path, importDiagnostics, appendLog]);

  const updateParams = useCallback(
    (mutator: (draft: EngineParams) => void) => {
      paramsEdited.current = true;
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
    arrangerRevision.current++;
    if (!engineClient.isTauri()) {
      const name = filePath.split("/").pop() ?? filePath;
      const lower = name.toLowerCase();
      const isSty =
        lower.endsWith(".sty") ||
        lower.endsWith(".prs") ||
        lower.endsWith(".sst") ||
        lower.endsWith(".bcs");
      setArranger((prev) => ({
        ...prev,
        file_name: name,
        file_path: filePath,
        source_format: isSty ? "sty" : "music",
      }));
      setStatusMsg(`Preview selecionado: ${name}`);
      return;
    }
    setStatusMsg("Importando pacote...");
    try {
      const nextState = await engineClient.loadMusicPackage(filePath);
      arrangerRevision.current++;
      setArranger(nextState);
      setStatusMsg(
        `Carregado ${nextState.file_name}: ${nextState.instruments.length} timbres e ${nextState.styles.length} estilo(s)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao carregar pacote: ${String(err)}`);
    }
  }, []);

  const handlePickFileDialog = useCallback(async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Importador nativo disponível no aplicativo desktop Tauri");
      return;
    }
    arrangerRevision.current++;
    setStatusMsg("Abrindo explorador de arquivos...");
    try {
      const nextState = await engineClient.pickMusicPackageDialog();
      if (!nextState) {
        setStatusMsg("Importação cancelada.");
        return;
      }
      arrangerRevision.current++;
      setArranger(nextState);
      setPackages((prev) => mergePackageEntry(prev, nextState, "music"));
      setStatusMsg(
        `Carregado ${nextState.file_name}: ${nextState.instruments.length} timbres e ${nextState.styles.length} estilo(s)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao abrir arquivo .Music: ${String(err)}`);
    }
  }, []);

  const handlePickStyDialog = useCallback(async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Importador de ritmos .STY disponível no aplicativo desktop Tauri");
      return;
    }
    arrangerRevision.current++;
    setStatusMsg("Selecionando arquivo(s) .STY...");
    try {
      const nextState = await engineClient.pickStyDialog();
      if (!nextState) {
        setStatusMsg("Importação .STY cancelada.");
        return;
      }
      arrangerRevision.current++;
      setArranger(nextState);
      setPackages((prev) => mergePackageEntry(prev, nextState, "sty"));
      const sfInfo = nextState.soundfont_name ? ` • SF2: ${nextState.soundfont_name}` : "";
      setStatusMsg(
        `Carregado ${nextState.file_name}: ${nextState.styles.length} estilo(s) .STY${sfInfo}`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao abrir arquivo .STY: ${String(err)}`);
    }
  }, []);

  const handlePickStyFolderDialog = useCallback(async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Importador de pasta .STY disponível no aplicativo desktop Tauri");
      return;
    }
    arrangerRevision.current++;
    setStatusMsg("Selecionando pasta de ritmos .STY...");
    try {
      const nextState = await engineClient.pickStyFolderDialog();
      if (!nextState) {
        setStatusMsg("Seleção de pasta .STY cancelada.");
        return;
      }
      arrangerRevision.current++;
      setArranger(nextState);
      setPackages((prev) => mergePackageEntry(prev, nextState, "sty"));
      const sfInfo = nextState.soundfont_name ? ` • SF2: ${nextState.soundfont_name}` : "";
      setStatusMsg(
        `Pasta .STY carregada (${nextState.styles.length} ritmo(s))${sfInfo}`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao carregar pasta .STY: ${String(err)}`);
    }
  }, []);

  const handlePickSoundfontDialog = useCallback(async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Importador de SoundFont .SF2 disponível no aplicativo desktop Tauri");
      return;
    }
    arrangerRevision.current++;
    setStatusMsg("Selecionando SoundFont (.sf2)...");
    try {
      const nextState = await engineClient.pickSoundfontDialog();
      if (!nextState) {
        setStatusMsg("Seleção de SoundFont cancelada.");
        return;
      }
      arrangerRevision.current++;
      setArranger(nextState);
      setStatusMsg(
        `SoundFont carregado: ${nextState.soundfont_name ?? "SF2"} (${nextState.instruments.length} timbres)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao carregar SoundFont .SF2: ${String(err)}`);
    }
  }, []);

  const handlePickInstrumentDialog = useCallback(async () => {
    if (!engineClient.isTauri()) {
      setStatusMsg("Importador de timbres .Instrument disponível no aplicativo desktop Tauri");
      return;
    }
    arrangerRevision.current++;
    setStatusMsg("Selecionando timbre (.Instrument)...");
    try {
      const nextState = await engineClient.pickInstrumentDialog(true);
      if (!nextState) {
        setStatusMsg("Importação de timbre cancelada.");
        return;
      }
      arrangerRevision.current++;
      setArranger(nextState);
      const activeInst = nextState.instruments.find(
        (i) => i.index === nextState.upper1_instrument,
      );
      setStatusMsg(
        `Timbre importado em Upper 1: ${activeInst?.name ?? "Instrument"} (${nextState.instruments.length} timbres no pacote)`,
      );
    } catch (err) {
      setStatusMsg(`Falha ao importar timbre .Instrument: ${String(err)}`);
    }
  }, []);

  const handleMasterVolumeChange = useCallback(
    (nextDb: number) => {
      const clamped = Math.max(-36, Math.min(6, Math.round(nextDb * 2) / 2));
      setMasterVolumeDb(clamped);
      updateParams((d) => {
        d.volume_db = clamped;
      });
    },
    [updateParams],
  );

  const handleBufferSizeChange = useCallback(async (nextSize: number) => {
    if (bufferPending.current || ![64, 128, 256, 512, 1024].includes(nextSize)) return;
    if (!engineClient.isTauri()) {
      setBufferSize(nextSize);
      return;
    }
    setBufferSize(nextSize);
    bufferPending.current = true;
    bufferRevision.current++;
    setBufferBusy(true);
    try {
      const nextTelemetry = await engineClient.setAudioBufferSize(nextSize);
      setTelemetry(nextTelemetry);
      setBufferSize(nextTelemetry.requested_buffer_size ?? nextSize);
      setStatusMsg(`Buffer estrito aceito pelo backend: ${nextSize} frames. Sem tamanho alternativo; o callback observado e a estimativa de saída são mostrados separadamente.`);

    } catch (err) {
      setTelemetry((current) => current ? { ...current, requested_buffer_size: nextSize, buffer_size: 0, latency_ms: 0, output_latency_ms: null, audio_active: false } : current);
      setStatusMsg(`Falha ao configurar buffer estrito ${nextSize}: ${String(err)}. Nenhum buffer alternativo foi solicitado.`);
      setLogsOpen(true);
    } finally {
      bufferPending.current = false;
      setBufferBusy(false);
    }
  }, []);

  const handleTempoLockToggle = useCallback(async () => {
    const nextLocked = !arranger.tempo_locked;
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, tempo_locked: nextLocked }));
      return;
    }
    const nextState = await engineClient.setArrangerTransport(
      arranger.rhythm_playing,
      null,
      null,
      null,
      nextLocked,
    );
    arrangerRevision.current++;
    setArranger(nextState);
  }, [arranger.rhythm_playing, arranger.tempo_locked]);

  const handleSelectStyle = useCallback(async (idx: number) => {
    if (!Number.isFinite(idx)) return;
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, active_style_index: idx }));
      return;
    }
    const nextState = await engineClient.setArrangerStyle(idx);
    arrangerRevision.current++;
    setArranger(nextState);
  }, []);

  const handleSectionClick = useCallback(async (sectionName: string) => {
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, active_section: sectionName, pending_section: null }));
      return;
    }
    const nextState = await engineClient.setArrangerSection(sectionName, false);
    arrangerRevision.current++;
    setArranger(nextState);
  }, []);

  const handleTogglePlay = useCallback(async () => {
    const nextPlay = !arranger.rhythm_playing;
    if (!engineClient.isTauri()) {
      setArranger((prev) => ({ ...prev, rhythm_playing: nextPlay }));
      return;
    }
    const nextState = await engineClient.setArrangerTransport(nextPlay, null, null, null);
    arrangerRevision.current++;
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
      arrangerRevision.current++;
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
      arrangerRevision.current++;
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
      chordMode?: string | null,
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
          chord_mode: chordMode ?? prev.chord_mode,
        }));
        return;
      }
      const nextState =
        chordMode !== undefined
          ? await engineClient.setArrangerKeyboardVoice(
              slot,
              instIdx,
              splitKey,
              splitEnabled,
              chordScanLower,
              chordScanUpper,
              upperOctave,
              transpose,
              chordMode,
            )
          : await engineClient.setArrangerKeyboardVoice(
              slot,
              instIdx,
              splitKey,
              splitEnabled,
              chordScanLower,
              chordScanUpper,
              upperOctave,
              transpose,
            );
      arrangerRevision.current++;
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
    arrangerRevision.current++;
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
    arrangerRevision.current++;
    setArranger(nextState);
  }, []);

  const handleTrackInstrumentChange = useCallback(
    async (trackIndex: number, instrumentIndex: number | null) => {
      if (!engineClient.isTauri()) {
        setArranger((prev) => {
          const nextInsts = [...(prev.track_instruments ?? Array(8).fill(null))];
          nextInsts[trackIndex] = instrumentIndex;
          return { ...prev, track_instruments: nextInsts };
        });
        return;
      }
      try {
        const nextState = await engineClient.setArrangerTrackInstrument(
          trackIndex,
          instrumentIndex,
        );
        arrangerRevision.current++;
        setArranger(nextState);
      } catch (err) {
        setStatusMsg(`Falha ao alterar timbre da trilha: ${String(err)}`);
      }
    },
    [],
  );

  const handleApplyOts = useCallback(async (otsIndex: number) => {
    if (!engineClient.isTauri()) {
      setArranger((prev) => {
        const preset = (prev.ots_presets ?? []).find((p) => p.index === otsIndex);
        if (!preset) return prev;
        return {
          ...prev,
          upper1_instrument: preset.upper1_instrument,
          upper2_instrument: preset.upper2_instrument,
          lower_instrument: preset.lower_instrument,
        };
      });
      return;
    }
    try {
      const nextState = await engineClient.applyArrangerOts(otsIndex);
      arrangerRevision.current++;
      setArranger(nextState);
      const applied = (nextState.ots_presets ?? []).find((p) => p.index === otsIndex);
      if (applied) {
        setStatusMsg(`OTS aplicado: ${applied.name}`);
      }
    } catch (err) {
      setStatusMsg(`Falha ao aplicar OTS: ${String(err)}`);
    }
  }, []);

  const commitMidiConfig = useCallback((next: MidiConfig) => {
    midiEdited.current = true;
    midiConfigRef.current = next;
    setMidiConfig(next);
    setSnapshot((current) => current ? { ...current, midi_config: next } : current);
  }, []);

  const handleVelocityCurve = useCallback((curve: [number, number][]) => {
    const previous = midiConfigRef.current;
    const next = { ...previous, velocity_curve: curve };
    commitMidiConfig(next);
    if (!engineClient.isTauri()) return;
    // Serialize full-config writes with port connection, so neither can restore stale fields.
    midiWriteQueue.current = midiWriteQueue.current.then(() => engineClient.updateMidiConfig(next)).catch((err) => {
      if (midiConfigRef.current === next) commitMidiConfig(previous);
      setStatusMsg(`Erro ao ajustar curva de velocidade: ${String(err)}`);
    });
  }, [commitMidiConfig]);

  const handleMidiConnect = useCallback((portName: string) => {
    const previous = midiConfigRef.current;
    const next = { ...previous, selected_port: portName || null };
    commitMidiConfig(next);
    setSelectedMidiPort(portName);
    if (!engineClient.isTauri()) return;
    midiWriteQueue.current = midiWriteQueue.current.then(async () => {
      if (portName) {
        const msg = await engineClient.connectMidiDevice(portName);
        setStatusMsg(`MIDI conectado: ${msg}`);
      } else {
        await engineClient.updateMidiConfig(next);
      }
    }).catch((err) => {
      if (midiConfigRef.current === next) {
        commitMidiConfig(previous);
        setSelectedMidiPort(previous.selected_port ?? "");
      }
      setStatusMsg(`Erro MIDI: ${String(err)}`);
    });
  }, [commitMidiConfig]);

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

  const handleExportWav = useCallback(async () => {
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
  }, []);

  const runTool = useCallback(async (action: () => Promise<void>) => {
    if (toolsPending.current) return;
    if (!engineClient.isTauri()) {
      setStatusMsg("Ferramentas disponíveis no aplicativo desktop Tauri");
      return;
    }
    toolsPending.current = true;
    setToolsBusy(true);
    try { await action(); }
    catch (err) { setStatusMsg(`Falha na ferramenta: ${String(err)}`); }
    finally { toolsPending.current = false; setToolsBusy(false); }
  }, []);

  const handleResetInstruments = () => runTool(async () => {
    arrangerRevision.current++;
    const next = await engineClient.resetArrangerInstruments();
    arrangerRevision.current++;
    setArranger(next);
    setStatusMsg("Timbres restaurados aos padrões do pacote.");
  });
  const handleSaveConfiguration = () => runTool(async () => {
    await midiWriteQueue.current;
    await engineClient.saveCurrentConfiguration();
    setStatusMsg("Configuração atual salva.");
  });
  const handleClearConfiguration = () => {
    if (!window.confirm("Remover somente a configuração salva? Os ajustes em uso não serão alterados.")) return;
    void runTool(async () => {
      await engineClient.clearSavedConfiguration();
      setStatusMsg("Configuração salva removida. Ajustes em uso preservados.");
    });
  };
  const handleScanDuplicates = () => runTool(async () => {
    setStatusMsg("Analisando conteúdo dos ritmos duplicados...");
    const scan = await engineClient.scanStyleDuplicates();
    setDuplicateScan(scan);
    setStatusMsg("");
  });
  const handleCleanupDuplicates = () => {
    if (!duplicateScan) return;
    void runTool(async () => {
      const report = await engineClient.cleanupStyleDuplicates(duplicateScan);
      setCleanupReport(report);
      setDuplicateScan(null);
      setStatusMsg("");
    });
  };
  const handleRestoreDuplicates = () => {
    if (!cleanupReport?.manifest_path) return;
    void runTool(async () => {
      const report = await engineClient.restoreStyleDuplicates(cleanupReport.manifest_path);
      setCleanupReport(report);
      setStatusMsg("");
    });
  };

  const activeStyle = arranger.styles[arranger.active_style_index] ?? arranger.styles[0] ?? null;
  const availableSections = useMemo(
    () => new Set(activeStyle?.sections ?? []),
    [activeStyle],
  );

  return (
    <div className="studio-shell">
      <ArrangerTopbar
        packages={packages}
        activePackagePath={arranger.file_path}
        styles={arranger.styles}
        activeStyleIndex={arranger.active_style_index}
        midiPorts={midiPorts}
        selectedMidiPort={selectedMidiPort}
        masterVolumeDb={masterVolumeDb}
        bufferSize={bufferSize}
        telemetry={telemetry}
        onLoadPackage={handleLoadPackage}
        onSelectStyle={handleSelectStyle}
        onPickMusicDialog={handlePickFileDialog}
        onPickStyDialog={handlePickStyDialog}
        onPickStyFolderDialog={handlePickStyFolderDialog}
        onMidiConnect={handleMidiConnect}
        onOpenLogs={() => setLogsOpen(true)}
        onVelocitySettings={() => setVelocityOpen(true)}
        velocitySettingsReady={midiConfigReady}
        onExportWav={handleExportWav}
        onResetInstruments={handleResetInstruments}
        onSaveConfiguration={handleSaveConfiguration}
        onClearConfiguration={handleClearConfiguration}
        onScanDuplicates={handleScanDuplicates}
        toolsBusy={toolsBusy}
        bufferBusy={bufferBusy}
        onMasterVolumeChange={handleMasterVolumeChange}
        onBufferSizeChange={handleBufferSizeChange}
      />

      {logsOpen && <LogsDialog entries={logs} telemetry={telemetry} onClear={() => setLogs([])} onClose={() => setLogsOpen(false)} />}
      {velocityOpen && <VelocitySettings config={midiConfig} onChange={handleVelocityCurve} onClose={() => setVelocityOpen(false)} />}
      <main className="arranger-workspace">
        {/* ROW 1: 3-Column Hardware Arranger Console (Mixer | LCD & Chords | Voices & Split/Scan) */}
        <div className="arranger-console-grid">
          <MixerPanel
            masterVolumeDb={masterVolumeDb}
            trackVolumes={arranger.track_volumes}
            trackInstruments={arranger.track_instruments}
            instruments={arranger.instruments}
            soundfontName={arranger.soundfont_name}
            onMasterVolumeChange={handleMasterVolumeChange}
            onTrackVolumeChange={handleTrackVolume}
            onTrackInstrumentChange={handleTrackInstrumentChange}
            onPickSoundfont={handlePickSoundfontDialog}
          />

          <LcdPanel
            arranger={arranger}
            onTogglePlay={handleTogglePlay}
            onBpmChange={handleBpmChange}
            onTempoLockToggle={handleTempoLockToggle}
            onApplyOts={handleApplyOts}
            onChordClick={handleChordClick}
          />

          <VoicesSplitPanel
            arranger={arranger}
            onVoiceChange={handleVoiceChange}
            onPickInstrumentDialog={handlePickInstrumentDialog}
          />
        </div>

        {/* ROW 2: Hardware Intro / Variation / Fill / Ending & Start/Stop Strip */}
        <section className="panel-card arranger-rhythm-strip">
          <div className="rhythm-strip-top">
            <div className="arranger-transport-strip">
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

      {duplicateScan && (
        <section className="library-action-panel" role="dialog" aria-label="Confirmar limpeza de ritmos">
          <strong>{duplicateScan.redundant_files} arquivo(s) redundante(s) por conteúdo idêntico</strong>
          <p>{duplicateScan.scanned_files} arquivo(s) analisado(s) em {duplicateScan.root}. {(duplicateScan.reclaimable_bytes / 1048576).toFixed(2)} MB recuperáveis.</p>
          <p>Apenas cópias idênticas serão movidas para quarentena recuperável. Originais preservados.</p>
          <button type="button" className="btn" disabled={toolsBusy || !duplicateScan.redundant_files} onClick={handleCleanupDuplicates}>Confirmar quarentena</button>
          <button type="button" className="btn" disabled={toolsBusy} onClick={() => setDuplicateScan(null)}>Cancelar</button>
        </section>
      )}
      {cleanupReport && (
        <section className="library-action-panel" aria-label="Resultado da limpeza" role={cleanupReport.status === "recovery_required" || cleanupReport.errors.length ? "alert" : "status"}>
          <strong>{cleanupReport.status === "recovery_required" ? "Recuperação necessária — limpeza não concluída" : cleanupReport.errors.length ? "Operação com erros" : cleanupReport.status === "rolled_back" ? "Operação revertida — nenhum arquivo removido" : cleanupReport.status === "restored" ? "Restauração concluída" : cleanupReport.status === "quarantined" ? "Quarentena concluída" : cleanupReport.status === "unchanged" ? "Nenhum arquivo movido" : `Estado da operação: ${cleanupReport.status}`}</strong>
          <span>Movidos: {cleanupReport.moved_files} • Restaurados: {cleanupReport.restored_files}</span>
          <code>Manifesto: {cleanupReport.manifest_path}</code>
          {cleanupReport.errors.map((error, index) => <p key={index}>{error}</p>)}
          <button type="button" className="btn" disabled={toolsBusy || !cleanupReport.manifest_path} onClick={handleRestoreDuplicates}>Desfazer quarentena</button>
          <button type="button" className="btn" disabled={toolsBusy} onClick={() => setCleanupReport(null)}>Fechar</button>
        </section>
      )}
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

          </div>

          <span className="keyboard-hint">Teclado PC: Z–M (Acordes) / Q–P (Solo)</span>

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
