import { useEffect, useRef } from "react";
import type { AppResult } from "../types/launcher";
import styles from "./ResultsList.module.css";

interface ResultsListProps {
  results: AppResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onLaunch: () => void;
}

export function ResultsList({
  results,
  selectedIndex,
  onSelect,
  onLaunch,
}: ResultsListProps) {
  const listRef = useRef<HTMLUListElement>(null);
  const selectedRef = useRef<HTMLLIElement>(null);

  useEffect(() => {
    if (selectedRef.current) {
      selectedRef.current.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  }, [selectedIndex]);

  if (results.length === 0) {
    return <div className={styles.empty}>No applications found</div>;
  }

  return (
    <ul className={styles.list} ref={listRef}>
      {results.map((app, index) => (
        <li
          key={`${app.exec}-${index}`}
          ref={index === selectedIndex ? selectedRef : null}
          className={`${styles.item} ${
            index === selectedIndex ? styles.selected : ""
          }`}
          onMouseEnter={() => onSelect(index)}
          onClick={onLaunch}
        >
          <div className={styles.appIcon}>
            <span className={styles.appIconPlaceholder}>
              {app.name.charAt(0).toUpperCase()}
            </span>
          </div>
          <div className={styles.appInfo}>
            <span className={styles.appName}>{app.name}</span>
            {app.description && (
              <span className={styles.appDescription}>{app.description}</span>
            )}
          </div>
        </li>
      ))}
    </ul>
  );
}
