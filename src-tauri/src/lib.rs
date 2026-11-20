pub mod audio;
pub mod commands;
pub mod contracts;
pub mod effects;
pub mod instruments;
pub mod midi;
pub mod presets;
pub mod synthesis;

use audio::AudioEngineManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = AudioEngineManager::start();

    tauri::Builder::default()
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            commands::get_engine_snapshot,
            commands::get_telemetry,
            commands::note_on,
            commands::note_off,
            commands::all_notes_off,
            commands::set_pedal,
            commands::update_engine_params,
            commands::select_instrument,
            commands::select_preset,
            commands::morph_presets,
            commands::save_user_preset_cmd,
            commands::list_midi_devices,
            commands::connect_midi_device,
            commands::update_midi_config,
            commands::play_test_chord,
            commands::export_demo_wav,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Piano Tect Tauri application");
}
