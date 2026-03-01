# cosmic-md

A fast, minimal markdown viewer for [COSMIC](https://github.com/pop-os/cosmic-epoch) desktop.

Opens a `.md` file and renders it. That's it.

- Native COSMIC theming (dark/light follows system)
- Syntax-highlighted code blocks
- Clickable links (opens in default browser)
- ~100 lines of Rust

## Install

### From .deb (Pop!_OS / Ubuntu)

Download the latest `.deb` from [Releases](https://github.com/peterbenoit/cosmic-md/releases) and install:

```bash
sudo dpkg -i cosmic-md_*.deb
```

### From source

Requires Rust 1.90+ and system dependencies:

```bash
sudo apt install just libexpat1-dev libfontconfig-dev libfreetype-dev libinput-dev libxkbcommon-dev libwayland-dev pkg-config cmake
```

Then build and install:

```bash
just install
```

## Usage

```bash
cosmic-md README.md
```

Or double-click any `.md` file in COSMIC Files after installing.

## Building

```bash
just build        # release build
just run FILE.md  # build and run with a file
just clean        # clean build artifacts
```

## License

MIT
