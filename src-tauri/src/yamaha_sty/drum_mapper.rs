use std::fmt::Write;

/// Classified Yamaha / GM percussion kit family for drum note remapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumKitKind {
    XgStandard,
    XgPower,
    XgRoom,
    XgElectronic,
    XgBrush,
    XgJazz,
    XgLatinCuban,
    XgSfx,
    PsrExpansionDrum,
}

/// Diagnostic span and note-count summary for a single `.STY` section.
#[derive(Debug, Clone)]
pub struct StySectionDiagnostic {
    pub raw_name: String,
    pub canonical_name: String,
    pub raw_start_tick: u32,
    pub raw_end_tick: u32,
    pub length_ticks_500: u32,
    pub note_count: usize,
}

/// Diagnostic CASM channel routing and SoundFont voice resolution entry.
#[derive(Debug, Clone)]
pub struct StyChannelDiagnostic {
    pub section: String,
    pub src_ch: u8,
    pub src_ch_midi_1based: u8,
    pub dest_ch: u8,
    pub dest_ch_midi_1based: u8,
    pub casm_name: String,
    pub ntr: String,
    pub ntt: String,
    pub msb: u8,
    pub lsb: u8,
    pub program: u8,
    pub sf2_bank: u16,
    pub sf2_program: u16,
    pub is_drum: bool,
    pub uses_expansion_bank: bool,
}

/// Aggregated diagnostics produced when parsing and remapping a Yamaha `.STY` file.
#[derive(Debug, Clone, Default)]
pub struct StyDiagnostics {
    pub file_name: String,
    pub format: String,
    pub ppq: u16,
    pub bpm: f32,
    pub time_sig: String,
    pub sections: Vec<StySectionDiagnostic>,
    pub channels: Vec<StyChannelDiagnostic>,
    pub raw_drum_note_histogram: Vec<(u8, usize)>,
    pub remapped_drum_note_histogram: Vec<(u8, usize)>,
    pub remapped_drum_hits: usize,
    pub uses_expansion_kit: bool,
    pub has_audio_style_chunks: bool,
    pub audio_chunk_tags: Vec<String>,
    pub warnings: Vec<String>,
}

impl StyDiagnostics {
    /// Formats a concise, readable multi-line diagnostics summary covering container metadata,
    /// section spans, CASM channel routing, drum note histograms, expansion/audio flags, and warnings.
    pub fn summary_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "File: {} | Format: {} | PPQ: {} | BPM: {:.1} | TimeSig: {}",
            self.file_name, self.format, self.ppq, self.bpm, self.time_sig
        );
        let audio_tags = if self.audio_chunk_tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", self.audio_chunk_tags.join(", "))
        };
        let _ = writeln!(
            out,
            "ExpansionKit: {} | AudioStyleChunks: {}{} | RemappedDrumHits: {}",
            self.uses_expansion_kit,
            self.has_audio_style_chunks,
            audio_tags,
            self.remapped_drum_hits
        );

        let _ = writeln!(out, "Sections ({}):", self.sections.len());
        if self.sections.is_empty() {
            let _ = writeln!(out, "  (none)");
        } else {
            for sec in &self.sections {
                let _ = writeln!(
                    out,
                    "  - {} (raw \"{}\"): {}..{} (len500={}), notes={}",
                    sec.canonical_name,
                    sec.raw_name,
                    sec.raw_start_tick,
                    sec.raw_end_tick,
                    sec.length_ticks_500,
                    sec.note_count
                );
            }
        }

        let _ = writeln!(out, "CASM Channels ({}):", self.channels.len());
        if self.channels.is_empty() {
            let _ = writeln!(out, "  (none)");
        } else {
            for ch in &self.channels {
                let drum_tag = if ch.is_drum { " [DRUM]" } else { "" };
                let exp_tag = if ch.uses_expansion_bank { " [EXP]" } else { "" };
                let _ = writeln!(
                    out,
                    "  - [{}] MIDI ch {} -> {} (raw {}->{}) \"{}\" | MSB/LSB/PC {}/{}/{} -> SF2 {}/{} | NTR={} NTT={}{}{}",
                    ch.section,
                    ch.src_ch_midi_1based,
                    ch.dest_ch_midi_1based,
                    ch.src_ch,
                    ch.dest_ch,
                    ch.casm_name,
                    ch.msb,
                    ch.lsb,
                    ch.program,
                    ch.sf2_bank,
                    ch.sf2_program,
                    ch.ntr,
                    ch.ntt,
                    drum_tag,
                    exp_tag
                );
            }
        }

        let _ = writeln!(
            out,
            "Top Raw Drum Notes: {}",
            format_top_histogram(&self.raw_drum_note_histogram, 10)
        );
        let _ = writeln!(
            out,
            "Top Remapped Drum Notes: {}",
            format_top_histogram(&self.remapped_drum_note_histogram, 10)
        );

        if self.warnings.is_empty() {
            let _ = writeln!(out, "Warnings: none");
        } else {
            let _ = writeln!(out, "Warnings ({}):", self.warnings.len());
            for warning in &self.warnings {
                let _ = writeln!(out, "  - {}", warning);
            }
        }

        out
    }
}

