# Context: TustPortal

## Glossary

### Campus Network
A network environment where the device is connected to the Tianjin University of Science and Technology (TUST) campus infrastructure. Synonymous with "校园网环境" in the UI.

**Detection criteria:** A device is considered on the Campus Network **only if** both of the following are true:
1. It is connected to a WiFi network whose SSID starts with `TUST` or `CU_TUST` (case-insensitive).
2. Its local IPv4 address starts with `10.`.

If either condition is false, the device is **not** on the Campus Network, and the auto-login background loop will skip intervention unless the user has enabled the **Ignore SSID** setting.

### Ignore SSID
A user-configurable setting that removes the SSID condition from Campus Network detection. When enabled, the app considers any `10.x` IPv4 network as the Campus Network, regardless of WiFi SSID or wired connection type.

### Auto-Login
The background behavior that detects a login-captive state on the Campus Network and automatically submits stored credentials to the campus portal (`10.10.102.50:801`). It operates on a 20-second polling interval.

### Portal
The campus network authentication endpoint at `http://10.10.102.50:801/eportal/portal/login`.

### Auto-Start
A Windows-only feature that registers the application to launch automatically when the user logs into Windows. It is controlled by a user preference that defaults to enabled and can be changed in Settings. The macOS version does not implement Auto-Start.

### First Launch
The initial execution of the application on a device where no credentials have been saved. On Windows, the First Launch experience includes showing the Settings window immediately and prompting the user for the Auto-Start preference.
