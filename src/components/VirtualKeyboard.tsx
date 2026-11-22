import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";

interface VirtualKeyboardProps {
  activeNotes: number[];
  extendedRange: boolean;
  splitKey?: number;
  splitEnabled?: boolean;
  onNoteOn: (note: number, velocity: number) => void;
  onNoteOff: (note: number) => void;
}

const BLACK_SEMITONES = new Set([1, 3, 6, 8, 10]);
const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

const COMPUTER_KEY_MAP: Record<string, number> = {
  z: 48,
  s: 49,
  x: 50,
  d: 51,
  c: 52,
  v: 53,
  g: 54,
  b: 55,
  h: 56,
  n: 57,
  j: 58,
  m: 59,
  q: 60,
  "2": 61,
  w: 62,
  "3": 63,
  e: 64,
  r: 65,
  "5": 66,
  t: 67,
  "6": 68,
  y: 69,
  "7": 70,
  u: 71,
  i: 72,
  "9": 73,
  o: 74,
  "0": 75,
  p: 76,
};

function noteLabel(midi: number): string {
  const octave = Math.floor(midi / 12) - 1;
  return `${NOTE_NAMES[midi % 12]}${octave}`;
}

export const VirtualKeyboard: React.FC<VirtualKeyboardProps> = ({
  activeNotes,
  extendedRange,
  splitKey = 53,
  splitEnabled = false,
  onNoteOn,
  onNoteOff,
}) => {
  const [pressedLocal, setPressedLocal] = useState<Set<number>>(new Set());
  const activeSet = useMemo(() => {
    const merged = new Set(activeNotes);
    pressedLocal.forEach((n) => merged.add(n));
    return merged;
  }, [activeNotes, pressedLocal]);

  // 61-key arranger keyboard (C2=36..C7=96, matching Music Studio octaves 3..8), or 105 extended
  const minNote = extendedRange ? 12 : 36;
  const maxNote = extendedRange ? 116 : 96;

  const { whiteKeys, blackKeys, splitDividerLeftPct, firstUpperWhite } = useMemo(() => {
    const whites: number[] = [];
    const blacks: { midi: number; leftPct: number }[] = [];
    for (let m = minNote; m <= maxNote; m++) {
      if (!BLACK_SEMITONES.has(m % 12)) {
        whites.push(m);
      }
    }
    const totalWhites = whites.length;
    let whiteIdx = 0;
    for (let m = minNote; m <= maxNote; m++) {
      if (!BLACK_SEMITONES.has(m % 12)) {
        whiteIdx++;
      } else {
        const leftPct = (whiteIdx / totalWhites) * 100;
        blacks.push({ midi: m, leftPct });
      }
    }
    const upperWhitesStart = whites.findIndex((m) => m > splitKey);
    const dividerWhites = upperWhitesStart >= 0 ? upperWhitesStart : whites.length;
    const splitPct = (dividerWhites / totalWhites) * 100;
    const upperWhiteMidi = upperWhitesStart >= 0 ? whites[upperWhitesStart] : -1;
    return {
      whiteKeys: whites,
      blackKeys: blacks,
      splitDividerLeftPct: splitPct,
      firstUpperWhite: upperWhiteMidi,
    };
  }, [minNote, maxNote, splitKey]);

  const triggerDown = useCallback(
    (midi: number, velocity: number) => {
      setPressedLocal((prev) => {
        if (prev.has(midi)) return prev;
        const next = new Set(prev);
        next.add(midi);
        return next;
      });
      onNoteOn(midi, velocity);
    },
    [onNoteOn],
  );

  const triggerUp = useCallback(
    (midi: number) => {
      setPressedLocal((prev) => {
        if (!prev.has(midi)) return prev;
        const next = new Set(prev);
        next.delete(midi);
        return next;
      });
      onNoteOff(midi);
    },
    [onNoteOff],
  );

  const heldComputerKeys = useRef<Set<string>>(new Set());

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.repeat) return;
      const tag = (e.target as HTMLElement | null)?.tagName;
      if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
      const k = e.key.toLowerCase();
      const midi = COMPUTER_KEY_MAP[k];
      if (midi !== undefined && !heldComputerKeys.current.has(k)) {
        heldComputerKeys.current.add(k);
        triggerDown(midi, 88);
      }
    };
    const onKeyUp = (e: KeyboardEvent) => {
      const k = e.key.toLowerCase();
      const midi = COMPUTER_KEY_MAP[k];
      if (midi !== undefined && heldComputerKeys.current.has(k)) {
        heldComputerKeys.current.delete(k);
        triggerUp(midi);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
    };
  }, [triggerDown, triggerUp]);

  const computeVelocity = (e: React.MouseEvent<HTMLDivElement>): number => {
    const rect = e.currentTarget.getBoundingClientRect();
    const relY = Math.max(0.15, Math.min(1, (e.clientY - rect.top) / rect.height));
    return Math.round(28 + relY * 96);
  };

  return (
    <div className="piano-keys-scroll" data-testid="virtual-keyboard">
      <div className="piano-keys-track" style={{ minWidth: "960px" }}>
        {whiteKeys.map((midi) => {
          const isC = midi % 12 === 0;
          const letter = NOTE_NAMES[midi % 12];
          const displayOctave = Math.floor(midi / 12); // Music Studio numbers C2 as 3..C7 as 8
          const isActive = activeSet.has(midi);
          const isLeftSplit = splitEnabled && midi <= splitKey;
          const isSplitEdge = splitEnabled && midi === firstUpperWhite;
          return (
            <div
              key={midi}
              role="button"
              aria-label={`Key ${noteLabel(midi)}`}
              className={`white-key ${isActive ? "active" : ""} ${
                isLeftSplit ? "split-chord-zone" : ""
              } ${isSplitEdge ? "split-point-edge" : ""}`}
              onMouseDown={(e) => {
                e.stopPropagation();
                triggerDown(midi, computeVelocity(e));
              }}
              onMouseUp={() => triggerUp(midi)}
              onMouseLeave={() => {
                if (pressedLocal.has(midi)) triggerUp(midi);
              }}
            >
              {isC && <span className={`key-oct-num key-oct-${displayOctave}`}>{displayOctave}</span>}
              <span className="key-note-letter">{letter}</span>
            </div>
          );
        })}

        {splitEnabled && (
          <div
            data-testid="split-divider-line"
            className="split-divider-line"
            style={{ left: `${splitDividerLeftPct}%` }}
            title={`Split Divider (${noteLabel(splitKey)})`}
          />
        )}

        {blackKeys.map(({ midi, leftPct }) => {
          const isActive = activeSet.has(midi);
          const isLeftSplit = splitEnabled && midi <= splitKey;
          return (
            <div
              key={midi}
              role="button"
              aria-label={`Key ${noteLabel(midi)}`}
              className={`black-key ${isActive ? "active" : ""} ${
                isLeftSplit ? "split-chord-zone" : ""
              }`}
              style={{ left: `${leftPct}%` }}
              onMouseDown={(e) => {
                e.stopPropagation();
                triggerDown(midi, computeVelocity(e));
              }}
              onMouseUp={() => triggerUp(midi)}
              onMouseLeave={() => {
                if (pressedLocal.has(midi)) triggerUp(midi);
              }}
            />
          );
        })}
      </div>
    </div>
  );
};
