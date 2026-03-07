# Niri Touch Deck (`niri-remote`)

A real-time, interactive Terminal User Interface (TUI) and Command Line multi-tool for the [Niri Wayland Compositor](https://github.com/YaLTeR/niri). 

Built with Rust and Ratatui, this tool visualizes your exact multi-monitor workspace and window layout in the terminal. It is built from the ground up to be used over **SSH from a tablet or phone**, acting as a physical touch-screen "Stream Deck" to control your Linux desktop. 

Don't want to use the UI? It also doubles as a blazing-fast CLI to instantly move, close, fullscreen, or swap windows using smart title-matching.

## ✨ Features

* **Proportional Real-Time Rendering:** Maps the exact pixel dimensions (`width`/`height`) of your Niri layout and draws them proportionally in the terminal.
* **Smart Workspace Swapping:** Swap entire workspaces between monitors with a single tap or command. It automatically recalculates and snaps window percentages (e.g., preserving perfect 50/50 splits) so your layout doesn't break when moving between ultrawide and standard 16:9 displays.
* **Multi-Monitor Support:** Automatically detects physical monitor layout (Left/Right) and draws an edge-to-edge map of your screens.
* **Touch-Screen / Mouse Control:** Click or tap any ASCII window box to instantly focus it on your real desktop.
* **Command Toolbar:** A built-in touch toolbar to move windows across columns, shift them to different monitors, toggle fullscreen, or close apps.
* **Smart CLI Automation:** Pass arguments directly to the binary to manage windows by their ID or by loosely matching their Title string.

## 🛠️ Prerequisites

* A Linux system running the **Niri** Wayland compositor.
* **Rust & Cargo** (only if you are building from source).

## 🚀 Installation & Build

Clone the repository and build the optimized release binary:

```bash
git clone [https://github.com/DeliciousGoatmeal/niri-remote.git](https://github.com/DeliciousGoatmeal/niri-remote.git)
cd niri-remote
cargo build --release
```

The compiled binary will be located at `target/release/niri-remote`.

---

## 💻 Usage: CLI Mode

You can interact with your Wayland windows purely from the command line without ever launching the visual TUI. The tool features smart string-matching, so you don't need to type the exact window title!

```bash
# List all active windows and their IDs
niri-remote list

# Swap workspaces between your two monitors (preserves layout proportions!)
niri-remote swap

# Move a window to a specific display (by ID or Title)
niri-remote move 123 to 2
niri-remote move brave to DP-1

# Fullscreen a window
niri-remote fullscreen discord

# Close a window
niri-remote close steam
```

---

## 📱 Usage: The Tablet / SSH Setup (TUI Mode)

If you run the app with no arguments (`niri-remote`), it launches the interactive terminal UI. 

While you can run this locally in your terminal, the real magic is running it from an external touch device (like an iPad or Android tablet) via SSH.

**The Wayland Socket Gotcha:**
Because SSH sessions are blind to your Wayland graphical environment, the app needs to know where the Niri IPC socket is located. To make this a seamless 1-click launch from your tablet and enable CLI arguments, add an alias to your main PC's shell config file.

**If you use Bash/Zsh (`~/.bashrc` or `~/.zshrc`):**
```bash
niri-remote() {
    export NIRI_SOCKET=$(find /run/user/$(id -u)/ -maxdepth 1 -type s -name "niri.*.sock" | head -n 1)
    ~/path/to/niri-remote/target/release/niri-remote "$@"
}
```

**If you use Fish (`~/.config/fish/config.fish`):**
```fish
function niri-remote
    set -x NIRI_SOCKET (find /run/user/(id -u)/ -maxdepth 1 -type s -name "niri.*.sock" | head -n 1)
    ~/path/to/niri-remote/target/release/niri-remote $argv
end
```

Now, simply SSH into your PC from your tablet and run:
```bash
niri-remote
```

## ⌨️ Native Niri Keybindings
Because the CLI tool handles all the heavy lifting, you can easily bind these commands directly to your Niri config (`~/.config/niri/config.kdl`) for native keyboard shortcuts!

```kdl
// Example: Bind Super + Shift + S to instantly swap your monitors
Mod+Shift+S { spawn "/absolute/path/to/niri-remote/target/release/niri-remote" "swap"; }
```

## 🏗️ Built With
* [Rust](https://www.rust-lang.org/)
* [Ratatui](https://github.com/ratatui-org/ratatui)
* [Serde](https://serde.rs/)
