# Cloud Authentication & Control

| Mode | Discovery and status | Commands |
| --- | --- | --- |
| LAN first | LAN and configured cloud services | Prefer LAN; use an available cloud transport when the operation is unavailable locally or LAN fails. |
| LAN only | LAN only | Local operations only. |
| Cloud | Configured cloud services only | Cloud operations only. |

The default is **LAN first**. The same mode applies to physical devices, scenes, segments, DreamView and saved One Click actions. Functions unavailable through the selected transports are not executed through another transport.

## Authentication

- **Govee API Key** authorizes the official Platform API.
- **Govee Account Token** authorizes account services, including saved One Click actions and IoT. A configured token takes priority over email and password.
- Account email and password remain available for password sign-in when no account token is configured.

Cloud credentials are not used in LAN only mode. LAN discovery and UDP commands are not used in Cloud mode.

## Discovery

DreamView centers are identified automatically using account topology and supported device metadata. Saved One Click actions retain their names from Govee Home. No manual center selection or scene-name configuration is required.

Home Assistant discovery is preserved during temporary synchronization failures. Deleted actions are removed after a complete successful synchronization. One Click availability follows the authenticated account connection.

| App option | Environment variable |
| --- | --- |
| `control_mode` | `GOVEE_CONTROL_MODE` |
| `govee_api_key` | `GOVEE_API_KEY` |
| `govee_token` | `GOVEE_ACCOUNT_TOKEN` |
