use crate::audio::AudioEngineManager;
use crate::contracts::{
    EngineParams, EngineSnapshot, EngineTelemetry, MidiConfig, MidiPortInfo, MidiTransportState,
    PresetData,
};
use crate::midi::list_midi_input_ports;
use crate::music_studio::{
    MusicPackageEntry, MusicPackageSummary, USER_DOCUMENTS_INSTRUMENTS_DIR,
    USER_DOCUMENTS_KEYBOARD_DIR, USER_DOCUMENTS_RHYTHMS_DIR,
};
use crate::style_library::{self, CleanupReport, DuplicateScan};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_engine_snapshot(engine: State<Arc<AudioEngineManager>>) -> EngineSnapshot {
    engine.snapshot()
}

#[tauri::command]
pub fn get_telemetry(engine: State<Arc<AudioEngineManager>>) -> EngineTelemetry {
    engine.shared.telemetry.read().clone()
}

#[tauri::command]
pub fn note_on(engine: State<Arc<AudioEngineManager>>, note: u8, velocity: u8) {
    engine.note_on(note, velocity);
}

#[tauri::command]
pub fn note_off(engine: State<Arc<AudioEngineManager>>, note: u8) {
    engine.note_off(note);
}

#[tauri::command]
pub fn all_notes_off(engine: State<Arc<AudioEngineManager>>) {
    engine.all_notes_off();
}

#[tauri::command]
pub fn set_pedal(engine: State<Arc<AudioEngineManager>>, pedal: String, value: f32) {
    engine.set_pedal(&pedal, value);
}

#[tauri::command]
pub fn update_engine_params(engine: State<Arc<AudioEngineManager>>, params: EngineParams) {
    engine.set_params(params);
}

#[tauri::command]
pub fn select_instrument(
    engine: State<Arc<AudioEngineManager>>,
    instrument_id: String,
) -> Result<EngineSnapshot, String> {
    engine.select_instrument(&instrument_id)
}

#[tauri::command]
pub fn select_preset(
    engine: State<Arc<AudioEngineManager>>,
    preset_id: String,
) -> Result<EngineSnapshot, String> {
    engine.select_preset(&preset_id)
}

#[tauri::command]
pub fn morph_presets(
    engine: State<Arc<AudioEngineManager>>,
    preset_a_id: String,
    preset_b_id: String,
    t: f32,
) -> Result<EngineParams, String> {
    engine.morph_ab(&preset_a_id, &preset_b_id, t)
}

#[tauri::command]
pub fn save_user_preset_cmd(
    engine: State<Arc<AudioEngineManager>>,
    name: String,
    category: String,
) -> Result<PresetData, String> {
    engine.save_preset(&name, &category)
}

#[tauri::command]
pub async fn list_midi_devices() -> Result<Vec<MidiPortInfo>, String> {
    tauri::async_runtime::spawn_blocking(list_midi_input_ports)
        .await
        .map_err(|e| format!("Falha na enumeração MIDI: {e}"))
}

#[tauri::command]
pub async fn connect_midi_device(
    engine: State<'_, Arc<AudioEngineManager>>,
    port_name: String,
) -> Result<String, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.connect_midi(&port_name))
        .await
        .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub fn update_midi_config(engine: State<Arc<AudioEngineManager>>, config: MidiConfig) {
    engine.set_midi_config(config);
}

#[tauri::command]
pub fn play_test_chord(engine: State<Arc<AudioEngineManager>>) {
    let notes = [48_u8, 60, 64, 67, 71];
    for (idx, n) in notes.iter().enumerate() {
        engine.note_on(*n, 74 + (idx as u8 * 4));
    }
}

#[tauri::command]
pub async fn export_demo_wav(
    engine: State<'_, Arc<AudioEngineManager>>,
    filename: Option<String>,
) -> Result<String, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let fname = filename.unwrap_or_else(|| "piano-tect-demo.wav".to_string());
        let path = std::env::temp_dir().join(fname);
        engine.export_demo_wav(PathBuf::from(path))
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub fn play_demo_midi(engine: State<Arc<AudioEngineManager>>) -> MidiTransportState {
    engine.play_demo_midi()
}

#[tauri::command]
pub fn stop_midi_transport(engine: State<Arc<AudioEngineManager>>) -> MidiTransportState {
    engine.stop_midi_transport()
}

#[tauri::command]
pub fn toggle_midi_record(engine: State<Arc<AudioEngineManager>>) -> MidiTransportState {
    engine.toggle_midi_record()
}

#[tauri::command]
pub fn play_recorded_midi(engine: State<Arc<AudioEngineManager>>) -> MidiTransportState {
    engine.play_recorded_midi()
}

