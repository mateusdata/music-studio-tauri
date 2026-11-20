import React, { useMemo, useState } from "react";
import { ChevronLeft, ChevronRight, Music, RotateCcw, Volume2 } from "lucide-react";
import type { NoteEditOffset } from "../types/engine";
import { ParameterSlider } from "./ParameterSlider";

export interface NoteEditPanelProps {
  noteEdit: Record<number, NoteEditOffset>;
  diapasonHz?: number;
  onChangeNoteEdit: (nextNoteEdit: Record<number, NoteEditOffset>) => void;
  onAuditionNote?: (midiNote: number) => void;
}

const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const BLACK_SEMITONES = new Set([1, 3, 6, 8, 10]);

export const MIN_MIDI_KEY = 21; // A0
export const MAX_MIDI_KEY = 108; // C8

const DEFAULT_OFFSET: NoteEditOffset = {
  detune_cents: 0,
  gain_db: 0,
  hammer_hardness_offset: 0,
  impedance_offset: 0,
};

export function formatMidiNoteName(midi: number): string {
  const octave = Math.floor(midi / 12) - 1;
  return `${NOTE_NAMES[((midi % 12) + 12) % 12]}${octave}`;
}

function isZeroOffset(offset: NoteEditOffset): boolean {
  return (
    Math.abs(offset.detune_cents) < 1e-6 &&
    Math.abs(offset.gain_db) < 1e-6 &&
    Math.abs(offset.hammer_hardness_offset) < 1e-6 &&
    Math.abs(offset.impedance_offset) < 1e-6
  );
}

