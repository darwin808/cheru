# Cheru

**A fast, lightweight desktop launcher for macOS and Linux.**

Cheru is an open-source alternative to Raycast and Alfred, built with Tauri v2 (Rust) and React 19 (TypeScript). It stays out of your way until you need it — summon it with a global hotkey, find what you're looking for in milliseconds, and get back to work.

![Cheru Demo](demo.gif)

---

## Features

- **Fuzzy App Search** — indexes all installed applications at startup and searches using nucleo-matcher, the same fuzzy matching engine used by the Helix editor
- **Folder Search** — indexes common directories up to 3 levels deep for fast filesystem navigation
- **Image Search** — finds PNG, JPG, GIF, WebP, and SVG files across your home directory, capped at 5000 entries
- **Directory Drill-Down** — type `downloads/` to browse a folder's contents, then keep drilling with `/` to go deeper
- **Image Preview Panel** — a slide-in panel renders a preview when an image result is selected; GIF animation is supported
- **Configurable Hotkey** — toggle the launcher from anywhere; default `Alt+Space`, customizable via `~/.config/cheru/config.toml`
- **System Tray** — Show/Quit menu available in the menu bar / system tray
- **Real App Icons** — `.icns` files are converted to PNG in the background and cached at `~/.cache/cheru/icons/`
- **Keyboard-Driven** — arrow keys navigate, Enter launches or drills, Escape hides
- **Themes** — built-in Gruvbox (default), Dracula, Atom One Dark, and Dark themes; fully customizable colors via config

---

## Installation

### Homebrew (macOS)

```sh
brew tap darwin808/cheru
brew install --cask cheru
```

If macOS Gatekeeper blocks the app, remove the quarantine attribute and open manually:

```sh
xattr -cr /Applications/Cheru.app
open /Applications/Cheru.app
```

### Build from Source

**Prerequisites**

- Node.js 22+
- Rust 1.75+
- Linux only: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`

```sh
git clone https://github.com/darwin808/cheru.git
cd cheru
npm install

# Development
npm run tauri dev

# Production build
npm run tauri build
```

---

## Keyboard Shortcuts

| Key | Action |
|---|---|
| `Alt+Space` (default) | Toggle launcher (configurable) |
| `↑` / `↓` | Navigate results |
| `Enter` | Launch app / Open folder or image / Drill into folder |
| `Escape` | Hide launcher |
| Type `/` | Enter browse mode (e.g., `downloads/`) |

---

## Configuration

Cheru reads its config from `~/.config/cheru/config.toml`. A default file is created on first launch.

```toml
# Hotkey to toggle the launcher window
hotkey = "Cmd+D"

# Theme: "gruvbox" (default), "dark", "dracula", "one-dark"
theme = "gruvbox"

# Custom color overrides (optional)
# These override any theme's colors. Use CSS color values.
# [colors]
# bg_primary = "rgba(40, 40, 40, 0.92)"
# accent = "#d79921"
# text_primary = "#ebdbb2"
```

### Available Themes

| Theme | Description |
|---|---|
| `gruvbox` | Warm retro groove (default) |
| `dark` | Neutral dark with blue accent |
| `dracula` | Purple-accented dark theme |
| `one-dark` | Atom One Dark colors |

### Custom Colors

All CSS color values are supported. Available keys:

`bg_primary`, `bg_secondary`, `bg_hover`, `bg_selected`, `bg_actionbar`, `text_primary`, `text_secondary`, `text_placeholder`, `accent`, `border`

---

## Architecture

```
cheru/
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri setup, plugins, state, tray, hotkey
│       ├── commands.rs           # IPC commands + AppState
│       ├── config.rs             # Config file reader (~/.config/cheru/config.toml)
│       ├── matcher.rs            # nucleo-matcher fuzzy search wrapper
│       └── indexer/
│           ├── mod.rs            # AppEntry struct, folder/image indexing
│           ├── linux.rs          # .desktop file parsing
│           └── macos.rs          # .app bundle scanning + icon conversion
└── src/                          # React frontend
    ├── App.tsx                   # Root component, keyboard handling
    ├── App.css                   # Global styles, CSS variables
    ├── themes.ts                 # Built-in theme definitions + applicator
    ├── types/
    │   └── launcher.ts           # AppResult interface
    ├── hooks/
    │   └── useLauncher.ts        # Search state, browse mode, IPC
    └── components/
        ├── SearchBar.tsx         # Search input
        ├── ResultsList.tsx       # Grouped results with sections
        ├── PreviewPanel.tsx      # Image preview slide-in
        └── ActionBar.tsx         # Keyboard shortcut hints
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Tauri v2 (Rust) |
| Frontend | React 19 + TypeScript + Vite 6 |
| Fuzzy matching | nucleo-matcher 0.3 |
| Linux app discovery | freedesktop-desktop-entry |
| macOS app discovery | plist crate + sips |
| Global hotkey | tauri-plugin-global-shortcut |
| Styling | CSS Modules, 4 built-in themes |
| Configuration | TOML config file |

