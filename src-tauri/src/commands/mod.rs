use crate::audio::AudioEngineManager;
use crate::contracts::{
    EngineParams, EngineSnapshot, EngineTelemetry, MidiConfig, MidiPortInfo, PresetData,
};
use crate::midi::list_midi_input_ports;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn get_engine_snapshot(engine: State<AudioEngineManager>) -> EngineSnapshot {
    engine.snapshot()
}

#[tauri::command]
pub fn get_telemetry(engine: State<AudioEngineManager>) -> EngineTelemetry {
    engine.shared.telemetry.read().clone()
}

#[tauri::command]
pub fn note_on(engine: State<AudioEngineManager>, note: u8, velocity: u8) {
    engine.note_on(note, velocity);
}

#[tauri::command]
pub fn note_off(engine: State<AudioEngineManager>, note: u8) {
    engine.note_off(note);
}

#[tauri::command]
pub fn all_notes_off(engine: State<AudioEngineManager>) {
    engine.all_notes_off();
}

#[tauri::command]
pub fn set_pedal(engine: State<AudioEngineManager>, pedal: String, value: f32) {
    engine.set_pedal(&pedal, value);
}

#[tauri::command]
pub fn update_engine_params(engine: State<AudioEngineManager>, params: EngineParams) {
    engine.set_params(params);
}

#[tauri::command]
pub fn select_instrument(
    engine: State<AudioEngineManager>,
    instrument_id: String,
) -> Result<EngineSnapshot, String> {
    engine.select_instrument(&instrument_id)
}

#[tauri::command]
pub fn select_preset(
    engine: State<AudioEngineManager>,
    preset_id: String,
) -> Result<EngineSnapshot, String> {
    engine.select_preset(&preset_id)
}

#[tauri::command]
pub fn morph_presets(
    engine: State<AudioEngineManager>,
    preset_a_id: String,
    preset_b_id: String,
    t: f32,
) -> Result<EngineParams, String> {
    engine.morph_ab(&preset_a_id, &preset_b_id, t)
}

#[tauri::command]
pub fn save_user_preset_cmd(
    engine: State<AudioEngineManager>,
    name: String,
    category: String,
) -> Result<PresetData, String> {
    engine.save_preset(&name, &category)
}

#[tauri::command]
pub fn list_midi_devices() -> Vec<MidiPortInfo> {
    list_midi_input_ports()
}

#[tauri::command]
pub fn connect_midi_device(
    engine: State<AudioEngineManager>,
    port_name: String,
) -> Result<String, String> {
    engine.connect_midi(&port_name)
}

#[tauri::command]
pub fn update_midi_config(engine: State<AudioEngineManager>, config: MidiConfig) {
    engine.set_midi_config(config);
}

#[tauri::command]
pub fn play_test_chord(engine: State<AudioEngineManager>) {
    let notes = [48_u8, 60, 64, 67, 71];
    for (idx, n) in notes.iter().enumerate() {
        engine.note_on(*n, 74 + (idx as u8 * 4));
    }
}

#[tauri::command]
pub fn export_demo_wav(
    engine: State<AudioEngineManager>,
    filename: Option<String>,
) -> Result<String, String> {
    let fname = filename.unwrap_or_else(|| "piano-tect-demo.wav".to_string());
    let path = std::env::temp_dir().join(fname);
    engine.export_demo_wav(PathBuf::from(path))
}