export const NoteEditPanel: React.FC<NoteEditPanelProps> = ({
  noteEdit,
  diapasonHz = 440,
  onChangeNoteEdit,
  onAuditionNote,
}) => {
  const [selectedMidi, setSelectedMidi] = useState<number>(60); // Middle C4

  const allKeys = useMemo(() => {
    const keys: number[] = [];
    for (let m = MIN_MIDI_KEY; m <= MAX_MIDI_KEY; m++) {
      keys.push(m);
    }
    return keys;
  }, []);

  const modifiedKeys = useMemo(() => {
    return Object.entries(noteEdit)
      .filter(([, val]) => val && !isZeroOffset(val))
      .map(([k]) => Number(k))
      .filter((n) => Number.isFinite(n))
      .sort((a, b) => a - b);
  }, [noteEdit]);

  const currentOffset: NoteEditOffset = noteEdit[selectedMidi] ?? DEFAULT_OFFSET;

  const updateKeyOffset = (patch: Partial<NoteEditOffset>) => {
    const merged: NoteEditOffset = {
      ...currentOffset,
      ...patch,
    };
    const next = { ...noteEdit };
    if (isZeroOffset(merged)) {
      delete next[selectedMidi];
    } else {
      next[selectedMidi] = merged;
    }
    onChangeNoteEdit(next);
  };

  const resetSelectedKey = () => {
    if (!(selectedMidi in noteEdit)) return;
    const next = { ...noteEdit };
    delete next[selectedMidi];
    onChangeNoteEdit(next);
  };

  const clearAllKeys = () => {
    if (modifiedKeys.length === 0) return;
    onChangeNoteEdit({});
  };

  const targetFreqHz =
    diapasonHz *
    Math.pow(2, (selectedMidi - 69 + currentOffset.detune_cents / 100) / 12);

  return (
    <section className="panel-card panel-card-wide" data-testid="note-edit-panel">
      <div className="panel-title">
        <span>
          <Music size={14} style={{ verticalAlign: "text-bottom", marginRight: 6 }} />
          PRO Per-Note Edit (A0–C8)
        </span>
        <span className="badge-subtle">
          {modifiedKeys.length} modified {modifiedKeys.length === 1 ? "key" : "keys"}
        </span>
      </div>

      {/* Key Selector Bar */}
      <div className="note-edit-toolbar">
        <div className="note-nav-group">
          <button
            type="button"
            className="btn btn-compact"
            aria-label="Previous MIDI Key"
            disabled={selectedMidi <= MIN_MIDI_KEY}
            onClick={() => setSelectedMidi((m) => Math.max(MIN_MIDI_KEY, m - 1))}
          >
            <ChevronLeft size={14} />
          </button>
          <select
            className="studio-select"
            aria-label="Select MIDI Key"
            value={selectedMidi}
            onChange={(e) => setSelectedMidi(Number(e.target.value))}
          >
            {allKeys.map((midi) => {
              const hasEdit =
                noteEdit[midi] !== undefined && !isZeroOffset(noteEdit[midi]);
              return (
                <option key={midi} value={midi}>
                  {formatMidiNoteName(midi)} (MIDI {midi})
                  {hasEdit ? " *" : ""}
                </option>
              );
            })}
          </select>
          <button
            type="button"
            className="btn btn-compact"
            aria-label="Next MIDI Key"
            disabled={selectedMidi >= MAX_MIDI_KEY}
            onClick={() => setSelectedMidi((m) => Math.min(MAX_MIDI_KEY, m + 1))}
          >
            <ChevronRight size={14} />
          </button>
        </div>

        <div className="note-readout-pill">
          <strong>{formatMidiNoteName(selectedMidi)}</strong>
          <span>MIDI {selectedMidi}</span>
          <span>{targetFreqHz.toFixed(2)} Hz</span>
        </div>

        <div className="note-actions-group">
          {onAuditionNote && (
            <button
              type="button"
              className="btn btn-compact"
              onClick={() => onAuditionNote(selectedMidi)}
            >
              <Volume2 size={12} /> Audition
            </button>
          )}
          <button
            type="button"
            className="btn btn-compact"
            onClick={resetSelectedKey}
            disabled={isZeroOffset(currentOffset)}
          >
            <RotateCcw size={12} /> Reset Key
          </button>
          <button
            type="button"
            className="btn btn-compact"
            onClick={clearAllKeys}
            disabled={modifiedKeys.length === 0}
          >
            Reset All
          </button>
        </div>
      </div>

      {/* 88-Key Mini Strip */}
      <div
        className="note-edit-strip"
        data-testid="note-edit-keyboard"
        role="listbox"
        aria-label="88-Key Note Edit Selector"
      >
        {allKeys.map((midi) => {
          const isBlack = BLACK_SEMITONES.has(midi % 12);
          const isSelected = midi === selectedMidi;
          const isEdited =
            noteEdit[midi] !== undefined && !isZeroOffset(noteEdit[midi]);
          return (
            <button
              key={midi}
              type="button"
              role="option"
              aria-selected={isSelected}
              aria-label={`Edit ${formatMidiNoteName(midi)} (MIDI ${midi})`}
              title={`${formatMidiNoteName(midi)} (MIDI ${midi})`}
              className={[
                "note-strip-key",
                isBlack ? "black" : "white",
                isSelected ? "selected" : "",
                isEdited ? "edited" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              onClick={() => setSelectedMidi(midi)}
            />
          );
        })}
      </div>

      {/* 4 Per-Note Offsets */}
      <div className="note-sliders-grid">
        <ParameterSlider
          label="Detune"
          value={currentOffset.detune_cents}
          min={-50}
          max={50}
          step={0.5}
          unit="ct"
          onChange={(v) => updateKeyOffset({ detune_cents: v })}
        />
        <ParameterSlider
          label="Gain"
          value={currentOffset.gain_db}
          min={-18}
          max={18}
          step={0.2}
          unit="dB"
          onChange={(v) => updateKeyOffset({ gain_db: v })}
        />
        <ParameterSlider
          label="Hammer Hardness Offset"
          value={currentOffset.hammer_hardness_offset}
          min={-0.6}
          max={0.6}
          step={0.02}
          onChange={(v) => updateKeyOffset({ hammer_hardness_offset: v })}
        />
        <ParameterSlider
          label="Impedance Offset"
          value={currentOffset.impedance_offset}
          min={-0.5}
          max={0.5}
          step={0.02}
          onChange={(v) => updateKeyOffset({ impedance_offset: v })}
        />
      </div>

      {/* Modified keys quick-jump chips */}
      {modifiedKeys.length > 0 && (
        <div className="modified-keys-bar">
          <span className="control-label">Edited Keys:</span>
          {modifiedKeys.map((midi) => (
            <button
              key={midi}
              type="button"
              className={`chip-btn ${midi === selectedMidi ? "active" : ""}`}
              onClick={() => setSelectedMidi(midi)}
            >
              {formatMidiNoteName(midi)}
            </button>
          ))}
        </div>
      )}
    </section>
  );
};
