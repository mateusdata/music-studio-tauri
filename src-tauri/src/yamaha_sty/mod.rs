use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub mod drum_mapper;
pub use drum_mapper::{
    classify_drum_kit, remap_yamaha_drum_note, DrumKitKind, StyChannelDiagnostic,
    StyDiagnostics, StySectionDiagnostic,
};

/// Binary container / Style File Format dialect detected in a `.STY` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyFormat {
    Sff1,
    Sff2,
    LegacyStyl,
}

/// Yamaha CASM Note Transposition Rule (NTR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtrRule {
    RootTrans = 0,
    RootFixed = 1,
    Guitar = 2,
}

/// Yamaha CASM Note Transposition Table (NTT).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NttRule {
    Bypass = 0,
    Melody = 1,
    Chord = 2,
    Bass = 3,
    MelodicMinor = 4,
    HarmonicMinor = 5,
}

/// Yamaha CASM Retrigger Rule (RTR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtrRule {
    Stop = 0,
    PitchShift = 1,
    PitchShiftToRoot = 2,
    Retrigger = 3,
    RetriggerToRoot = 4,
    NoteGenerator = 5,
}

/// Target chord parameters consumed by the CASM rule matcher and transposer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetChordSpec {
    pub root_pc: u8,
    pub yamaha_bit: u8,
    pub third_iv: i16,
    pub fifth_iv: i16,
    pub seventh_iv: i16,
    pub add9: bool,
    pub is_major_family: bool,
}

/// Per-range transposition rule inside a `Ctab` (1 range) or `Ctb2` (3 ranges) record.
#[derive(Debug, Clone)]
pub struct CasmRangeRule {
    pub ntr: NtrRule,
    pub ntt: NttRule,
    pub bass_on: bool,
    pub high_key: u8,
    pub note_low: u8,
    pub note_high: u8,
    pub rtr: RtrRule,
}

/// Parsed CASM channel assignment and transposition rule.
#[derive(Debug, Clone)]
pub struct CasmRule {
    pub src_ch: u8,
    pub dest_ch: u8,
    pub name: String,
    pub editable: u8,
    pub note_mute: u16,
    pub chord_mute: u64,
    pub src_root: u8,
    pub src_chord_type: u8,
    pub low_split: Option<u8>,
    pub high_split: Option<u8>,
    pub ranges: [CasmRangeRule; 3],
}

impl CasmRule {
    /// Returns whether this CASM rule should sound for the given target chord.
    pub fn is_active(&self, spec: TargetChordSpec) -> bool {
        if self.chord_mute != 0 {
            let bit = spec.yamaha_bit.min(39);
            if (self.chord_mute & (1_u64 << bit)) == 0 {
                return false;
            }
        }
        let root_mask = self.note_mute & 0x0FFF;
        if root_mask != 0 && root_mask != 0x0FFF {
            let key_bit = 1_u16 << (spec.root_pc % 12);
            if (root_mask & key_bit) == 0 {
                return false;
            }
        }
        true
    }

    /// Selects the active range rule for a raw source MIDI note (SFF2 Ctb2 uses 3 split zones).
    pub fn select_range(&self, raw_note: u8) -> &CasmRangeRule {
        if let (Some(lo), Some(hi)) = (self.low_split, self.high_split) {
            if raw_note <= lo {
                &self.ranges[0]
            } else if raw_note <= hi {
                &self.ranges[1]
            } else {
                &self.ranges[2]
            }
        } else {
            &self.ranges[0]
        }
    }

    /// Transposes a source MIDI note according to the CASM range, source chord, and target chord.
    /// Returns `None` when a MegaVoice keyswitch/FX zone should be filtered out.
    pub fn transpose_note(
        &self,
        raw_note: u8,
        spec: TargetChordSpec,
        is_drum: bool,
        is_bass: bool,
    ) -> Option<u8> {
        let r = self.select_range(raw_note);

        // Filter out MegaVoice articulation keyswitches when Ctb2 splits them into a fixed FX zone.
        if !is_drum && self.dest_ch != 8 && self.dest_ch != 9 {
            if self.high_split == Some(95) && raw_note > 95 {
                return None;
            }
            if self.low_split.is_some()
                && raw_note < 24
                && r.ntr == NtrRule::RootFixed
                && r.ntt == NttRule::Bypass
            {
                return None;
            }
        }

        // Drums and RootFixed+Bypass zones pass through unaltered.
        if is_drum
            || self.dest_ch == 8
            || self.dest_ch == 9
            || (r.ntr == NtrRule::RootFixed && r.ntt == NttRule::Bypass)
        {
            return Some(raw_note);
        }

        let src_root = (self.src_root % 12) as i16;
        let tgt_root = (spec.root_pc % 12) as i16;

        // SFF1/SFF2 RootTrans + Bypass transposes strictly by root delta without altering chord intervals.
        if r.ntt == NttRule::Bypass {
            let mut delta = (tgt_root - src_root).rem_euclid(12);
            if tgt_root > r.high_key as i16 && (raw_note as i16 + delta - 12) >= 24 {
                delta -= 12;
            }
            return Some(fold_note_into_range(raw_note as i16 + delta, r));
        }

        let iv = (raw_note as i16 - src_root).rem_euclid(12);
        let target_iv = map_chord_interval(
            iv,
            self.src_chord_type,
            r.ntt,
            r.bass_on || is_bass,
            spec,
        );

        let raw_transposed = match r.ntr {
            NtrRule::RootTrans => {
                let mut delta = (tgt_root - src_root).rem_euclid(12);
                let iv_adj = target_iv - iv;
                if tgt_root > r.high_key as i16
                    && (raw_note as i16 + iv_adj + delta - 12) >= 24
                {
                    delta -= 12;
                }
                raw_note as i16 + iv_adj + delta
            }
            NtrRule::RootFixed => {
                let target_pc = (tgt_root + target_iv).rem_euclid(12);
                let raw_pc = (raw_note as i16).rem_euclid(12);
                let mut diff = (target_pc - raw_pc).rem_euclid(12);
                if diff > 6 {
                    diff -= 12;
                }
                raw_note as i16 + diff
            }
            NtrRule::Guitar => {
                let target_pc = (tgt_root + target_iv).rem_euclid(12);
                let raw_pc = (raw_note as i16).rem_euclid(12);
                let mut diff = (target_pc - raw_pc).rem_euclid(12);
                if diff > 6 {
                    diff -= 12;
                }
                let candidate = raw_note as i16 + diff;
                if r.note_low == 0 && r.note_high == 127 {
                    let mut p = candidate;
                    while p < 40 {
                        p += 12;
                    }
                    while p > 84 {
                        p -= 12;
                    }
                    p
                } else {
                    candidate
                }
            }
        };

        Some(fold_note_into_range(raw_transposed, r))
    }
}

/// MIDI voice specification captured from `SInt` and per-section setup events.
#[derive(Debug, Clone)]
pub struct StyVoiceSpec {
    pub channel: u8,
    pub msb: u8,
    pub lsb: u8,
    pub program: u8,
    pub volume: u8,
    pub pan: i32,
    pub name_hint: String,
}

/// Normalized note event (500 ticks per quarter note, relative to section start).
#[derive(Debug, Clone)]
pub struct StyNoteEvent {
    pub start_tick: u32,
    pub duration_ticks: u32,
    pub midi_note: u8,
    pub velocity: u8,
}

/// Single accompaniment or percussion track within a style section.
#[derive(Debug, Clone)]
pub struct StyTrack {
    pub src_ch: u8,
    pub dest_ch: u8,
    pub track_slot: usize,
    pub voice: StyVoiceSpec,
    pub is_drum: bool,
    pub casm_rule: Option<CasmRule>,
    pub events: Vec<StyNoteEvent>,
}

/// Canonical style section (`Intro 1..4`, `Variation 1..4`, `Fill 1..4`, `Break`, `Ending 1..4`).
#[derive(Debug, Clone)]
pub struct StySection {
    pub id: u32,
    pub raw_name: String,
    pub name: String,
    pub length_ticks: u32,
    pub is_loop: bool,
    pub jump_to: i32,
    pub tracks: Vec<StyTrack>,
}

/// One Touch Setting (OTS) registration extracted from the `OTSc` chunk.
#[derive(Debug, Clone)]
pub struct OtsPreset {
    pub index: usize,
    pub name: String,
    pub right1_bank_msb: u8,
    pub right1_bank_lsb: u8,
    pub right1_program: u8,
    pub right2_program: Option<(u8, u8, u8)>,
    pub left_program: Option<(u8, u8, u8)>,
}

/// Complete parsed Yamaha `.STY` style document.
#[derive(Debug, Clone)]
pub struct StyDocument {
    pub title: String,
    pub file_name: String,
    pub file_path: String,
    pub format: StyFormat,
    pub tpqn: u16,
    pub bpm: f32,
    pub time_sig_num: u8,
    pub time_sig_den: u8,
    pub sections: Vec<StySection>,
    pub ots_presets: Vec<OtsPreset>,
    pub has_audio_style_chunks: bool,
    pub audio_chunk_tags: Vec<String>,
    pub diagnostics: StyDiagnostics,
}

/// Resolves a Yamaha Bank MSB/LSB/Program + track hint into a GM SoundFont `(sf2_bank, sf2_program, is_drum)`.
pub fn resolve_yamaha_voice_to_gm(
    dest_ch: u8,
    msb: u8,
    _lsb: u8,
    prog: u8,
    track_name: &str,
) -> (u16, u16, bool) {
    let upper = track_name.to_ascii_uppercase();

    // Percussion & Drum kits: dest channels 8/9, PSR Expansion drums (62), XG SFX (126), XG Drums (127).
    if dest_ch == 8 || dest_ch == 9 || msb == 62 || msb == 126 || msb == 127 {
        let sf2_prog = if msb == 127 {
            (prog & 0x7f) as u16
        } else if msb == 62 {
            16
        } else {
            0
        };
        return (128, sf2_prog, true);
    }

    // Yamaha PSR Expansion melodic banks (msb == 63): slots are pack-local, NOT GM program numbers.
    if msb == 63 {
        let gm_prog = infer_gm_program_from_hint_and_dest(&upper, dest_ch);
        return (0, gm_prog, false);
    }

    // Yamaha MegaVoice bank (msb == 8): map guitar/bass slots cleanly instead of raw GM 0..3 pianos.
    if msb == 8 {
        let gm_prog = if dest_ch == 10
            || upper.contains("BASS")
            || upper.contains("BAIXO")
            || upper == "BS"
            || upper.starts_with("BS ")
            || upper.contains("MELLOWFI")
            || (16..=31).contains(&prog)
        {
            if upper.contains("PICK") {
                34
            } else {
                33
            }
        } else if upper.contains("NYLON") || upper.contains("VIOL") {
            24
        } else if upper.contains("STEEL") || upper.contains("STGT") || prog == 1 {
            25
        } else {
            27
        };
        return (0, gm_prog, false);
    }

    // Standard GM / XG / Sweet! / Live! / Cool! / GM2 voices use the GM program number directly.
    (0, (prog & 0x7f) as u16, false)
}

fn infer_gm_program_from_hint_and_dest(upper: &str, dest_ch: u8) -> u16 {
    if upper.contains("SANF")
        || upper.contains("ACORD")
        || upper.contains("ACCORD")
        || upper.contains("MUSETT")
        || upper.contains("TANGO")
    {
        21
    } else if upper.contains("BAIXO")
        || upper.contains("BASS")
        || upper.contains("CONTR")
        || upper == "BS"
        || upper.starts_with("BS ")
    {
        33
    } else if upper.contains("GTR")
        || upper.contains("GUIT")
        || upper.contains("GT")
        || upper.contains("BAIXAO")
    {
        if upper.contains("NYLON") || upper.contains("STEEL") || upper.contains("BAIXAO") {
            25
        } else {
            27
        }
    } else if upper.contains("VIOL") {
        // "VIOLAO" vs "VIOLIN"
        if upper.contains("VIOLIN") {
            48
        } else {
            25
        }
    } else if upper.contains("EP")
        || upper.contains("E.P")
        || upper.contains("RHOD")
        || upper.contains("WURL")
    {
        4
    } else if upper.contains("PIANO") || upper.contains("GRAND") || upper == "PF" {
        0
    } else if upper.contains("ORG") || upper.contains("OGN") {
        16
    } else if upper.contains("STR") || upper.contains("CELLO") {
        48
    } else if upper.contains("PAD") {
        89
    } else if upper.contains("BRASS")
        || upper.contains("MET")
        || upper.contains("TROM")
        || upper.contains("TRUMP")
        || upper.contains("HORN")
    {
        61
    } else if upper.contains("SAX") {
        65
    } else if upper.contains("LEAD") || upper.contains("SYN") {
        81
    } else {
        match dest_ch {
            10 => 33,
            11 => 27,
            12 => 21,
            13 => 48,
            14 => 21,
            15 => 61,
            _ => 0,
        }
    }
}