#[tauri::command]
pub fn set_midi_playback_speed(
    engine: State<Arc<AudioEngineManager>>,
    speed: f32,
) -> MidiTransportState {
    engine.set_midi_playback_speed(speed)
}

#[tauri::command]
pub fn arm_midi_cc_learn(
    engine: State<Arc<AudioEngineManager>>,
    param_path: Option<String>,
) -> Option<String> {
    engine.arm_midi_cc_learn(param_path)
}

#[tauri::command]
pub async fn list_music_packages(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<Vec<MusicPackageEntry>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || Ok(engine.list_music_packages()))
        .await
        .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn load_music_package(
    engine: State<'_, Arc<AudioEngineManager>>,
    path: String,
) -> Result<MusicPackageSummary, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.load_music_package(&PathBuf::from(path)))
        .await
        .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn pick_music_package_dialog(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<Option<MusicPackageSummary>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
    let default_dir = if std::path::Path::new(USER_DOCUMENTS_RHYTHMS_DIR).exists() {
        format!("{}/", USER_DOCUMENTS_RHYTHMS_DIR)
    } else {
        "/home/data/Documents/".to_string()
    };

    let output = std::process::Command::new("/usr/bin/zenity")
        .arg("--file-selection")
        .arg("--title=Selecionar Ritmo / Pacote .Music ou .STY")
        .arg(format!("--filename={}", default_dir))
        .arg("--file-filter=Ritmos (*.Music *.Template *.sty *.prs *.sst *.bcs) | *.Music *.music *.Template *.template *.sty *.STY *.prs *.PRS *.sst *.SST *.bcs *.BCS")
        .arg("--file-filter=Music Studio (*.Music *.Template) | *.Music *.music *.Template *.template")
        .arg("--file-filter=Yamaha Style (*.sty *.prs *.sst *.bcs) | *.sty *.STY *.prs *.PRS *.sst *.SST *.bcs *.BCS")
        .arg("--file-filter=Todos os arquivos | *")
        .output()
        .map_err(|e| format!("Falha ao abrir explorador de arquivos (zenity): {}", e))?;

    if !output.status.success() {
        return Ok(None);
    }
    let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if chosen.is_empty() {
        return Ok(None);
    }
    let summary = engine.load_music_package(&PathBuf::from(chosen))?;
    Ok(Some(summary))
    }).await.map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub fn get_arranger_state(engine: State<Arc<AudioEngineManager>>) -> MusicPackageSummary {
    engine.arranger_state()
}

#[tauri::command]
pub fn set_arranger_section(
    engine: State<Arc<AudioEngineManager>>,
    section_name: String,
    immediate: bool,
) -> MusicPackageSummary {
    engine.set_arranger_section(&section_name, immediate)
}

#[tauri::command]
pub fn set_arranger_transport(
    engine: State<Arc<AudioEngineManager>>,
    playing: bool,
    sync_start: Option<bool>,
    bpm: Option<f32>,
    auto_fill: Option<bool>,
    tempo_locked: Option<bool>,
) -> MusicPackageSummary {
    engine.set_arranger_transport(playing, sync_start, bpm, auto_fill, tempo_locked)
}

