# mkcert-tauri-tool

Lightweight cross-platform desktop UI for managing `mkcert`-based local development certificates.

## What it does

- Checks whether `mkcert` is installed and whether the local root CA exists.
- Installs and trusts the local root CA through `mkcert -install`.
- Opens the root CA directory for inspection.
- Generates `local.pem` and `local-key.pem` for one or more comma-separated domains.
- Stores the last-used form values in a local config file.
- Can optionally schedule daily certificate generation:
  - Windows: `schtasks`
  - Unix-like systems: `crontab`

## Requirements

- Node.js and npm
- Rust toolchain compatible with the Tauri version in this repo
- `mkcert` installed on the host machine
- Platform-specific build prerequisites for Tauri

You also need the OS certificate trust tooling that `mkcert` relies on.

## Development

Install dependencies:

```bash
npm install
```

Run the app in development mode:

```bash
npm run dev
```

Build a production bundle:

```bash
npm run build
```

## Project Layout

- `src/index.html` - application markup
- `src/main.js` - frontend logic and Tauri command invocations
- `src/style.css` - UI styling
- `src-tauri/src/main.rs` - native entry point and environment setup
- `src-tauri/src/lib.rs` - Tauri app bootstrap and command registration
- `src-tauri/src/commands.rs` - commands exposed to the frontend
- `src-tauri/src/services/mkcert.rs` - `mkcert` integration
- `src-tauri/src/services/scheduler.rs` - daily renewal scheduling
- `src-tauri/src/settings.rs` - config load/save
- `src-tauri/src/utils.rs` - config path helper

## Runtime Flow

1. The frontend loads saved settings from the native backend.
2. The app checks whether `mkcert` and the local root CA are available.
3. If needed, the user can install the root CA or open the CA folder.
4. The user enters domains, chooses a destination folder, and generates certificates.
5. Settings are written to the app config directory after successful generation.

## Configuration Storage

Settings are stored as JSON in the platform app config directory.

Fallback path:

```text
~/.config/mkcert-gui-tool/config.json
```

Saved values:

- `domains`
- `directory`
- `auto_schedule`

## Certificate Output

When no destination directory is chosen, certificates are written to:

```text
~/.local/share/mkcert-certs
```

The generated files are:

- `local.pem`
- `local-key.pem`

## Scheduling Behavior

Enabling automatic renewal updates the system scheduler from the native backend.

- On Windows, the app creates or replaces a task named `MkcertAutoRenewalTaskTauri`.
- On Unix-like systems, the app writes a cron entry marked with `# mkcert-gui-tool`.

The scheduled worker runs the same binary with `--worker` and reloads the saved settings before generating certificates.

## Notes for Development

- The frontend talks to Rust through Tauri `invoke` calls.
- The app expects `mkcert` to be discoverable on `PATH`.
- `src-tauri/src/main.rs` includes Linux-specific environment cleanup for Snap/GTK/WebKit behavior.
- This repo currently uses the `src/` directory as the frontend dist target in `src-tauri/tauri.conf.json`.

## Troubleshooting

- If certificate generation fails, verify `mkcert` is installed and trusted locally.
- If the app cannot find `mkcert`, confirm it is available in your shell `PATH`.
- If scheduling fails on Linux or macOS, confirm `crontab` is available and writable.
- If scheduling fails on Windows, confirm Task Scheduler access is permitted.

## License

MIT License because it is a permissive license that lets others use, copy, modify, merge, publish, distribute, sublicense, and sell the software with only the requirement to keep the license and copyright notice.
