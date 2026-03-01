export PATH := env('HOME') + "/.cargo/bin:" + env('PATH')

# Build release binary
build:
    cargo build --release

# Run with a markdown file
run FILE:
    cargo run --release -- {{FILE}}

# Install binary + desktop entry, register MIME type
install: build
    cp target/release/cosmic-md ~/.local/bin/
    cp res/com.cosmic.md-viewer.desktop ~/.local/share/applications/
    update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
    xdg-mime default com.cosmic.md-viewer.desktop text/markdown
    xdg-mime default com.cosmic.md-viewer.desktop text/x-markdown
    @echo "Installed cosmic-md"

# Uninstall
uninstall:
    rm -f ~/.local/bin/cosmic-md
    rm -f ~/.local/share/applications/com.cosmic.md-viewer.desktop
    update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
    @echo "Uninstalled cosmic-md"

# Clean build artifacts
clean:
    cargo clean
