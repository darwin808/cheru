import { useEffect, useRef, useState, useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { AppResult } from "../types/launcher";
import styles from "./ResultsList.module.css";

interface ResultsListProps {
  results: AppResult[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onLaunch: () => void;
  isKeyboardNav: React.RefObject<boolean>;
}

interface Section {
  label: string;
  items: { result: AppResult; globalIndex: number }[];
}

const TYPE_LABELS: Record<string, string> = {
  Calculator: "Calculator",
  App: "Applications",
  System: "System",
  Folder: "Folders",
  Image: "Images",
  WebSearch: "Web Search",
};

function groupByType(results: AppResult[]): Section[] {
  const sections: Section[] = [];
  let currentType: string | null = null;
  let currentSection: Section | null = null;

  results.forEach((result, index) => {
    if (result.result_type !== currentType) {
      currentType = result.result_type;
      currentSection = {
        label: TYPE_LABELS[result.result_type] ?? result.result_type,
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
  isKeyboardNav,
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
        <div key={`${section.label}-${sections.indexOf(section)}`} className={styles.section}>
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
                onMouseEnter={() => { if (!isKeyboardNav.current) onSelect(globalIndex); }}
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
                        : result.result_type === "Image"
                        ? "\u{1F5BC}"
                        : result.result_type === "Calculator"
                        ? "\u{1F5A9}"
                        : result.result_type === "System"
                        ? "\u{2699}"
                        : result.result_type === "WebSearch"
                        ? "\u{1F50D}"
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
                  {TYPE_LABELS[result.result_type] ?? result.result_type}
                </span>
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
}
