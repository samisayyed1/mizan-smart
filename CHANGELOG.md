# Changelog

All notable changes to Mizan desktop ship from this file. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/spec/v2.0.0.html).

## [3.3.4] — 2026-05-06

### Removed

- Diagnostic amber marker bar and `[mizan-connect]` console log on the Mizan
  Connect settings page. The fix from v3.3.3 is verified against the live
  backend — sign-in, broker connection (Alpaca Paper), and signed-in /
  signed-out transitions all work end-to-end. This release is the clean handoff
  cut for external sharing.

## [3.3.3] — 2026-05-06

### Fixed

- **Settings sidebar navigation hardened.** The sidebar's `NavLink to="connect"`
  is now resolved to the absolute `/settings/connect` rather than relying on
  React Router's relative-path resolver. The route tree contains TWO routes
  named `connect` (one at AppLayout for `/connect`, one at SettingsLayout for
  `/settings/connect`). Static analysis confirms relative resolution is correct
  in isolation, but the absolute path is defensive against any runtime edge
  case.

### Diagnostic

- ConnectSettingsPage now logs to the WebView console on mount and renders a
  small amber marker bar at the top showing the live `pathname`, `isEnabled`,
  and `isConnected` values. Right-click → Inspect → Console to read the log
  line. The marker is removed in the next release once routing is verified
  healthy in production.

## [3.3.2] — 2026-05-06

### Fixed

- **Mizan Connect navigation still broken in v3.3.1 builds.** The release
  workflow had two Vite invocations — an explicit "Build frontend" step followed
  by tauri-action's own beforeBuildCommand. Only the latter received the
  `CONNECT_*` env vars. When Tauri's `generate_context!` macro caches against
  the dist/ produced by the first invocation, the second build's correctly-baked
  bundle gets shadowed. The fix is to set the env block on both steps; the chain
  is now idempotent.

### Changed

- Tauri `devtools` feature is now enabled in production builds. Right- click in
  the app and choose **Inspect** to open the WebView devtools console. This is a
  temporary diagnostic aid for the founding-member period and will be disabled
  before live-broker launch.

## [3.3.1] — 2026-05-06

### Fixed

- **Mizan Connect navigation broken in production builds.** The 3.3.0 installer
  shipped with `CONNECT_AUTH_URL` and `CONNECT_AUTH_PUBLISHABLE_KEY` empty at
  build time, which set `CONNECT_ENABLED = false` and gated the Mizan Connect
  provider / routes into a "disabled" placeholder. Clicking the sidebar entry
  appeared to do nothing — the route resolved, but the page rendered a silent
  disabled state instead of the sign-in form. Build-time variables are now
  configured on the repo and bake into every release.

## [3.3.0] — 2026-05-05

Initial public release of the Mizan desktop app under the Mizan brand, forking
from the upstream lineage. Includes the Mizan Connect cloud integration
(Supabase auth, broker sync via SnapTrade), the Mizan Compass instrument, the
editorial landing site, and the cross-platform release pipeline (macOS Apple
Silicon, macOS Intel, Windows x64, Linux AppImage).
