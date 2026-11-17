# DreamView recovery without Community login

A successful Govee Account/IoT connection does not imply a successful Community login. A Community error must not remove previously discovered One Click scenes.

## API-key-only video center

Set these Home Assistant app options (use the actual stable device ID, not an IP):

```yaml
govee_api_key: YOUR_API_KEY
video_dreamview_sku: R2A80
video_dreamview_id: "DEVICE_ID_FROM_PLATFORM_API"
```

Email/password are optional for this path. Select the device you have verified as your Movie DreamView video center. A generic `dreamViewToggle` capability on another model is not sufficient to identify a video center. The explicit SKU and device ID must match, and the Platform metadata must advertise both on and off values. Invalid or missing metadata does not create replacement scenes.

For a selected center without LAN discovery, this explicit configuration authorizes Platform API control even when `lan_cloud_fallback` is disabled. Real LAN devices retain LAN-first routing and use only the configured `disabled`, `iot`, or `platform` fallback. This does not add guessed IP addresses or UDP support to R2A80.

The dedicated switch is optimistic: its displayed state can reflect the last accepted command until a Platform state observation arrives. The two command scenes have no state. An accepted command does not prove a physical change; check both ON and OFF on your TV. The documented R2A80 ON command was confirmed physically before this release; OFF still needs live verification.

## Existing automation scenes

The defaults are:

| Action | Full MQTT scene name |
| --- | --- |
| ON | `One-Click: Default: Включить DreamView` |
| OFF | `One-Click: Default: Выключить DreamView` |

If the originals used different capitalization, spacing or group names, set `dreamview_on_scene` and `dreamview_off_scene` to their exact full names. Read them from a backup, MQTT retained discovery `payload_on`, or the former scene metadata. The prior `unique_id` is `gv2mqtt-one-click-` plus the unhyphenated UUID v5 (DNS namespace, UTF-8 full name); the new scenes use the same algorithm, bridge device identity, discovery topic, command topic and payload.

This preserves an existing registry identity when the names match. An old `entity_id` such as `scene.govee_to_mqtt_one_click_default_vkliuchit_dreamview` does not uniquely determine the original spelling. Automatic restoration of that exact entity ID cannot be guaranteed from the entity ID alone. Do not delete the HA registry or retained discovery to fix this.

If the original scene is fetched and IoT is available, the original executes. Otherwise only these two explicitly configured aliases can use replacement commands. A deduplicated name set publishes one discovery entity per exact name. Logs distinguish original IoT actions from DreamView replacements. Platform failure is returned as an error without recording successful state.

The Night Mode One Click, ScenicView, Music DreamView and group membership are not reconstructed from a toggle. Their original action definitions/topology remain necessary.

## Discovery during outages

After a successful complete fetch, only scene names are saved to `gv2mqtt-one-click-names.json` in the normal cache directory. A failed fetch republishes known original scene identities as unavailable and protects previously retained One Click topics from cleanup, including topics recorded by older versions. If optional account topology fails, stale discovery cleanup is deferred until that source recovers. Protected topics remain in the discovery-topic index across restarts.

A successful empty list is authoritative and can remove deleted scenes, except explicitly configured compatibility aliases. Expired API results are not silently reused to authorize deletion. MQTT publication failures abort cleanup. Original scene availability uses `gv2mqtt/oneclick/availability`; this does not change physical-device availability.

The first update cannot recover names already deleted from both retained discovery and the entity registry. Existing older discovery that has never been republished cannot acquire the new availability topic until its original name is known. No SQLite cache, retained discovery, or HA entity registry is automatically cleared by this fix.

## Experimental home-layout authentication

`one_click_auth` defaults to `community`. Set `account` to try the working Account bearer token on the home-layout endpoint, or `auto` to try Account then Community. These paths use separate response caches, validate application status, and do not log token values in the new diagnostics. They are experimental and have not been validated against this user's live account. Browser login success does not prove compatibility with either API. Original One Click execution still requires account credentials and IoT.

| Home Assistant option | Environment variable |
| --- | --- |
| `video_dreamview_sku` | `GOVEE_VIDEO_DREAMVIEW_SKU` |
| `video_dreamview_id` | `GOVEE_VIDEO_DREAMVIEW_ID` |
| `dreamview_on_scene` | `GOVEE_DREAMVIEW_ON_SCENE` |
| `dreamview_off_scene` | `GOVEE_DREAMVIEW_OFF_SCENE` |
| `one_click_auth` | `GOVEE_ONE_CLICK_AUTH` |
