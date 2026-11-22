import { useEffect, useRef } from "react";
import type { EngineTelemetry } from "../types/engine";

export interface StudioLogEntry {
  id: number;
  time: string;
  source: string;
  message: string;
  level: "info" | "error";
}

export function LogsDialog({ entries, telemetry, onClear, onClose }: {
  entries: StudioLogEntry[];
  telemetry: EngineTelemetry | null;
  onClear: () => void;
  onClose: () => void;
}) {
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    panel.current?.querySelector<HTMLButtonElement>("button")?.focus();
    return () => previous?.focus();
  }, []);
  return (
    <div className="logs-backdrop">
      <section ref={panel} className="logs-dialog" role="dialog" aria-modal="true" aria-label="Logs" onKeyDown={(event) => {
        if (event.key === "Escape") { event.stopPropagation(); onClose(); }
        if (event.key === "Tab") {
          const buttons = panel.current?.querySelectorAll<HTMLButtonElement>("button");
          if (!buttons?.length) return;
          const first = buttons[0], last = buttons[buttons.length - 1];
          if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
          else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
        }
      }}>
        <div className="panel-title"><span>Logs</span><button type="button" className="btn" onClick={onClose}>Fechar logs</button></div>
        <p>Mensagens desta sessão e avisos de importação. Histórico limitado às últimas 200 entradas.</p>
        <div className="logs-audio-info">
          <strong>Diagnóstico de áudio</strong>
          <span>Solicitado: {telemetry?.requested_buffer_size ?? "—"} frames · Callback observado: {telemetry?.buffer_size || "—"} frames · Taxa: {telemetry?.sample_rate || "—"} Hz</span>
          <span>{telemetry?.buffer_size && telemetry.sample_rate ? `${telemetry.buffer_size} ÷ ${telemetry.sample_rate} × 1000 = ${(telemetry.buffer_size / telemetry.sample_rate * 1000).toFixed(2)} ms por callback.` : "Aguardando callback de áudio do dispositivo."}</span>
          <span>Saída estimada pelo driver (timestamp callback → reprodução): {telemetry?.audio_active && telemetry.output_latency_ms != null && telemetry.output_latency_ms > 0 ? `${telemetry.output_latency_ms.toFixed(2)} ms` : "indisponível"}.</span>
          <span>Modo estrito: somente o buffer escolhido é solicitado. Se rejeitado, o áudio para; selecione outro valor manualmente.</span>
          <span>A latência total tecla/MIDI → som não foi medida. O período do bloco e a estimativa do driver não são uma medição ponta a ponta.</span>
        </div>
        <ol className="logs-entries" aria-label="Histórico de logs">
          {entries.length === 0 && <li>Nenhuma mensagem registrada.</li>}
          {entries.map((entry) => <li key={entry.id} className={`log-entry log-${entry.level}`}>
            <span className="log-entry-meta"><time>{entry.time}</time> · {entry.source}</span>
            <pre>{entry.message}</pre>
          </li>)}
        </ol>
        <div className="logs-actions"><button type="button" className="btn" onClick={onClear}>Limpar logs</button></div>
      </section>
    </div>
  );
}
