//! Explicit video-center selection; never infer a video role from a generic toggle.
use crate::service::device::Device;
use crate::service::state::StateHandle;
use anyhow::Context;

#[derive(Clone, Debug)]
pub struct VideoDreamViewConfig {
    pub sku: String,
    pub device_id: String,
    pub on_scene: String,
    pub off_scene: String,
}

impl VideoDreamViewConfig {
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        let read = |key: &str| std::env::var(key).ok().filter(|s| !s.trim().is_empty());
        match (read("GOVEE_VIDEO_DREAMVIEW_SKU"), read("GOVEE_VIDEO_DREAMVIEW_ID")) {
            (None, None) => Ok(None),
            (Some(sku), Some(device_id)) => {
                let config = Self {
                    sku: sku.trim().to_string(),
                    device_id: device_id.trim().to_string(),
                    on_scene: read("GOVEE_DREAMVIEW_ON_SCENE").unwrap_or_else(||
                        "One-Click: Default: Включить DreamView".to_string()),
                    off_scene: read("GOVEE_DREAMVIEW_OFF_SCENE").unwrap_or_else(||
                        "One-Click: Default: Выключить DreamView".to_string()),
                };
                anyhow::ensure!(config.on_scene != config.off_scene, "DreamView ON/OFF scene names must differ");
                Ok(Some(config))
            }
            _ => anyhow::bail!("Set both video_dreamview_sku and video_dreamview_id (SKU and stable Platform device ID, not an IP address)"),
        }
    }

    pub fn matches(&self, device: &Device) -> bool {
        self.sku.eq_ignore_ascii_case(&device.sku)
            && self.device_id.eq_ignore_ascii_case(&device.id)
    }

    pub fn action(&self, name: &str) -> Option<bool> {
        if name == self.on_scene {
            Some(true)
        } else if name == self.off_scene {
            Some(false)
        } else {
            None
        }
    }
}

/// Require the actual API capability and both advertised values before exposing aliases.
pub fn has_platform_toggle(device: &Device) -> bool {
    device
        .get_capability_by_instance("dreamViewToggle")
        .is_some_and(|cap| {
            cap.enum_parameter_by_name("on").is_some()
                && cap.enum_parameter_by_name("off").is_some()
        })
}

