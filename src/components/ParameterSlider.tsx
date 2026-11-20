import React from "react";

interface ParameterSliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
}

export const ParameterSlider: React.FC<ParameterSliderProps> = ({
  label,
  value,
  min,
  max,
  step = 0.01,
  unit = "",
  onChange,
}) => {
  const formatted =
    Math.abs(step) >= 1 ? Math.round(value).toString() : value.toFixed(2);

  return (
    <label className="control-row">
      <span className="control-label">
        <span>{label}</span>
        <span className="control-val">
          {formatted}
          {unit ? ` ${unit}` : ""}
        </span>
      </span>
      <input
        type="range"
        className="studio-range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
      />
    </label>
  );
};
