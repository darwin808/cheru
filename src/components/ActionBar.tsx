import { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";
import type { AppResult } from "../types/launcher";
import styles from "./ActionBar.module.css";

interface ActionBarProps {
  selectedResult: AppResult | null;
}

export function ActionBar({ selectedResult }: ActionBarProps) {
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  const actionLabel = selectedResult
    ? selectedResult.result_type === "Folder"
      ? "Open Folder"
      : selectedResult.result_type === "Image"
        ? "Open Image"
        : "Open Application"
    : "Open";

  return (
    <div className={styles.actionBar}>
      <div className={styles.left}>
        {version && <span className={styles.version}>v{version}</span>}
        {selectedResult && (
          <span className={styles.selectedType}>
            {selectedResult.result_type === "App"
              ? "Application"
              : selectedResult.result_type === "Folder"
                ? "Folder"
                : "Image"}
          </span>
        )}
      </div>
      <div className={styles.right}>
        <div className={styles.action}>
          <span className={styles.actionLabel}>{actionLabel}</span>
          <kbd className={styles.kbd}>↵</kbd>
        </div>
        <div className={styles.action}>
          <span className={styles.actionLabel}>Hide</span>
          <kbd className={styles.kbd}>esc</kbd>
        </div>
        <div className={styles.action}>
          <span className={styles.actionLabel}>Navigate</span>
          <kbd className={styles.kbd}>↑↓</kbd>
        </div>
      </div>
    </div>
  );
}
