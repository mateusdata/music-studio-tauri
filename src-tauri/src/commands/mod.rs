use crate::audio::AudioEngineManager;
use crate::contracts::{
    EngineParams, EngineSnapshot, EngineTelemetry, MidiConfig, MidiPortInfo,
    MidiTransportState, PresetData,
};
use crate::midi::list_midi_input_ports;
use crate::music_studio::{MusicPackageEntry, MusicPackageSummary};
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

#[tauri::command]
pub fn play_demo_midi(engine: State<AudioEngineManager>) -> MidiTransportState {
    engine.play_demo_midi()
}

#[tauri::command]
pub fn stop_midi_transport(engine: State<AudioEngineManager>) -> MidiTransportState {
    engine.stop_midi_transport()
}

#[tauri::command]
pub fn toggle_midi_record(engine: State<AudioEngineManager>) -> MidiTransportState {
    engine.toggle_midi_record()
}

#[tauri::command]
pub fn play_recorded_midi(engine: State<AudioEngineManager>) -> MidiTransportState {
    engine.play_recorded_midi()
}

#[tauri::command]
pub fn set_midi_playback_speed(
    engine: State<AudioEngineManager>,
    speed: f32,
) -> MidiTransportState {
    engine.set_midi_playback_speed(speed)
}

#[tauri::command]
pub fn arm_midi_cc_learn(
    engine: State<AudioEngineManager>,
    param_path: Option<String>,
) -> Option<String> {
    engine.arm_midi_cc_learn(param_path)
}

#[tauri::command]
pub fn list_music_packages(engine: State<AudioEngineManager>) -> Vec<MusicPackageEntry> {
    engine.list_music_packages()
}

#[tauri::command]
pub fn load_music_package(
    engine: State<AudioEngineManager>,
    path: String,
) -> Result<MusicPackageSummary, String> {
    engine.load_music_package(&PathBuf::from(path))
}

#[tauri::command]
pub fn get_arranger_state(engine: State<AudioEngineManager>) -> MusicPackageSummary {
    engine.arranger_state()
}

#[tauri::command]
pub fn set_arranger_section(
    engine: State<AudioEngineManager>,
    section_name: String,
    immediate: bool,
) -> MusicPackageSummary {
    engine.set_arranger_section(&section_name, immediate)
}

#[tauri::command]
pub fn set_arranger_transport(
    engine: State<AudioEngineManager>,
    playing: bool,
    sync_start: Option<bool>,
    bpm: Option<f32>,
    auto_fill: Option<bool>,
) -> MusicPackageSummary {
    engine.set_arranger_transport(playing, sync_start, bpm, auto_fill)
}

#[tauri::command]
pub fn set_arranger_style(
    engine: State<AudioEngineManager>,
    style_index: usize,
) -> MusicPackageSummary {
    engine.set_arranger_style(style_index)
}

#[tauri::command]
pub fn set_arranger_keyboard_voice(
    engine: State<AudioEngineManager>,
    slot: String,
    instrument_index: Option<usize>,
    split_key: Option<u8>,
    split_enabled: Option<bool>,
) -> MusicPackageSummary {
    engine.set_arranger_voice(&slot, instrument_index, split_key, split_enabled)
}

#[tauri::command]
pub fn set_arranger_chord(
    engine: State<AudioEngineManager>,
    root_pc: u8,
    quality: String,
) -> MusicPackageSummary {
    engine.set_arranger_chord(root_pc, &quality)
}

#[tauri::command]
pub fn set_arranger_track_volume(
    engine: State<AudioEngineManager>,
    track_index: usize,
    volume: f32,
) -> MusicPackageSummary {
    engine.set_arranger_track_volume(track_index, volume)
}
