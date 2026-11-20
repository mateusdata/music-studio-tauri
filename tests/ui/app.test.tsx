import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import App from "../../src/App";

describe("Piano Tect Tauri App baseline", () => {
  it("renders title, virtual keyboard, and pedal buttons", () => {
    render(<App />);
    expect(screen.getByText("Piano Tect Tauri")).toBeInTheDocument();
    expect(screen.getByTestId("virtual-keyboard")).toBeInTheDocument();
    expect(screen.getByText("Sustain (Damper)")).toBeInTheDocument();
  });
});