fn fold_note_into_range(raw_transposed: i16, r: &CasmRangeRule) -> u8 {
    let lo = r.note_low.min(r.note_high).max(21) as i16;
    let hi = r.note_high.max(r.note_low).min(108) as i16;
    let mut pitch = raw_transposed;
    while pitch < lo && pitch + 12 <= hi {
        pitch += 12;
    }
    while pitch > hi && pitch - 12 >= lo {
        pitch -= 12;
    }
    pitch.clamp(lo, hi) as u8
}

fn source_chord_reference(src_chord_type: u8) -> (i16, i16, i16, bool) {
    match src_chord_type {
        // Maj, Maj6, Maj(9), Maj6(9)
        0 | 1 | 4 | 6 => (4, 7, 11, false),
        // Maj7, Maj7(#11), Maj7(9)
        2 | 3 | 5 => (4, 7, 11, false),
        // Aug, 7aug, Maj7aug
        7 | 28 | 29 => (4, 8, if src_chord_type == 28 { 11 } else { 10 }, false),
        // min, min6, min(9), min7, min7(9), min7(11)
        8 | 9 | 10 | 12 | 13 | 14 => (3, 7, 10, true),
        // min7b5, dim, dim7
        11 | 17 | 18 => (3, 6, if src_chord_type == 18 { 9 } else { 10 }, true),
        // minMaj7, minMaj7(9)
        15 | 16 => (3, 7, 11, true),
        // 7th family
        19 | 22 | 23 | 24 | 25 | 26 | 27 => (4, 7, 10, false),
        // 7sus4
        20 => (5, 7, 10, false),
        // 7b5
        21 => (4, 6, 10, false),
        // sus4
        32 => (5, 7, 10, false),
        // 1+2+5 / sus2
        33 => (2, 7, 10, false),
        // Default major triad/7 reference
        _ => (4, 7, 10, false),
    }
}

fn map_chord_interval(
    iv: i16,
    src_chord_type: u8,
    ntt: NttRule,
    is_bass_mode: bool,
    spec: TargetChordSpec,
) -> i16 {
    let (src_3rd, src_5th, src_7th, src_is_minor) = source_chord_reference(src_chord_type);
    let safe_7th = if spec.seventh_iv > 0 {
        spec.seventh_iv
    } else {
        0
    };

    if iv == 0 {
        return 0;
    }

    // Bass / BassOn preserves root strictly and maps 3rd/5th/7th chord tones cleanly.
    if is_bass_mode || ntt == NttRule::Bass {
        if iv == src_3rd {
            return spec.third_iv;
        }
        if iv == src_5th {
            return spec.fifth_iv;
        }
        if iv == src_7th || iv == 10 || iv == 11 {
            return safe_7th;
        }
    }

    if iv == src_3rd {
        return spec.third_iv;
    }
    if iv == src_5th {
        return spec.fifth_iv;
    }
    if iv == src_7th {
        return safe_7th;
    }

    match iv {
        // Minor/Major alternate 3rd
        3 | 4 => {
            if ntt == NttRule::Chord || ntt == NttRule::Bass {
                spec.third_iv
            } else if iv == 3 && src_is_minor {
                spec.third_iv
            } else if iv == 4 && !spec.is_major_family {
                spec.third_iv
            } else if iv == 3 && !spec.is_major_family {
                spec.third_iv
            } else {
                iv
            }
        }
        // 2nd / 9th
        2 => {
            if spec.third_iv == 2 || spec.add9 {
                2
            } else if ntt == NttRule::Chord || ntt == NttRule::Bass {
                0
            } else {
                2
            }
        }
        // 4th / 11th
        5 => {
            if spec.third_iv == 5 {
                5
            } else if ntt == NttRule::Chord {
                spec.third_iv
            } else {
                5
            }
        }
        // Tritone / b5
        6 => {
            if spec.fifth_iv == 6 {
                6
            } else if ntt == NttRule::Chord || ntt == NttRule::Bass {
                spec.fifth_iv
            } else {
                6
            }
        }
        // #5 / b6
        8 => {
            if spec.fifth_iv == 8 {
                8
            } else if ntt == NttRule::Chord || ntt == NttRule::Bass {
                spec.fifth_iv
            } else if ntt == NttRule::HarmonicMinor || !spec.is_major_family {
                8
            } else {
                9
            }
        }
        // 6th / dim7
        9 => {
            if spec.seventh_iv == 9 {
                9
            } else if ntt == NttRule::Chord || ntt == NttRule::Bass {
                spec.fifth_iv
            } else if ntt == NttRule::MelodicMinor || spec.is_major_family {
                9
            } else {
                8
            }
        }
        // b7 / Maj7
        10 => {
            if ntt == NttRule::Chord || ntt == NttRule::Bass {
                safe_7th
            } else if ntt == NttRule::HarmonicMinor && !spec.is_major_family {
                11
            } else if spec.seventh_iv == 10 || spec.seventh_iv == 11 {
                spec.seventh_iv
            } else {
                10
            }
        }
        11 => {
            if ntt == NttRule::Chord || ntt == NttRule::Bass {
                safe_7th
            } else if !spec.is_major_family && ntt == NttRule::Melody {
                if spec.seventh_iv == 11 {
                    11
                } else {
                    10
                }
            } else {
                11
            }
        }
        // b2 / b9
        1 => {
            if ntt == NttRule::Chord || ntt == NttRule::Bass {
                0
            } else {
                1
            }
        }
        _ => iv,
    }
}

/// Parses a Yamaha `.STY` file (`SFF1`, `SFF2`, or legacy `STYL`) into a `StyDocument`.
pub fn parse_sty_file(path: &Path) -> Result<StyDocument, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read STY file {}: {}", path.display(), e))?;
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("style.sty")
        .to_string();
    let file_path = path.to_string_lossy().to_string();

    let doc = parse_sty_bytes(&data, file_name, file_path)?;
    #[cfg(not(test))]
    eprintln!("[STY DIAGNOSTICS]\n{}", doc.diagnostics.summary_text());
    Ok(doc)
}

