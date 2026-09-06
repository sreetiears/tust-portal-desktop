# PRD: Windows Port for TustPortal

## Problem Statement

TustPortal is a Tauri-based desktop application that automatically logs users into the Tianjin University of Science and Technology (TUST) campus network portal. Currently, the application only supports macOS. Windows users — including students on lab computers, personal laptops, and dorm desktops — cannot use the tool, forcing them to manually log into the captive portal every time their session expires.

The macOS exclusivity is not inherent to the application's architecture (Tauri is cross-platform), but stems from a small set of platform-specific networking and UX assumptions that have no Windows equivalent implemented.

## Solution

Port TustPortal to Windows by:

1. Implementing Windows-specific network discovery (WiFi SSID, local IP) behind the existing platform abstraction.
2. Adapting the tray and window lifecycle to match Windows user expectations (first-launch window visibility, left-click tray behavior).
3. Adding a Windows-only auto-start on login feature so the application can fulfill its purpose as a background utility.
4. Providing a Windows installer (NSIS `.exe` as primary, MSI as secondary) and a dedicated Windows CI/CD pipeline.
5. Creating Windows-specific user and developer documentation.

The macOS version remains untouched except for the necessary `#[cfg]` conditional compilation gates required to coexist with the new Windows modules.

## User Stories

### Network Detection
1. As a Windows student user, I want the app to detect that I am connected to the `TUST` or `CU_TUST` WiFi network, so that it knows when to attempt automatic login.
2. As a Windows student user, I want the app to recognize my `10.x` campus IP address, so that it only attempts login when I am actually on the campus network.
3. As a Windows user on Ethernet or an unknown WiFi, I want the option to ignore SSID detection, so that the app can still auto-login based on IP range alone.
4. As a Windows user with multiple network adapters, I want the app to identify the currently active WiFi interface, so that SSID detection is accurate even if I have virtual or secondary adapters.
5. As a Chinese Windows user, I want SSID detection to work regardless of my system language locale, so that the app functions on both Chinese and English Windows editions.
6. As a Windows user with no WiFi adapter, I want the app to degrade gracefully (show "未检测到WiFi"), so that I understand why auto-login is not triggering.

### Window & Tray Behavior
7. As a Windows user installing the app for the first time, I want the Settings window to appear immediately after launch, so that I can enter my credentials without guessing where the app went.
8. As a Windows user, I want the app to minimize to the system tray after I have saved my credentials, so that it stays out of my way while running in the background.
9. As a Windows user, I want left-clicking the tray icon to open the Settings window directly, so that I can quickly check or change my credentials.
10. As a Windows user, I want right-clicking the tray icon to show a context menu (Trigger Login, Pause, Logs, Settings, Quit), so that I have access to all actions.
11. As a Windows user, I want the tray tooltip to show "天科大校园网自动登录", so that I can identify the app among other tray icons.

### Auto-Login
12. As a Windows user, I want the app to check my network status every 20 seconds, so that it catches captive portal redirects as soon as they happen.
13. As a Windows user, I want the app to automatically submit my saved credentials when a login is needed, so that I don't have to open a browser.
14. As a Windows user, I want the app to retry up to 3 times if the login fails, so that transient network errors don't leave me disconnected.
15. As a Windows user, I want the app to verify Baidu connectivity after login, so that I know whether the login actually succeeded.
16. As a Windows user, I want to pause auto-login from the tray menu, so that I can temporarily prevent automatic authentication when needed.

