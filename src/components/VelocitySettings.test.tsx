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
    expect(onChange).toHaveBeenLastCalledWith([[0, .5], [.5, .5], [1, .5]]);
    for (const label of ["Lo — toque suave", "Mid — toque médio", "Hi — toque forte"])
      expect(screen.getByRole("slider", { name: label })).toHaveValue("50");
    fireEvent.change(screen.getByRole("slider", { name: "Mid — toque médio" }), { target: { value: "75" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, .5], [.5, .75], [1, .5]]);
    fireEvent.click(screen.getByText("Restaurar padrão MIDI"));
    expect(onChange).toHaveBeenLastCalledWith(DEFAULT_VELOCITY_CURVE);
  });

  it("supports fixed output at every strength, custom curves and safe minimum", () => {
    const onChange = vi.fn();
    render(<VelocitySettings config={config} onChange={onChange} onClose={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("Modo da curva de velocidade"), { target: { value: "fixed" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, .5], [1, .5]]);
    fireEvent.change(screen.getByRole("slider", { name: "Velocidade fixa" }), { target: { value: "80" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, .8], [1, .8]]);
    expect(velocityCurveFor("fixed", [50,50,50], 0)).toEqual([[0,1/127],[1,1/127]]);
    expect(velocityCurveFor("custom", [80,50,20], 50)).toEqual([[0,.8],[.5,.5],[1,.2]]);
  });

  it("reopens a saved custom curve without resetting its values", () => {
    const onChange=vi.fn();
    render(<VelocitySettings config={{...config,velocity_curve:[[0,.2],[.5,.6],[1,.9]]}} onChange={onChange} onClose={vi.fn()} />);
    expect(screen.getByLabelText("Modo da curva de velocidade")).toHaveValue("custom");
    expect(screen.getByRole("slider", { name: "Lo — toque suave" })).toHaveValue("20");
    expect(screen.getByRole("slider", { name: "Mid — toque médio" })).toHaveValue("60");
    expect(screen.getByRole("slider", { name: "Hi — toque forte" })).toHaveValue("90");
    expect(onChange).not.toHaveBeenCalled();
  });

  it("supports up to 130% volume boost in sliders, curve output, and reopening saved boosted curves", () => {
    const onChange = vi.fn();
    const { unmount } = render(<VelocitySettings config={config} onChange={onChange} onClose={vi.fn()} />);
    fireEvent.change(screen.getByLabelText("Modo da curva de velocidade"), { target: { value: "custom" } });
    fireEvent.change(screen.getByRole("slider", { name: "Mid — toque médio" }), { target: { value: "130" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 0.5], [0.5, 1.3], [1, 0.5]]);
    fireEvent.change(screen.getByLabelText("Modo da curva de velocidade"), { target: { value: "fixed" } });
    fireEvent.change(screen.getByRole("slider", { name: "Velocidade fixa" }), { target: { value: "130" } });
    expect(onChange).toHaveBeenLastCalledWith([[0, 1.3], [1, 1.3]]);
    expect(velocityCurveFor("custom", [50, 115, 200], 50)).toEqual([[0, 0.5], [0.5, 1.15], [1, 1.3]]);
    expect(velocityCurveFor("fixed", [50, 50, 50], 180)).toEqual([[0, 1.3], [1, 1.3]]);
    unmount();

    const reopenChange = vi.fn();
    render(<VelocitySettings config={{ ...config, velocity_curve: [[0, 0.5], [0.5, 1.15], [1, 1.3]] }} onChange={reopenChange} onClose={vi.fn()} />);
    expect(screen.getByLabelText("Modo da curva de velocidade")).toHaveValue("custom");
    expect(screen.getByRole("slider", { name: "Lo — toque suave" })).toHaveValue("50");
    expect(screen.getByRole("slider", { name: "Mid — toque médio" })).toHaveValue("115");
    expect(screen.getByRole("slider", { name: "Hi — toque forte" })).toHaveValue("130");
    expect(reopenChange).not.toHaveBeenCalled();
  });
});
