# Standard Terminal Graphics (STG)

A high-performance Rust library for terminal graphics with complete support for colors, animations, advanced input handling, and multi-platform packaging.

## 🚀 Key Features

### Core Graphics
- **Optimized FrameBuffers** with styled character support
- **Smart rendering** with dirty regions and optimized cache
- **Smooth animations** at 60 FPS with performance control
- **Complete color system** RGB and predefined palettes
- **Advanced input handling** (keyboard, mouse, resize)

### UI Components
- **Window Manager** with drag & drop, resize, Z-ordering
- **Modular widget system** and extensible
- **Robust event handling** and non-blocking
- **Flexible layout engine** and responsive

### Media Support
- **Image viewer** with automatic Braille conversion
- **Video player** with animated patterns and controls
- **ASCII art rendering** optimized for terminal

### Cross-Platform Packaging
- **Debian packages** (.deb) with `cargo-deb`
- **Alpine Linux packages** (.apk) with automatic target detection
- **Cross-compilation** for multiple architectures (x86, ARM, etc.)
- **Automatic installer/uninstaller** with configuration management

## 📦 Installation

### From Cargo (Rust)
```bash
cargo add standard_terminal_graphics
```

### Debian Package (.deb)
```bash
# Install cargo-deb if not present
cargo install cargo-deb

# Compile and create package
cargo deb

# Install the package
sudo dpkg -i target/debian/stg_*.deb
```

### Alpine Linux Package (.apk)
```bash
# On Alpine Linux
sudo apk add rust cargo musl-dev

# Use automatic installer
./install-alpine.sh

# Or compile manually
cargo build --target x86_64-unknown-linux-musl --release
```

### Build from Source
```bash
git clone https://github.com/yourrepo/standard_terminal_graphics
cd standard_terminal_graphics
cargo build --release
```

## 🎮 Desktop Environment Demo

The main demo shows a complete desktop environment:

```bash
# Run the demo
cargo run --bin stg-demo

# Or if installed via package
stg-demo
```

### Demo Controls
- **F1**: New terminal
- **F2**: New text editor
- **Mouse**: Click and drag to move/resize windows
- **Window corners**: Drag to resize
- **Window borders**: Drag to move
- **Window buttons**: Close (✕) and Minimize (─)

### Available Windows
- **Terminal**: Working terminal with commands (ls, pwd, date, help, clear, exit)
- **Image Viewer**: Viewer with animated Braille graphics
- **Video Player**: Video simulator with dynamic patterns
- **File Manager**: Simplified file browser
- **Text Editor**: Basic text editor

## 🔧 Testing and Quality

### Advanced Test System
```bash
# Complete test
./test-package.sh

# Test with options
./test-package.sh --skip-clippy           # Skip linting
./test-package.sh --skip-format           # Skip formatting  
./test-package.sh --allow-warnings        # Allow warnings
./test-package.sh --help                  # Show help
```

### Automated Tests
- ✅ **Multi-target compilation** with conflict resolution
- ✅ **Execution** with safety timeout
- ✅ **Packaging** for Debian and Alpine
- ✅ **Configurable linting** with Clippy
- ✅ **Formatting** with rustfmt
- ✅ **Cross-compilation** for Alpine architectures
- ✅ **Installer/uninstaller scripts** with syntax testing

## 🏗️ Architecture

```
standard_terminal_graphics/
├── src/
│   ├── lib.rs              # Core library and FrameBuffer
│   ├── input.rs            # Input management (keyboard/mouse)
│   ├── renderer.rs         # Smart rendering engine
│   ├── animation.rs        # Animation system
│   └── ui.rs              # Widgets and UI components
├── examples/
│   └── demo.rs            # Desktop environment demo
├── tests/                 # Complete test suite
├── docs/                  # API documentation
├── packaging/
│   ├── APKBUILD           # Alpine Linux package config
│   ├── stg.post-install   # Alpine post-install script
│   ├── stg.pre-deinstall  # Alpine pre-uninstall script
│   └── install-alpine.sh  # Alpine automatic installer
└── scripts/
    └── test-package.sh    # Advanced test system
```

## 📚 Usage Examples

### Basic FrameBuffer
```rust
use standard_terminal_graphics::*;

let mut fb = FrameBuffer::new(80, 24);
fb.set(10, 5, 'X');
fb.draw_line(0, 0, 20, 10, '#');
println!("{}", fb.to_string());
```

