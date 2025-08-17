# awesome-omarchy-tui

Terminal UI for browsing [awesome-omarchy](https://github.com/aorumbayev/awesome-omarchy) repository.

## Features

- 🔍 **Full-text search** across repositories and descriptions
- 🎯 **Interactive navigation** with vim-style keybindings
- 🚀 **Auto-updates** - always stay on the latest version
- ⚡ **Lightweight** - ~6MB optimized binary
- 🌍 **Cross-platform** - Linux, macOS, Windows (x64/ARM64)

## Install

**Cargo:**
```bash
cargo install awesome-omarchy-tui
```

**Unix/Linux/macOS:**
```bash
curl -fsSL https://tui.awesome-omarchy.com/install.sh | bash
```

**Windows:**
```powershell
iwr https://tui.awesome-omarchy.com/install.ps1 | iex
```

**Manual:** Download from [releases](https://github.com/aorumbayev/awesome-omarchy-tui/releases)

## Usage

```bash
awsomarchy           # Launch TUI
awsomarchy version   # Show version  
awsomarchy update    # Update to latest (direct install only)
```

**Updates:**
- Direct install: `awsomarchy update`
- Cargo install: `cargo install awesome-omarchy-tui`

**Navigation:**
- `h/l` - Switch sidebar/content
- `j/k` - Navigate items
- `/` - Search
- `Enter` - Open repository
- `Q` - Quit

## Build

```bash
git clone https://github.com/aorumbayev/awesome-omarchy-tui.git
cd awesome-omarchy-tui
cargo build --release
```

Binary size: ~6MB (optimized with LTO, stripped)

## License

MIT
