import React, { useEffect, useMemo, useRef, useState } from "react";
import { ChevronRight, Clock, Layers, Search, X } from "lucide-react";
import type { InstrumentModel, PresetData } from "../types/engine";
import { ParameterSlider } from "./ParameterSlider";

export interface PackDefinition {
  id: string;
  title: string;
  column: 1 | 2 | 3;
  match: (inst: InstrumentModel) => boolean;
}

function textIncludesAny(inst: InstrumentModel, tokens: string[]): boolean {
  const hay = `${inst.id} ${inst.name}`.toLowerCase();
  return tokens.some((tok) => hay.includes(tok));
}

export const PIANOTEQ_PACKS: PackDefinition[] = [
  // Column 1 (12 packs)
  {
    id: "steinway-d",
    title: "Steinway D Instrument Pack",
    column: 1,
    match: (i) =>
      textIncludesAny(i, ["grand-ny-d", "grand-hb-d", "steinway d", "steinway-d"]),
  },
  {
    id: "steinway-b",
    title: "Steinway B Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["steinway-b", "model b", "steinway b"]),
  },
  {
    id: "kawai",
    title: "Kawai Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["sk-ex", "shigeru", "kawai"]),
  },
  {
    id: "yfx",
    title: "YFX Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["yfx", "cfx"]),
  },
  {
    id: "bosendorfer",
    title: "Bösendorfer Instrument Pack",
    column: 1,
    match: (i) =>
      textIncludesAny(i, ["bosendorfer", "bösendorfer", "280vc", "vienna concert"]),
  },
  {
    id: "bechstein-dg",
    title: "BechsteinDG Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["bechstein-dg", "bechstein dg", "d 282", "berlin concert"]),
  },
  {
    id: "petrof",
    title: "Petrof Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["petrof", "p 275", "bohemian", "mistral"]),
  },
  {
    id: "guitar",
    title: "Guitar Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["guitar"]),
  },
  {
    id: "steingraeber",
    title: "Steingraeber Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["steingraeber", "e-272", "bayreuth"]),
  },
  {
    id: "grotrian",
    title: "Grotrian Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["grotrian", "braunschweig"]),
  },
  {
    id: "bluthner",
    title: "Blüthner Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["bluthner", "blüthner", "leipzig", "aliquot"]),
  },
  {
    id: "yc5",
    title: "YC5 Instrument Pack",
    column: 1,
    match: (i) => textIncludesAny(i, ["yc5"]),
  },

  // Column 2 (10 packs)
  {
    id: "k2",
    title: "K2 Instrument Pack",
    column: 2,
    match: (i) => textIncludesAny(i, ["grand-k2", "grand k2"]),
  },
  {
    id: "u4",
    title: "U4 Instrument Pack",
    column: 2,
    match: (i) =>
      i.category === "UprightPiano" || textIncludesAny(i, ["upright-u4", "upright u4"]),
  },
  {
    id: "electric",
    title: "Electric Instrument Pack",
    column: 2,
    match: (i) =>
      !textIncludesAny(i, ["kivir", "cp80", "cp-80"]) &&
      textIncludesAny(i, ["tines", "reeds", "rhodes", "wurlitzer", "mk1", "mk2"]),
  },
  {
    id: "hohner",
    title: "Hohner Instrument Pack",
    column: 2,
    match: (i) => textIncludesAny(i, ["clavinet", "pianet", "hohner", "electra"]),
  },
  {
    id: "syngular",
    title: "Syngular Instrument Pack",
    column: 2,
    match: (i) => i.category === "Synthesizer" || textIncludesAny(i, ["syngular"]),
  },
  {
    id: "vibes",
    title: "Vibes Instrument Pack",
    column: 2,
    match: (i) => textIncludesAny(i, ["vibraphone", "vibe"]),
  },
  {
    id: "celeste",
    title: "Celeste Instrument Pack",
    column: 2,
    match: (i) =>
      textIncludesAny(i, ["celesta", "celeste", "glockenspiel", "toy-piano", "toy piano", "kalimba"]),
  },
  {
    id: "xylo",
    title: "Xylo Instrument Pack",
    column: 2,
    match: (i) => textIncludesAny(i, ["xylo", "marimba"]),
  },
  {
    id: "steelpans",
    title: "Steelpans Instrument Pack",
    column: 2,
    match: (i) =>
      textIncludesAny(i, ["steel-drum", "steel drum", "steelpan", "spacedrum", "hand-pan", "hand pan", "tank-drum", "tank drum"]),
  },
  {
    id: "harpsichord",
    title: "Harpsichord Instrument Pack",
    column: 2,
    match: (i) => textIncludesAny(i, ["ruckers"]),
  },

  // Column 3 (6 packs)
  {
    id: "harp",
    title: "Harp Instrument Pack",
    column: 3,
    match: (i) =>
      textIncludesAny(i, ["harp"]) && !textIncludesAny(i, ["harpsichord"]),
  },
  {
    id: "karsten",
    title: "Karsten Instrument Pack",
    column: 3,
    match: (i) =>
      !textIncludesAny(i, ["kivir", "walter"]) &&
      textIncludesAny(i, [
        "karsten",
        "salodiensis",
        "giusti",
        "schmidt",
        "weimes",
        "wiener-pianoforte",
        "ny-steinway-square",
        "steinway square",
      ]),
  },
  {
    id: "kremsegg1",
    title: "Kremsegg1 Instrument Pack",
    column: 3,
    match: (i) =>
      textIncludesAny(i, ["kremsegg1", "dohnal", "besendorfer-1829", "erard", "érard", "streicher"]),
  },
  {
    id: "kremsegg2",
    title: "Kremsegg2 Instrument Pack",
    column: 3,
    match: (i) =>
      textIncludesAny(i, ["kremsegg2", "broadwood", "pleyel", "frenzel", "bechstein-1899", "bechstein (1899)"]),
  },
  {
    id: "morphing-layers",
    title: "Morphing/Layers",
    column: 3,
    match: () => false,
  },
  {
    id: "kivir-bells",
    title: "KIViR Instruments & Bells",
    column: 3,
    match: (i) =>
      i.category === "Prepared" ||
      textIncludesAny(i, [
        "kivir",
        "clavichord",
        "cymbalum",
        "cimbalom",
        "walter",
        "cp80",
        "cp-80",
        "bell",
        "carillon",
        "prepared",
      ]),
  },
];

