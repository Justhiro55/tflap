# tflap

[![Crates.io](https://img.shields.io/crates/v/tflap.svg)](https://crates.io/crates/tflap)
[![Downloads](https://img.shields.io/crates/d/tflap.svg)](https://crates.io/crates/tflap)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

> A classic Flappy Bird game in your terminal 🐦

Play the addictive Flappy Bird game directly in your terminal with smooth animations, persistent high scores, and simple controls.

## Features

- 🎮 Smooth terminal-based gameplay
- 🏆 Persistent high score tracking
- 🎨 Colorful ASCII graphics
- ⌨️  Simple keyboard controls
- 🚀 Cross-platform (macOS, Linux, Windows)
- 💾 Auto-save high scores to `~/.tflap_highscore`

## Installation

### Using Cargo

```bash
cargo install tflap
```

### From source

```bash
git clone https://github.com/yourusername/tflap
cd tflap
cargo install --path .
```

## Usage

Simply run the command to start playing:

```bash
tflap
```

### Controls

- **Space**: Jump (during gameplay)
- **R**: Retry (after game over)
- **Q / Esc**: Quit the game
- **Ctrl+C**: Force quit

### Gameplay

- Navigate the bird through the pipes by tapping Space to jump
- Avoid hitting the pipes or the ground
- Score points by passing through pipes
- Try to beat your high score!

## How to Play

1. Press **Space** to make the bird jump
2. Avoid colliding with pipes or boundaries
3. Each pipe you pass increases your score by 1
4. When you set a new record, you'll see a special celebration screen!

## Requirements

- Rust 1.70 or later
- A terminal with Unicode support

## License

Licensed under either of MIT or Apache-2.0 at your option. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
