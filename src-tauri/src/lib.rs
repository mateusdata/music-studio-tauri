pub mod audio;
pub mod commands;
pub mod contracts;
pub mod effects;
pub mod instruments;
pub mod midi;
pub mod music_studio;
pub mod presets;
pub mod synthesis;

use audio::AudioEngineManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = AudioEngineManager::start();

    tauri::Builder::default()
        .manage(engine)
        .setup(|app| {
            if let (Some(win), Some(icon)) = (
                app.get_webview_window("main"),
                app.default_window_icon().cloned(),
            ) {
                let _ = win.set_icon(icon);
            }
            Ok(())
        })
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
            commands::play_demo_midi,
            commands::stop_midi_transport,
            commands::toggle_midi_record,
            commands::play_recorded_midi,
            commands::set_midi_playback_speed,
            commands::arm_midi_cc_learn,
            commands::list_music_packages,
            commands::load_music_package,
            commands::pick_music_package_dialog,
            commands::import_instrument_file,
            commands::pick_instrument_dialog,
            commands::get_arranger_state,
            commands::set_arranger_section,
            commands::set_arranger_transport,
            commands::set_arranger_style,
            commands::set_arranger_keyboard_voice,
            commands::set_arranger_chord,
            commands::set_arranger_track_volume,
            commands::set_audio_buffer_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Piano Tect Tauri application");
}