fn parse_sty_bytes(
    data: &[u8],
    file_name: String,
    file_path: String,
) -> Result<StyDocument, String> {
    if data.len() < 14 {
        return Err("STY file too short (< 14 bytes)".to_string());
    }

    let hdr_tag = &data[0..4];
    let mut format = if hdr_tag == b"MThd" {
        StyFormat::Sff1
    } else if hdr_tag == b"STYL" {
        StyFormat::LegacyStyl
    } else {
        return Err(format!(
            "Unsupported STY header {:?} (expected MThd or STYL)",
            hdr_tag
        ));
    };

    let hlen = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if 8 + hlen > data.len() || hlen < 6 {
        return Err(format!("Invalid STY header chunk length {}", hlen));
    }
    let tpqn_raw = u16::from_be_bytes([data[12], data[13]]);
    let tpqn = if tpqn_raw == 0 || (tpqn_raw & 0x8000) != 0 {
        480
    } else {
        tpqn_raw
    };

    let (mtrk_chunks, casm_chunk, otsc_chunk, top_audio_tags) =
        collect_top_level_chunks(data, 8 + hlen);
    if mtrk_chunks.is_empty() {
        return Err("No MTrk MIDI data chunk found in STY file".to_string());
    }

    let mut audio_chunk_tags = top_audio_tags;
    for tag in detect_audio_style_chunks(data) {
        if !audio_chunk_tags.contains(&tag) {
            audio_chunk_tags.push(tag);
        }
    }
    let has_audio_style_chunks = !audio_chunk_tags.is_empty();

    let casm = match casm_chunk {
        Some(chunk) => parse_casm_chunk(chunk),
        None => CasmData::default(),
    };
    if format != StyFormat::LegacyStyl && casm.has_ctb2 {
        format = StyFormat::Sff2;
    }

    let ots_presets = match otsc_chunk {
        Some(chunk) => parse_otsc_chunk(chunk),
        None => Vec::new(),
    };

    let mut midi = parse_midi_tracks(&mtrk_chunks)?;
    if format != StyFormat::LegacyStyl && midi.detected_sff2 {
        format = StyFormat::Sff2;
    }

    let title = if !midi.title.is_empty() {
        std::mem::take(&mut midi.title)
    } else {
        Path::new(&file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Yamaha Style")
            .to_string()
    };

    let bpm = midi.bpm.clamp(20.0, 300.0);
    let time_sig_num = midi.time_sig_num;
    let time_sig_den = midi.time_sig_den;
    let (sections, section_diagnostics) = build_sections(&mut midi, &casm, tpqn);
    let diagnostics = build_sty_diagnostics(
        &file_name,
        format,
        tpqn,
        bpm,
        time_sig_num,
        time_sig_den,
        &sections,
        section_diagnostics,
        has_audio_style_chunks,
        &audio_chunk_tags,
    );

    Ok(StyDocument {
        title,
        file_name,
        file_path,
        format,
        tpqn,
        bpm,
        time_sig_num,
        time_sig_den,
        sections,
        ots_presets,
        has_audio_style_chunks,
        audio_chunk_tags,
        diagnostics,
    })
}

fn is_valid_ascii_tag(tag: &[u8]) -> bool {
    tag.len() == 4 && tag.iter().all(|&b| (0x20..0x7f).contains(&b))
}

fn is_known_chunk_tag(tag: &[u8]) -> bool {
    matches!(
        tag,
        b"MTrk"
            | b"CASM"
            | b"OTSc"
            | b"FNRc"
            | b"FNRP"
            | b"MDB "
            | b"MHhd"
            | b"MspI"
            | b"CSEG"
            | b"AASM"
            | b"AFil"
            | b"AWav"
    )
}

fn detect_audio_style_chunks(data: &[u8]) -> Vec<String> {
    let mut tags = Vec::new();
    if data.len() < 8 {
        return tags;
    }
    let mut i = 0usize;
    while i + 8 <= data.len() {
        let candidate = &data[i..i + 4];
        if matches!(candidate, b"AASM" | b"AFil" | b"AWav") {
            let be_len =
                u32::from_be_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
            let le_len =
                u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
            let plausible_be = be_len > 0 && be_len < data.len();
            let plausible_le = le_len > 0 && le_len < data.len();
            if plausible_be || plausible_le {
                let tag_str = String::from_utf8_lossy(candidate).to_string();
                if !tags.contains(&tag_str) {
                    tags.push(tag_str);
                }
            }
        }
        i += 1;
    }
    tags
}

fn find_next_chunk_tag(data: &[u8], start: usize) -> Option<usize> {
    let targets: [&[u8; 4]; 3] = [b"CASM", b"OTSc", b"MTrk"];
    let mut i = start;
    while i + 8 <= data.len() {
        let candidate = &data[i..i + 4];
        if targets.iter().any(|t| *t == candidate) {
            let clen = u32::from_be_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]])
                as usize;
            if clen > 0 && i + 8 + clen <= data.len() + 4 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn collect_top_level_chunks<'a>(
    data: &'a [u8],
    start_pos: usize,
) -> (Vec<&'a [u8]>, Option<&'a [u8]>, Option<&'a [u8]>, Vec<String>) {
    let mut mtrks = Vec::new();
    let mut casm: Option<&'a [u8]> = None;
    let mut otsc: Option<&'a [u8]> = None;
    let mut audio_tags: Vec<String> = Vec::new();

    let mut pos = start_pos;
    while pos + 8 <= data.len() {
        // Odd-length MTrk chunks in older PSR styles often insert 0x00 pad byte(s) before CASM.
        while pos < data.len() && data[pos] == 0 {
            pos += 1;
        }
        if pos + 8 > data.len() {
            break;
        }
        let tag = &data[pos..pos + 4];
        if !is_valid_ascii_tag(tag) && !is_known_chunk_tag(tag) {
            if let Some(recovered) = find_next_chunk_tag(data, pos + 1) {
                pos = recovered;
                continue;
            }
            break;
        }
        let clen_be =
            u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
        let is_audio_chunk = matches!(tag, b"AASM" | b"AFil" | b"AWav");
        let clen = if is_audio_chunk {
            let clen_le = u32::from_le_bytes([
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;
            if clen_be
                .checked_add(pos + 8)
                .is_some_and(|end| end <= data.len())
            {
                clen_be
            } else if clen_le > 0
                && clen_le
                    .checked_add(pos + 8)
                    .is_some_and(|end| end <= data.len())
            {
                clen_le
            } else {
                clen_be
            }
        } else {
            clen_be
        };
        let payload_start = pos + 8;
        let payload_end = payload_start.saturating_add(clen).min(data.len());
        let payload = &data[payload_start..payload_end];

        if tag == b"MTrk" {
            mtrks.push(payload);
        } else if tag == b"CASM" {
            casm = Some(payload);
        } else if tag == b"OTSc" {
            otsc = Some(payload);
        } else if is_audio_chunk {
            let tag_str = String::from_utf8_lossy(tag).to_string();
            if !audio_tags.contains(&tag_str) {
                audio_tags.push(tag_str);
            }
        }

        pos = payload_end;
        if payload_end == data.len() {
            break;
        }
    }

    // Defensive fallback scan in case a non-zero corrupt pad byte appeared after an odd chunk.
    if casm.is_none() {
        if let Some(idx) = find_next_chunk_tag(data, start_pos) {
            if &data[idx..idx + 4] == b"CASM" && idx + 8 <= data.len() {
                let clen = u32::from_be_bytes([
                    data[idx + 4],
                    data[idx + 5],
                    data[idx + 6],
                    data[idx + 7],
                ]) as usize;
                let end = (idx + 8).saturating_add(clen).min(data.len());
                casm = Some(&data[idx + 8..end]);
            }
        }
    }
    if otsc.is_none() {
        let mut scan = start_pos;
        while scan + 8 <= data.len() {
            if &data[scan..scan + 4] == b"OTSc" {
                let clen = u32::from_be_bytes([
                    data[scan + 4],
                    data[scan + 5],
                    data[scan + 6],
                    data[scan + 7],
                ]) as usize;
                let end = (scan + 8).saturating_add(clen).min(data.len());
                otsc = Some(&data[scan + 8..end]);
                break;
            }
            scan += 1;
        }
    }

    (mtrks, casm, otsc, audio_tags)
}

#[derive(Default)]
struct CasmData {
    has_ctb2: bool,
    by_section: HashMap<String, Vec<CasmRule>>,
    all_rules: Vec<CasmRule>,
}

fn decode_latin1(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| b as char).collect()
}

fn decode_ntr(v: u8) -> NtrRule {
    match v {
        0 => NtrRule::RootTrans,
        1 => NtrRule::RootFixed,
        2 => NtrRule::Guitar,
        _ => NtrRule::RootFixed,
    }
}

fn decode_ntt(v: u8) -> NttRule {
    match v & 0x7f {
        0 => NttRule::Bypass,
        1 => NttRule::Melody,
        2 => NttRule::Chord,
        3 => NttRule::Bass,
        4 => NttRule::MelodicMinor,
        5 => NttRule::HarmonicMinor,
        _ => NttRule::Chord,
    }
}

fn decode_rtr(v: u8) -> RtrRule {
    match v {
        0 => RtrRule::Stop,
        1 => RtrRule::PitchShift,
        2 => RtrRule::PitchShiftToRoot,
        3 => RtrRule::Retrigger,
        4 => RtrRule::RetriggerToRoot,
        5 => RtrRule::NoteGenerator,
        _ => RtrRule::PitchShift,
    }
}

fn decode_range_rule(b: &[u8]) -> CasmRangeRule {
    CasmRangeRule {
        ntr: decode_ntr(b[0]),
        ntt: decode_ntt(b[1]),
        bass_on: (b[1] & 0x80) != 0,
        high_key: b[2].min(11),
        note_low: b[3],
        note_high: b[4],
        rtr: decode_rtr(b[5]),
    }
}

fn parse_casm_chunk(chunk: &[u8]) -> CasmData {
    let mut casm = CasmData::default();
    let mut pos = 0usize;

    while pos + 8 <= chunk.len() {
        while pos < chunk.len() && chunk[pos] == 0 {
            pos += 1;
        }
        if pos + 8 > chunk.len() {
            break;
        }
        let tag = &chunk[pos..pos + 4];
        let clen =
            u32::from_be_bytes([chunk[pos + 4], chunk[pos + 5], chunk[pos + 6], chunk[pos + 7]])
                as usize;
        let end = (pos + 8).saturating_add(clen).min(chunk.len());
        let sub = &chunk[pos + 8..end];
        pos = end;

        if tag.eq_ignore_ascii_case(b"CSEG") {
            let mut sdec_tokens: Vec<String> = Vec::new();
            let mut rules: Vec<CasmRule> = Vec::new();
            let mut cntts: Vec<(u8, u8)> = Vec::new();

            let mut sp = 0usize;
            while sp + 8 <= sub.len() {
                while sp < sub.len() && sub[sp] == 0 {
                    sp += 1;
                }
                if sp + 8 > sub.len() {
                    break;
                }
                let stag = &sub[sp..sp + 4];
                let slen = u32::from_be_bytes([
                    sub[sp + 4],
                    sub[sp + 5],
                    sub[sp + 6],
                    sub[sp + 7],
                ]) as usize;
                let send = (sp + 8).saturating_add(slen).min(sub.len());
                let sd = &sub[sp + 8..send];
                sp = send;

                if stag.eq_ignore_ascii_case(b"Sdec") {
                    let text = decode_latin1(sd);
                    for tok in text.split(',') {
                        let trimmed = tok.trim();
                        if !trimmed.is_empty() {
                            sdec_tokens.push(trimmed.to_ascii_lowercase());
                        }
                    }
                } else if stag.eq_ignore_ascii_case(b"Ctab") && sd.len() >= 26 {
                    let r = decode_range_rule(&sd[20..26]);
                    rules.push(CasmRule {
                        src_ch: sd[0],
                        dest_ch: sd[9],
                        name: decode_latin1(&sd[1..9]).trim().to_string(),
                        editable: sd[10],
                        note_mute: u16::from_be_bytes([sd[11], sd[12]]),
                        chord_mute: ((sd[13] as u64) << 32)
                            | ((sd[14] as u64) << 24)
                            | ((sd[15] as u64) << 16)
                            | ((sd[16] as u64) << 8)
                            | (sd[17] as u64),
                        src_root: sd[18],
                        src_chord_type: sd[19],
                        low_split: None,
                        high_split: None,
                        ranges: [r.clone(), r.clone(), r],
                    });
                } else if stag.eq_ignore_ascii_case(b"Ctb2") && sd.len() >= 40 {
                    casm.has_ctb2 = true;
                    rules.push(CasmRule {
                        src_ch: sd[0],
                        dest_ch: sd[9],
                        name: decode_latin1(&sd[1..9]).trim().to_string(),
                        editable: sd[10],
                        note_mute: u16::from_be_bytes([sd[11], sd[12]]),
                        chord_mute: ((sd[13] as u64) << 32)
                            | ((sd[14] as u64) << 24)
                            | ((sd[15] as u64) << 16)
                            | ((sd[16] as u64) << 8)
                            | (sd[17] as u64),
                        src_root: sd[18],
                        src_chord_type: sd[19],
                        low_split: Some(sd[20]),
                        high_split: Some(sd[21]),
                        ranges: [
                            decode_range_rule(&sd[22..28]),
                            decode_range_rule(&sd[28..34]),
                            decode_range_rule(&sd[34..40]),
                        ],
                    });
                } else if stag.eq_ignore_ascii_case(b"Cntt") && sd.len() >= 2 {
                    cntts.push((sd[0], sd[1]));
                }
            }

            if !cntts.is_empty() {
                for (ch, ntt_byte) in cntts {
                    let ntt = decode_ntt(ntt_byte);
                    let bass_on = (ntt_byte & 0x80) != 0;
                    for rule in &mut rules {
                        if rule.src_ch == ch {
                            for range in &mut rule.ranges {
                                range.ntt = ntt;
                                range.bass_on = bass_on;
                            }
                        }
                    }
                }
            }

            casm.all_rules.extend(rules.iter().cloned());
            for sec_key in sdec_tokens {
                casm.by_section
                    .entry(sec_key)
                    .or_default()
                    .extend(rules.iter().cloned());
            }
        }
    }

    casm
}

fn read_vlq(data: &[u8], pos: &mut usize) -> Option<u32> {
    let mut val = 0_u32;
    for _ in 0..4 {
        let b = *data.get(*pos)?;
        *pos += 1;
        val = (val << 7) | (b & 0x7f) as u32;
        if (b & 0x80) == 0 {
            return Some(val);
        }
    }
    Some(val)
}

fn parse_otsc_chunk(chunk: &[u8]) -> Vec<OtsPreset> {
    let mut presets = Vec::new();
    let mut pos = 0usize;
    let mut idx = 0usize;

    while pos + 8 <= chunk.len() && idx < 4 {
        while pos < chunk.len() && chunk[pos] == 0 {
            pos += 1;
        }
        if pos + 8 > chunk.len() {
            break;
        }
        let tag = &chunk[pos..pos + 4];
        let clen =
            u32::from_be_bytes([chunk[pos + 4], chunk[pos + 5], chunk[pos + 6], chunk[pos + 7]])
                as usize;
        let end = (pos + 8).saturating_add(clen).min(chunk.len());
        let payload = &chunk[pos + 8..end];
        pos = end;

        if tag != b"MTrk" {
            continue;
        }

        let mut ch_msb = [0_u8; 16];
        let mut ch_lsb = [0_u8; 16];
        let mut ch_prog = [0_u8; 16];
        let mut ch_has_pc = [false; 16];
        let mut name_hint = String::new();

        let mut p = 0usize;
        let mut running_status = 0_u8;
        while p < payload.len() {
            if read_vlq(payload, &mut p).is_none() || p >= payload.len() {
                break;
            }
            let first = payload[p];
            let status = if (first & 0x80) != 0 {
                p += 1;
                if first < 0xf0 {
                    running_status = first;
                } else {
                    running_status = 0;
                }
                first
            } else {
                if running_status == 0 {
                    break;
                }
                running_status
            };

            if status == 0xff {
                if p >= payload.len() {
                    break;
                }
                let mtype = payload[p];
                p += 1;
                let Some(mlen) = read_vlq(payload, &mut p) else {
                    break;
                };
                let mend = p.saturating_add(mlen as usize).min(payload.len());
                if matches!(mtype, 1 | 3 | 6) && name_hint.is_empty() {
                    let s = decode_latin1(&payload[p..mend]).trim().to_string();
                    if !s.is_empty() {
                        name_hint = s;
                    }
                }
                p = mend;
                if mtype == 0x2f {
                    break;
                }
            } else if status == 0xf0 || status == 0xf7 {
                let Some(slen) = read_vlq(payload, &mut p) else {
                    break;
                };
                p = p.saturating_add(slen as usize).min(payload.len());
            } else {
                let cmd = status & 0xf0;
                let ch = (status & 0x0f) as usize;
                if cmd == 0xc0 || cmd == 0xd0 {
                    let Some(&d1) = payload.get(p) else {
                        break;
                    };
                    p += 1;
                    if cmd == 0xc0 {
                        ch_prog[ch] = d1;
                        ch_has_pc[ch] = true;
                    }
                } else {
                    let (Some(&d1), Some(&d2)) = (payload.get(p), payload.get(p + 1)) else {
                        break;
                    };
                    p += 2;
                    if cmd == 0xb0 {
                        if d1 == 0 {
                            ch_msb[ch] = d2;
                        } else if d1 == 32 {
                            ch_lsb[ch] = d2;
                        }
                    }
                }
            }
        }

        if ch_has_pc[0] || ch_has_pc[1] || ch_has_pc[3] {
            presets.push(OtsPreset {
                index: idx,
                name: if name_hint.is_empty() {
                    format!("OTS {}", idx + 1)
                } else {
                    name_hint
                },
                right1_bank_msb: ch_msb[0],
                right1_bank_lsb: ch_lsb[0],
                right1_program: ch_prog[0],
                right2_program: if ch_has_pc[1] {
                    Some((ch_msb[1], ch_lsb[1], ch_prog[1]))
                } else {
                    None
                },
                left_program: if ch_has_pc[3] {
                    Some((ch_msb[3], ch_lsb[3], ch_prog[3]))
                } else {
                    None
                },
            });
        }
        idx += 1;
    }

    presets
}

#[derive(Clone, Copy)]
struct ChannelState {
    msb: u8,
    lsb: u8,
    program: u8,
    volume: u8,
    pan: i32,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            msb: 0,
            lsb: 0,
            program: 0,
            volume: 100,
            pan: 0,
        }
    }
}

