# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cyrano is a macOS speech-to-text desktop application built with Tauri. It provides local transcription powered by OpenAI's Whisper model with Metal GPU acceleration. Users can record audio via global keyboard shortcuts and have transcriptions automatically inserted into other applications or copied to the clipboard.

## Development Commands

```bash
# Run the app in development mode
npm run tauri:dev

# Run frontend only (no Tauri)
npm run dev

# Build for production
npm run tauri:build

# Run all checks (REQUIRED before completing any task)
npm run check:all

# Auto-fix what can be fixed
npm run fix:all

# Run tests
npm run test:run          # Frontend (Vitest)
npm run rust:test         # Backend (Cargo)

# Regenerate TypeScript bindings from Rust
npm run rust:bindings
```

## Architecture

### Stack

- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS, Radix UI, Zustand, TanStack Query
- **Backend**: Rust, Tauri 2, cpal (audio), whisper-rs (transcription with Metal)
- **Type Safety**: specta/tauri-specta generates TypeScript bindings from Rust commands

### Directory Structure

```
src/                          # Frontend (React/TypeScript)
├── components/
│   ├── ui/                   # Radix UI + shadcn-style components
│   ├── layout/               # Main window layout
│   ├── preferences/          # Settings dialog and panes
│   ├── quick-pane/           # Global shortcut popup
│   └── recording-overlay/    # Recording state UI
├── lib/
│   ├── tauri-bindings.ts     # Auto-generated from Rust (don't edit manually)
│   └── commands/             # Frontend command system
├── hooks/                    # Custom React hooks
├── store/                    # Zustand stores
└── i18n/                     # i18next config (en, fr, ar)

src-tauri/                    # Backend (Rust)
├── src/
│   ├── lib.rs                # App entry, plugin setup
│   ├── commands/             # Tauri command handlers (exposed to frontend)
│   ├── services/             # Business logic (recording, transcription, shortcuts)
│   ├── infrastructure/       # External adapters (whisper, cpal, macOS accessibility)
│   ├── domain/               # Error types, state management
│   └── traits/               # Abstractions (AudioCapture, Transcriber)
└── Cargo.toml
```

### Key Patterns

**Tauri Commands**: Backend functions in `src-tauri/src/commands/` are exposed to the frontend via specta. After modifying Rust commands, run `npm run rust:bindings` to regenerate `src/lib/tauri-bindings.ts`.

**Multiple Windows**: The app has three windows:

- `main` - Primary application window
- `quick-pane` - Global shortcut popup (entry: `src/quick-pane-main.tsx`)
- `recording-overlay` - Recording state indicator

**Global Shortcuts**: Managed via `tauri-plugin-global-shortcut`. Registration happens in `lib.rs` setup, with shortcut handlers in `services/shortcut_service.rs`.

**Accessibility APIs**: macOS accessibility (for cursor text insertion) is handled in `infrastructure/permissions/macos_accessibility.rs`.

## Task Completion Verification

**Before declaring any task complete**, run the full verification suite:

```bash
npm run check:all
```

This runs:

- TypeScript type checking (`tsc --noEmit`)
- ESLint with zero warnings tolerance
- ast-grep scanning
- Prettier formatting check
- Rust formatting check (`cargo fmt --check`)
- Rust clippy linting (`cargo clippy -- -D warnings`)
- Frontend tests (Vitest)
- Backend tests (Cargo)

### Rules

1. **Never skip verification** - Even for "small" changes
2. **Fix all errors** - All checks must pass before marking complete
3. **No warnings** - ESLint uses `--max-warnings 0`
4. **Both stacks matter** - Frontend and backend checks are equally important

If checks fail, run `npm run fix:all` then re-run `npm run check:all`.