pub async fn activate_alias(state: &StateHandle, name: &str) -> anyhow::Result<bool> {
    let Some(config) = state.video_dreamview_config().await else {
        return Ok(false);
    };
    let Some(enabled) = config.action(name) else {
        return Ok(false);
    };
    let device = state
        .devices()
        .await
        .into_iter()
        .find(|device| config.matches(device))
        .context("Configured video DreamView center has not been discovered")?;
    anyhow::ensure!(
        config.matches(&device) && has_platform_toggle(&device),
        "Configured video DreamView center has no matching SKU and Platform on/off capability"
    );
    log::info!(
        "Executing DreamView replacement command (not original One Click), enabled={enabled}"
    );
    let device = state.resolve_device_for_control(&device.id).await?;
    state.device_set_dreamview(&device, enabled).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_requires_both_sku_and_stable_id() {
        let config = VideoDreamViewConfig {
            sku: "R2A80".into(),
            device_id: "AA:BB".into(),
            on_scene: "on".into(),
            off_scene: "off".into(),
        };
        assert!(config.matches(&Device::new("r2a80", "aa:bb")));
        assert!(!config.matches(&Device::new("H606A", "AA:BB")));
        assert!(!config.matches(&Device::new("R2A80", "CC:DD")));
        assert_eq!(config.action("on"), Some(true));
        assert_eq!(config.action("off"), Some(false));
        assert_eq!(config.action("night mode"), None);
        assert!(!has_platform_toggle(&Device::new("R2A80", "AA:BB")));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::platform_api::{GoveeApiClient, HttpDeviceInfo};
    use crate::service::hass::HassClient;
    use crate::service::state::State;
    use axum::{routing::post, Json, Router};
    use serde_json::{json, Value};
    use std::sync::Arc;

    async fn setup(
        code: u32,
    ) -> (
        StateHandle,
        tokio::sync::mpsc::Receiver<Value>,
        tokio::task::JoinHandle<()>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let app = Router::new().route("/control", post(move |Json(body): Json<Value>| {
            let tx = tx.clone();
            async move {
                tx.send(body.clone()).await.unwrap();
                Json(json!({"requestId": body["requestId"], "code": code, "msg": "test",
                    "capability": {"type": "devices.capabilities.toggle", "instance": "dreamViewToggle",
                    "value": body["payload"]["capability"]["value"], "state": {"status":"success"}}}))
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let state = Arc::new(State::new());
        state
            .set_platform_client(
                GoveeApiClient::new("test-key")
                    .unwrap()
                    .with_control_url(format!("http://{addr}/control")),
            )
            .await;
        state.set_hass_client(HassClient::new_test()).await;
        let info: HttpDeviceInfo = serde_json::from_value(json!({
            "sku":"R2A80", "device":"AA:BB", "deviceName":"TV", "type":"devices.types.light",
            "capabilities":[{"type":"devices.capabilities.toggle", "instance":"dreamViewToggle",
                "parameters":{"dataType":"ENUM", "options":[{"name":"on","value":7},{"name":"off","value":3}]}}]
        })).unwrap();
        state
            .device_mut("R2A80", "AA:BB")
            .await
            .set_http_device_info(info);
        state
            .set_video_dreamview_config(Some(VideoDreamViewConfig {
                sku: "R2A80".into(),
                device_id: "AA:BB".into(),
                on_scene: "on".into(),
                off_scene: "off".into(),
            }))
            .await;
        (state, rx, server)
    }

    #[tokio::test]
    async fn api_key_only_aliases_use_advertised_on_and_off_without_lan_or_topology() {
        let (state, mut rx, server) = setup(200).await;
        let device = state.device_by_id("AA:BB").await.unwrap();
        assert!(state.device_supports_dreamview(&device).await);
        assert!(
            !state
                .device_supports_dreamview(&Device::new("H606A", "CC:DD"))
                .await
        );
        assert!(activate_alias(&state, "on").await.unwrap());
        let on = rx.recv().await.unwrap();
        assert_eq!(on["payload"]["capability"]["value"], 7);
        assert_eq!(on["payload"]["sku"], "R2A80");
        assert_eq!(on["payload"]["device"], "AA:BB");
        assert!(activate_alias(&state, "off").await.unwrap());
        let off = rx.recv().await.unwrap();
        assert_eq!(off["payload"]["capability"]["value"], 3);
        assert_ne!(on["requestId"], off["requestId"]);
        assert_eq!(
            state
                .device_by_id("AA:BB")
                .await
                .unwrap()
                .dreamview_enabled(),
            Some(false)
        );
        assert!(!activate_alias(&state, "night mode").await.unwrap());
        server.abort();
    }

    #[tokio::test]
    async fn logical_api_failure_does_not_record_a_successful_state_change() {
        let (state, _rx, server) = setup(400).await;
        assert!(activate_alias(&state, "on").await.is_err());
        assert_eq!(
            state
                .device_by_id("AA:BB")
                .await
                .unwrap()
                .dreamview_enabled(),
            None
        );
        server.abort();
    }

    #[tokio::test]
    async fn wrong_sku_cannot_control_another_device() {
        let (state, mut rx, server) = setup(200).await;
        state
            .set_video_dreamview_config(Some(VideoDreamViewConfig {
                sku: "H606A".into(),
                device_id: "AA:BB".into(),
                on_scene: "on".into(),
                off_scene: "off".into(),
            }))
            .await;
        assert!(activate_alias(&state, "on").await.is_err());
        assert!(rx.try_recv().is_err());
        server.abort();
    }
}
