import { type KeyboardEvent, useEffect, useMemo, useRef, useState } from "react";
import { Search, X } from "lucide-react";

export interface SearchableOption {
  value: string;
  label: string;
  sublabel?: string;
}

export interface SearchableOptionGroup {
  label: string;
  items: SearchableOption[];
}

export interface SearchableSelectProps {
  ariaLabel: string;
  value: string;
  onChange: (value: string) => void;
  groups?: SearchableOptionGroup[];
  options?: SearchableOption[];
  searchPlaceholder?: string;
  className?: string;
}

export function SearchableSelect({
  ariaLabel,
  value,
  onChange,
  groups,
  options,
  searchPlaceholder,
  className,
}: SearchableSelectProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [highlightIdx, setHighlightIdx] = useState(0);

  const containerRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  const allItems = useMemo(() => {
    if (groups && groups.length > 0) {
      return groups.flatMap((g) => g.items);
    }
    return options ?? [];
  }, [groups, options]);

  const selectedOption = useMemo(
    () => allItems.find((item) => item.value === value) ?? allItems[0],
    [allItems, value],
  );

  const tokens = useMemo(
    () =>
      query
        .trim()
        .toLowerCase()
        .split(/\s+/)
        .filter((t) => t.length > 0),
    [query],
  );

  const filteredGroups = useMemo(() => {
    if (!groups || groups.length === 0) return [];
    if (tokens.length === 0) return groups.filter((g) => g.items.length > 0);
    return groups
      .map((group) => ({
        label: group.label,
        items: group.items.filter((item) => {
          const haystack = `${item.label} ${item.sublabel ?? ""} ${group.label}`.toLowerCase();
          return tokens.every((tok) => haystack.includes(tok));
        }),
      }))
      .filter((group) => group.items.length > 0);
  }, [groups, tokens]);

  const filteredOptions = useMemo(() => {
    if (groups && groups.length > 0) return [];
    const list = options ?? [];
    if (tokens.length === 0) return list;
    return list.filter((opt) => {
      const haystack = `${opt.label} ${opt.sublabel ?? ""}`.toLowerCase();
      return tokens.every((tok) => haystack.includes(tok));
    });
  }, [groups, options, tokens]);

  const flatFiltered = useMemo(() => {
    if (groups && groups.length > 0) {
      return filteredGroups.flatMap((g) => g.items);
    }
    return filteredOptions;
  }, [groups, filteredGroups, filteredOptions]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setHighlightIdx(0);
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handleOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handleOutside);
    return () => document.removeEventListener("mousedown", handleOutside);
  }, [open]);

  const selectItem = (val: string) => {
    onChange(val);
    setOpen(false);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      e.preventDefault();
      setOpen(false);
      return;
    }
    if (flatFiltered.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setHighlightIdx((prev) => (prev + 1) % flatFiltered.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setHighlightIdx((prev) => (prev - 1 + flatFiltered.length) % flatFiltered.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      const target = flatFiltered[Math.min(highlightIdx, flatFiltered.length - 1)];
      if (target) {
        selectItem(target.value);
      }
    }
  };

  const testIdPrefix = ariaLabel.toLowerCase().replace(/\s+/g, "-");
  let runningIdx = 0;

  return (
    <div className={`searchable-select-wrap ${className ?? ""}`} ref={containerRef}>
      <select
        aria-label={ariaLabel}
        className="sr-only-select"
        tabIndex={-1}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {groups && groups.length > 0
          ? groups.map((group) => (
              <optgroup key={group.label} label={group.label}>
                {group.items.map((item) => (
                  <option key={item.value} value={item.value}>
                    {item.label}
                  </option>
                ))}
              </optgroup>
            ))
          : (options ?? []).map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
      </select>

      <button
        type="button"
        className="studio-select searchable-select-trigger"
        aria-label={`${ariaLabel} Trigger`}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((prev) => !prev)}
      >
        {selectedOption?.label ?? "Select..."}
      </button>

      {open && (
        <div
          className="searchable-select-popover"
          data-testid={`${testIdPrefix}-popover`}
        >
          <div className="searchable-select-search-row">
            <Search size={13} className="searchable-select-icon" />
            <input
              ref={inputRef}
              type="text"
              className="studio-input searchable-select-input"
              aria-label={`${ariaLabel} Search`}
              placeholder={searchPlaceholder ?? "Pesquisar..."}
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setHighlightIdx(0);
              }}
              onKeyDown={handleKeyDown}
            />
            {query.length > 0 && (
              <button
                type="button"
                className="searchable-select-clear"
                aria-label="Clear search"
                onClick={() => {
                  setQuery("");
                  setHighlightIdx(0);
                  inputRef.current?.focus();
                }}
              >
                <X size={12} />
              </button>
            )}
          </div>

          <div className="searchable-select-list" role="listbox" aria-label={`${ariaLabel} Options`}>
            {flatFiltered.length === 0 ? (
              <div className="searchable-select-empty">Nenhum resultado encontrado</div>
            ) : groups && groups.length > 0 ? (
              filteredGroups.map((group) => (
                <div key={group.label} className="searchable-select-group">
                  <div className="searchable-select-group-label">{group.label}</div>
                  {group.items.map((item) => {
                    const idx = runningIdx++;
                    const isSelected = item.value === value;
                    const isHighlighted = idx === highlightIdx;
                    return (
                      <button
                        key={item.value}
                        type="button"
                        role="option"
                        aria-selected={isSelected}
                        className={`searchable-select-option ${isSelected ? "active" : ""} ${
                          isHighlighted ? "highlighted" : ""
                        }`}
                        onMouseEnter={() => setHighlightIdx(idx)}
                        onClick={() => selectItem(item.value)}
                      >
                        <span className="searchable-select-option-label">{item.label}</span>
                        {item.sublabel && (
                          <span className="searchable-select-option-sub">{item.sublabel}</span>
                        )}
                      </button>
                    );
                  })}
                </div>
              ))
            ) : (
              filteredOptions.map((opt, idx) => {
                const isSelected = opt.value === value;
                const isHighlighted = idx === highlightIdx;
                return (
                  <button
                    key={opt.value}
                    type="button"
                    role="option"
                    aria-selected={isSelected}
                    className={`searchable-select-option ${isSelected ? "active" : ""} ${
                      isHighlighted ? "highlighted" : ""
                    }`}
                    onMouseEnter={() => setHighlightIdx(idx)}
                    onClick={() => selectItem(opt.value)}
                  >
                    <span className="searchable-select-option-label">{opt.label}</span>
                    {opt.sublabel && (
                      <span className="searchable-select-option-sub">{opt.sublabel}</span>
                    )}
                  </button>
                );
              })
            )}
          </div>
        </div>
      )}
    </div>
  );
}