#[tauri::command]
pub async fn set_audio_buffer_size(
    engine: State<'_, Arc<AudioEngineManager>>,
    buffer_size: u32,
) -> Result<EngineTelemetry, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.set_audio_buffer_size(buffer_size))
        .await
        .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub fn set_arranger_style(
    engine: State<Arc<AudioEngineManager>>,
    style_index: usize,
) -> MusicPackageSummary {
    engine.set_arranger_style(style_index)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn set_arranger_keyboard_voice(
    engine: State<Arc<AudioEngineManager>>,
    slot: String,
    instrument_index: Option<usize>,
    split_key: Option<u8>,
    split_enabled: Option<bool>,
    chord_scan_lower: Option<bool>,
    chord_scan_upper: Option<bool>,
    upper_octave: Option<i8>,
    transpose: Option<i8>,
    chord_mode: Option<String>,
) -> MusicPackageSummary {
    engine.set_arranger_voice(
        &slot,
        instrument_index,
        split_key,
        split_enabled,
        chord_scan_lower,
        chord_scan_upper,
        upper_octave,
        transpose,
        chord_mode.as_deref(),
    )
}

#[tauri::command]
pub fn set_arranger_chord(
    engine: State<Arc<AudioEngineManager>>,
    root_pc: u8,
    quality: String,
) -> MusicPackageSummary {
    engine.set_arranger_chord(root_pc, &quality)
}

#[tauri::command]
pub fn set_arranger_track_volume(
    engine: State<Arc<AudioEngineManager>>,
    track_index: usize,
    volume: f32,
) -> MusicPackageSummary {
    engine.set_arranger_track_volume(track_index, volume)
}

#[tauri::command]
pub async fn import_instrument_file(
    engine: State<'_, Arc<AudioEngineManager>>,
    path: String,
    assign_upper1: Option<bool>,
) -> Result<MusicPackageSummary, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        engine.import_instrument_file(&PathBuf::from(path), assign_upper1.unwrap_or(true))
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn pick_instrument_dialog(
    engine: State<'_, Arc<AudioEngineManager>>,
    assign_upper1: Option<bool>,
) -> Result<Option<MusicPackageSummary>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let default_dir = if std::path::Path::new(USER_DOCUMENTS_INSTRUMENTS_DIR).exists() {
            format!("{}/", USER_DOCUMENTS_INSTRUMENTS_DIR)
        } else if std::path::Path::new(USER_DOCUMENTS_RHYTHMS_DIR).exists() {
            format!("{}/", USER_DOCUMENTS_RHYTHMS_DIR)
        } else {
            "/home/data/Documents/".to_string()
        };

        let output = std::process::Command::new("/usr/bin/zenity")
            .arg("--file-selection")
            .arg("--multiple")
            .arg("--separator=\n")
            .arg("--title=Selecionar Timbre (.Instrument)")
            .arg(format!("--filename={}", default_dir))
            .arg("--file-filter=Music Studio Instrument (*.Instrument) | *.Instrument *.instrument")
            .arg("--file-filter=Todos os arquivos | *")
            .output()
            .map_err(|e| format!("Falha ao abrir explorador de arquivos (zenity): {}", e))?;

        if !output.status.success() {
            return Ok(None);
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        let paths: Vec<String> = raw
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if paths.is_empty() {
            return Ok(None);
        }

        let assign_last = assign_upper1.unwrap_or(true);
        let mut latest = None;
        for (idx, p) in paths.iter().enumerate() {
            let should_assign = assign_last && idx + 1 == paths.len();
            latest = Some(engine.import_instrument_file(&PathBuf::from(p), should_assign)?);
        }
        Ok(latest)
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

fn default_sty_dialog_dir() -> String {
    if Path::new(USER_DOCUMENTS_KEYBOARD_DIR).exists() {
        format!("{}/", USER_DOCUMENTS_KEYBOARD_DIR)
    } else if Path::new(USER_DOCUMENTS_RHYTHMS_DIR).exists() {
        format!("{}/", USER_DOCUMENTS_RHYTHMS_DIR)
    } else {
        "/home/data/Documents/".to_string()
    }
}

#[tauri::command]
pub async fn load_sty_files(
    engine: State<'_, Arc<AudioEngineManager>>,
    paths: Vec<String>,
) -> Result<MusicPackageSummary, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let path_bufs: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        engine.load_sty_files(&path_bufs)
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn pick_sty_dialog(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<Option<MusicPackageSummary>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
    let default_dir = default_sty_dialog_dir();
    let output = std::process::Command::new("/usr/bin/zenity")
        .arg("--file-selection")
        .arg("--multiple")
        .arg("--separator=\n")
        .arg("--title=Selecionar Ritmos Yamaha (.STY)")
        .arg(format!("--filename={}", default_dir))
        .arg("--file-filter=Yamaha Style (*.sty *.STY *.prs *.sst *.bcs) | *.sty *.STY *.prs *.PRS *.sst *.SST *.bcs *.BCS")
        .arg("--file-filter=Todos os arquivos | *")
        .output()
        .map_err(|e| format!("Falha ao abrir explorador de arquivos (zenity): {}", e))?;

    if !output.status.success() {
        return Ok(None);
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let paths: Vec<PathBuf> = raw
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();
    if paths.is_empty() {
        return Ok(None);
    }
    Ok(Some(engine.load_sty_files(&paths)?))
    }).await.map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn pick_sty_folder_dialog(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<Option<MusicPackageSummary>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let default_dir = default_sty_dialog_dir();
        let output = std::process::Command::new("/usr/bin/zenity")
            .arg("--file-selection")
            .arg("--directory")
            .arg("--title=Selecionar Pasta de Ritmos .STY")
            .arg(format!("--filename={}", default_dir))
            .output()
            .map_err(|e| format!("Falha ao abrir explorador de arquivos (zenity): {}", e))?;

        if !output.status.success() {
            return Ok(None);
        }
        let chosen_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if chosen_dir.is_empty() {
            return Ok(None);
        }
        let collected = style_library::collect_style_files(&[PathBuf::from(&chosen_dir)])?;
        let mut paths = style_library::deduplicate_style_paths(&collected)?;
        paths.sort_by(|a, b| {
            let an = a
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_lowercase();
            let bn = b
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_lowercase();
            an.cmp(&bn).then_with(|| a.cmp(b))
        });
        paths.truncate(200);
        if paths.is_empty() {
            return Err(format!(
                "Nenhum arquivo .STY/.PRS/.SST/.BCS encontrado em '{}'",
                chosen_dir
            ));
        }
        Ok(Some(engine.load_sty_files(&paths)?))
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn load_soundfont_file(
    engine: State<'_, Arc<AudioEngineManager>>,
    path: String,
) -> Result<MusicPackageSummary, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.load_soundfont_file(&PathBuf::from(path)))
        .await
        .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub async fn pick_soundfont_dialog(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<Option<MusicPackageSummary>, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let default_dir = if Path::new("/usr/share/sounds/sf2").exists() {
            "/usr/share/sounds/sf2/".to_string()
        } else {
            "/home/data/Documents/".to_string()
        };

        let output = std::process::Command::new("/usr/bin/zenity")
            .arg("--file-selection")
            .arg("--title=Selecionar SoundFont (.SF2)")
            .arg(format!("--filename={}", default_dir))
            .arg("--file-filter=SoundFont 2 (*.sf2 *.SF2) | *.sf2 *.SF2")
            .arg("--file-filter=Todos os arquivos | *")
            .output()
            .map_err(|e| format!("Falha ao abrir explorador de arquivos (zenity): {}", e))?;

        if !output.status.success() {
            return Ok(None);
        }
        let chosen = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if chosen.is_empty() {
            return Ok(None);
        }
        Ok(Some(engine.load_soundfont_file(&PathBuf::from(chosen))?))
    })
    .await
    .map_err(|e| format!("Falha na tarefa: {e}"))?
}

