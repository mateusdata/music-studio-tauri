import { invoke } from "@tauri-apps/api/core";
import type {
  EngineParams,
  EngineSnapshot,
  EngineTelemetry,
  MidiConfig,
  MidiPortInfo,
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
};
