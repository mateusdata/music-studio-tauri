import React, { useEffect, useRef, useState } from "react";
import { Save, X, Check } from "lucide-react";

export interface SavePresetPopoverProps {
  defaultName?: string;
  onSavePreset: (name: string, category: string) => Promise<void> | void;
}

const PRESET_CATEGORIES = [
  "User",
  "Studio",
  "Intimate",
  "Binaural",
  "Concert",
  "Historical",
  "Experimental",
];

export const SavePresetPopover: React.FC<SavePresetPopoverProps> = ({
  defaultName = "",
  onSavePreset,
}) => {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState(defaultName);
  const [category, setCategory] = useState("User");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setError(null);
      setName(defaultName);
      setTimeout(() => inputRef.current?.focus(), 0);
    }
  }, [open, defaultName]);

  useEffect(() => {
    if (!open) return;
    const handleDocClick = (e: MouseEvent) => {
      if (wrapRef.current && !wrapRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleDocClick);
    window.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleDocClick);
      window.removeEventListener("keydown", handleKey);
    };
  }, [open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      setError("Enter a preset name");
      return;
    }
    setSaving(true);
    setError(null);
    try {
      await onSavePreset(trimmed, category);
      setOpen(false);
      setName("");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="popover-anchor" ref={wrapRef}>
      <button
        type="button"
        className={`btn ${open ? "btn-active" : ""}`}
        aria-expanded={open}
        aria-haspopup="dialog"
        onClick={() => setOpen((prev) => !prev)}
      >
        <Save size={13} /> Save User Preset
      </button>

      {open && (
        <div
          className="preset-popover"
          role="dialog"
          aria-label="Save User Preset"
          data-testid="save-preset-popover"
        >
          <div className="popover-header">
            <span>Save User Preset</span>
            <button
              type="button"
              className="icon-btn"
              aria-label="Close Save Preset"
              onClick={() => setOpen(false)}
            >
              <X size={13} />
            </button>
          </div>
          <form className="popover-form" onSubmit={handleSubmit}>
            <label className="control-row">
              <span className="control-label">Preset Name</span>
              <input
                ref={inputRef}
                type="text"
                className="studio-input"
                aria-label="Preset Name"
                placeholder="e.g. Late Night Felt"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </label>
            <label className="control-row">
              <span className="control-label">Category</span>
              <select
                className="studio-select"
                aria-label="Preset Category"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
              >
                {PRESET_CATEGORIES.map((cat) => (
                  <option key={cat} value={cat}>
                    {cat}
                  </option>
                ))}
              </select>
            </label>
            {error && <div className="popover-error">{error}</div>}
            <div className="popover-actions">
              <button
                type="button"
                className="btn"
                onClick={() => setOpen(false)}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn-primary"
                disabled={saving || !name.trim()}
              >
                <Check size={13} /> {saving ? "Saving..." : "Save"}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
};
