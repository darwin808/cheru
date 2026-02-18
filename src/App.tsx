import { useRef, useCallback, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "./themes";
import { SearchBar } from "./components/SearchBar";
import { ResultsList } from "./components/ResultsList";
import { ActionBar } from "./components/ActionBar";
import { PreviewPanel } from "./components/PreviewPanel";
import { useLauncher } from "./hooks/useLauncher";
import "./App.css";

function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const isKeyboardNav = useRef(false);
  const {
    query,
    results,
    selectedIndex,
    setSelectedIndex,
    browsePath,
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
          isKeyboardNav.current = true;
          moveSelection("down");
          break;
        case "ArrowUp":
          e.preventDefault();
          isKeyboardNav.current = true;
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

  // Refocus input and reload results when window becomes visible
  useEffect(() => {
    let cancelled = false;
    let unlistenFn: (() => void) | null = null;

    const currentWindow = getCurrentWindow();
    currentWindow
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return;
        if (focused) {
          if (inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
          }
          // Repopulate results after hide() cleared them
          search("");
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
  }, [search]);

  // Load theme from config
  useEffect(() => {
    invoke<{ theme: string; colors: Record<string, string> }>("get_theme").then(
      (cfg) => applyTheme(cfg.theme, cfg.colors)
    );
  }, []);

  const selectedResult = results[selectedIndex] ?? null;
  const showPreview = selectedResult?.result_type === "Image";

  return (
    <div className="launcher" onKeyDown={handleKeyDown} onMouseMove={() => { isKeyboardNav.current = false; }}>
      <SearchBar query={query} onQueryChange={search} inputRef={inputRef} />
      {browsePath && (
        <div className="breadcrumb">
          {browsePath}
        </div>
      )}
      <div className="content">
        <ResultsList
          results={results}
          selectedIndex={selectedIndex}
          onSelect={setSelectedIndex}
          onLaunch={launch}
          isKeyboardNav={isKeyboardNav}
        />
        {showPreview && <PreviewPanel result={selectedResult} />}
      </div>
      <ActionBar selectedResult={selectedResult} />
    </div>
  );
}

export default App;