export interface InstrumentPackBrowserProps {
  instruments: InstrumentModel[];
  presets: PresetData[];
  activeInstrumentId: string;
  activePresetId: string;
  presetBId: string | null;
  morphing: number;
  recentPresetIds: string[];
  onSelectInstrument: (instrumentId: string) => Promise<void> | void;
  onSelectPreset: (presetId: string) => Promise<void> | void;
  onMorphPresets: (presetAId: string, presetBId: string, t: number) => Promise<void> | void;
}

export const InstrumentPackBrowser: React.FC<InstrumentPackBrowserProps> = ({
  instruments,
  presets,
  activeInstrumentId,
  activePresetId,
  presetBId,
  morphing,
  recentPresetIds,
  onSelectInstrument,
  onSelectPreset,
  onMorphPresets,
}) => {
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [inspectedInstrumentId, setInspectedInstrumentId] = useState<string>(activeInstrumentId);
  const [morphAId, setMorphAId] = useState<string>(activePresetId);
  const [morphBId, setMorphBId] = useState<string>(
    presetBId ?? presets[1]?.id ?? activePresetId,
  );
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    setInspectedInstrumentId(activeInstrumentId);
    setMorphAId(activePresetId);
    setMorphBId(presetBId ?? presets[1]?.id ?? activePresetId);
    setSearchQuery("");
    const focusTimer = setTimeout(() => searchInputRef.current?.focus(), 0);
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      clearTimeout(focusTimer);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [open, activeInstrumentId, activePresetId, presetBId, presets]);

  const query = searchQuery.trim().toLowerCase();

  const presetsByInstrument = useMemo(() => {
    const map = new Map<string, PresetData[]>();
    for (const preset of presets) {
      const list = map.get(preset.instrument_id) ?? [];
      list.push(preset);
      map.set(preset.instrument_id, list);
    }
    return map;
  }, [presets]);

  const { packItems, unmatchedInstruments } = useMemo(() => {
    const claimed = new Set<string>();
    const map = new Map<string, InstrumentModel[]>();
    for (const pack of PIANOTEQ_PACKS) {
      const matched = instruments.filter((inst) => {
        if (claimed.has(inst.id)) return false;
        if (pack.match(inst)) {
          claimed.add(inst.id);
          return true;
        }
        return false;
      });
      map.set(pack.id, matched);
    }
    const unmatched = instruments.filter((inst) => !claimed.has(inst.id));
    return { packItems: map, unmatchedInstruments: unmatched };
  }, [instruments]);

  const recentPresets = useMemo(() => {
    const byId = new Map(presets.map((p) => [p.id, p]));
    return recentPresetIds
      .map((id) => byId.get(id))
      .filter((p): p is PresetData => Boolean(p));
  }, [presets, recentPresetIds]);

  const matchesFilter = (packTitle: string, inst: InstrumentModel): boolean => {
    if (!query) return true;
    if (packTitle.toLowerCase().includes(query)) return true;
    if (inst.name.toLowerCase().includes(query) || inst.id.toLowerCase().includes(query)) {
      return true;
    }
    const instPresets = presetsByInstrument.get(inst.id) ?? [];
    return instPresets.some((p) => p.name.toLowerCase().includes(query));
  };

  const inspectedInstrument =
    instruments.find((i) => i.id === inspectedInstrumentId) ?? instruments[0];
  const inspectedPresets = inspectedInstrument
    ? presetsByInstrument.get(inspectedInstrument.id) ?? []
    : [];

  const visibleInspectedPresets = query
    ? inspectedPresets.filter(
        (p) =>
          p.name.toLowerCase().includes(query) ||
          (inspectedInstrument?.name.toLowerCase().includes(query) ?? false),
      )
    : inspectedPresets;

  const handleChooseInstrument = async (instId: string) => {
    await onSelectInstrument(instId);
    setOpen(false);
  };

  const handleChoosePreset = async (presetId: string) => {
    await onSelectPreset(presetId);
    setOpen(false);
  };

  const renderPackColumn = (col: 1 | 2 | 3) => {
    const colPacks = PIANOTEQ_PACKS.filter((p) => p.column === col);
    return (
      <div className="packs-col" data-testid={`packs-col-${col}`}>
        {colPacks.map((pack) => {
          if (pack.id === "morphing-layers") {
            if (query && !pack.title.toLowerCase().includes(query)) return null;
            return (
              <div key={pack.id} className="pack-group pack-morph-card">
                <div className="pack-header">
                  <span>{pack.title}</span>
                  <Layers size={12} />
                </div>
                <div className="pack-morph-body">
                  <div className="two-col-mini">
                    <label className="control-row">
                      <span className="control-label">Layer / Preset A</span>
                      <select
                        aria-label="Morph Preset A"
                        className="studio-select studio-select-sm"
                        value={morphAId}
                        onChange={(e) => {
                          const nextA = e.target.value;
                          setMorphAId(nextA);
                          onMorphPresets(nextA, morphBId, morphing);
                        }}
                      >
                        {presets.map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label className="control-row">
                      <span className="control-label">Layer / Preset B</span>
                      <select
                        aria-label="Morph Preset B"
                        className="studio-select studio-select-sm"
                        value={morphBId}
                        onChange={(e) => {
                          const nextB = e.target.value;
                          setMorphBId(nextB);
                          onMorphPresets(morphAId, nextB, morphing);
                        }}
                      >
                        {presets.map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                  <ParameterSlider
                    label="Morph A ↔ B"
                    value={morphing}
                    min={0}
                    max={1}
                    step={0.01}
                    onChange={(t) => onMorphPresets(morphAId, morphBId, t)}
                  />
                </div>
              </div>
            );
          }

          const rawItems = packItems.get(pack.id) ?? [];
          const filteredItems = rawItems.filter((inst) => matchesFilter(pack.title, inst));
          if (query && filteredItems.length === 0 && !pack.title.toLowerCase().includes(query)) {
            return null;
          }

          return (
            <div key={pack.id} className="pack-group">
              <div className="pack-header">
                <span>{pack.title}</span>
                {rawItems.length > 0 && (
                  <span className="pack-count">{rawItems.length}</span>
                )}
              </div>
              {filteredItems.length > 0 ? (
                <div className="pack-items">
                  {filteredItems.map((inst) => {
                    const isActive = inst.id === activeInstrumentId;
                    const isInspected = inst.id === inspectedInstrument?.id;
                    const instPresets = presetsByInstrument.get(inst.id) ?? [];
                    return (
                      <div
                        key={inst.id}
                        className={`pack-inst-row ${isActive ? "active" : ""} ${
                          isInspected ? "inspected" : ""
                        }`}
                        onMouseEnter={() => setInspectedInstrumentId(inst.id)}
                      >
                        <button
                          type="button"
                          className="pack-inst-btn"
                          onClick={() => handleChooseInstrument(inst.id)}
                          onFocus={() => setInspectedInstrumentId(inst.id)}
                        >
                          <span className="pack-inst-name">{inst.name}</span>
                        </button>
                        <button
                          type="button"
                          className="pack-submenu-btn"
                          aria-label={`Presets for ${inst.name}`}
                          title={`Browse ${instPresets.length} presets for ${inst.name}`}
                          onClick={(e) => {
                            e.stopPropagation();
                            setInspectedInstrumentId(inst.id);
                          }}
                        >
                          <span className="pack-preset-count">{instPresets.length}</span>
                          <ChevronRight size={12} />
                        </button>
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div className="pack-empty-row">Included in Full Library</div>
              )}
            </div>
          );
        })}

        {col === 3 && unmatchedInstruments.length > 0 && (
          <div className="pack-group">
            <div className="pack-header">
              <span>Other Instruments</span>
              <span className="pack-count">{unmatchedInstruments.length}</span>
            </div>
            <div className="pack-items">
              {unmatchedInstruments
                .filter((inst) => matchesFilter("Other Instruments", inst))
                .map((inst) => (
                  <div
                    key={inst.id}
                    className={`pack-inst-row ${
                      inst.id === activeInstrumentId ? "active" : ""
                    }`}
                    onMouseEnter={() => setInspectedInstrumentId(inst.id)}
                  >
                    <button
                      type="button"
                      className="pack-inst-btn"
                      onClick={() => handleChooseInstrument(inst.id)}
                    >
                      <span className="pack-inst-name">{inst.name}</span>
                    </button>
                    <button
                      type="button"
                      className="pack-submenu-btn"
                      aria-label={`Presets for ${inst.name}`}
                      onClick={() => setInspectedInstrumentId(inst.id)}
                    >
                      <ChevronRight size={12} />
                    </button>
                  </div>
                ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  return (
    <>
      <button
        type="button"
        className={`btn ${open ? "btn-active" : ""}`}
        aria-haspopup="dialog"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
      >
        All Packs & Presets
      </button>

      {open && (
        <div
          className="packs-backdrop"
          data-testid="pack-browser-backdrop"
          onClick={() => setOpen(false)}
        >
          <div
            className="packs-megamenu"
            role="dialog"
            aria-label="Instrument Packs Browser"
            data-testid="instrument-packs-browser"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="packs-topbar">
              <div className="packs-title-area">
                <span className="packs-dialog-title">Pianoteq Instrument Packs & Presets</span>
                <span className="badge-subtle">{instruments.length} instruments</span>
              </div>

              <div className="packs-search-wrap">
                <Search size={13} className="packs-search-icon" />
                <input
                  ref={searchInputRef}
                  type="text"
                  className="studio-input packs-search-input"
                  aria-label="Search Instruments and Presets"
                  placeholder="Filter packs, instruments, or presets..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
              </div>

              <button
                type="button"
                className="icon-btn"
                aria-label="Close Packs Browser"
                onClick={() => setOpen(false)}
              >
                <X size={16} />
              </button>
            </div>

            {/* Recent Presets strip at top left */}
            <div className="recent-presets-strip" data-testid="recent-presets-strip">
              <span className="recent-label">
                <Clock size={12} /> Recent Presets:
              </span>
              {recentPresets.length > 0 ? (
                recentPresets.map((preset) => (
                  <button
                    key={preset.id}
                    type="button"
                    className={`chip-btn ${
                      preset.id === activePresetId ? "active" : ""
                    }`}
                    onClick={() => handleChoosePreset(preset.id)}
                  >
                    {preset.name}
                  </button>
                ))
              ) : (
                <span className="pack-empty-inline">No recent presets yet</span>
              )}
            </div>

            <div className="packs-body-layout">
              <div className="packs-columns-grid" data-testid="packs-columns-grid">
                {renderPackColumn(1)}
                {renderPackColumn(2)}
                {renderPackColumn(3)}
              </div>

              {/* Right-side Presets Submenu for selected/hovered instrument */}
              <aside
                className="packs-presets-submenu"
                aria-label="Instrument Presets Submenu"
                data-testid="pack-presets-submenu"
              >
                <div className="submenu-header">
                  <div className="submenu-inst-name">
                    {inspectedInstrument?.name ?? "Select Instrument"}
                  </div>
                  <div className="submenu-inst-cat">
                    {inspectedInstrument?.category ?? ""}
                  </div>
                </div>
                {inspectedInstrument?.description && (
                  <p className="submenu-desc">{inspectedInstrument.description}</p>
                )}
                <div className="submenu-list">
                  {visibleInspectedPresets.length > 0 ? (
                    visibleInspectedPresets.map((preset) => (
                      <button
                        key={preset.id}
                        type="button"
                        className={`submenu-preset-btn ${
                          preset.id === activePresetId ? "active" : ""
                        }`}
                        onClick={() => handleChoosePreset(preset.id)}
                      >
                        <span>{preset.name}</span>
                        <span className="submenu-preset-cat">{preset.category}</span>
                      </button>
                    ))
                  ) : (
                    <div className="pack-empty-row">No matching presets</div>
                  )}
                </div>
              </aside>
            </div>
          </div>
        </div>
      )}
    </>
  );
};
