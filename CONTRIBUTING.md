# Contributing to PANOS

Thank you for your interest in contributing to PANOS! :rocket:

We welcome contributions of all kinds - bug fixes, new features, documentation improvements, and bug reports. This document will help you get started.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style & Standards](#code-style--standards)
- [Project Structure](#project-structure)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)
- [Bug Reports](#bug-reports)
- [Feature Requests](#feature-requests)
- [Documentation](#documentation)
- [Community](#community)

---

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For version control
- **GitHub Account** - For pull requests and issues

### Quick Start

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/panos.git
cd panos

# 3. Add upstream remote
git remote add upstream https://github.com/Nonbangkok/panos.git

# 4. Install dependencies and build
cargo build
cargo test
```

---

## Development Setup

### Environment Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Install useful tools
rustup component add clippy rustfmt
cargo install cargo-tarpaulin  # For test coverage
cargo install cargo-watch      # For auto-reloading during development
```

### IDE Configuration

**VS Code Extensions (Recommended):**
- `rust-analyzer` - Rust language server
- `CodeLLDB` - Debugging support
- `Better TOML` - TOML file editing

**Configuration (`.vscode/settings.json`):**
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "editor.formatOnSave": true,
    "files.exclude": {
        "**/target": true
    }
}
```

---

## Development Workflow

### 1. Create a Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Changes

- Write code following our [style guidelines](#code-style--standards)
- Add tests for new functionality
- Update documentation
- Ensure all tests pass

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Build in release mode
cargo build --release
```

### 4. Commit Changes

```bash
# Stage changes
git add .

# Commit with conventional message
git commit -m "feat: add pattern matching for file names"
# or
git commit -m "fix: resolve panic when handling empty directories"
```

### 5. Submit Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
# Link to any relevant issues
# Request review from maintainers
```

---

## Code Style & Standards

### Rust Guidelines

- Use `cargo fmt` for automatic formatting
- Use `cargo clippy` for linting (no warnings allowed)
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable and function names
- Add doc comments for all public APIs

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(rules): add glob pattern support for file matching
fix(file_ops): handle permission denied errors gracefully
docs(readme): update installation instructions
```

### Code Organization

```rust
//! Module documentation
//! 
//! Brief description of what this module does.

use anyhow::Result;
use std::path::Path;
use tracing::{debug, info};

/// Function documentation
/// 
/// # Arguments
/// 
/// * `path` - The path to process
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, `Err` on failure
/// 
/// # Examples
/// 
/// ```
/// use panos::process_file;
/// 
/// let result = process_file(&Path::new("test.txt"))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn process_file(path: &Path) -> Result<()> {
    info!("Processing file: {:?}", path);
    // Implementation here
    Ok(())
}
```

---

## Project Structure

```
panos/
|
+--- src/
|    |
|    +--- cli/           # Command line interface
|    |    +--- mod.rs
|    |    +--- args.rs
|    |
|    +--- config/        # Configuration management
|    |    +--- mod.rs
|    |    +--- loader.rs
|    |
|    +--- file_ops/      # File operations
|    |    +--- mod.rs
|    |    +--- mover.rs
|    |    +--- remover.rs
|    |
|    +--- organizer/     # Core organization logic
|    |    +--- mod.rs
|    |    +--- scanner.rs
|    |
|    +--- rules/         # Pattern matching engine
|    |    +--- mod.rs
|    |    +--- matcher.rs
|    |
|    +--- lib.rs         # Library entry point
|    +--- main.rs        # CLI binary entry
|
+--- tests/               # Integration tests
|    +--- integration_tests.rs
|
+--- benches/            # Performance benchmarks
|
+--- docs/               # Additional documentation
|
+--- assets/             # Static assets (logo, etc.)
|
+--- panos.toml          # Example configuration
+--- README.md
+--- CONTRIBUTING.md
+--- LICENSE
+--- Cargo.toml
```

### Module Responsibilities

- **`cli/`**: Command line argument parsing and help generation
- **`config/`**: Loading, parsing, and validating configuration files
- **`file_ops/`**: File system operations (move, delete, create directories)
- **`organizer/`**: High-level orchestration and scanning logic
- **`rules/`**: Pattern matching and rule evaluation engine

---

## Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test_input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_error_handling() {
        // Test error cases
        assert!(function_that_can_fail().is_err());
    }
}
```

### Test Categories

1. **Unit Tests** - Test individual functions in isolation
2. **Integration Tests** - Test module interactions
3. **Property Tests** - Test invariants and edge cases
4. **Performance Tests** - Benchmark critical paths

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_organize_files

# Run with output
cargo test -- --nocapture

# Run in release mode (faster)
cargo test --release

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

### Test Data

Use `tempfile` crate for temporary test directories:

```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_file_organization() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source");
    let dest = temp_dir.path().join("dest");
    
    fs::create_dir_all(&source).unwrap();
    
    // Create test files
    fs::write(source.join("test.txt"), "content").unwrap();
    
    // Test organization
    organize_files(&source, &dest).unwrap();
    
    // Verify results
    assert!(dest.join("test.txt").exists());
}
```

---

## Submitting Changes

### Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code follows style guidelines (`cargo fmt`, `cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] PR description is clear and concise
- [ ] Related issues are linked
- [ ] Breaking changes are documented

### Pull Request Template

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated

## Related Issues
Closes #123
```

### Review Process

1. **Automated Checks** - CI runs tests, formatting, and linting
2. **Peer Review** - At least one maintainer reviews the PR
3. **Discussion** - Address feedback and make requested changes
4. **Approval** - PR is approved and merged

---

## Bug Reports

### Before Reporting

1. Check existing issues for duplicates
2. Verify you're using the latest version
3. Try to reproduce in a minimal environment

### Bug Report Template

```markdown
**Bug Description**
Clear and concise description of the bug.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment**
- OS: [e.g. macOS 13.0]
- Rust version: [e.g. 1.70.0]
- PANOS version: [e.g. 0.1.0]

**Additional Context**
Add any other context about the problem here.

**Logs**
Include relevant log output if available.
```

---

## Feature Requests

### Before Requesting

1. Check if feature already exists
2. Search existing feature requests
3. Consider if it fits the project scope

### Feature Request Template

```markdown
**Feature Description**
Clear and concise description of the feature.

**Problem Statement**
What problem does this feature solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
What other approaches did you consider?

**Additional Context**
Add any other context about the feature request.
```

---

## Documentation

### Types of Documentation

1. **Code Documentation** - Doc comments for public APIs
2. **User Documentation** - README, examples, tutorials
3. **Developer Documentation** - Architecture docs, design decisions
4. **API Documentation** - Generated from doc comments

### Documentation Guidelines

- Use clear, simple language
- Include examples for complex functionality
- Keep documentation up to date with code changes
- Use consistent formatting and style

### Example Documentation

```rust
/// Organizes files according to configuration rules.
/// 
/// This function scans the source directory and moves files to their
/// appropriate destinations based on the provided configuration.
/// 
/// # Arguments
/// 
/// * `config` - Configuration containing rules and settings
/// * `dry_run` - If true, only shows what would be done without moving files
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or `Err` if organization fails.
/// 
/// # Examples
/// 
/// ```rust
/// use panos::{Config, organize};
/// 
/// let config = Config::default();
/// organize(&config, true)?; // Dry run
/// # Ok::<(), anyhow::Error>(())
/// ```
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - The source directory doesn't exist
/// - Permission is denied when accessing files
/// - A file operation fails
pub fn organize(config: &Config, dry_run: bool) -> Result<()> {
    // Implementation
}
```

---

## Community

### Getting Help

- **GitHub Issues** - For bug reports and feature requests
- **GitHub Discussions** - For questions and general discussion
- **Discord/Slack** - (Coming soon) Real-time chat

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and considerate
- Use inclusive language
- Focus on constructive feedback
- Help others learn and grow

### Recognition

Contributors are recognized in:
- README.md contributors section
- Release notes for significant contributions
- Annual contributor highlights

---

## License

By contributing to PANOS, you agree that your contributions will be licensed under the [GPL-3.0 License](LICENSE).

---

## Getting Help

If you need help with contributing:

1. Check existing issues and discussions
2. Read the code and documentation
3. Ask questions in GitHub Discussions
4. Start a draft PR to get early feedback

---

Thank you for contributing to PANOS! :heart:

Your contributions help make file organization better for everyone.