#[derive(Clone)]
struct RawNote {
    channel: u8,
    start_tick: u32,
    end_tick: u32,
    midi_note: u8,
    velocity: u8,
    voice: ChannelState,
}

#[derive(Clone)]
struct RawMarker {
    tick: u32,
    name: String,
    event_order: usize,
}

struct ParsedMidi {
    title: String,
    bpm: f32,
    time_sig_num: u8,
    time_sig_den: u8,
    detected_sff2: bool,
    markers: Vec<RawMarker>,
    notes: Vec<RawNote>,
    max_tick: u32,
    drum_msb_channels: HashSet<u8>,
}

fn cc10_to_signed_pan(cc10: u8) -> i32 {
    if cc10 == 64 {
        0
    } else if cc10 < 64 {
        (((cc10 as i32 - 64) * 100) / 64).clamp(-100, 100)
    } else {
        (((cc10 as i32 - 64) * 100) / 63).clamp(-100, 100)
    }
}

fn parse_midi_tracks(mtrks: &[&[u8]]) -> Result<ParsedMidi, String> {
    let mut title = String::new();
    let mut bpm = 120.0_f32;
    let mut has_bpm = false;
    let mut time_sig_num = 4_u8;
    let mut time_sig_den = 4_u8;
    let mut has_tsig = false;
    let mut detected_sff2 = false;
    let mut markers: Vec<RawMarker> = Vec::new();
    let mut notes: Vec<RawNote> = Vec::new();
    let mut max_tick = 0_u32;
    let mut drum_msb_channels: HashSet<u8> = HashSet::new();

    for trk in mtrks {
        let mut p = 0usize;
        let mut tick = 0_u32;
        let mut running_status = 0_u8;
        let mut ch_state = [ChannelState::default(); 16];
        // Active note-ons: (channel, pitch) -> index into `notes`
        let mut active: [[Option<usize>; 128]; 16] = [[None; 128]; 16];

        while p < trk.len() {
            let Some(dt) = read_vlq(trk, &mut p) else {
                break;
            };
            tick = tick.saturating_add(dt);
            if tick > max_tick {
                max_tick = tick;
            }
            if p >= trk.len() {
                break;
            }
            let first = trk[p];
            let status = if (first & 0x80) != 0 {
                p += 1;
                if first < 0xf0 {
                    running_status = first;
                } else {
                    running_status = 0;
                }
                first
            } else {
                if running_status == 0 {
                    break;
                }
                running_status
            };

            if status == 0xff {
                if p >= trk.len() {
                    break;
                }
                let mtype = trk[p];
                p += 1;
                let Some(mlen) = read_vlq(trk, &mut p) else {
                    break;
                };
                let mend = p.saturating_add(mlen as usize).min(trk.len());
                let mdata = &trk[p..mend];
                p = mend;

                match mtype {
                    0x03 => {
                        if title.is_empty() {
                            let t = decode_latin1(mdata).trim().to_string();
                            if !t.is_empty() {
                                title = t;
                            }
                        }
                    }
                    0x06 => {
                        let m = decode_latin1(mdata).trim().to_string();
                        if m.eq_ignore_ascii_case("SFF2") {
                            detected_sff2 = true;
                        } else if !m.is_empty()
                            && !m.eq_ignore_ascii_case("SFF1")
                            && !m.eq_ignore_ascii_case("SInt")
                        {
                            markers.push(RawMarker {
                                tick,
                                name: m,
                                event_order: notes.len(),
                            });
                        }
                    }
                    0x51 if mdata.len() >= 3 => {
                        if !has_bpm {
                            let us = ((mdata[0] as u32) << 16)
                                | ((mdata[1] as u32) << 8)
                                | (mdata[2] as u32);
                            if us > 0 {
                                bpm = 60_000_000.0 / us as f32;
                                has_bpm = true;
                            }
                        }
                    }
                    0x58 if mdata.len() >= 2 => {
                        if !has_tsig {
                            let num = mdata[0].max(1);
                            let den = 1_u8.checked_shl((mdata[1] as u32).min(6)).unwrap_or(4);
                            time_sig_num = num;
                            time_sig_den = den;
                            has_tsig = true;
                        }
                    }
                    0x2f => {
                        break;
                    }
                    _ => {}
                }
            } else if status == 0xf0 || status == 0xf7 {
                let Some(slen) = read_vlq(trk, &mut p) else {
                    break;
                };
                p = p.saturating_add(slen as usize).min(trk.len());
            } else {
                let cmd = status & 0xf0;
                let ch = (status & 0x0f) as usize;
                if cmd == 0xc0 || cmd == 0xd0 {
                    let Some(&d1) = trk.get(p) else {
                        break;
                    };
                    p += 1;
                    if cmd == 0xc0 {
                        ch_state[ch].program = d1 & 0x7f;
                    }
                } else {
                    let (Some(&d1), Some(&d2)) = (trk.get(p), trk.get(p + 1)) else {
                        break;
                    };
                    p += 2;
                    match cmd {
                        0xb0 => match d1 {
                            0 => {
                                let msb = d2 & 0x7f;
                                ch_state[ch].msb = msb;
                                if matches!(msb, 62 | 126 | 127) {
                                    drum_msb_channels.insert(ch as u8);
                                }
                            }
                            32 => ch_state[ch].lsb = d2 & 0x7f,
                            7 => ch_state[ch].volume = d2 & 0x7f,
                            10 => ch_state[ch].pan = cc10_to_signed_pan(d2 & 0x7f),
                            _ => {}
                        },
                        0x90 if d2 > 0 => {
                            let pitch = (d1 & 0x7f) as usize;
                            if let Some(prev_idx) = active[ch][pitch].take() {
                                if notes[prev_idx].end_tick == 0 {
                                    notes[prev_idx].end_tick = tick.max(notes[prev_idx].start_tick + 1);
                                }
                            }
                            let idx = notes.len();
                            notes.push(RawNote {
                                channel: ch as u8,
                                start_tick: tick,
                                end_tick: 0,
                                midi_note: pitch as u8,
                                velocity: d2 & 0x7f,
                                voice: ch_state[ch],
                            });
                            active[ch][pitch] = Some(idx);
                        }
                        0x80 | 0x90 => {
                            let pitch = (d1 & 0x7f) as usize;
                            if let Some(prev_idx) = active[ch][pitch].take() {
                                notes[prev_idx].end_tick = tick.max(notes[prev_idx].start_tick + 1);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Close any still-open notes at EOF tick.
        for ch_slots in &mut active {
            for slot in ch_slots.iter_mut() {
                if let Some(idx) = slot.take() {
                    notes[idx].end_tick = max_tick.max(notes[idx].start_tick + 1);
                }
            }
        }
    }

    Ok(ParsedMidi {
        title,
        bpm,
        time_sig_num,
        time_sig_den,
        detected_sff2,
        markers,
        notes,
        max_tick,
        drum_msb_channels,
    })
}

fn scale_ticks(raw: u32, tpqn: u16) -> u32 {
    let div = tpqn.max(1) as u128;
    ((raw as u128 * 500) / div) as u32
}

fn canonical_section_meta(raw_name: &str) -> (String, bool, i32) {
    let trimmed = raw_name.trim();
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "intro a" => ("Intro 1".to_string(), false, -2),
        "intro b" => ("Intro 2".to_string(), false, -2),
        "intro c" => ("Intro 3".to_string(), false, -2),
        "intro d" => ("Intro 4".to_string(), false, -2),
        "main a" => ("Variation 1".to_string(), true, 0),
        "main b" => ("Variation 2".to_string(), true, 0),
        "main c" => ("Variation 3".to_string(), true, 0),
        "main d" => ("Variation 4".to_string(), true, 0),
        "fill in aa" => ("Fill 1".to_string(), false, -2),
        "fill in bb" => ("Fill 2".to_string(), false, -2),
        "fill in cc" => ("Fill 3".to_string(), false, -2),
        "fill in dd" => ("Fill 4".to_string(), false, -2),
        "fill in ab" => ("Fill AB".to_string(), false, -2),
        "fill in ba" => ("Break".to_string(), false, -2),
        "ending a" => ("Ending 1".to_string(), false, 0),
        "ending b" => ("Ending 2".to_string(), false, 0),
        "ending c" => ("Ending 3".to_string(), false, 0),
        "ending d" => ("Ending 4".to_string(), false, 0),
        _ => (trimmed.to_string(), false, 0),
    }
}

fn dest_ch_to_track_slot(dest_ch: u8) -> usize {
    match dest_ch {
        9 => 0,
        8 => 1,
        10 => 2,
        11 => 3,
        12 => 4,
        13 => 5,
        14 => 6,
        15 => 7,
        0 => 3,
        1 => 4,
        2 => 5,
        3 => 6,
        4 => 7,
        5 => 5,
        6 => 6,
        7 => 7,
        _ => 3,
    }
}

fn fallback_dest_ch(src_ch: u8) -> u8 {
    match src_ch {
        8..=15 => src_ch,
        0 => 11,
        1 => 12,
        2 => 13,
        3 => 14,
        4 => 15,
        5 => 13,
        6 => 14,
        7 => 15,
        _ => 11,
    }
}

fn lookup_casm_rules<'a>(casm: &'a CasmData, raw_name: &str) -> Vec<&'a CasmRule> {
    let key = raw_name.trim().to_ascii_lowercase();
    if let Some(rules) = casm.by_section.get(&key) {
        return rules.iter().collect();
    }
    // Alias fallbacks for cross-fill / partial Sdec styles.
    let aliases: &[&str] = match key.as_str() {
        "fill in ab" => &["fill in bb", "fill in aa", "main b", "main a"],
        "fill in ba" => &["fill in aa", "main a"],
        "fill in aa" => &["main a"],
        "fill in bb" => &["main b", "main a"],
        "fill in cc" => &["main c", "main a"],
        "fill in dd" => &["main d", "main a"],
        "intro a" => &["main a"],
        "intro b" => &["main b", "main a"],
        "intro c" => &["main c", "main a"],
        "intro d" => &["main d", "main a"],
        "ending a" => &["main a"],
        "ending b" => &["main b", "main a"],
        "ending c" => &["main c", "main a"],
        "ending d" => &["main d", "main a"],
        _ => &[],
    };
    for alias in aliases {
        if let Some(rules) = casm.by_section.get(*alias) {
            return rules.iter().collect();
        }
    }
    Vec::new()
}

fn is_dummy_sint_voice(v: &ChannelState) -> bool {
    v.msb == 0 && v.program == 0 && (v.lsb == 113 || v.lsb == 0)
}

fn record_global_casm_role(
    rule: &CasmRule,
    global_ch_role: &mut HashMap<u8, (u8, String)>,
    global_drum_channels: &mut HashSet<u8>,
) {
    let is_drum_rule = rule.dest_ch == 8 || rule.dest_ch == 9;
    if is_drum_rule {
        global_drum_channels.insert(rule.src_ch);
    }
    match global_ch_role.get_mut(&rule.src_ch) {
        Some((existing_dest, existing_name)) => {
            let existing_is_drum = *existing_dest == 8 || *existing_dest == 9;
            if is_drum_rule && !existing_is_drum {
                *existing_dest = rule.dest_ch;
                *existing_name = rule.name.clone();
            }
        }
        None => {
            global_ch_role.insert(rule.src_ch, (rule.dest_ch, rule.name.clone()));
        }
    }
}

fn resolve_effective_voice(
    raw_notes: &[&RawNote],
    src_ch: u8,
    target_is_drum: bool,
    global_ch_real_voice: &HashMap<u8, ChannelState>,
    global_ch_drum_voice: &HashMap<u8, ChannelState>,
    global_ch_melodic_voice: &HashMap<u8, ChannelState>,
) -> ChannelState {
    let first_voice = raw_notes[0].voice;
    if !is_dummy_sint_voice(&first_voice) {
        return first_voice;
    }
    if let Some(n) = raw_notes.iter().find(|n| {
        !is_dummy_sint_voice(&n.voice)
            && (target_is_drum || !matches!(n.voice.msb, 62 | 126 | 127))
    }) {
        return n.voice;
    }
    if target_is_drum {
        if let Some(&v) = global_ch_drum_voice.get(&src_ch) {
            return v;
        }
        if let Some(&v) = global_ch_real_voice.get(&src_ch) {
            if !matches!(v.msb, 63 | 8) {
                return v;
            }
        }
    } else if let Some(&v) = global_ch_melodic_voice.get(&src_ch) {
        return v;
    }
    first_voice
}

struct BuiltSection {
    section: StySection,
    raw_start: u32,
    raw_end: u32,
}

fn build_sections(
    midi: &mut ParsedMidi,
    casm: &CasmData,
    tpqn: u16,
) -> (Vec<StySection>, Vec<StySectionDiagnostic>) {
    let mut raw_sections: Vec<(String, u32, u32)> = Vec::new();

    if midi.markers.is_empty() {
        if midi.max_tick > 0 || !midi.notes.is_empty() {
            raw_sections.push(("Main A".to_string(), 0, midi.max_tick.max(tpqn as u32)));
        }
    } else {
        // Keep chronological order of markers.
        midi.markers
            .sort_by_key(|m| (m.tick, m.event_order));
        for i in 0..midi.markers.len() {
            let start = midi.markers[i].tick;
            let end = if i + 1 < midi.markers.len() {
                midi.markers[i + 1].tick
            } else {
                midi.max_tick
            };
            if end <= start {
                continue;
            }
            raw_sections.push((midi.markers[i].name.clone(), start, end));
        }
    }

    // Build style-wide role, drum-channel, and real-voice inheritance maps before processing sections.
    let mut casm_keys: Vec<&String> = casm.by_section.keys().collect();
    casm_keys.sort_by_key(|k| (!k.starts_with("main "), k.as_str()));

    let mut global_ch_role: HashMap<u8, (u8, String)> = HashMap::new();
    let mut global_drum_channels: HashSet<u8> = midi.drum_msb_channels.clone();
    for k in casm_keys {
        for rule in &casm.by_section[k] {
            record_global_casm_role(rule, &mut global_ch_role, &mut global_drum_channels);
        }
    }
    for rule in &casm.all_rules {
        record_global_casm_role(rule, &mut global_ch_role, &mut global_drum_channels);
    }

    let mut global_ch_real_voice: HashMap<u8, ChannelState> = HashMap::new();
    let mut global_ch_drum_voice: HashMap<u8, ChannelState> = HashMap::new();
    let mut global_ch_melodic_voice: HashMap<u8, ChannelState> = HashMap::new();
    for note in &midi.notes {
        if matches!(note.voice.msb, 62 | 126 | 127) {
            global_drum_channels.insert(note.channel);
            global_ch_drum_voice.entry(note.channel).or_insert(note.voice);
        } else if !is_dummy_sint_voice(&note.voice) {
            global_ch_melodic_voice
                .entry(note.channel)
                .or_insert(note.voice);
        }
        if !is_dummy_sint_voice(&note.voice) {
            global_ch_real_voice
                .entry(note.channel)
                .or_insert(note.voice);
        }
    }

    // Deduplicate identical section names keeping the richest/longest span.
    let mut deduped: Vec<BuiltSection> = Vec::new();
    let mut seen_canon_idx: HashMap<String, usize> = HashMap::new();

    for (raw_name, raw_start, raw_end) in raw_sections {
        let (canon_name, is_loop, jump_to) = canonical_section_meta(&raw_name);
        let length_ticks = scale_ticks(raw_end.saturating_sub(raw_start), tpqn).max(500);

        // Group notes falling into [raw_start, raw_end) by source MIDI channel.
        let mut notes_by_ch: HashMap<u8, Vec<&RawNote>> = HashMap::new();
        for note in &midi.notes {
            if note.start_tick >= raw_start && note.start_tick < raw_end {
                notes_by_ch.entry(note.channel).or_default().push(note);
            }
        }

        let section_rules = lookup_casm_rules(casm, &raw_name);
        let mut tracks: Vec<StyTrack> = Vec::new();

        let mut active_channels: Vec<u8> = notes_by_ch.keys().copied().collect();
        active_channels.sort_unstable();

        for src_ch in active_channels {
            let raw_notes = &notes_by_ch[&src_ch];
            if raw_notes.is_empty() {
                continue;
            }

            // Collect all distinct (src_ch, dest_ch, chord_mute) CASM rules matching src_ch.
            let mut matching_rules: Vec<&CasmRule> = Vec::new();
            let mut seen_rule_keys: HashSet<(u8, u8, u64)> = HashSet::new();
            for rule in &section_rules {
                if rule.src_ch == src_ch {
                    let k = (rule.src_ch, rule.dest_ch, rule.chord_mute);
                    if seen_rule_keys.insert(k) {
                        matching_rules.push(*rule);
                    }
                }
            }

            if matching_rules.is_empty() {
                let (mut dest_ch, mut name_hint) = match global_ch_role.get(&src_ch) {
                    Some((d, n)) => (*d, n.clone()),
                    None => {
                        let d = fallback_dest_ch(src_ch);
                        (d, format!("Ch {}", d + 1))
                    }
                };
                if global_drum_channels.contains(&src_ch) {
                    dest_ch = if dest_ch == 8 || dest_ch == 9 {
                        dest_ch
                    } else {
                        9
                    };
                    if name_hint.starts_with("Ch ") {
                        name_hint = if dest_ch == 8 {
                            "Rhythm1".to_string()
                        } else {
                            "Rhythm2".to_string()
                        };
                    }
                }
                let target_is_drum = dest_ch == 8
                    || dest_ch == 9
                    || global_drum_channels.contains(&src_ch)
                    || matches!(raw_notes[0].voice.msb, 62 | 126 | 127);
                let effective_voice = resolve_effective_voice(
                    raw_notes,
                    src_ch,
                    target_is_drum,
                    &global_ch_real_voice,
                    &global_ch_drum_voice,
                    &global_ch_melodic_voice,
                );
                let is_drum = target_is_drum || matches!(effective_voice.msb, 62 | 126 | 127);
                let is_megavoice = !is_drum && effective_voice.msb == 8;
                let events = build_track_events(
                    raw_notes,
                    raw_start,
                    raw_end,
                    length_ticks,
                    tpqn,
                    is_megavoice,
                );
                if !events.is_empty() {
                    tracks.push(StyTrack {
                        src_ch,
                        dest_ch,
                        track_slot: dest_ch_to_track_slot(dest_ch),
                        voice: StyVoiceSpec {
                            channel: src_ch,
                            msb: effective_voice.msb,
                            lsb: effective_voice.lsb,
                            program: effective_voice.program,
                            volume: effective_voice.volume,
                            pan: effective_voice.pan,
                            name_hint,
                        },
                        is_drum,
                        casm_rule: None,
                        events,
                    });
                }
            } else {
                let mut cached_standard: Option<Vec<StyNoteEvent>> = None;
                let mut cached_megavoice: Option<Vec<StyNoteEvent>> = None;
                for rule in matching_rules {
                    let rule_is_drum = rule.dest_ch == 8
                        || rule.dest_ch == 9
                        || matches!(raw_notes[0].voice.msb, 62 | 126 | 127);
                    let effective_voice = resolve_effective_voice(
                        raw_notes,
                        src_ch,
                        rule_is_drum,
                        &global_ch_real_voice,
                        &global_ch_drum_voice,
                        &global_ch_melodic_voice,
                    );
                    let is_drum = rule.dest_ch == 8
                        || rule.dest_ch == 9
                        || matches!(effective_voice.msb, 62 | 126 | 127);
                    let is_megavoice = !is_drum && effective_voice.msb == 8;
                    let events = if is_megavoice {
                        cached_megavoice
                            .get_or_insert_with(|| {
                                build_track_events(
                                    raw_notes,
                                    raw_start,
                                    raw_end,
                                    length_ticks,
                                    tpqn,
                                    true,
                                )
                            })
                            .clone()
                    } else {
                        cached_standard
                            .get_or_insert_with(|| {
                                build_track_events(
                                    raw_notes,
                                    raw_start,
                                    raw_end,
                                    length_ticks,
                                    tpqn,
                                    false,
                                )
                            })
                            .clone()
                    };
                    if events.is_empty() {
                        continue;
                    }
                    tracks.push(StyTrack {
                        src_ch,
                        dest_ch: rule.dest_ch,
                        track_slot: dest_ch_to_track_slot(rule.dest_ch),
                        voice: StyVoiceSpec {
                            channel: src_ch,
                            msb: effective_voice.msb,
                            lsb: effective_voice.lsb,
                            program: effective_voice.program,
                            volume: effective_voice.volume,
                            pan: effective_voice.pan,
                            name_hint: rule.name.clone(),
                        },
                        is_drum,
                        casm_rule: Some(rule.clone()),
                        events,
                    });
                }
            }
        }

        tracks.sort_by_key(|t| (t.track_slot, t.dest_ch, t.src_ch));

        let total_notes: usize = tracks.iter().map(|t| t.events.len()).sum();
        if let Some(&existing_idx) = seen_canon_idx.get(&canon_name) {
            let existing_notes: usize = deduped[existing_idx]
                .section
                .tracks
                .iter()
                .map(|t| t.events.len())
                .sum();
            if total_notes > existing_notes {
                let keep_id = deduped[existing_idx].section.id;
                deduped[existing_idx] = BuiltSection {
                    section: StySection {
                        id: keep_id,
                        raw_name,
                        name: canon_name,
                        length_ticks,
                        is_loop,
                        jump_to,
                        tracks,
                    },
                    raw_start,
                    raw_end,
                };
            }
        } else {
            let id = (deduped.len() + 1) as u32;
            seen_canon_idx.insert(canon_name.clone(), deduped.len());
            deduped.push(BuiltSection {
                section: StySection {
                    id,
                    raw_name,
                    name: canon_name,
                    length_ticks,
                    is_loop,
                    jump_to,
                    tracks,
                },
                raw_start,
                raw_end,
            });
        }
    }

    // Wire Fill 1..4 / Fill AB `jump_to` to their target Variation section IDs (> 0).
    let variation_ids: HashMap<String, i32> = deduped
        .iter()
        .filter(|b| b.section.name.starts_with("Variation "))
        .map(|b| (b.section.name.clone(), b.section.id as i32))
        .collect();

    for built in &mut deduped {
        let target_name = match built.section.name.as_str() {
            "Fill 1" => Some("Variation 1"),
            "Fill 2" | "Fill AB" => Some("Variation 2"),
            "Fill 3" => Some("Variation 3"),
            "Fill 4" => Some("Variation 4"),
            _ => None,
        };
        if let Some(tname) = target_name {
            built.section.jump_to = variation_ids.get(tname).copied().unwrap_or(-2);
        }
    }

    let mut sections = Vec::with_capacity(deduped.len());
    let mut section_diagnostics = Vec::with_capacity(deduped.len());
    for built in deduped {
        let note_count = built.section.tracks.iter().map(|t| t.events.len()).sum();
        section_diagnostics.push(StySectionDiagnostic {
            raw_name: built.section.raw_name.clone(),
            canonical_name: built.section.name.clone(),
            raw_start_tick: built.raw_start,
            raw_end_tick: built.raw_end,
            length_ticks_500: built.section.length_ticks,
            note_count,
        });
        sections.push(built.section);
    }

    (sections, section_diagnostics)
}

fn build_track_events(
    raw_notes: &[&RawNote],
    raw_start: u32,
    raw_end: u32,
    length_ticks: u32,
    tpqn: u16,
    filter_megavoice_keyswitches: bool,
) -> Vec<StyNoteEvent> {
    if length_ticks == 0 {
        return Vec::new();
    }
    let mut events = Vec::with_capacity(raw_notes.len());
    for note in raw_notes {
        if filter_megavoice_keyswitches && (note.midi_note < 24 || note.midi_note >= 96) {
            continue;
        }
        let rel_start = note.start_tick.saturating_sub(raw_start);
        let clamped_end = if note.end_tick > note.start_tick {
            note.end_tick.min(raw_end)
        } else {
            (note.start_tick + 1).min(raw_end)
        };
        let rel_dur = clamped_end.saturating_sub(note.start_tick).max(1);

        let start_tick = scale_ticks(rel_start, tpqn).min(length_ticks - 1);
        let max_dur = (length_ticks - start_tick).max(1);
        let duration_ticks = scale_ticks(rel_dur, tpqn).clamp(1, max_dur);

        events.push(StyNoteEvent {
            start_tick,
            duration_ticks,
            midi_note: note.midi_note,
            velocity: note.velocity,
        });
    }
    events
}

#[allow(clippy::too_many_arguments)]
fn build_sty_diagnostics(
    file_name: &str,
    format: StyFormat,
    tpqn: u16,
    bpm: f32,
    time_sig_num: u8,
    time_sig_den: u8,
    sections: &[StySection],
    section_diagnostics: Vec<StySectionDiagnostic>,
    has_audio_style_chunks: bool,
    audio_chunk_tags: &[String],
) -> StyDiagnostics {
    let mut channels = Vec::new();
    let mut raw_hist: BTreeMap<u8, usize> = BTreeMap::new();
    let mut remapped_hist: BTreeMap<u8, usize> = BTreeMap::new();
    let mut remapped_drum_hits = 0usize;
    let mut uses_expansion_kit = false;

    for sec in sections {
        for tr in &sec.tracks {
            let (sf2_bank, sf2_program, _) = resolve_yamaha_voice_to_gm(
                tr.dest_ch,
                tr.voice.msb,
                tr.voice.lsb,
                tr.voice.program,
                &tr.voice.name_hint,
            );
            let uses_expansion_bank = tr.voice.msb == 62 || tr.voice.msb == 63;
            if uses_expansion_bank {
                uses_expansion_kit = true;
            }
            let (casm_name, ntr, ntt) = match &tr.casm_rule {
                Some(rule) => (
                    rule.name.clone(),
                    format!("{:?}", rule.ranges[0].ntr),
                    format!("{:?}", rule.ranges[0].ntt),
                ),
                None => (
                    tr.voice.name_hint.clone(),
                    "Fallback".to_string(),
                    "Fallback".to_string(),
                ),
            };
            channels.push(StyChannelDiagnostic {
                section: sec.name.clone(),
                src_ch: tr.src_ch,
                src_ch_midi_1based: tr.src_ch + 1,
                dest_ch: tr.dest_ch,
                dest_ch_midi_1based: tr.dest_ch + 1,
                casm_name,
                ntr,
                ntt,
                msb: tr.voice.msb,
                lsb: tr.voice.lsb,
                program: tr.voice.program,
                sf2_bank,
                sf2_program,
                is_drum: tr.is_drum,
                uses_expansion_bank,
            });

            let kit_opt: Option<DrumKitKind> = classify_drum_kit(
                tr.dest_ch,
                tr.voice.msb,
                tr.voice.lsb,
                tr.voice.program,
            );
            if tr.is_drum || kit_opt.is_some() {
                let kit = kit_opt.unwrap_or(DrumKitKind::XgStandard);
                for ev in &tr.events {
                    *raw_hist.entry(ev.midi_note).or_insert(0) += 1;
                    let remapped = remap_yamaha_drum_note(ev.midi_note, kit);
                    *remapped_hist.entry(remapped).or_insert(0) += 1;
                    if remapped != ev.midi_note {
                        remapped_drum_hits += 1;
                    }
                }
            }
        }
    }

    let mut warnings = Vec::new();
    if uses_expansion_kit {
        warnings.push(
            "Este estilo usa kits/vozes de expansão Yamaha (MSB 62/63) ausentes no .STY — remapeado para GM/SF2. Você pode trocar o timbre no mixer ou importar .Instrument/.SF2.".to_string(),
        );
    }
    if has_audio_style_chunks {
        warnings.push(format!(
            "Este estilo contém faixas de áudio proprietárias ({}) que não são reproduzidas pelo motor MIDI.",
            audio_chunk_tags.join("/")
        ));
    }

    let format_str = match format {
        StyFormat::Sff1 => "SFF1".to_string(),
        StyFormat::Sff2 => "SFF2".to_string(),
        StyFormat::LegacyStyl => "STYL".to_string(),
    };

    StyDiagnostics {
        file_name: file_name.to_string(),
        format: format_str,
        ppq: tpqn,
        bpm,
        time_sig: format!("{}/{}", time_sig_num, time_sig_den),
        sections: section_diagnostics,
        channels,
        raw_drum_note_histogram: raw_hist.into_iter().collect(),
        remapped_drum_note_histogram: remapped_hist.into_iter().collect(),
        remapped_drum_hits,
        uses_expansion_kit,
        has_audio_style_chunks,
        audio_chunk_tags: audio_chunk_tags.to_vec(),
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn collect_sty_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_sty_files(&p, out);
            } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "sty" | "prs" | "sst" | "bcs"
                ) {
                    out.push(p);
                }
            }
        }
    }

    fn chord_spec(
        root_pc: u8,
        yamaha_bit: u8,
        third_iv: i16,
        fifth_iv: i16,
        seventh_iv: i16,
        add9: bool,
        is_major_family: bool,
    ) -> TargetChordSpec {
        TargetChordSpec {
            root_pc,
            yamaha_bit,
            third_iv,
            fifth_iv,
            seventh_iv,
            add9,
            is_major_family,
        }
    }

    #[test]
    fn parses_all_available_real_sty_files_on_disk() {
        let root = Path::new("/home/data/Documents/Ritmos de Teclados");
        if !root.exists() {
            return;
        }
        let mut files = Vec::new();
        collect_sty_files(root, &mut files);
        files.sort();
        // This is the user's mutable library, not a fixed test fixture.
        // Content-deduplication/import may legitimately change its file count.
        assert!(!files.is_empty(), "No styles available under {}", root.display());

        let mut total_events = 0usize;
        for fp in &files {
            let doc = parse_sty_file(fp)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", fp.display(), e));
            assert!(
                (40.0..=240.0).contains(&doc.bpm),
                "{}: BPM {} out of range",
                fp.display(),
                doc.bpm
            );
            assert!(
                !doc.sections.is_empty(),
                "{}: sections empty",
                fp.display()
            );
            assert!(
                !doc.diagnostics.sections.is_empty(),
                "{}: diagnostics.sections empty",
                fp.display()
            );
            assert!(
                !doc.diagnostics.channels.is_empty(),
                "{}: diagnostics.channels empty",
                fp.display()
            );
            assert!(
                !doc.diagnostics.summary_text().is_empty(),
                "{}: diagnostics summary empty",
                fp.display()
            );
            let file_notes: usize = doc
                .sections
                .iter()
                .flat_map(|s| s.tracks.iter())
                .map(|t| t.events.len())
                .sum();
            assert!(
                file_notes > 0,
                "{}: zero note events across all sections",
                fp.display()
            );
            for sec in &doc.sections {
                assert!(sec.length_ticks >= 500, "{}: {} too short", fp.display(), sec.name);
                for tr in &sec.tracks {
                    assert!(tr.track_slot < 8, "{}: invalid slot {}", fp.display(), tr.track_slot);
                    let (bank, gm_prog, resolved_drum) = resolve_yamaha_voice_to_gm(
                        tr.dest_ch,
                        tr.voice.msb,
                        tr.voice.lsb,
                        tr.voice.program,
                        &tr.voice.name_hint,
                    );
                    assert!(gm_prog < 128);
                    assert_eq!(resolved_drum, bank == 128);
                    if tr.is_drum
                        || tr.dest_ch == 8
                        || tr.dest_ch == 9
                        || matches!(tr.voice.msb, 62 | 126 | 127)
                    {
                        assert_ne!(
                            bank, 0,
                            "{}: {} drum track src={} dst={} msb={} resolved to melodic bank 0!",
                            fp.display(),
                            sec.name,
                            tr.src_ch,
                            tr.dest_ch,
                            tr.voice.msb
                        );
                        assert_eq!(bank, 128);
                        assert!(resolved_drum);
                    }
                    for ev in &tr.events {
                        assert!(
                            ev.start_tick < sec.length_ticks,
                            "{}: {} start_tick {} >= len {}",
                            fp.display(),
                            sec.name,
                            ev.start_tick,
                            sec.length_ticks
                        );
                        assert!(ev.duration_ticks >= 1);
                        assert!(
                            ev.start_tick + ev.duration_ticks <= sec.length_ticks,
                            "{}: {} start {} + dur {} > len {}",
                            fp.display(),
                            sec.name,
                            ev.start_tick,
                            ev.duration_ticks,
                            sec.length_ticks
                        );
                        if let Some(rule) = &tr.casm_rule {
                            for spec in [
                                chord_spec(0, 0, 4, 7, 11, false, true),
                                chord_spec(9, 8, 3, 7, 10, false, false),
                                chord_spec(7, 19, 4, 7, 10, false, true),
                                chord_spec(2, 22, 4, 7, 10, true, true),
                            ] {
                                if rule.is_active(spec) {
                                    if let Some(out) = rule.transpose_note(
                                        ev.midi_note,
                                        spec,
                                        tr.is_drum,
                                        tr.dest_ch == 10,
                                    ) {
                                        assert!(out <= 127);
                                        if tr.is_drum || tr.dest_ch == 8 || tr.dest_ch == 9 {
                                            assert_eq!(out, ev.midi_note);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            total_events += file_notes;
        }
        assert!(total_events > 50_000, "Unexpectedly low total note count: {}", total_events);
    }

    #[test]
    fn parses_sff1_o_som_do_povo() {
        let p = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Ritmos PSR/O Som do Povo/O Som do Povo.S837.STY",
        );
        if !p.exists() {
            return;
        }
        let doc = parse_sty_file(p).expect("SFF1 parse");
        assert_eq!(doc.format, StyFormat::Sff1);
        assert!(doc.sections.iter().any(|s| s.name == "Variation 1" && s.is_loop));
        assert!(doc.sections.iter().any(|s| s.name == "Fill 1" && !s.is_loop && s.jump_to > 0));
    }

    #[test]
    fn parses_sff2_forro_boys_and_expansion_voices() {
        let p = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Washington Brasileiro e Forro Boys/Forro Boys/(35) FORRÓ BOYS.sty",
        );
        if !p.exists() {
            return;
        }
        let doc = parse_sty_file(p).expect("SFF2 FORRO BOYS parse");
        assert_eq!(doc.format, StyFormat::Sff2);
        let var1 = doc
            .sections
            .iter()
            .find(|s| s.name == "Variation 1")
            .expect("Variation 1");

        // Real PSR-S650 voices (MSB 62/63) live inside Main A, overriding SInt dummy (0, 113, 0).
        let has_exp63 = var1.tracks.iter().any(|t| t.voice.msb == 63);
        let has_drum62 = var1.tracks.iter().any(|t| t.voice.msb == 62 && t.is_drum);
        assert!(has_exp63, "Expected MSB=63 expansion melodic voice in Variation 1");
        assert!(has_drum62, "Expected MSB=62 expansion drum voice in Variation 1");

        // Verify expansion voice heuristic resolution.
        assert_eq!(resolve_yamaha_voice_to_gm(12, 63, 0, 6, "Sanfona"), (0, 21, false));
        assert_eq!(resolve_yamaha_voice_to_gm(11, 63, 0, 9, "Guitar1"), (0, 27, false));
        assert_eq!(resolve_yamaha_voice_to_gm(10, 63, 0, 0, "Baixo"), (0, 33, false));
        assert_eq!(resolve_yamaha_voice_to_gm(9, 62, 0, 6, "Drum"), (128, 16, true));
        assert_eq!(resolve_yamaha_voice_to_gm(9, 127, 0, 24, "Std"), (128, 24, true));
    }

    #[test]
    fn parses_sff2_calcinha_preta_chord_mute_major_vs_minor() {
        let p = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Ritmos PSR/Calcinha Preta/CALCINHAPRETA GB.sty",
        );
        if !p.exists() {
            return;
        }
        let doc = parse_sty_file(p).expect("CALCINHAPRETA GB parse");
        assert_eq!(doc.format, StyFormat::Sff2);

        let var1 = doc
            .sections
            .iter()
            .find(|s| s.name == "Variation 1")
            .expect("Variation 1");

        let dst11_tracks: Vec<&StyTrack> = var1.tracks.iter().filter(|t| t.dest_ch == 11).collect();
        assert_eq!(
            dst11_tracks.len(),
            2,
            "Expected 2 CASM tracks targeting dest_ch=11 (src=11 Major, src=7 Minor)"
        );

        let t_maj = dst11_tracks.iter().find(|t| t.src_ch == 11).unwrap();
        let t_min = dst11_tracks.iter().find(|t| t.src_ch == 7).unwrap();
        let r_maj = t_maj.casm_rule.as_ref().unwrap();
        let r_min = t_min.casm_rule.as_ref().unwrap();

        let c_maj = chord_spec(0, 0, 4, 7, 11, false, true);
        let c_min = chord_spec(0, 8, 3, 7, 10, false, false);
        let g_7 = chord_spec(7, 19, 4, 7, 10, false, true);

        assert!(r_maj.is_active(c_maj));
        assert!(!r_maj.is_active(c_min));
        assert!(r_maj.is_active(g_7));

        assert!(!r_min.is_active(c_maj));
        assert!(r_min.is_active(c_min));
        assert!(!r_min.is_active(g_7));
    }

    #[test]
    fn parses_legacy_styl_and_leading_zero_pad_before_casm() {
        let p_styl = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Forro_novos_2026-11-08/Mastruz/Mastruz.STY",
        );
        if p_styl.exists() {
            let doc = parse_sty_file(p_styl).expect("Legacy STYL parse");
            assert_eq!(doc.format, StyFormat::LegacyStyl);
            assert!(doc.sections.len() >= 5);
            assert!(doc.sections.iter().all(|s| !s.tracks.is_empty()));
        }

        let p_pad = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Forro_novos_2026-11-08/Forro_bandas/CALPRETA.STY",
        );
        if p_pad.exists() {
            let doc = parse_sty_file(p_pad).expect("Odd MTrk + 0x00 pad before CASM");
            assert_eq!(doc.format, StyFormat::Sff1);
            let var1 = doc.sections.iter().find(|s| s.name == "Variation 1").unwrap();
            assert!(var1.tracks.iter().any(|t| t.casm_rule.is_some()));
        }
    }

    #[test]
    fn parses_otsc_presets() {
        let p = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Forro_novos_2026-11-08/Forro_bandas/CAVIAR.S119.STY",
        );
        if !p.exists() {
            return;
        }
        let doc = parse_sty_file(p).expect("OTSc parse");
        assert_eq!(doc.ots_presets.len(), 4);
        assert_eq!(doc.ots_presets[0].right1_bank_lsb, 112);
        assert_eq!(doc.ots_presets[0].right1_program, 23);
        assert_eq!(doc.ots_presets[0].right2_program, Some((0, 113, 21)));
        assert_eq!(doc.ots_presets[0].left_program, Some((0, 114, 4)));
    }

    #[test]
    fn casm_transpose_note_major_minor_dom7_dom9_bass_and_drums() {
        let chord_rule = CasmRule {
            src_ch: 11,
            dest_ch: 11,
            name: "Chord1".to_string(),
            editable: 1,
            note_mute: 0x0FFF,
            chord_mute: 0x03_ffff_ffff,
            src_root: 0,
            src_chord_type: 2, // CMaj7 source
            low_split: None,
            high_split: None,
            ranges: [
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Chord,
                    bass_on: false,
                    high_key: 6, // F#
                    note_low: 48,
                    note_high: 84,
                    rtr: RtrRule::PitchShift,
                },
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Chord,
                    bass_on: false,
                    high_key: 6,
                    note_low: 48,
                    note_high: 84,
                    rtr: RtrRule::PitchShift,
                },
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Chord,
                    bass_on: false,
                    high_key: 6,
                    note_low: 48,
                    note_high: 84,
                    rtr: RtrRule::PitchShift,
                },
            ],
        };

        // 1. Major -> Minor (C Maj7: C4=60, E4=64, G4=67, B4=71 -> Am: A3=57, C4=60, E4=64, G4=67)
        let a_min = chord_spec(9, 8, 3, 7, 10, false, false);
        assert!(chord_rule.is_active(a_min));
        assert_eq!(chord_rule.transpose_note(60, a_min, false, false), Some(57));
        assert_eq!(chord_rule.transpose_note(64, a_min, false, false), Some(60));
        assert_eq!(chord_rule.transpose_note(67, a_min, false, false), Some(64));
        assert_eq!(chord_rule.transpose_note(71, a_min, false, false), Some(67));

        // 2. Dom7 (G7: G4=67, B4=71, D5=74, F5=77; high_key=6 folds -12 -> G3=55, B3=59, D4=62, F4=65)
        let g7 = chord_spec(7, 19, 4, 7, 10, false, true);
        assert_eq!(chord_rule.transpose_note(60, g7, false, false), Some(55));
        assert_eq!(chord_rule.transpose_note(64, g7, false, false), Some(59));
        assert_eq!(chord_rule.transpose_note(67, g7, false, false), Some(62));
        assert_eq!(chord_rule.transpose_note(71, g7, false, false), Some(65));

        // 3. Dom9 add9 tone (D4=62 in C -> D stays 9th color when add9=true, folds to root when false)
        let c9 = chord_spec(0, 22, 4, 7, 10, true, true);
        let c_plain = chord_spec(0, 0, 4, 7, 11, false, true);
        assert_eq!(chord_rule.transpose_note(62, c9, false, false), Some(62));
        assert_eq!(chord_rule.transpose_note(62, c_plain, false, false), Some(60));

        // 4. RootFixed keeps closest register
        let mut fixed_rule = chord_rule.clone();
        fixed_rule.ranges[0].ntr = NtrRule::RootFixed;
        let f_maj = chord_spec(5, 0, 4, 7, 11, false, true);
        assert_eq!(fixed_rule.transpose_note(60, f_maj, false, false), Some(65));
        assert_eq!(fixed_rule.transpose_note(64, f_maj, false, false), Some(69));
        assert_eq!(fixed_rule.transpose_note(67, f_maj, false, false), Some(72));

        // 5. Bass preserves root on F and Am (with high_key=3 (D#), F=5 folds -12 into F1=29 inside [28..=60])
        let mut bass_rule = CasmRule {
            src_ch: 10,
            dest_ch: 10,
            name: "Bass".to_string(),
            editable: 1,
            note_mute: 0x0FFF,
            chord_mute: 0x03_ffff_ffff,
            src_root: 0,
            src_chord_type: 0,
            low_split: None,
            high_split: None,
            ranges: [
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Bass,
                    bass_on: true,
                    high_key: 3,
                    note_low: 28,
                    note_high: 60,
                    rtr: RtrRule::RetriggerToRoot,
                },
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Bass,
                    bass_on: true,
                    high_key: 3,
                    note_low: 28,
                    note_high: 60,
                    rtr: RtrRule::RetriggerToRoot,
                },
                CasmRangeRule {
                    ntr: NtrRule::RootTrans,
                    ntt: NttRule::Bass,
                    bass_on: true,
                    high_key: 3,
                    note_low: 28,
                    note_high: 60,
                    rtr: RtrRule::RetriggerToRoot,
                },
            ],
        };
        assert_eq!(bass_rule.transpose_note(36, f_maj, false, true), Some(29)); // F1 (5 > high_key=3, >= 28)
        assert_eq!(bass_rule.transpose_note(36, a_min, false, true), Some(33)); // A1
        bass_rule.ranges[0].high_key = 6;
        assert_eq!(bass_rule.transpose_note(36, f_maj, false, true), Some(41)); // F2 (5 <= high_key=6)

        // 6. Drums pass through unchanged
        assert_eq!(chord_rule.transpose_note(36, a_min, true, false), Some(36));

        // 7. Ctb2 range selection & MegaVoice keyswitch filtering
        let mut ctb2_rule = chord_rule.clone();
        ctb2_rule.low_split = Some(40);
        ctb2_rule.high_split = Some(95);
        ctb2_rule.ranges[2] = CasmRangeRule {
            ntr: NtrRule::RootFixed,
            ntt: NttRule::Bypass,
            bass_on: false,
            high_key: 6,
            note_low: 0,
            note_high: 127,
            rtr: RtrRule::PitchShift,
        };
        assert_eq!(ctb2_rule.transpose_note(100, a_min, false, false), None);

        // 8. RootTrans + Bypass transposes strictly by root without altering chord quality
        let mut root_bypass_rule = chord_rule.clone();
        root_bypass_rule.ranges[0].ntt = NttRule::Bypass;
        assert_eq!(root_bypass_rule.transpose_note(60, f_maj, false, false), Some(65));
        assert_eq!(root_bypass_rule.transpose_note(63, a_min, false, false), Some(60));

        // 9. Guitar NTR keeps voicings in guitar fretboard register
        let mut gtr_rule = chord_rule.clone();
        gtr_rule.ranges[0].ntr = NtrRule::Guitar;
        gtr_rule.ranges[0].note_low = 0;
        gtr_rule.ranges[0].note_high = 127;
        assert_eq!(gtr_rule.transpose_note(60, g7, false, false), Some(55));
        assert_eq!(gtr_rule.transpose_note(64, g7, false, false), Some(59));
    }

    #[test]
    fn parses_cntt_and_sparse_otsc_and_megavoice() {
        let p_cntt = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Ritmos PSR/O Som do Povo/HT.POVO.STY",
        );
        if p_cntt.exists() {
            let doc = parse_sty_file(p_cntt).expect("HT.POVO parse");
            let var1 = doc.sections.iter().find(|s| s.name == "Variation 1").unwrap();
            let bass = var1.tracks.iter().find(|t| t.dest_ch == 10).unwrap();
            let rule = bass.casm_rule.as_ref().unwrap();
            assert!(rule.ranges[0].bass_on);
            assert_eq!(rule.ranges[0].ntt, NttRule::Melody);
        }

        let p_ots = Path::new(
            "/home/data/Documents/Ritmos de Teclados/Ritmos PSR/Zé Vaqueiro/PACK PISEIRO INSCRITOS 2K21/04-PISADA DE BARÃO .sty",
        );
        if p_ots.exists() {
            let doc = parse_sty_file(p_ots).expect("04-PISADA DE BARÃO parse");
            assert_eq!(doc.ots_presets.len(), 1);
            assert_eq!(doc.ots_presets[0].index, 3);
            assert_eq!(doc.ots_presets[0].right1_bank_lsb, 96);
            assert_eq!(doc.ots_presets[0].right1_program, 111);
        }

        let p_mega = Path::new(
            "/home/data/Documents/Ritmos de Teclados/KIT DILSON SHOW V1 2019 PSR-S650/RITIMOS 2019/(04) ARROCHASETANEJO.sty",
        );
        if p_mega.exists() {
            let doc = parse_sty_file(p_mega).expect("ARROCHASETANEJO parse");
            let var1 = doc.sections.iter().find(|s| s.name == "Variation 1").unwrap();
            for tr in var1.tracks.iter().filter(|t| t.voice.msb == 8) {
                assert!(
                    tr.events.iter().all(|e| (24..96).contains(&e.midi_note)),
                    "MegaVoice track {} retained keyswitch outside 24..96",
                    tr.voice.name_hint
                );
            }
        }
    }

    fn push_vlq(out: &mut Vec<u8>, mut val: u32) {
        let mut buf = [0u8; 4];
        let mut idx = 3usize;
        buf[idx] = (val & 0x7f) as u8;
        val >>= 7;
        while val > 0 && idx > 0 {
            idx -= 1;
            buf[idx] = ((val & 0x7f) as u8) | 0x80;
            val >>= 7;
        }
        out.extend_from_slice(&buf[idx..]);
    }

    fn push_chunk(out: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
        out.extend_from_slice(tag);
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(payload);
    }

    fn push_meta_marker(trk: &mut Vec<u8>, delta: u32, text: &str) {
        push_vlq(trk, delta);
        trk.push(0xff);
        trk.push(0x06);
        push_vlq(trk, text.len() as u32);
        trk.extend_from_slice(text.as_bytes());
    }

    fn push_cc(trk: &mut Vec<u8>, delta: u32, ch: u8, cc: u8, val: u8) {
        push_vlq(trk, delta);
        trk.push(0xb0 | (ch & 0x0f));
        trk.push(cc & 0x7f);
        trk.push(val & 0x7f);
    }

    fn push_pc(trk: &mut Vec<u8>, delta: u32, ch: u8, prog: u8) {
        push_vlq(trk, delta);
        trk.push(0xc0 | (ch & 0x0f));
        trk.push(prog & 0x7f);
    }

    fn push_note(trk: &mut Vec<u8>, delta_on: u32, dur: u32, ch: u8, note: u8, vel: u8) {
        push_vlq(trk, delta_on);
        trk.push(0x90 | (ch & 0x0f));
        trk.push(note & 0x7f);
        trk.push(vel & 0x7f);
        push_vlq(trk, dur);
        trk.push(0x80 | (ch & 0x0f));
        trk.push(note & 0x7f);
        trk.push(0);
    }

    fn push_eot(trk: &mut Vec<u8>, delta: u32) {
        push_vlq(trk, delta);
        trk.extend_from_slice(&[0xff, 0x2f, 0x00]);
    }

    fn build_ctab(src_ch: u8, name: &str, dest_ch: u8) -> Vec<u8> {
        let mut buf = Vec::with_capacity(26);
        buf.push(src_ch);
        let mut name_bytes = [b' '; 8];
        for (i, b) in name.as_bytes().iter().take(8).enumerate() {
            name_bytes[i] = *b;
        }
        buf.extend_from_slice(&name_bytes);
        buf.push(dest_ch);
        buf.push(1); // editable
        buf.extend_from_slice(&0x0FFF_u16.to_be_bytes());
        buf.extend_from_slice(&[0x03, 0xff, 0xff, 0xff, 0xff]); // chord_mute
        buf.push(0); // src_root = C
        buf.push(0); // src_chord_type = Maj
        buf.extend_from_slice(&[1, 0, 6, 0, 127, 1]); // RootFixed + Bypass
        buf
    }

    fn build_cseg(sdec: &str, ctabs: &[Vec<u8>]) -> Vec<u8> {
        let mut cseg = Vec::new();
        push_chunk(&mut cseg, b"Sdec", sdec.as_bytes());
        for ctab in ctabs {
            push_chunk(&mut cseg, b"Ctab", ctab);
        }
        cseg
    }

    #[test]
    fn casm_transpose_note_preserves_drums_on_dest_ch_8_and_9_across_chords() {
        let chords = [
            chord_spec(0, 0, 4, 7, 11, false, true),  // C Major
            chord_spec(9, 8, 3, 7, 10, false, false), // A Minor
            chord_spec(6, 8, 3, 7, 10, false, false), // F# Minor
            chord_spec(7, 19, 4, 7, 10, false, true), // G7
            chord_spec(2, 22, 4, 7, 10, true, true),  // D9
        ];
        for dest_ch in [8u8, 9u8] {
            let drum_rule = CasmRule {
                src_ch: 6,
                dest_ch,
                name: format!("Rhythm{}", if dest_ch == 8 { 1 } else { 2 }),
                editable: 1,
                note_mute: 0x0FFF,
                chord_mute: 0x03_ffff_ffff,
                src_root: 0,
                src_chord_type: 0,
                low_split: Some(24),
                high_split: Some(95),
                ranges: [
                    CasmRangeRule {
                        ntr: NtrRule::RootTrans,
                        ntt: NttRule::Chord,
                        bass_on: false,
                        high_key: 6,
                        note_low: 36,
                        note_high: 84,
                        rtr: RtrRule::PitchShift,
                    },
                    CasmRangeRule {
                        ntr: NtrRule::RootTrans,
                        ntt: NttRule::Chord,
                        bass_on: false,
                        high_key: 6,
                        note_low: 36,
                        note_high: 84,
                        rtr: RtrRule::PitchShift,
                    },
                    CasmRangeRule {
                        ntr: NtrRule::RootTrans,
                        ntt: NttRule::Chord,
                        bass_on: false,
                        high_key: 6,
                        note_low: 36,
                        note_high: 84,
                        rtr: RtrRule::PitchShift,
                    },
                ],
            };
            for note in [22u8, 36, 38, 42, 60, 99] {
                for spec in chords {
                    assert_eq!(drum_rule.transpose_note(note, spec, false, false), Some(note));
                    assert_eq!(drum_rule.transpose_note(note, spec, true, false), Some(note));
                }
            }
        }
    }

    #[test]
    fn detects_audio_style_chunks_aasm_awav_and_emits_warning() {
        let mut trk = Vec::new();
        push_meta_marker(&mut trk, 0, "Main A");
        push_cc(&mut trk, 0, 9, 0, 127);
        push_pc(&mut trk, 0, 9, 0);
        push_note(&mut trk, 0, 240, 9, 36, 100);
        push_eot(&mut trk, 240);

        let mut casm = Vec::new();
        let cseg = build_cseg("Main A", &[build_ctab(9, "Rhythm2", 9)]);
        push_chunk(&mut casm, b"CSEG", &cseg);

        let mut aasm = Vec::new();
        push_chunk(&mut aasm, b"AFil", b"audio_loop.wav");
        push_chunk(&mut aasm, b"AWav", &[1, 2, 3, 4, 5, 6, 7, 8]);

        let mut sty = Vec::new();
        // MThd header: format 0, 1 track, 480 TPQN
        push_chunk(&mut sty, b"MThd", &[0, 0, 0, 1, 0x01, 0xe0]);
        push_chunk(&mut sty, b"MTrk", &trk);
        push_chunk(&mut sty, b"AASM", &aasm);
        push_chunk(&mut sty, b"CASM", &casm);

        let doc = parse_sty_bytes(
            &sty,
            "AudioStyle.STY".to_string(),
            "/tmp/AudioStyle.STY".to_string(),
        )
        .expect("parse synthetic audio STY");

        assert!(doc.has_audio_style_chunks);
        assert!(doc.audio_chunk_tags.contains(&"AASM".to_string()));
        assert!(doc.audio_chunk_tags.contains(&"AFil".to_string()));
        assert!(doc.audio_chunk_tags.contains(&"AWav".to_string()));
        assert!(doc.diagnostics.has_audio_style_chunks);
        assert!(
            doc.diagnostics
                .warnings
                .iter()
                .any(|w| w.contains("AASM") && w.contains("AWav")),
            "Expected audio chunk warning, got {:?}",
            doc.diagnostics.warnings
        );
        // Ensure skipping over top-level AASM did not lose the subsequent CASM chunk.
        let var1 = doc.sections.iter().find(|s| s.name == "Variation 1").unwrap();
        assert!(var1.tracks.iter().any(|t| t.casm_rule.is_some()));
    }

    #[test]
    fn inherits_global_drum_role_and_real_voice_when_intro_c_omitted_from_sdec() {
        // Case A: Intro C omitted from Sdec (only Main A in Sdec); src_ch=6 has dummy 0/113/0 in Intro C,
        // while Main A defines src_ch=6 -> dest_ch=9 and expansion drum voice (62, 0, 6).
        let mut trk = Vec::new();
        push_meta_marker(&mut trk, 0, "SInt");
        push_cc(&mut trk, 0, 6, 0, 0);
        push_cc(&mut trk, 0, 6, 32, 113);
        push_pc(&mut trk, 0, 6, 0);

        push_meta_marker(&mut trk, 480, "Intro C");
        // Emit a note-on at tick 480 whose note-off happens AFTER the Main A marker (tick 1440)
        // to exercise loop-boundary duration clamping.
        push_vlq(&mut trk, 0);
        trk.extend_from_slice(&[0x96, 36, 110]);
        push_meta_marker(&mut trk, 960, "Main A"); // tick 1440
        push_vlq(&mut trk, 240);
        trk.extend_from_slice(&[0x86, 36, 0]); // note-off at tick 1680 (past Intro C end)
        push_cc(&mut trk, 0, 6, 0, 62);
        push_cc(&mut trk, 0, 6, 32, 0);
        push_pc(&mut trk, 0, 6, 6);
        push_note(&mut trk, 0, 240, 6, 38, 100);
        push_eot(&mut trk, 240);

        let mut casm_main_only = Vec::new();
        let cseg_main_a = build_cseg("Main A", &[build_ctab(6, "Rhythm2", 9)]);
        push_chunk(&mut casm_main_only, b"CSEG", &cseg_main_a);

        let mut sty_alias = Vec::new();
        push_chunk(&mut sty_alias, b"MThd", &[0, 0, 0, 1, 0x01, 0xe0]);
        push_chunk(&mut sty_alias, b"MTrk", &trk);
        push_chunk(&mut sty_alias, b"CASM", &casm_main_only);

        let doc_alias = parse_sty_bytes(
            &sty_alias,
            "SynthAlias.sty".to_string(),
            "/tmp/SynthAlias.sty".to_string(),
        )
        .expect("parse alias synthetic STY");

        let intro3 = doc_alias
            .sections
            .iter()
            .find(|s| s.name == "Intro 3")
            .expect("Intro 3");
        let drum_tr = intro3
            .tracks
            .iter()
            .find(|t| t.src_ch == 6)
            .expect("src_ch=6 track in Intro 3");
        assert_eq!(drum_tr.dest_ch, 9);
        assert!(drum_tr.is_drum);
        assert_eq!(drum_tr.voice.msb, 62);
        let (bank, _prog, is_drum_gm) = resolve_yamaha_voice_to_gm(
            drum_tr.dest_ch,
            drum_tr.voice.msb,
            drum_tr.voice.lsb,
            drum_tr.voice.program,
            &drum_tr.voice.name_hint,
        );
        assert_eq!(bank, 128);
        assert!(is_drum_gm);
        assert!(doc_alias.diagnostics.uses_expansion_kit);
        for ev in &drum_tr.events {
            assert!(ev.start_tick < intro3.length_ticks);
            assert!(ev.start_tick + ev.duration_ticks <= intro3.length_ticks);
        }

        // Case B: Main C exists in Sdec WITHOUT src_ch=6 (only src_ch=11), forcing Intro C src_ch=6
        // down the pure unmatched fallback branch -> must inherit dest_ch=9 + is_drum + msb=62!
        let mut casm_partial_main_c = Vec::new();
        push_chunk(&mut casm_partial_main_c, b"CSEG", &cseg_main_a);
        let cseg_main_c = build_cseg("Main C", &[build_ctab(11, "Chord1", 11)]);
        push_chunk(&mut casm_partial_main_c, b"CSEG", &cseg_main_c);

        let mut sty_fallback = Vec::new();
        push_chunk(&mut sty_fallback, b"MThd", &[0, 0, 0, 1, 0x01, 0xe0]);
        push_chunk(&mut sty_fallback, b"MTrk", &trk);
        push_chunk(&mut sty_fallback, b"CASM", &casm_partial_main_c);

        let doc_fallback = parse_sty_bytes(
            &sty_fallback,
            "SynthFallback.sty".to_string(),
            "/tmp/SynthFallback.sty".to_string(),
        )
        .expect("parse fallback synthetic STY");

        let intro3_fb = doc_fallback
            .sections
            .iter()
            .find(|s| s.name == "Intro 3")
            .expect("Intro 3 fallback");
        let drum_tr_fb = intro3_fb
            .tracks
            .iter()
            .find(|t| t.src_ch == 6)
            .expect("src_ch=6 fallback track in Intro 3");
        assert_eq!(drum_tr_fb.dest_ch, 9);
        assert!(drum_tr_fb.is_drum);
        assert_eq!(drum_tr_fb.voice.msb, 62);
        let (fb_bank, _, fb_drum) = resolve_yamaha_voice_to_gm(
            drum_tr_fb.dest_ch,
            drum_tr_fb.voice.msb,
            drum_tr_fb.voice.lsb,
            drum_tr_fb.voice.program,
            &drum_tr_fb.voice.name_hint,
        );
        assert_eq!(fb_bank, 128);
        assert!(fb_drum);
    }
}
