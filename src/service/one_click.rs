//! Discovery metadata only: never persist IoT messages, credentials or bearer tokens here.
use crate::hass_mqtt::base::{Device, EntityConfig, Origin};
use crate::hass_mqtt::scene::SceneConfig;
use crate::service::hass::{availability_topic, oneclick_topic};
use std::collections::BTreeSet;
use std::path::PathBuf;
use uuid::Uuid;

pub const AVAILABILITY_TOPIC: &str = "gv2mqtt/oneclick/availability";

pub fn unique_id(name: &str) -> String {
    format!("gv2mqtt-one-click-{}", Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()).simple())
}

pub fn scene_config(name: &str, replacement: bool) -> SceneConfig {
    SceneConfig {
        base: EntityConfig {
            availability_topic: if replacement { availability_topic() } else { AVAILABILITY_TOPIC.to_string() },
            availability: vec![], availability_mode: None,
            name: Some(name.to_string()), entity_category: None,
            origin: Origin::default(), device: Device::this_service(),
            unique_id: unique_id(name), device_class: None, icon: None,
        },
        command_topic: oneclick_topic(), payload_on: name.to_string(),
    }
}

pub fn preserve_topic(topic: &str, complete: bool, preserve_topology: bool) -> bool {
    preserve_topology || (!complete && topic.contains("/scene/gv2mqtt-one-click-"))
        || (!complete && topic.ends_with("-dreamViewToggle/config"))
}

fn names_path() -> PathBuf {
    let mut path = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("gv2mqtt-one-click-names.json");
    path
}

pub fn load_names() -> BTreeSet<String> {
    std::fs::read(names_path()).ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}

pub fn save_names(names: &BTreeSet<String>) -> anyhow::Result<()> {
    let path = names_path();
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(names)?)?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_preserve_discovery_but_confirmed_deletions_do_not() {
        let topic = "homeassistant/scene/gv2mqtt-one-click-abc/config";
        assert!(preserve_topic(topic, false, false));
        assert!(!preserve_topic(topic, true, false));
        assert!(preserve_topic(topic, true, true));
        assert!(!preserve_topic("homeassistant/light/device/config", false, false));
    }

    #[test]
    fn replacement_uses_original_discovery_identity_and_command() {
        let name = "One-Click: Default: Включить DreamView";
        let original = scene_config(name, false);
        let replacement = scene_config(name, true);
        assert_eq!(original.base.unique_id, replacement.base.unique_id);
        assert_eq!(original.base.unique_id, format!("gv2mqtt-one-click-{}",
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()).simple()));
        assert_eq!(original.command_topic, replacement.command_topic);
        assert_eq!(original.payload_on, replacement.payload_on);
        assert_eq!(original.base.availability_topic, AVAILABILITY_TOPIC);
        assert_eq!(replacement.base.availability_topic, availability_topic());
    }
}
