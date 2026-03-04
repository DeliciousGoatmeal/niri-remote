# Niri Touch Deck (`niri-ascii`)


*(Tip: Replace the line above with an actual screenshot or GIF of your tablet running the UI!)*

A real-time, interactive Terminal User Interface (TUI) and Command Line multi-tool for the [Niri Wayland Compositor](https://github.com/YaLTeR/niri). 

Built with Rust and Ratatui, this tool visualizes your exact multi-monitor workspace and window layout in the terminal. It is built from the ground up to be used over **SSH from a tablet or phone**, acting as a physical touch-screen "Stream Deck" to control your Linux desktop. 

Don't want to use the UI? It also doubles as a blazing-fast CLI to instantly move, close, or fullscreen windows using smart title-matching.

## ✨ Features

* **Proportional Real-Time Rendering:** Maps the exact pixel dimensions (`width`/`height`) of your Niri layout and draws them proportionally in the terminal.
* **Multi-Monitor Support:** Automatically detects physical monitor layout (Left/Right) and draws an edge-to-edge map of your screens.
* **Touch-Screen / Mouse Control:** Click or tap any ASCII window box to instantly focus it on your real desktop.
* **Command Toolbar:** A built-in touch toolbar to move windows across columns, shift them to different monitors, toggle fullscreen, or close apps.
* **Smart CLI Automation:** Pass arguments directly to the binary to manage windows by their ID or by loosely matching their Title string.

## 🛠️ Prerequisites

* A Linux system running the **Niri** Wayland compositor.
* **Rust & Cargo** (to compile the binary).

## 🚀 Installation & Build

Clone the repository and build the optimized release binary:

```bash
git clone [https://github.com/yourusername/niri-ascii.git](https://github.com/yourusername/niri-ascii.git)
cd niri-ascii
cargo build --release
The compiled binary will be located at target/release/niri-ascii.

💻 Usage: CLI Mode
You can interact with your Wayland windows purely from the command line without ever launching the visual TUI. The tool features smart string-matching, so you don't need to type the exact window title!

Bash
# List all active windows and their IDs
niri-remote list

# Move a window to a specific display (by ID or Title)
niri-remote move 123 to 2
niri-remote move brave to DP-1

# Fullscreen a window
niri-remote fullscreen discord

# Close a window
niri-remote close steam
📱 Usage: The Tablet / SSH Setup (TUI Mode)
If you run the app with no arguments (niri-remote), it launches the interactive terminal UI.

While you can run this locally in your terminal, the real magic is running it from an external touch device (like an iPad or Android tablet) via SSH.

The Wayland Socket Gotcha:
Because SSH sessions are blind to your Wayland graphical environment, the app needs to know where the Niri IPC socket is located. To make this a seamless 1-click launch from your tablet and enable CLI arguments, add an alias to your main PC's shell config file.

If you use Bash/Zsh (~/.bashrc or ~/.zshrc):

Bash
niri-remote() {
    export NIRI_SOCKET=$(find /run/user/$(id -u)/ -maxdepth 1 -type s -name "niri.*.sock" | head -n 1)
    ~/path/to/niri-ascii/target/release/niri-ascii "$@"
}
If you use Fish (~/.config/fish/config.fish):

Code snippet
function niri-remote
    set -x NIRI_SOCKET (find /run/user/(id -u)/ -maxdepth 1 -type s -name "niri.*.sock" | head -n 1)
    ~/path/to/niri-ascii/target/release/niri-ascii $argv
end
Now, simply SSH into your PC from your tablet and run:

Bash
niri-remote
⌨️ Local Keybindings
If you are running the TUI locally using a keyboard instead of touch:

q: Quit the application

h, j, k, l or Arrow Keys: Move focus between Niri windows

Shift + h, j, k, l or Arrow Keys: Physically move the window around your screen

🏗️ Built With
Rust

Ratatui

Serde
