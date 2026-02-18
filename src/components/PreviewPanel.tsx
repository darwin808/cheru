import { useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AppResult } from "../types/launcher";
import styles from "./PreviewPanel.module.css";

interface PreviewPanelProps {
  result: AppResult;
}

export function PreviewPanel({ result }: PreviewPanelProps) {
  const imageUrl = useMemo(
    () => (result.icon ? convertFileSrc(result.icon) : ""),
    [result.icon]
  );

  const isGif = result.name.toLowerCase().endsWith(".gif");

  return (
    <div className={styles.panel}>
      <div className={styles.imageContainer}>
        {imageUrl && (
          <img
            className={styles.previewImage}
            src={imageUrl}
            alt={result.name}
          />
        )}
      </div>
      <div className={styles.info}>
        <span className={styles.fileName}>{result.name}</span>
        {result.description && (
          <span className={styles.filePath}>{result.description}</span>
        )}
        {isGif && <span className={styles.badge}>GIF</span>}
      </div>
    </div>
  );
}
