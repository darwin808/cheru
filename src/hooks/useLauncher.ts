import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppResult } from "../types/launcher";

export function useLauncher() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<AppResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [browsePath, setBrowsePath] = useState<string | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const resolvedBasesRef = useRef<Map<string, string>>(new Map());

  const resolveFirstSegment = useCallback(async (segment: string): Promise<string | null> => {
    // Check cache first
    const cached = resolvedBasesRef.current.get(segment.toLowerCase());
    if (cached) return cached;

    // Use search_folders to find the best match
    const folders = await invoke<AppResult[]>("search_folders", { query: segment });
    if (folders.length > 0) {
      const resolved = folders[0].exec;
      resolvedBasesRef.current.set(segment.toLowerCase(), resolved);
      return resolved;
    }
    return null;
  }, []);

  const search = useCallback((q: string) => {
    setQuery(q);
    setSelectedIndex(0);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(async () => {
      setIsLoading(true);
      try {
        if (q.includes("/")) {
          // Browse mode: parse path segments
          const slashIndex = q.indexOf("/");
          const firstSegment = q.substring(0, slashIndex);
          const rest = q.substring(slashIndex + 1);

          if (!firstSegment) {
            setResults([]);
            setBrowsePath(null);
            setIsLoading(false);
            return;
          }

          // Resolve the first segment to an actual path
          const basePath = await resolveFirstSegment(firstSegment);
          if (!basePath) {
            setResults([]);
            setBrowsePath(null);
            setIsLoading(false);
            return;
          }

          // Parse remaining path: everything up to the last / is directory, after is filter
          let dirPath = basePath;
          let filter = "";

          if (rest === "") {
            // Just "downloads/" — browse the base
            filter = "";
          } else if (rest.endsWith("/")) {
            // "downloads/anime/" — browse downloads/anime
            dirPath = basePath + "/" + rest.slice(0, -1);
            filter = "";
          } else if (rest.includes("/")) {
            // "downloads/anime/one" — browse downloads/anime, filter "one"
            const lastSlash = rest.lastIndexOf("/");
            dirPath = basePath + "/" + rest.substring(0, lastSlash);
            filter = rest.substring(lastSlash + 1);
          } else {
            // "downloads/ani" — browse downloads, filter "ani"
            filter = rest;
          }

          setBrowsePath(dirPath);
          const browsed = await invoke<AppResult[]>("browse_directory", {
            path: dirPath,
            filter,
          });
          setResults(browsed);
        } else {
          // Normal search mode
          setBrowsePath(null);
          const [apps, folders, images, calcResult] = await Promise.all([
            invoke<AppResult[]>("search_apps", { query: q }),
            invoke<AppResult[]>("search_folders", { query: q }),
            invoke<AppResult[]>("search_images", { query: q }),
            invoke<string | null>("eval_expression", { expr: q }),
          ]);
          // Deduplicate by exec path
          const seen = new Set<string>();
          const merged: AppResult[] = [];

          // Prepend calculator result if available
          if (calcResult) {
            merged.push({
              name: `= ${calcResult}`,
              exec: `calc:${calcResult}`,
              icon: null,
              description: q,
              result_type: "Calculator",
            });
          }

          for (const r of [...apps, ...folders, ...images]) {
            if (!seen.has(r.exec)) {
              seen.add(r.exec);
              merged.push(r);
            }
          }

          // Web search fallback when few results
          if (q.length >= 2 && merged.length < 3) {
            merged.push({
              name: `Search Google for "${q}"`,
              exec: `https://www.google.com/search?q=${encodeURIComponent(q)}`,
              icon: null,
              description: "Open in browser",
              result_type: "WebSearch",
            });
          }

          setResults(merged);
        }
      } catch (err) {
        console.error("Search failed:", err);
        setResults([]);
      } finally {
        setIsLoading(false);
      }
    }, 300);
  }, [resolveFirstSegment]);

  const launch = useCallback(async () => {
    if (results.length === 0 || selectedIndex >= results.length) return;

    const app = results[selectedIndex];

    // If it's a folder in browse mode, drill into it instead of opening
    if (app.result_type === "Folder" && browsePath !== null) {
      // Reconstruct the query to drill into this folder
      const folderName = app.name;
      // Find the current query up to the last segment and append the folder name
      const lastSlash = query.lastIndexOf("/");
      const prefix = query.substring(0, lastSlash + 1);
      const newQuery = prefix + folderName + "/";
      search(newQuery);
      return;
    }

    try {
      if (app.result_type === "Calculator") {
        // Calculator results are display-only, do nothing on Enter
        return;
      } else if (app.result_type === "WebSearch") {
        await invoke("open_url", { url: app.exec });
      } else if (app.result_type === "System") {
        const id = app.exec.replace("system:", "");
        await invoke("run_system_command", { id });
      } else if (app.result_type === "Folder" || app.result_type === "Image") {
        await invoke("open_path", { path: app.exec });
      } else {
        await invoke("launch_app", { exec: app.exec });
      }
      await invoke("hide_launcher_window");
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
      setBrowsePath(null);
    } catch (err) {
      console.error("Launch failed:", err);
    }
  }, [results, selectedIndex, browsePath, query, search]);

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
      setBrowsePath(null);
    } catch (err) {
      console.error("Hide failed:", err);
    }
  }, []);

  // Load all apps on mount (no debounce)
  useEffect(() => {
    let cancelled = false;
    Promise.all([
      invoke<AppResult[]>("search_apps", { query: "" }),
      invoke<AppResult[]>("search_folders", { query: "" }),
      invoke<AppResult[]>("search_images", { query: "" }),
    ]).then(([apps, folders, images]) => {
      if (!cancelled) {
        const seen = new Set<string>();
        const merged = [...apps, ...folders, ...images].filter((r) => {
          if (seen.has(r.exec)) return false;
          seen.add(r.exec);
          return true;
        });
        setResults(merged);
      }
    });
    return () => { cancelled = true; };
  }, []);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  return {
    query,
    results,
    selectedIndex,
    setSelectedIndex,
    isLoading,
    browsePath,
    search,
    launch,
    moveSelection,
    hide,
  };
}
