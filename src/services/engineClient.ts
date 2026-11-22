import { invoke } from "@tauri-apps/api/core";
import type {
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiConfig,
  MidiPortInfo,
  MidiTransportState,
  MusicPackageEntry,
  MusicPackageSummary,
  PresetData,
} from "../types/engine";

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const engineClient = {
  isTauri: isTauriRuntime,

  async getSnapshot(): Promise<EngineSnapshot> {
    return invoke<EngineSnapshot>("get_engine_snapshot");
  },

  async getTelemetry(): Promise<EngineTelemetry> {
    return invoke<EngineTelemetry>("get_telemetry");
  },

  async noteOn(note: number, velocity: number): Promise<void> {
    return invoke("note_on", { note, velocity });
  },

  async noteOff(note: number): Promise<void> {
    return invoke("note_off", { note });
  },

  async allNotesOff(): Promise<void> {
    return invoke("all_notes_off");
  },

  async setPedal(pedal: string, value: number): Promise<void> {
    return invoke("set_pedal", { pedal, value });
  },

  async updateParams(params: EngineParams): Promise<void> {
    return invoke("update_engine_params", { params });
  },

  async selectInstrument(instrumentId: string): Promise<EngineSnapshot> {
    return invoke<EngineSnapshot>("select_instrument", { instrumentId });
  },

  async selectPreset(presetId: string): Promise<EngineSnapshot> {
    return invoke<EngineSnapshot>("select_preset", { presetId });
  },

  async morphPresets(presetAId: string, presetBId: string, t: number): Promise<EngineParams> {
    return invoke<EngineParams>("morph_presets", { presetAId, presetBId, t });
  },

  async saveUserPreset(name: string, category: string): Promise<PresetData> {
    return invoke<PresetData>("save_user_preset_cmd", { name, category });
  },

  async listMidiDevices(): Promise<MidiPortInfo[]> {
    return invoke<MidiPortInfo[]>("list_midi_devices");
  },

  async connectMidiDevice(portName: string): Promise<string> {
    return invoke<string>("connect_midi_device", { portName });
  },

  async updateMidiConfig(config: MidiConfig): Promise<void> {
    return invoke("update_midi_config", { config });
  },

  async playTestChord(): Promise<void> {
    return invoke("play_test_chord");
  },

  async exportDemoWav(filename?: string): Promise<string> {
    return invoke<string>("export_demo_wav", { filename: filename ?? null });
  },

  async playDemoMidi(): Promise<MidiTransportState> {
    return invoke<MidiTransportState>("play_demo_midi");
  },

  async stopMidiTransport(): Promise<MidiTransportState> {
    return invoke<MidiTransportState>("stop_midi_transport");
  },

  async toggleMidiRecord(): Promise<MidiTransportState> {
    return invoke<MidiTransportState>("toggle_midi_record");
  },

  async playRecordedMidi(): Promise<MidiTransportState> {
    return invoke<MidiTransportState>("play_recorded_midi");
  },

  async setMidiPlaybackSpeed(speed: number): Promise<MidiTransportState> {
    return invoke<MidiTransportState>("set_midi_playback_speed", { speed });
  },

  async armMidiCcLearn(paramPath: string | null): Promise<string | null> {
    return invoke<string | null>("arm_midi_cc_learn", { paramPath });
  },

  async listMusicPackages(): Promise<MusicPackageEntry[]> {
    return invoke<MusicPackageEntry[]>("list_music_packages");
  },

  async loadMusicPackage(path: string): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("load_music_package", { path });
  },

  async getArrangerState(): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("get_arranger_state");
  },

  async setArrangerSection(sectionName: string, immediate: boolean): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_section", { sectionName, immediate });
  },

  async setArrangerTransport(
    playing: boolean,
    syncStart: boolean | null,
    bpm: number | null,
    autoFill: boolean | null,
  ): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_transport", {
      playing,
      syncStart,
      bpm,
      autoFill,
    });
  },

  async setArrangerStyle(styleIndex: number): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_style", { styleIndex });
  },

  async setArrangerKeyboardVoice(
    slot: "upper1" | "upper2" | "lower" | "split",
    instrumentIndex: number | null,
    splitKey: number | null,
    splitEnabled: boolean | null,
  ): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_keyboard_voice", {
      slot,
      instrumentIndex,
      splitKey,
      splitEnabled,
    });
  },

  async setArrangerChord(rootPc: number, quality: string): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_chord", { rootPc, quality });
  },

  async setArrangerTrackVolume(trackIndex: number, volume: number): Promise<MusicPackageSummary> {
    return invoke<MusicPackageSummary>("set_arranger_track_volume", { trackIndex, volume });
  },
};
