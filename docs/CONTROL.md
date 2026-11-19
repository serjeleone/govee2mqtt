# Cloud Authentication & Control

| Mode | Discovery and status | Commands |
| --- | --- | --- |
| LAN first | LAN and configured cloud services | Prefer LAN; use an available cloud transport when the operation is unavailable locally or LAN fails. |
| LAN only | LAN only | Local operations only. |
| Cloud | Configured cloud services only | Cloud operations only. |

The default is **LAN first**. The same mode applies to physical devices, scenes, segments, DreamView and saved One Click actions. Functions unavailable through the selected transports are not executed through another transport.

## Authentication

- **Govee API Key** authorizes the official Platform API.
- Account email and password authorize account services, including saved One Click actions and IoT. Verification codes remain supported when Govee requests them.

Cloud credentials are not used in LAN only mode. LAN discovery and UDP commands are not used in Cloud mode.

## Discovery

DreamView centers are identified automatically using account topology and supported device metadata. Saved One Click actions retain their names from Govee Home. No manual center selection or scene-name configuration is required.

Home Assistant discovery is preserved during temporary synchronization failures. Deleted actions are removed after a complete successful synchronization. One Click availability follows the authenticated account connection.

| App option | Environment variable |
| --- | --- |
| `control_mode` | `GOVEE_CONTROL_MODE` |
| `govee_api_key` | `GOVEE_API_KEY` |
| `govee_email` / `govee_password` | `GOVEE_EMAIL` / `GOVEE_PASSWORD` |


## DreamView and ScenicView commands

The Web UI provides separate **On** and **Off** command buttons for video DreamView and virtual groups/scenes. A successful request means the command was accepted, not that the mode was observed. Physical lights keep their normal power switch and have separate DreamView buttons when identified as video centers. R2A80 supports video DreamView using the Platform API key without account topology; a generic `dreamViewToggle` alone does not classify other models as video centers.

ScenicView uses the saved `DreamViewScenic` object's own advertised `powerSwitch`. It never reuses a physical center's generic `dreamViewToggle` to activate an arbitrary saved scene. Control works independently of account topology when the object is present in the Platform device list. Old `/api/virtual-control/{id}/power/on` and `/off` requests also resolve those Platform IDs.

The state endpoint can return HTTP 200 with JSON `code:400` for virtual objects; this is an API error, not OFF. Empty DreamView state values remain unknown. Member power does not identify an active ScenicView. Real Platform push values update the mode even before the first poll; command responses do not manufacture mode state.

Home Assistant receives separate DreamView and ScenicView On/Off buttons. Existing switch unique IDs and command topics remain compatible with automations; unknown mode state is published as unknown, without optimistic state changes. One Click identities and discovery protection remain intact.

The user-supplied Account Token option introduced in 2026-11-17 has been removed. Keep the **Govee API Key** for Platform features. Optional account email/password and verification-code sign-in remain available for IoT and original One Click actions; the API key alone does not replace that sign-in.

The diagnostic snapshots confirm that DreamView state is not readable through these responses. Real R2A80 ON was tested by the user; hardware OFF and saved ScenicView control still need verification on the devices. Automated tests verify the exact outgoing capabilities and values and error handling.