### Styled Graphics
```rust
let mut sfb = StyledFrameBuffer::new(80, 24);
let styled_char = StyledChar::new('█')
    .with_fg(Color::Red)
    .with_bg(Color::Blue);
sfb.set(10, 10, styled_char);
```

### Input Handling
```rust
use standard_terminal_graphics::input::{InputManager, InputEvent};

let mut input = InputManager::new()?;
while let Some(event) = input.poll_event(Duration::from_millis(16))? {
    match event {
        InputEvent::Key(key) => println!("Key: {:?}", key),
        InputEvent::Mouse { x, y, kind } => println!("Mouse: {},{} {:?}", x, y, kind),
        InputEvent::Quit => break,
        _ => {}
    }
}
```

### Smart Rendering
```rust
let mut renderer = SmartRenderer::new()?;
let fb = StyledFrameBuffer::new(80, 24);

// Mark only changed regions
renderer.mark_dirty(Rect::new(10, 10, 20, 5));
renderer.render(&fb)?;
```

### Animations
```rust
let mut anim_manager = AnimationManager::new();
let timer = FrameTimer::new(60); // 60 FPS

loop {
    // Update logic
    anim_manager.update(timer.delta_time());
    
    // Render
    renderer.render(&fb)?;
    timer.wait_for_next_frame();
}
```

## 🎯 Supported Targets

### Architectures
- **x86_64** (Intel/AMD 64-bit)
- **x86** (Intel/AMD 32-bit) 
- **aarch64** (ARM 64-bit)
- **armv7** (ARM 32-bit)

### Operating Systems
- **Linux** (Ubuntu, Debian, Alpine, Arch, etc.)
- **macOS** (Intel and Apple Silicon)
- **Windows** (via WSL or native)

### Package Formats
- **Debian/Ubuntu**: `.deb` packages
- **Alpine Linux**: `.apk` packages
- **Cargo**: Rust package manager
- **Source**: Direct compilation

## ⚡ Performance

### Optimizations
- **Dirty Region Tracking**: Rendering only modified areas
- **Buffer Pooling**: Memory reuse to reduce allocations
- **SIMD Operations**: Vector operations where possible
- **Parallel Rendering**: Multi-threaded rendering for complex scenes
- **Cache-Friendly Layout**: Data structures optimized for cache

### Benchmarks
- **Stable 60 FPS** on multiple windows (tested on average hardware)
- **<1ms latency** for mouse/keyboard input
- **<5MB RAM** for typical applications
- **Cross-compilation** in <30 seconds

## 🔌 Dependencies

### Runtime
- `crossterm` - Cross-platform terminal input/output
- `image` - Image processing and Braille conversion
- `rayon` - Parallelization of intensive operations

### Development
- `cargo-deb` - Debian package creation
- `rustfmt` - Code formatting
- `clippy` - Advanced linting

## 🚀 Roadmap

### v1.1 (Next Release)
- [ ] Expanded widget library (button, slider, progress bar)
- [ ] Basic audio support (beep, tones)
- [ ] Plugin system for extensions
- [ ] Complete API documentation

### v1.2 (Future)
- [ ] Network rendering (client/server)
- [ ] 3D ASCII rendering engine
- [ ] Game engine components
- [ ] Package manager integration (brew, chocolatey)

### v2.0 (Long-term)
- [ ] GPU acceleration where possible
- [ ] WebAssembly target
- [ ] Mobile support (termux)
- [ ] Cloud deployment tools

## 🤝 Contributing

Contributions are welcome! Please:

1. **Fork** the repository
2. **Create** a branch for your feature (`git checkout -b feature/amazing-feature`)
3. **Test** your changes (`./test-package.sh`)
4. **Commit** your changes (`git commit -m 'Add amazing feature'`)
5. **Push** to the branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

### Guidelines
- Follow standard Rust style (`cargo fmt`)
- Add tests for new functionality
- Update documentation
- Maintain API compatibility where possible

## 📄 License

This project is released under the **MIT** license. See the [LICENSE](LICENSE) file for details.

## 🏆 Acknowledgments

- **crossterm** team for excellent terminal support
- **Rust** community for tools and ecosystem
- **ASCII art** projects for graphics inspiration
- **Alpine Linux** team for packaging support

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourrepo/stg/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourrepo/stg/discussions)
- **Documentation**: [docs.rs](https://docs.rs/standard_terminal_graphics)
- **Examples**: `examples/` folder in the repository

---

*Made with ❤️ for the terminal graphics community*