### Auto-Start on Login
17. As a Windows user, I want the app to ask me on first launch whether it should start automatically when I log in, so that I can choose convenience versus manual control.
18. As a Windows user, I want the default answer to the auto-start prompt to be "Yes", so that most users get the background utility behavior out of the box.
19. As a Windows user, I want to change the auto-start preference later in Settings, so that I am not locked into my initial choice.
20. As a Windows user who disabled auto-start, I want the app to respect my choice across reinstalls (as long as settings persist), so that I don't have to disable it repeatedly.
21. As a Windows user on a restricted lab computer, I want the auto-start mechanism to fail gracefully (log the error, don't crash) if the registry is locked down, so that the app remains usable even if auto-start is impossible.

### Credentials & Settings
22. As a Windows user, I want to save my username, password, and carrier (校园网 / 中国联通) in the app, so that I don't have to re-enter them on every login.
23. As a Windows user, I want my credentials to persist across app restarts, so that the background auto-login loop has the data it needs.
24. As a Windows user, I want to see my current network status (SSID, IPv4, IPv6, campus network yes/no) in the Settings page, so that I can diagnose why auto-login is or isn't triggering.
25. As a Windows user, I want to view real-time logs of the auto-login process, so that I can troubleshoot connection issues.

### Installation & Distribution
26. As a Windows user without administrator rights, I want to install the app using the `.exe` installer without needing an admin password, so that I can use it on lab computers or shared devices.
27. As a Windows user who prefers system-wide installs, I want an `.msi` option, so that I can install it for all users or manage it via standard Windows tools.
28. As a Windows user, I want the app to have a proper Windows icon in the Start Menu and taskbar, so that it looks like a native application.
29. As a developer, I want the Windows build to be produced automatically via GitHub Actions when a version tag is pushed, so that releases are consistent and reproducible.

## Implementation Decisions

### 1. Platform Network Provider (Deep Module)

The application exposes a unified platform-agnostic interface for network discovery:

- `get_wifi_ssid() -> Option<String>`
- `get_local_ipv4() -> Option<String>`
- `get_local_ipv6() -> Option<String>`

The module is split into two platform-specific implementations gated by `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "windows")]`:

- **macOS implementation** (existing): Uses `ipconfig`, `networksetup`, and `ifconfig`.
- **Windows implementation** (new): Uses `netsh wlan show interfaces` for SSID detection and `ipconfig` for IPv6. IPv4 continues to use the cross-platform `local_ip_address` crate.

This is a **deep module** because it encapsulates the full complexity of shell command execution, locale-aware output parsing, adapter enumeration, and error handling behind three simple functions. The rest of the application consumes this interface without knowledge of the underlying OS mechanics.

**Technical clarification:** Empirical testing on Chinese Windows confirmed that the `netsh wlan show interfaces` SSID field key is the literal string `SSID` even on localized systems. The parser uses exact key matching (`key == "SSID"`) to avoid false matches with `AP BSSID`, and performs block-aware scanning to prefer the adapter whose status indicates "connected" (`已连接` / `connected`) when multiple interfaces exist.

The IPv6 parser scans for lines containing "IPv6" and a colon, extracts the value, skips any value starting with `fe80` (link-local), and strips any `%zone` suffix before returning the address.

### 2. Tray Event Router (Shallow Adapter)

The tray construction logic was extended to support platform-specific icon click behavior without duplicating the menu definition.

- **macOS**: Retains current behavior. No custom left-click handler; the existing menu is attached directly. The `ActivationPolicy::Accessory` call is guarded with `#[cfg(target_os = "macos")]`.
- **Windows**: A left-click handler (`on_tray_icon_event`) is registered that opens the Settings window directly. Right-click continues to display the standard menu.

This is intentionally a shallow adapter because the behavior is thin glue between the tray framework and the existing window management functions.

### 3. Auto-Start Controller (Deep Module)

A new module encapsulates all Windows auto-start mechanics behind a minimal interface:

- `is_enabled() -> bool`
- `set_enabled(bool) -> Result<(), String>`

The implementation uses the `reg.exe` command-line tool rather than a native registry API or third-party crate:
- `reg query` to check whether `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TustPortal` exists.
- `reg add` to create the value, using `std::env::current_exe()` to capture the correct executable path.
- `reg delete` to remove the value.

This approach avoids adding a dependency that would be unused on macOS, and keeps the implementation consistent with the existing shell-out style used for network detection.

This is a **deep module** because it hides all command execution, path serialization, error translation, and fallback logic. Callers simply toggle a boolean.

### 4. First Launch Coordinator (Shallow Module)

The application setup phase uses platform-specific first-launch logic via a pure decision function:

- **macOS**: Unchanged. The initial hidden window is destroyed immediately; the app lives in the tray.
- **Windows**: If no saved credentials exist, the initial window is kept visible and shown centered.

The auto-start preference prompt is **not** a native system dialog. Instead, the Vue frontend detects the first-launch condition (no credentials + platform is Windows) and renders an in-app modal overlay asking whether to enable auto-start. This keeps the UX consistent with the Settings window that is already open, and avoids adding a native dialog dependency.

### 5. Platform Metadata Provider (Shallow Module)

A single new Tauri command is exposed to the frontend:

- `get_platform() -> String` (returns `"windows"` or `"macos"`)

The Vue frontend uses this to conditionally render the auto-start toggle in the Settings page. This prevents the toggle from appearing (and being non-functional) on macOS, keeping the shared UI clean across platforms.

### 6. CI/CD Windows Pipeline

A separate GitHub Actions workflow (`build-windows.yml`) was created. It triggers on `v*` tags, parallel to the existing `build-macos.yml`. The workflow:

1. Checks out the repository.
2. Sets up Node.js, pnpm, and Rust on `windows-latest`.
3. Runs `pnpm tauri build`.
4. Uploads both NSIS `.exe` and `.msi` artifacts to the GitHub Release.

**Decision:** Separate workflows rather than a unified matrix. This isolates the macOS release pipeline from any Windows-specific build instability and respects the project constraint of not modifying existing macOS infrastructure.

### 7. User-Agent String

The hardcoded User-Agent in the login HTTP request was changed from a macOS-specific string to a generic Chrome User-Agent (`Windows NT 10.0; Win64; x64`), because the portal is not known to gate requests by OS but the previous string misrepresented Windows traffic.

### 8. Credential Storage

Credentials continue to be stored as plaintext JSON in the application data directory. No encryption layer was introduced as part of this port. The Windows file path is `%AppData%\com.tust.portal\credentials.json`.

## Testing Decisions

### What Makes a Good Test

Tests verify **external behavior and contracts**, not internal implementation details. For example:
- A good test for the Platform Network Provider asserts that given a specific mocked `netsh` output, `get_wifi_ssid()` returns the expected SSID string.
- A bad test would assert the exact regex pattern used internally to parse that output.

### Modules Tested

1. **Platform Network Provider (Windows)** — 5 tests.
   - Extracts SSID from Chinese Windows `netsh` output.
   - Returns `None` when no SSID line is present.
   - Returns the connected adapter's SSID when multiple adapters exist (one disconnected with stale SSID, one connected).
   - Extracts global IPv6 from `ipconfig` while excluding `fe80::` link-local.
   - Returns `None` when only link-local IPv6 is present.
   - Prior art: None in the current codebase. This is a new testing surface.

2. **Auto-Start Controller** — 3 tests, mutex-guarded to prevent parallel registry collisions.
   - `is_enabled()` returns `false` when the registry key is absent.
   - `set_enabled(true)` creates the registry value; `is_enabled()` subsequently returns `true`.
   - `set_enabled(false)` removes the registry value; `is_enabled()` subsequently returns `false`.
   - Prior art: None. This is a new module.

3. **First Launch Coordinator** — 4 pure decision tests.
   - Windows + no credentials → show window.
   - Windows + credentials exist → hide window.
   - macOS + no credentials → hide window (preserves current behavior).
   - macOS + credentials exist → hide window.
   - Prior art: None. This logic was previously implicit in `lib.rs` setup.

### Total Test Count
12 unit tests across 3 modules.

### Modules Not Tested

- **Tray Event Router**: Thin glue calling existing Tauri window APIs. Testing would require mocking the entire Tauri runtime, which is not cost-effective.
- **CI/CD Pipeline**: Verified by the build itself succeeding and producing artifacts.
- **Vue Frontend Modal**: UI state transitions are simple enough to verify manually.

## Out of Scope

The following items are explicitly excluded:

1. **macOS behavior changes**: No modifications to macOS auto-start, tray behavior, window lifecycle, or CI pipeline.
2. **Credential encryption**: Passwords remain stored in plaintext JSON.
3. **macOS README modifications**: The existing `README.md` remains as-is. Windows documentation lives in `README_WD.md`.
4. **Linux port**: No Linux-specific modules or distribution targets.
5. **Microsoft Store (MSIX) distribution**: Only NSIS `.exe` and `.msi` are targeted.
6. **Ethernet-based campus detection without Ignore SSID**: The strict WiFi-bound rule remains. Users on Ethernet must enable "Ignore SSID" in Settings.
7. **Portal protocol changes**: The login URL, parameters, and User-Agent behavior (other than generic OS string) remain unchanged.
8. **System service / daemon mode**: Auto-start is per-user registry only, not a Windows Service.
9. **IPv6 campus portal support**: IPv6 detection is informational only; the portal interaction continues to be IPv4-driven.
10. **Auto-start on macOS**: Not implemented, per project constraints.
11. **Native first-run dialog**: The auto-start prompt is rendered in Vue, not as a native Win32 message box.

## Further Notes

- **Chinese Windows Compatibility:** The target user base primarily runs Simplified Chinese Windows 10/11. All `netsh` output parsing was empirically validated against a live Chinese Windows machine. The SSID field key is the literal ASCII string `SSID` even on localized systems.
- **Lab Computer Scenario:** A significant use case is shared lab computers where users lack admin rights. The NSIS per-user installer and registry-based auto-start (`HKCU`) are designed specifically for this environment.
- **Tauri Version:** The project uses Tauri 2. All new APIs are v2-compatible.
- **Future macOS Enhancement:** While out of scope, the module structure (deep platform providers, shallow coordinators) is designed so that an auto-start feature for macOS could be added later by implementing a macOS variant of the Auto-Start Controller without touching the rest of the codebase.