---

## IPC Commands

The frontend communicates with the Rust backend through Tauri's IPC bridge. All commands are invoked via `@tauri-apps/api/core`.

| Command | Arguments | Returns | Description |
|---|---|---|---|
| `search_apps` | `{ query }` | `AppResult[]` | Fuzzy search apps, max 50 results |
| `search_folders` | `{ query }` | `AppResult[]` | Fuzzy search folders, max 10 results |
| `search_images` | `{ query }` | `AppResult[]` | Fuzzy search images, max 20 results |
| `browse_directory` | `{ path, filter }` | `AppResult[]` | List directory contents, max 50 entries |
| `launch_app` | `{ exec }` | `void` | Launch application (allowlisted paths only) |
| `open_path` | `{ path }` | `void` | Open folder or image with system handler |
| `hide_launcher_window` | — | `void` | Hide the launcher window |
| `get_index_size` | — | `number` | Total number of indexed apps |
| `get_theme` | — | `ThemeConfig` | Returns theme name and custom color overrides |

---

## Data Flow

```
User types query
    |
    v
SearchBar -> useLauncher.search() -> 100ms debounce
    |
    v
+-- Normal mode:  search_apps + search_folders + search_images (parallel)
+-- Browse mode:  resolve first path segment -> browse_directory
    |
    v
ResultsList (grouped by type: Applications / Folders / Images)
    |
    v
+-- Arrow keys -> navigate results
+-- Enter      -> launch app / open file / drill into folder
+-- Escape     -> hide launcher
```

---

## Security

Cheru applies several layers of restrictions to prevent misuse of its launch and file-access capabilities:

- **Exec allowlist** — `launch_app` only accepts paths under `/Applications`, `/System/Applications`, `/usr/bin`, `/usr/local/bin`, `/opt`, and `~/Applications`
- **Path restrictions** — `open_path` and `browse_directory` are restricted to paths under `$HOME`
- **Content Security Policy** — locked down; no `eval`, no external resource loading
- **Icon path canonicalization** — icon paths are canonicalized before processing to prevent traversal
- **Field code stripping** — `%f`, `%u`, and other field codes are stripped from `.desktop` exec strings on Linux

---

## Known Limitations

- No Wayland layer-shell support; the window uses `alwaysOnTop` via XDG shell instead
- The global hotkey may not work on pure Wayland sessions — bind a compositor-level hotkey as a workaround
- The app index is built once at startup; there is no live refresh if you install new applications
- No extension or plugin system yet

---

## Platform Support

| Platform | Architecture | Status |
|---|---|---|
| macOS | ARM64 (Apple Silicon) | Supported |
| macOS | x64 (Intel) | Supported |
| Linux | x86_64 | Supported |

---

## Contributing

Pull requests are welcome. To run the Rust test suite:

```sh
cd src-tauri && cargo test
```

---

## Version

**0.2.3** — [github.com/darwin808/cheru](https://github.com/darwin808/cheru)
