# RDash - Vim-like Server Dashboard

A Vim-inspired terminal dashboard for managing and launching your server programs and utilities.

## Features

- **Vim-like Interface**: Full-screen TUI with familiar keyboard navigation
- **Custom Program Management**: Add your own programs with custom display names
- **Keyboard-Driven**: Everything works via keyboard shortcuts
- **Persistent Configuration**: Stores settings in `~/.config/rdash/config.json`
- **Easy Program Launch**: Launch programs directly from the dashboard
- **Hand-Editable Config**: JSON configuration can be edited manually
- **Sudo Support**: Run programs with elevated privileges (shows [SUDO] indicator)
- **Output Capture**: Capture and display program output in a popup box (shows [OUT] indicator)
- **Cross-Platform**: Built with Rust for performance and reliability

## Installation

### From Source
```bash
git clone https://github.com/rafaelsieber/rdash.git
cd rdash
cargo build --release
```

### Install to System
```bash
cargo install --path .
```

Or copy the binary to your PATH:
```bash
cp target/release/rdash ~/.local/bin/
# or
sudo cp target/release/rdash /usr/local/bin/
```

## Usage

Launch the dashboard:
```bash
rdash
```

### Keyboard Shortcuts

**Navigation:**
- `j` or `↓` - Move down
- `k` or `↑` - Move up
- `Enter` - Launch selected program

**Program Management:**
- `a` - Add new program
- `d` - Delete selected program
- `r` - Reload configuration

**Other:**
- `h` or `F1` - Show help
- `q` or `Esc` - Quit

### Adding Programs

1. Press `a` to enter add mode
2. Fill in the following information (7 steps):
   - **Program Name**: Unique identifier (e.g., "rfin")
   - **Display Name**: What appears on dashboard (e.g., "Controle Financeiro")
   - **Command**: Executable path or name (e.g., "rfin")
   - **Arguments**: Optional command-line arguments (e.g., "status")
   - **Description**: Optional description of the program
   - **Run with sudo**: y/n - whether to run with elevated privileges
   - **Show output**: y/n - whether to capture and display output in a popup
3. Review your input and press Enter to save

### Program Indicators

Programs show visual indicators for their configuration:
- `[SUDO]` - Program will run with sudo privileges
- `[OUT]` - Program output will be captured and displayed
- Both can be combined: `UFW Status [SUDO] [OUT] - Check firewall status`

### Output Display

For programs with output capture enabled:
- Output is displayed in a bordered popup box
- Shows both STDOUT and STDERR if present
- Press `SPACE` or `ESC` to close the output window
- Perfect for commands like `sudo ufw status`, `df -h`, `systemctl status`, etc.
   - **Arguments**: Space-separated arguments (optional)
   - **Description**: Brief description (optional)
3. Press `Enter` to proceed through each step
4. Review and press `Enter` to save

### Example Configuration

The configuration is stored in `~/.config/rdash/config.json`:

```json
{
  "programs": {
    "rfin": {
      "name": "rfin",
      "display_name": "Controle Financeiro",
      "command": "rfin",
      "args": [],
      "description": "Financial control system"
    },
    "htop": {
      "name": "htop",
      "display_name": "System Monitor",
      "command": "htop",
      "args": [],
      "description": "System resource monitor"
    }
  }
}
```

## Building from Source

Requirements:
- Rust 1.70+ (2021 edition)

```bash
git clone https://github.com/rafaelsieber/rdash.git
cd rdash
cargo build --release
```

## Dependencies

- `crossterm` - Cross-platform terminal manipulation
- `serde` - Serialization framework
- `serde_json` - JSON support
- `dirs` - System directories
- `clap` - Command line argument parsing

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- Built with [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal handling
- Inspired by Vim's keyboard-driven interface philosophy
