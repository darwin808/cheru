export interface ThemeColors {
  bg_primary: string;
  bg_secondary: string;
  bg_hover: string;
  bg_selected: string;
  bg_actionbar: string;
  text_primary: string;
  text_secondary: string;
  text_placeholder: string;
  accent: string;
  border: string;
  border_light: string;
  shadow: string;
}

const dark: ThemeColors = {
  bg_primary: "rgba(30, 30, 30, 0.85)",
  bg_secondary: "rgba(45, 45, 45, 0.9)",
  bg_hover: "rgba(60, 60, 60, 0.8)",
  bg_selected: "rgba(80, 120, 200, 0.35)",
  bg_actionbar: "rgba(25, 25, 25, 0.6)",
  text_primary: "#e4e4e4",
  text_secondary: "#999",
  text_placeholder: "#666",
  accent: "#5b9cf5",
  border: "rgba(255, 255, 255, 0.08)",
  border_light: "rgba(255, 255, 255, 0.12)",
  shadow: "0 8px 32px rgba(0, 0, 0, 0.4)",
};

const gruvbox: ThemeColors = {
  bg_primary: "rgba(40, 40, 40, 0.92)",
  bg_secondary: "rgba(60, 56, 54, 0.9)",
  bg_hover: "rgba(80, 73, 69, 0.8)",
  bg_selected: "rgba(215, 153, 33, 0.3)",
  bg_actionbar: "rgba(29, 32, 33, 0.6)",
  text_primary: "#ebdbb2",
  text_secondary: "#a89984",
  text_placeholder: "#665c54",
  accent: "#d79921",
  border: "rgba(235, 219, 178, 0.08)",
  border_light: "rgba(235, 219, 178, 0.12)",
  shadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
};

const dracula: ThemeColors = {
  bg_primary: "rgba(40, 42, 54, 0.92)",
  bg_secondary: "rgba(68, 71, 90, 0.9)",
  bg_hover: "rgba(68, 71, 90, 0.8)",
  bg_selected: "rgba(189, 147, 249, 0.25)",
  bg_actionbar: "rgba(33, 34, 44, 0.6)",
  text_primary: "#f8f8f2",
  text_secondary: "#6272a4",
  text_placeholder: "#44475a",
  accent: "#bd93f9",
  border: "rgba(248, 248, 242, 0.08)",
  border_light: "rgba(248, 248, 242, 0.12)",
  shadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
};

const oneDark: ThemeColors = {
  bg_primary: "rgba(40, 44, 52, 0.92)",
  bg_secondary: "rgba(53, 59, 69, 0.9)",
  bg_hover: "rgba(62, 68, 81, 0.8)",
  bg_selected: "rgba(97, 175, 239, 0.25)",
  bg_actionbar: "rgba(33, 37, 43, 0.6)",
  text_primary: "#abb2bf",
  text_secondary: "#5c6370",
  text_placeholder: "#4b5263",
  accent: "#61afef",
  border: "rgba(171, 178, 191, 0.08)",
  border_light: "rgba(171, 178, 191, 0.12)",
  shadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
};

export const themes: Record<string, ThemeColors> = {
  dark,
  gruvbox,
  dracula,
  "one-dark": oneDark,
};

export function applyTheme(
  themeName: string,
  overrides: Record<string, string> = {}
) {
  const base = themes[themeName] ?? themes.dark;
  const merged = { ...base, ...overrides };
  const root = document.documentElement;

  root.style.setProperty("--bg-primary", merged.bg_primary);
  root.style.setProperty("--bg-secondary", merged.bg_secondary);
  root.style.setProperty("--bg-hover", merged.bg_hover);
  root.style.setProperty("--bg-selected", merged.bg_selected);
  root.style.setProperty("--bg-actionbar", merged.bg_actionbar);
  root.style.setProperty("--text-primary", merged.text_primary);
  root.style.setProperty("--text-secondary", merged.text_secondary);
  root.style.setProperty("--text-placeholder", merged.text_placeholder);
  root.style.setProperty("--accent", merged.accent);
  root.style.setProperty("--border", merged.border);
  root.style.setProperty("--border-light", merged.border_light);
  root.style.setProperty("--shadow", merged.shadow);
}