fn format_top_histogram(hist: &[(u8, usize)], limit: usize) -> String {
    if hist.is_empty() || limit == 0 {
        return "(none)".to_string();
    }
    let mut sorted = hist.to_vec();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
        .into_iter()
        .take(limit)
        .map(|(note, count)| format!("{}:{}", note, count))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Classifies a Yamaha/MIDI destination channel and Bank Select / Program Change into a [`DrumKitKind`].
pub fn classify_drum_kit(dest_ch: u8, msb: u8, _lsb: u8, prog: u8) -> Option<DrumKitKind> {
    if msb == 62 {
        return Some(DrumKitKind::PsrExpansionDrum);
    }
    if msb == 126 {
        return if matches!(prog, 35 | 40 | 43 | 64) {
            Some(DrumKitKind::XgLatinCuban)
        } else {
            Some(DrumKitKind::XgSfx)
        };
    }
    if msb == 127 {
        return Some(match prog & 0x7f {
            16..=23 => DrumKitKind::XgPower,
            8..=15 => DrumKitKind::XgRoom,
            24..=31 => DrumKitKind::XgElectronic,
            32..=39 => DrumKitKind::XgJazz,
            40..=47 => DrumKitKind::XgBrush,
            _ => DrumKitKind::XgStandard,
        });
    }
    if dest_ch == 8 || dest_ch == 9 {
        return Some(DrumKitKind::XgStandard);
    }
    None
}

/// Remaps Yamaha XG, Latin/Cuban, and PSR Expansion drum notes into GM `FluidR3_GM.sf2` range `27..=87`.
pub fn remap_yamaha_drum_note(note: u8, kit: DrumKitKind) -> u8 {
    match note {
        // Yamaha XG low percussion & brush/kick/snare extensions (< 35)
        0..=12 => 36,
        13 => 86, // XG Surdo Mute
        14 => 87, // XG Surdo Open
        15 => 27,
        16 => 28,
        17 => 29,
        18 => 30,
        19 => 39,
        20..=23 => 37,
        24 => {
            if kit == DrumKitKind::PsrExpansionDrum {
                36
            } else {
                37
            }
        }
        25 => 38,
        26 => 40,
        27 => 38,
        28 => 40,
        29 => 38,
        30 => 85,
        31 => 38,
        32 => 31,
        33 => 35,
        34 => 37,

        // Core GM percussion range (35..=87), with targeted Latin/Cuban substitutions
        35..=87 => {
            if kit == DrumKitKind::XgLatinCuban {
                match note {
                    58 => 54, // Vibra Slap -> Tambourine / Pandeiro
                    71 => 54, // Samba Whistle H -> Tambourine / Pandeiro
                    72 => 82, // Samba Whistle L -> Shaker / Pandeiro groove
                    _ => note,
                }
            } else {
                note
            }
        }

        // Extended PSR upper percussion range (>= 88)
        88 => 86,
        89 => 82,
        90 => 80,
        91 => 81,
        92 => 54,
        93 => 56,
        94 => 62,
        95 => 63,
        96 => 64,
        97 => 65,
        98 => 66,
        99..=100 => 76,
        101..=102 => 77,
        103..=106 => 69,
        107..=127 => 82,
        _ => 82,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_KITS: [DrumKitKind; 9] = [
        DrumKitKind::XgStandard,
        DrumKitKind::XgPower,
        DrumKitKind::XgRoom,
        DrumKitKind::XgElectronic,
        DrumKitKind::XgBrush,
        DrumKitKind::XgJazz,
        DrumKitKind::XgLatinCuban,
        DrumKitKind::XgSfx,
        DrumKitKind::PsrExpansionDrum,
    ];

    #[test]
    fn all_notes_0_to_127_map_inside_fluidr3_gm_range_across_all_kits() {
        for &kit in &ALL_KITS {
            for note in 0u8..=127 {
                let mapped = remap_yamaha_drum_note(note, kit);
                assert!(
                    (27..=87).contains(&mapped),
                    "kit={:?} note={} mapped to out-of-range {}",
                    kit,
                    note,
                    mapped
                );
            }
        }
    }

    #[test]
    fn xg_and_psr_specific_low_and_high_remaps() {
        for kit in [DrumKitKind::XgStandard, DrumKitKind::PsrExpansionDrum] {
            assert_eq!(remap_yamaha_drum_note(33, kit), 35);
            assert_eq!(remap_yamaha_drum_note(34, kit), 37);
            assert_eq!(remap_yamaha_drum_note(31, kit), 38);
            assert_eq!(remap_yamaha_drum_note(88, kit), 86);
            assert_eq!(remap_yamaha_drum_note(89, kit), 82);
        }

        // Additional XG/PSR edge mappings
        assert_eq!(remap_yamaha_drum_note(0, DrumKitKind::XgStandard), 36);
        assert_eq!(remap_yamaha_drum_note(12, DrumKitKind::XgStandard), 36);
        assert_eq!(remap_yamaha_drum_note(13, DrumKitKind::XgStandard), 86);
        assert_eq!(remap_yamaha_drum_note(14, DrumKitKind::XgStandard), 87);
        assert_eq!(remap_yamaha_drum_note(24, DrumKitKind::XgStandard), 37);
        assert_eq!(remap_yamaha_drum_note(24, DrumKitKind::PsrExpansionDrum), 36);
        assert_eq!(remap_yamaha_drum_note(90, DrumKitKind::PsrExpansionDrum), 80);
        assert_eq!(remap_yamaha_drum_note(99, DrumKitKind::PsrExpansionDrum), 76);
        assert_eq!(remap_yamaha_drum_note(102, DrumKitKind::PsrExpansionDrum), 77);
        assert_eq!(remap_yamaha_drum_note(106, DrumKitKind::PsrExpansionDrum), 69);
        assert_eq!(remap_yamaha_drum_note(127, DrumKitKind::PsrExpansionDrum), 82);
    }

    #[test]
    fn latin_cuban_kit_remaps_whistle_and_vibra_slap_and_preserves_gm_percussion() {
        let kit = DrumKitKind::XgLatinCuban;
        assert_eq!(remap_yamaha_drum_note(58, kit), 54);
        assert_eq!(remap_yamaha_drum_note(71, kit), 54);
        assert_eq!(remap_yamaha_drum_note(72, kit), 82);

        for note in [36u8, 38, 42, 43, 66, 67, 68, 69, 70, 73, 74] {
            assert_eq!(
                remap_yamaha_drum_note(note, kit),
                note,
                "XgLatinCuban should preserve GM note {}",
                note
            );
        }
    }

    #[test]
    fn core_gm_notes_unchanged_for_xg_standard_and_psr_expansion() {
        for kit in [DrumKitKind::XgStandard, DrumKitKind::PsrExpansionDrum] {
            for note in (35u8..=57).chain(59..=70).chain(73..=87) {
                assert_eq!(
                    remap_yamaha_drum_note(note, kit),
                    note,
                    "kit={:?} altered core GM note {}",
                    kit,
                    note
                );
            }
            // Also 58, 71, 72 remain unchanged outside XgLatinCuban
            for note in [58u8, 71, 72] {
                assert_eq!(remap_yamaha_drum_note(note, kit), note);
            }
        }
    }

    #[test]
    fn classify_drum_kit_covers_msb_127_126_62_and_channel_fallbacks() {
        // MSB 62 -> PSR Expansion Drum
        assert_eq!(
            classify_drum_kit(9, 62, 0, 6),
            Some(DrumKitKind::PsrExpansionDrum)
        );

        // MSB 126 -> Latin/Cuban on 35/40/43/64, SFX otherwise
        for prog in [35u8, 40, 43, 64] {
            assert_eq!(
                classify_drum_kit(9, 126, 0, prog),
                Some(DrumKitKind::XgLatinCuban)
            );
        }
        assert_eq!(classify_drum_kit(9, 126, 0, 0), Some(DrumKitKind::XgSfx));
        assert_eq!(classify_drum_kit(9, 126, 0, 48), Some(DrumKitKind::XgSfx));

        // MSB 127 -> XG Drum family by program & 0x7f
        assert_eq!(classify_drum_kit(9, 127, 0, 0), Some(DrumKitKind::XgStandard));
        assert_eq!(classify_drum_kit(9, 127, 0, 8), Some(DrumKitKind::XgRoom));
        assert_eq!(classify_drum_kit(9, 127, 0, 15), Some(DrumKitKind::XgRoom));
        assert_eq!(classify_drum_kit(9, 127, 0, 16), Some(DrumKitKind::XgPower));
        assert_eq!(classify_drum_kit(9, 127, 0, 23), Some(DrumKitKind::XgPower));
        assert_eq!(
            classify_drum_kit(9, 127, 0, 24),
            Some(DrumKitKind::XgElectronic)
        );
        assert_eq!(
            classify_drum_kit(9, 127, 0, 31),
            Some(DrumKitKind::XgElectronic)
        );
        assert_eq!(classify_drum_kit(9, 127, 0, 32), Some(DrumKitKind::XgJazz));
        assert_eq!(classify_drum_kit(9, 127, 0, 39), Some(DrumKitKind::XgJazz));
        assert_eq!(classify_drum_kit(9, 127, 0, 40), Some(DrumKitKind::XgBrush));
        assert_eq!(classify_drum_kit(9, 127, 0, 47), Some(DrumKitKind::XgBrush));
        assert_eq!(
            classify_drum_kit(9, 127, 0, 48),
            Some(DrumKitKind::XgStandard)
        );

        // Channel 8/9 fallback when MSB is 0 (GM default percussion channels)
        assert_eq!(classify_drum_kit(8, 0, 0, 0), Some(DrumKitKind::XgStandard));
        assert_eq!(classify_drum_kit(9, 0, 0, 0), Some(DrumKitKind::XgStandard));

        // Non-drum melodic channel
        assert_eq!(classify_drum_kit(10, 0, 0, 33), None);
        assert_eq!(classify_drum_kit(10, 63, 0, 12), None);
    }

    #[test]
    fn diagnostics_summary_text_formats_all_sections_channels_and_histograms() {
        let diag = StyDiagnostics {
            file_name: "BossaBrasil.sty".to_string(),
            format: "SFF2".to_string(),
            ppq: 480,
            bpm: 124.5,
            time_sig: "4/4".to_string(),
            sections: vec![StySectionDiagnostic {
                raw_name: "Main A".to_string(),
                canonical_name: "Main A".to_string(),
                raw_start_tick: 0,
                raw_end_tick: 1920,
                length_ticks_500: 2000,
                note_count: 142,
            }],
            channels: vec![StyChannelDiagnostic {
                section: "Main A".to_string(),
                src_ch: 9,
                src_ch_midi_1based: 10,
                dest_ch: 9,
                dest_ch_midi_1based: 10,
                casm_name: "Rhythm2".to_string(),
                ntr: "RootFixed".to_string(),
                ntt: "Bypass".to_string(),
                msb: 126,
                lsb: 0,
                program: 40,
                sf2_bank: 128,
                sf2_program: 0,
                is_drum: true,
                uses_expansion_bank: false,
            }],
            raw_drum_note_histogram: vec![(71, 12), (36, 48)],
            remapped_drum_note_histogram: vec![(54, 12), (36, 48)],
            remapped_drum_hits: 12,
            uses_expansion_kit: true,
            has_audio_style_chunks: true,
            audio_chunk_tags: vec!["AUDI".to_string()],
            warnings: vec!["Detected audio style chunk".to_string()],
        };

        let text = diag.summary_text();
        assert!(text.contains("BossaBrasil.sty"));
        assert!(text.contains("SFF2"));
        assert!(text.contains("PPQ: 480"));
        assert!(text.contains("BPM: 124.5"));
        assert!(text.contains("4/4"));
        assert!(text.contains("Sections (1):"));
        assert!(text.contains("0..1920"));
        assert!(text.contains("MIDI ch 10 -> 10"));
        assert!(text.contains("126/0/40 -> SF2 128/0"));
        assert!(text.contains("NTR=RootFixed NTT=Bypass"));
        assert!(text.contains("Top Raw Drum Notes: 36:48, 71:12"));
        assert!(text.contains("Top Remapped Drum Notes: 36:48, 54:12"));
        assert!(text.contains("ExpansionKit: true"));
        assert!(text.contains("AudioStyleChunks: true [AUDI]"));
        assert!(text.contains("Detected audio style chunk"));
    }
}
