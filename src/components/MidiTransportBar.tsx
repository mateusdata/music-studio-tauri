import React, { useEffect, useRef, useState } from "react";
import { Play, Square, Circle, RotateCw, Radio } from "lucide-react";
import { engineClient } from "../services/engineClient";
import type { EngineTelemetry, MidiTransportState } from "../types/engine";

export interface MidiTransportBarProps {
  telemetry?: EngineTelemetry | null;
  onStatusMessage?: (msg: string) => void;
}

const DEFAULT_TRANSPORT: MidiTransportState = {
  is_playing: false,
  is_recording: false,
  playback_speed: 1.0,
  recorded_events_count: 0,
  current_track_name: null,
};

const CC_LEARN_TARGETS: { path: string; label: string }[] = [
  { path: "volume_db", label: "Master Volume" },
  { path: "morphing", label: "A/B Morphing" },
  { path: "dynamics_db", label: "Dynamics" },
  { path: "output.lid_position", label: "Lid Position" },
  { path: "reverb.mix", label: "Reverb Mix" },
];

export const MidiTransportBar: React.FC<MidiTransportBarProps> = ({
  telemetry,
  onStatusMessage,
}) => {
  const [transport, setTransport] = useState<MidiTransportState>(DEFAULT_TRANSPORT);
  const [selectedLearnPath, setSelectedLearnPath] = useState<string>("volume_db");
  const [learnTarget, setLearnTarget] = useState<string | null>(null);

  const prevTelTransportRef = useRef<string>("");
  const prevTelLearnRef = useRef<string | null | undefined>(undefined);

  useEffect(() => {
    const t = telemetry?.midi_transport;
    if (t) {
      const sig = `${t.is_playing}|${t.is_recording}|${t.playback_speed}|${t.recorded_events_count}|${t.current_track_name ?? ""}`;
      if (sig !== prevTelTransportRef.current) {
        prevTelTransportRef.current = sig;
        setTransport(t);
      }
    }
    if (telemetry && telemetry.midi_learn_target !== prevTelLearnRef.current) {
      prevTelLearnRef.current = telemetry.midi_learn_target;
      setLearnTarget(telemetry.midi_learn_target ?? null);
    }
  }, [telemetry]);

  const handlePlayDemo = async () => {
    setTransport((prev) => ({
      ...prev,
      is_playing: true,
      current_track_name: prev.current_track_name ?? "Demo Prelude",
    }));
    try {
      const next = await engineClient.playDemoMidi();
      if (next) setTransport(next);
      onStatusMessage?.("Playing Demo MIDI");
    } catch {
      onStatusMessage?.("Playing Demo MIDI (preview)");
    }
  };

  const handleStop = async () => {
    setTransport((prev) => ({
      ...prev,
      is_playing: false,
      is_recording: false,
    }));
    try {
      const next = await engineClient.stopMidiTransport();
      if (next) setTransport(next);
      onStatusMessage?.("MIDI transport stopped");
    } catch {
      onStatusMessage?.("MIDI transport stopped (preview)");
    }
  };

  const handleToggleRecord = async () => {
    setTransport((prev) => ({
      ...prev,
      is_recording: !prev.is_recording,
    }));
    try {
      const next = await engineClient.toggleMidiRecord();
      if (next) setTransport(next);
      onStatusMessage?.(next?.is_recording ? "MIDI recording armed" : "MIDI recording stopped");
    } catch {
      // keep optimistic state
    }
  };

  const handlePlayRec = async () => {
    setTransport((prev) => ({
      ...prev,
      is_playing: true,
      current_track_name: "Recorded Take",
    }));
    try {
      const next = await engineClient.playRecordedMidi();
      if (next) setTransport(next);
      onStatusMessage?.("Playing recorded MIDI");
    } catch {
      // keep optimistic state
    }
  };

  const handleSpeedChange = async (speed: number) => {
    const clamped = Math.max(0.25, Math.min(2.0, speed));
    setTransport((prev) => ({ ...prev, playback_speed: clamped }));
    try {
      const next = await engineClient.setMidiPlaybackSpeed(clamped);
      if (next) setTransport(next);
    } catch {
      // keep optimistic state
    }
  };

  const handleToggleCcLearn = async () => {
    const nextParam = learnTarget ? null : selectedLearnPath;
    setLearnTarget(nextParam);
    try {
      const result = await engineClient.armMidiCcLearn(nextParam);
      setLearnTarget(result ?? null);
      onStatusMessage?.(
        result ? `MIDI CC Learn armed for ${result}` : "MIDI CC Learn disarmed",
      );
    } catch {
      // keep optimistic state
    }
  };

  return (
    <div className="midi-transport-ribbon" data-testid="midi-transport-bar">
      <div className="transport-group">
        <span className="transport-label">MIDI Transport</span>
        <button
          type="button"
          className={`btn btn-compact ${transport.is_playing ? "btn-active" : ""}`}
          aria-pressed={transport.is_playing}
          onClick={handlePlayDemo}
        >
          <Play size={12} /> Play Demo
        </button>
        <button
          type="button"
          className="btn btn-compact"
          onClick={handleStop}
        >
          <Square size={11} /> Stop
        </button>
        <button
          type="button"
          className={`btn btn-compact ${transport.is_recording ? "btn-rec-active" : ""}`}
          aria-pressed={transport.is_recording}
          onClick={handleToggleRecord}
        >
          <Circle size={11} /> Record
        </button>
        <button
          type="button"
          className="btn btn-compact"
          onClick={handlePlayRec}
        >
          <RotateCw size={11} /> Play Rec
        </button>

        <label className="transport-speed-wrap">
          <span>Speed</span>
          <input
            type="range"
            aria-label="MIDI Playback Speed"
            className="studio-range transport-speed-range"
            min={0.25}
            max={2.0}
            step={0.05}
            value={transport.playback_speed}
            onChange={(e) => handleSpeedChange(parseFloat(e.target.value))}
          />
          <span className="control-val">{transport.playback_speed.toFixed(2)}x</span>
        </label>
      </div>

      <div className="transport-group">
        <span className="transport-status-pill">
          {transport.is_recording
            ? "REC"
            : transport.is_playing
              ? transport.current_track_name ?? "Playing"
              : "Idle"}
          {transport.recorded_events_count > 0
            ? ` • ${transport.recorded_events_count} ev`
            : ""}
        </span>

        <select
          className="studio-select studio-select-sm"
          aria-label="MIDI CC Learn Target"
          value={selectedLearnPath}
          onChange={(e) => {
            const nextPath = e.target.value;
            setSelectedLearnPath(nextPath);
            if (learnTarget !== null) {
              setLearnTarget(nextPath);
              engineClient.armMidiCcLearn(nextPath).catch(() => {});
            }
          }}
        >
          {CC_LEARN_TARGETS.map((item) => (
            <option key={item.path} value={item.path}>
              CC → {item.label}
            </option>
          ))}
        </select>

        <button
          type="button"
          className={`btn btn-compact ${learnTarget ? "btn-active" : ""}`}
          aria-pressed={Boolean(learnTarget)}
          onClick={handleToggleCcLearn}
        >
          <Radio size={12} />{" "}
          {learnTarget ? `MIDI CC Learn (${learnTarget})` : "MIDI CC Learn"}
        </button>
      </div>
    </div>
  );
};
