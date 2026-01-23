# Cyrano

A macOS speech-to-text desktop application with local transcription powered by OpenAI's Whisper model and Metal GPU acceleration.

## Features

- **Local Transcription** - Privacy-first speech recognition using Whisper (large-v3-turbo model) running entirely on your Mac
- **Metal GPU Acceleration** - Fast transcription leveraging Apple Silicon
- **Global Shortcuts** - Quick access from any application (default: `Cmd+Shift+.`)
- **Dual Output** - Automatic clipboard copy and cursor text insertion
- **Multi-language UI** - English, French, and Arabic with RTL support
- **Modern Interface** - Clean design with light/dark theme support

## Requirements

- macOS 13.0 or later
- Metal-compatible GPU (Apple Silicon or supported Intel Mac)

## Installation

Download the latest release from the [Releases](https://github.com/nvergez/cyrano/releases) page.

On first launch, grant the following permissions when prompted:

- **Microphone** - Required for audio recording
- **Accessibility** - Optional, enables text insertion at cursor position

## Usage

1. Press `Cmd+Shift+.` (or your configured shortcut) to start recording
2. Speak your text
3. Press the shortcut again to stop recording
4. Transcribed text is automatically copied to clipboard and inserted at cursor

### Settings

Access preferences via the app menu or `Cmd+,` to configure:

- Global keyboard shortcut
- Theme (Light / Dark / System)
- UI language

## Development

### Prerequisites

- Node.js 18+
- Rust 1.82+
- Xcode Command Line Tools

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Run frontend only
npm run dev
```

### Commands

```bash
npm run tauri:build      # Build for production
npm run check:all        # Run all verification checks
npm run fix:all          # Auto-fix linting/formatting
npm run test:run         # Frontend tests
npm run rust:test        # Backend tests
npm run rust:bindings    # Regenerate TypeScript bindings from Rust
```

### Project Structure

```
src/                     # React/TypeScript frontend
├── components/          # UI components (Radix UI + Tailwind)
├── hooks/               # Custom React hooks
├── store/               # Zustand state management
└── lib/                 # Utilities and Tauri bindings

src-tauri/               # Rust backend
├── src/
│   ├── commands/        # Tauri command handlers
│   ├── services/        # Business logic
│   ├── infrastructure/  # External adapters (Whisper, audio)
│   └── domain/          # Error types, state
└── Cargo.toml
```

### Tech Stack

**Frontend:** React 19, TypeScript, Vite, Tailwind CSS, Radix UI, Zustand

**Backend:** Rust, Tauri 2, whisper-rs (Metal), cpal

**Type Safety:** specta/tauri-specta for auto-generated TypeScript bindings

## License

[MIT](LICENSE) - Nicolas Vergez
