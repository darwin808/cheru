import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppResult } from "../types/launcher";

export function useLauncher() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<AppResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const search = useCallback((q: string) => {
    setQuery(q);
    setSelectedIndex(0);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(async () => {
      setIsLoading(true);
      try {
        const apps = await invoke<AppResult[]>("search_apps", { query: q });
        setResults(apps);
      } catch (err) {
        console.error("Search failed:", err);
        setResults([]);
      } finally {
        setIsLoading(false);
      }
    }, 60);
  }, []);

  const launch = useCallback(async () => {
    if (results.length === 0 || selectedIndex >= results.length) return;

    const app = results[selectedIndex];
    try {
      await invoke("launch_app", { exec: app.exec });
      await invoke("hide_launcher_window");
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
    } catch (err) {
      console.error("Launch failed:", err);
    }
  }, [results, selectedIndex]);

  const moveSelection = useCallback(
    (direction: "up" | "down") => {
      setSelectedIndex((prev) => {
        if (direction === "up") {
          return prev > 0 ? prev - 1 : results.length - 1;
        } else {
          return prev < results.length - 1 ? prev + 1 : 0;
        }
      });
    },
    [results.length]
  );

  const hide = useCallback(async () => {
    try {
      await invoke("hide_launcher_window");
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
    } catch (err) {
      console.error("Hide failed:", err);
    }
  }, []);

  // Load all apps on mount
  useEffect(() => {
    search("");
  }, [search]);

  return {
    query,
    results,
    selectedIndex,
    setSelectedIndex,
    isLoading,
    search,
    launch,
    moveSelection,
    hide,
  };
}
