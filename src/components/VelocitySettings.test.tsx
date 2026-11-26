import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_VELOCITY_CURVE, VelocitySettings, velocityCurveFor } from "./VelocitySettings";
import type { MidiConfig } from "../types/engine";

const config: MidiConfig = { selected_port: null, channel: 0, microtonal_multichannel: false,
  transpose: 0, key_range_mode: "Full88", velocity_curve: DEFAULT_VELOCITY_CURVE,
  cc_mappings: [] };

describe("Velocity settings", () => {
  it("preserves original response until explicitly changed", () => {
    const onChange = vi.fn();
    render(<VelocitySettings config={config} onChange={onChange} onClose={vi.fn()} />);
    expect(screen.getByLabelText("Modo da curva de velocidade")).toHaveValue("default");
    expect(onChange).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Modo da curva de velocidade"), { target: { value: "custom" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1], [.5, 1], [1, 1]]);
    for (const label of ["Lo — toque suave", "Mid — toque médio", "Hi — toque forte"])
      expect(screen.getByRole("slider", { name: label })).toHaveValue("50");
    fireEvent.change(screen.getByRole("slider", { name: "Mid — toque médio" }), { target: { value: "75" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1], [.5, 1.5], [1, 1]]);
    fireEvent.change(screen.getByRole("slider", { name: "Mid — toque médio" }), { target: { value: "100" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1], [.5, 2], [1, 1]]);
    fireEvent.click(screen.getByText("Restaurar padrão MIDI"));
    expect(onChange).toHaveBeenLastCalledWith(DEFAULT_VELOCITY_CURVE);
  });

  it("supports fixed output at every strength, custom curves and safe minimum", () => {
    const onChange = vi.fn();
    render(<VelocitySettings config={config} onChange={onChange} onClose={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("Modo da curva de velocidade"), { target: { value: "fixed" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1], [1, 1]]);
    fireEvent.change(screen.getByRole("slider", { name: "Velocidade fixa" }), { target: { value: "80" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1.6], [1, 1.6]]);
    fireEvent.change(screen.getByRole("slider", { name: "Velocidade fixa" }), { target: { value: "100" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 2], [1, 2]]);
    expect(velocityCurveFor("fixed", [50, 50, 50], 0)).toEqual([[0, 1 / 127], [1, 1 / 127]]);
    expect(velocityCurveFor("custom", [80, 50, 20], 50)).toEqual([[0, 1.6], [.5, 1], [1, .4]]);
    expect(velocityCurveFor("custom", [50, 100, 150], 50)).toEqual([[0, 1], [.5, 2], [1, 2]]);
  });

  it("reopens a saved custom or boosted curve on 0-100 UI scale without resetting", () => {
    const onChange = vi.fn();
    render(<VelocitySettings config={{ ...config, velocity_curve: [[0, .4], [.5, 1.2], [1, 2]] }} onChange={onChange} onClose={vi.fn()} />);
    expect(screen.getByLabelText("Modo da curva de velocidade")).toHaveValue("custom");
    for (const [label, value] of [["Lo — toque suave", "20"], ["Mid — toque médio", "60"], ["Hi — toque forte", "100"]] as const) {
      const slider = screen.getByRole("slider", { name: label });
      expect(slider).toHaveAttribute("max", "100");
      expect(slider).toHaveValue(value);
    }
    expect(onChange).not.toHaveBeenCalled();
  });
});
