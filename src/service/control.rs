use serde::Serialize;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum ControlMode {
    #[default]
    LanFirst = 0,
    LanOnly = 1,
    Cloud = 2,
}

impl ControlMode {
    pub fn parse_config(value: &str) -> anyhow::Result<Self> {
        match value
            .trim()
            .to_ascii_lowercase()
            .replace([' ', '-'], "_")
            .as_str()
        {
            "lan_first" => Ok(Self::LanFirst),
            "lan_only" => Ok(Self::LanOnly),
            "cloud" => Ok(Self::Cloud),
            _ => anyhow::bail!("control_mode must be LAN first, LAN only, or Cloud"),
        }
    }

    pub fn allows_lan(self) -> bool {
        self != Self::Cloud
    }
    pub fn allows_cloud(self) -> bool {
        self != Self::LanOnly
    }
}

impl std::fmt::Display for ControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LanFirst => "LAN first",
            Self::LanOnly => "LAN only",
            Self::Cloud => "Cloud",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_are_exclusive_and_lan_first_is_the_default() {
        assert_eq!(ControlMode::default(), ControlMode::LanFirst);
        for value in ["LAN first", "lan-first", "lan_first"] {
            assert_eq!(
                ControlMode::parse_config(value).unwrap(),
                ControlMode::LanFirst
            );
        }
        assert_eq!(
            ControlMode::parse_config("LAN only").unwrap(),
            ControlMode::LanOnly
        );
        assert_eq!(
            ControlMode::parse_config("Cloud").unwrap(),
            ControlMode::Cloud
        );
        assert!(!ControlMode::LanOnly.allows_cloud());
        assert!(!ControlMode::Cloud.allows_lan());
        assert!(ControlMode::LanFirst.allows_lan() && ControlMode::LanFirst.allows_cloud());
        assert!(ControlMode::parse_config("platform").is_err());
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    use crate::lan_api::LanDevice;
    use crate::platform_api::{GoveeApiClient, HttpDeviceInfo};
    use crate::service::state::State;
    use axum::{routing::post, Json, Router};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn transport_modes_gate_actual_lan_and_cloud_requests() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(16);
        let app = Router::new().route(
            "/control",
            post(move |Json(body): Json<Value>| {
                let tx = tx.clone();
                async move {
                    tx.send(body.clone()).await.unwrap();
                    Json(
                        json!({"requestId":body["requestId"],"code":200,"msg":"success",
                    "capability":{"type":body["payload"]["capability"]["type"],
                    "instance":body["payload"]["capability"]["instance"],
                    "value":body["payload"]["capability"]["value"],"state":{"status":"success"}}}),
                    )
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let udp = tokio::net::UdpSocket::bind("127.0.0.2:4003").await.unwrap();
        let state = Arc::new(State::new());
        state
            .set_platform_client(
                GoveeApiClient::new("test-key")
                    .unwrap()
                    .with_control_url(format!("http://{address}/control")),
            )
            .await;
        let info: HttpDeviceInfo = serde_json::from_value(json!({
            "sku":"H66A1","device":"TEST-TV","type":"devices.types.light","capabilities":[
                {"type":"devices.capabilities.on_off","instance":"powerSwitch","parameters":{"dataType":"ENUM","options":[{"name":"on","value":1},{"name":"off","value":0}]}},
                {"type":"devices.capabilities.toggle","instance":"dreamViewToggle","parameters":{"dataType":"ENUM","options":[{"name":"on","value":7},{"name":"off","value":3}]}}
            ]})).unwrap();
        {
            let mut device = state.device_mut("H66A1", "TEST-TV").await;
            device.set_http_device_info(info);
            device.set_lan_device(LanDevice {
                ip: "127.0.0.2".parse().unwrap(),
                device: "TEST-TV".into(),
                sku: "H66A1".into(),
                ble_version_hard: String::new(),
                ble_version_soft: String::new(),
                wifi_version_hard: String::new(),
                wifi_version_soft: String::new(),
            });
        }
        let device = state.device_by_id("TEST-TV").await.unwrap();
        assert!(state.device_supports_dreamview(&device).await);
        assert!(
            !state
                .device_supports_dreamview(&crate::service::device::Device::new("H606A", "OTHER"))
                .await
        );
        state.set_control_mode(ControlMode::Cloud);
        state.device_power_on(&device, true).await.unwrap();
        assert_eq!(
            rx.recv().await.unwrap()["payload"]["capability"]["instance"],
            "powerSwitch"
        );
        state.device_set_dreamview(&device, true).await.unwrap();
        assert_eq!(
            rx.recv().await.unwrap()["payload"]["capability"]["value"],
            7
        );
        state.device_set_dreamview(&device, false).await.unwrap();
        assert_eq!(
            rx.recv().await.unwrap()["payload"]["capability"]["value"],
            3
        );
        let mut buf = [0; 4096];
        assert!(
            tokio::time::timeout(Duration::from_millis(30), udp.recv_from(&mut buf))
                .await
                .is_err()
        );

        state.set_control_mode(ControlMode::LanOnly);
        assert!(state.get_platform_client().await.is_none());
        assert!(state.get_undoc_client().await.is_none());
        // No LAN query client is running, so the local attempt fails after sending.
        assert!(state.device_power_on(&device, false).await.is_err());
        assert!(
            tokio::time::timeout(Duration::from_millis(200), udp.recv_from(&mut buf))
                .await
                .is_ok()
        );
        state.device_list_scenes(&device).await.unwrap();
        assert!(rx.try_recv().is_err());

        state.set_control_mode(ControlMode::LanFirst);
        state.device_power_on(&device, false).await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(200), udp.recv_from(&mut buf))
                .await
                .is_ok()
        );
        assert_eq!(
            rx.recv().await.unwrap()["payload"]["capability"]["value"],
            0
        );
        server.abort();
    }
}
