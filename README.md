<p align="center">
  <img src="assets/panos_logo.png" width="350" alt="PANOS Logo">
</p>

# 🌌 PANOS: Universal File Organizer OS

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: GPL](https://img.shields.io/badge/License-GPL-yellow.svg)](https://opensource.org/licenses/GPL-3.0)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://GitHub.com/Nonbangkok/panos/graphs/commit-activity)

**PANOS** is a high-performance, rule-based CLI file management tool engineered for speed and reliability. It transforms cluttered directories into perfectly structured hierarchies using a "set-and-forget" automation approach.

---

## ✨ Features

- ⚡ **Lightning Fast**: High-performance directory traversal using `WalkDir` and pre-compiled glob patterns for O(1) matching speed during execution.
- 📐 **Intelligent Rule Engine**: Support for both extension-based sorting and complex glob pattern matching (e.g., `Invoice_*.pdf`).
- 🧪 **Safety First**: Comprehensive `--dry-run` mode and intelligent conflict resolution (automatic suffixing) to prevent data loss.
- 🧹 **Trash System**: specialized handling for temporary files (`.tmp`, `.crdownload`), moving them to a dedicated `.panos_trash` for safe review.
- 🌳 **Deep Clean**: Recursive removal of empty directories after organization to keep your filesystem pristine.
- 🔄 **Watch Mode**: (Target: Phase 3) Real-time daemonized monitoring.

---

## 🚀 Quick Start

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Nonbangkok/panos.git
cd panos

# Build for release
cargo build --release
```

### Basic Usage

```bash
# Run organization based on default panos.toml
./target/release/panos

# Preview changes without moving files
./target/release/panos --dry-run

# Specify a custom source directory
./target/release/panos --source ~/Downloads
```

---

## 🛠 Configuration (`panos.toml`)

PANOS uses a simple TOML configuration to define how your files should be handled.

```toml
source_dir = "Downloads"
watch_mode = false

[[rules]]
name = "Images"
extensions = ["jpg", "jpeg", "png", "gif"]
destination = "Media/Images"

[[rules]]
name = "Documents"
extensions = ["pdf", "docx", "txt"]
patterns = ["Invoice_*", "Report_*"]
destination = "Work/Documents"
```

## 🧠 Technical Core

PANOS is built with a focus on **efficiency** and **industrial-grade error handling**:

- **Pre-compiled Regex/Globs**: Patterns are compiled once at startup, ensuring that millions of files can be matched without re-parsing overhead.
- **Atomic Operations**: Moves are designed to be as atomic as possible, with robust error recovery if a file is "In Use" or "Permission Denied".
- **Recursive State-Awareness**: The scanner understands directory depth and ensures that empty branch nodes are pruned only after their children have been processed.

---

## 📂 Project Structure

The project follows a modular Rust architecture for maintainability and scalability:

- **`src/cli/`**: Command-line argument parsing and help template logic using `clap`.
- **`src/config/`**: Configuration management, including loading and parsing `panos.toml`.
- **`src/file_ops/`**: Core filesystem operations (move, delete, dry-run safety).
- **`src/organizer/`**: The "Brain" – high-level scanning logic and orchestration.
- **`src/rules/`**: Intelligent matching engine for file extensions and patterns.
- **`src/lib.rs`**: Library entry point exposing core functionality.
- **`src/main.rs`**: CLI binary entry point.

---

## 🏗 Development Guidelines

### Core Principles
- **Safety**: Leverage Rust's ownership model to ensure thread-safety and avoid race conditions.
- **Modularity**: Logic is strictly separated: `file_ops` handles "how", `rules` handles "what".
- **Efficiency**: Uses `WalkDir` for fast recursive directory traversal.

### Contribution Workflow
1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/amazing-feature`.
3. Standardize code style with `cargo fmt`.
4. Ensure all tests pass: `cargo test`.
5. Submit a Pull Request with a clear description of changes.

---

## 🗺 Roadmap

- [x] **Phase 1: Core Sorter**: Extension-based sorting & Conflict resolution.
- [x] **Phase 2: Advanced Logic**: Glob pattern matching & Temp file cleanup.
- [ ] **Phase 3: Automation**: Background `watch` mode using `notify` and `Undo` session logging.
- [ ] **Phase 4: Concurrency**: Integration with `Rayon` for ultra-large volume processing.

---

## 📝 License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  <i>PANOS - Organizing the chaos, one file at a time.</i>
</p>
