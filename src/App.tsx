import { useRef, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchBar } from "./components/SearchBar";
import { ResultsList } from "./components/ResultsList";
import { ActionBar } from "./components/ActionBar";
import { useLauncher } from "./hooks/useLauncher";
import "./App.css";

function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const {
    query,
    results,
    selectedIndex,
    setSelectedIndex,
    search,
    launch,
    moveSelection,
    hide,
  } = useLauncher();

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          moveSelection("down");
          break;
        case "ArrowUp":
          e.preventDefault();
          moveSelection("up");
          break;
        case "Enter":
          e.preventDefault();
          launch();
          break;
        case "Escape":
          e.preventDefault();
          hide();
          break;
      }
    },
    [moveSelection, launch, hide]
  );

  // Refocus input when window becomes visible
  useEffect(() => {
    let cancelled = false;
    let unlistenFn: (() => void) | null = null;

    const currentWindow = getCurrentWindow();
    currentWindow
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return;
        if (focused && inputRef.current) {
          inputRef.current.focus();
          inputRef.current.select();
        }
      })
      .then((fn) => {
        if (cancelled) {
          fn();
        } else {
          unlistenFn = fn;
        }
      });

    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  }, []);

  return (
    <div className="launcher" onKeyDown={handleKeyDown}>
      <SearchBar query={query} onQueryChange={search} inputRef={inputRef} />
      <ResultsList
        results={results}
        selectedIndex={selectedIndex}
        onSelect={setSelectedIndex}
        onLaunch={launch}
      />
      <ActionBar
        selectedResult={results[selectedIndex] ?? null}
      />
    </div>
  );
}

export default App;
