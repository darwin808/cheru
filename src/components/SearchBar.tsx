import styles from "./SearchBar.module.css";

interface SearchBarProps {
  query: string;
  onQueryChange: (query: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
}

export function SearchBar({ query, onQueryChange, inputRef }: SearchBarProps) {
  return (
    <div className={styles.searchBar}>
      <svg
        className={styles.searchIcon}
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        ref={inputRef}
        className={styles.input}
        type="text"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        placeholder="Search applications..."
        spellCheck={false}
        autoFocus
      />
    </div>
  );
}
