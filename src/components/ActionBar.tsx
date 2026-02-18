import type { AppResult } from "../types/launcher";
import styles from "./ActionBar.module.css";

interface ActionBarProps {
  selectedResult: AppResult | null;
}

export function ActionBar({ selectedResult }: ActionBarProps) {
  const actionLabel = selectedResult
    ? selectedResult.result_type === "Folder"
      ? "Open Folder"
      : "Open Application"
    : "Open";

  return (
    <div className={styles.actionBar}>
      <div className={styles.left}>
        {selectedResult && (
          <span className={styles.selectedType}>
            {selectedResult.result_type === "App" ? "Application" : "Folder"}
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
