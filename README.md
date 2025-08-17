# awesome-omarchy-tui

Terminal UI for browsing [awesome-omarchy](https://github.com/basecamp/omarchy) repository.

## Install

**Unix/Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/aorumbayev/awesome-omarchy-tui/main/install.sh | bash
```

**Windows:**
```powershell
iwr https://raw.githubusercontent.com/aorumbayev/awesome-omarchy-tui/main/install.ps1 | iex
```

**Manual:** Download from [releases](https://github.com/aorumbayev/awesome-omarchy-tui/releases)

## Usage

```bash
awsomarchy           # Launch TUI
awsomarchy version   # Show version  
awsomarchy update    # Update to latest
```

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

## License

MIT
