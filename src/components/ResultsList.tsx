import { useEffect, useRef, useState, useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AppResult } from "../types/launcher";
import styles from "./ResultsList.module.css";

interface ResultsListProps {
  results: AppResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onLaunch: () => void;
}

interface Section {
  label: string;
  items: { result: AppResult; globalIndex: number }[];
}

function groupByType(results: AppResult[]): Section[] {
  const sections: Section[] = [];
  let currentType: string | null = null;
  let currentSection: Section | null = null;

  results.forEach((result, index) => {
    if (result.result_type !== currentType) {
      currentType = result.result_type;
      currentSection = {
        label: result.result_type === "App" ? "Applications" : "Folders",
        items: [],
      };
      sections.push(currentSection);
    }
    currentSection!.items.push({ result, globalIndex: index });
  });

  return sections;
}

export function ResultsList({
  results,
  selectedIndex,
  onSelect,
  onLaunch,
}: ResultsListProps) {
  const listRef = useRef<HTMLDivElement>(null);
  const selectedRef = useRef<HTMLDivElement>(null);
  const [failedIcons, setFailedIcons] = useState<Set<string>>(new Set());

  // Memoize converted icon URLs
  const iconUrls = useMemo(() => {
    const urls = new Map<string, string>();
    for (const result of results) {
      if (result.icon && !urls.has(result.icon)) {
        urls.set(result.icon, convertFileSrc(result.icon));
      }
    }
    return urls;
  }, [results]);

  // Clear failed icons when results change
  useEffect(() => {
    setFailedIcons(new Set());
  }, [results]);

  useEffect(() => {
    if (selectedRef.current) {
      selectedRef.current.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  }, [selectedIndex]);

  if (results.length === 0) {
    return <div className={styles.empty}>No results found</div>;
  }

  const sections = groupByType(results);

  return (
    <div className={styles.list} ref={listRef}>
      {sections.map((section) => (
        <div key={section.label} className={styles.section}>
          <div className={styles.sectionHeader}>{section.label}</div>
          {section.items.map(({ result, globalIndex }) => {
            const iconUrl = result.icon ? iconUrls.get(result.icon) : null;
            const showIcon = iconUrl && !failedIcons.has(result.exec);

            return (
              <div
                key={`${result.exec}-${globalIndex}`}
                ref={globalIndex === selectedIndex ? selectedRef : null}
                className={`${styles.item} ${
                  globalIndex === selectedIndex ? styles.selected : ""
                }`}
                onMouseEnter={() => onSelect(globalIndex)}
                onClick={onLaunch}
              >
                <div className={styles.appIcon}>
                  {showIcon ? (
                    <img
                      className={styles.appIconImg}
                      src={iconUrl}
                      alt=""
                      onError={() =>
                        setFailedIcons((prev) => new Set(prev).add(result.exec))
                      }
                    />
                  ) : (
                    <span className={styles.appIconPlaceholder}>
                      {result.result_type === "Folder"
                        ? "\u{1F4C1}"
                        : result.name.charAt(0).toUpperCase()}
                    </span>
                  )}
                </div>
                <div className={styles.appInfo}>
                  <span className={styles.appName}>{result.name}</span>
                  {result.description && (
                    <span className={styles.appDescription}>
                      {result.description}
                    </span>
                  )}
                </div>
                <span className={styles.typeLabel}>
                  {result.result_type === "App" ? "Application" : "Folder"}
                </span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}