#[tauri::command]
pub fn set_arranger_track_instrument(
    engine: State<Arc<AudioEngineManager>>,
    track_index: usize,
    instrument_index: Option<usize>,
) -> MusicPackageSummary {
    engine.set_arranger_track_instrument(track_index, instrument_index)
}

#[tauri::command]
pub fn apply_arranger_ots(
    engine: State<Arc<AudioEngineManager>>,
    ots_index: usize,
) -> MusicPackageSummary {
    engine.apply_arranger_ots(ots_index)
}

#[tauri::command]
pub async fn reset_arranger_instruments(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<MusicPackageSummary, String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.reset_arranger_instruments())
        .await
        .map_err(|e| format!("Falha ao resetar instrumentos: {e}"))
}

#[tauri::command]
pub async fn save_current_configuration(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<(), String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.save_current_configuration())
        .await
        .map_err(|e| format!("Falha ao salvar: {e}"))?
}

#[tauri::command]
pub async fn clear_saved_configuration(
    engine: State<'_, Arc<AudioEngineManager>>,
) -> Result<(), String> {
    let engine = Arc::clone(engine.inner());
    tauri::async_runtime::spawn_blocking(move || engine.clear_saved_configuration())
        .await
        .map_err(|e| format!("Falha ao remover configuração: {e}"))?
}

#[tauri::command]
pub async fn scan_style_duplicates() -> Result<DuplicateScan, String> {
    tauri::async_runtime::spawn_blocking(|| {
        style_library::scan_keyboard_duplicates(Path::new(USER_DOCUMENTS_KEYBOARD_DIR))
    })
    .await
    .map_err(|e| format!("Falha ao analisar duplicados: {e}"))?
}

fn style_quarantine_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or_else(|| "Diretório de dados do usuário indisponível".to_string())?;
    Ok(base.join("piano-tect-tauri/style-quarantine"))
}

#[tauri::command]
pub async fn cleanup_style_duplicates(scan: DuplicateScan) -> Result<CleanupReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        style_library::quarantine_keyboard_duplicates(&scan, true, &style_quarantine_dir()?)
    })
    .await
    .map_err(|e| format!("Falha na limpeza: {e}"))?
}

#[tauri::command]
pub async fn restore_style_duplicates(manifest_path: String) -> Result<CleanupReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manifest = std::fs::canonicalize(&manifest_path).map_err(|e| e.to_string())?;
        let root = std::fs::canonicalize(style_quarantine_dir()?).map_err(|e| e.to_string())?;
        if !manifest.starts_with(&root) {
            return Err("Manifesto fora da quarentena do aplicativo".to_string());
        }
        style_library::restore_quarantine(&manifest)
    })
    .await
    .map_err(|e| format!("Falha na recuperação: {e}"))?
}
