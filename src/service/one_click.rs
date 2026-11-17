//! Discovery metadata only: never persist IoT messages, credentials or bearer tokens here.
use crate::hass_mqtt::base::{AvailabilityEntry, Device, EntityConfig, Origin};
use crate::hass_mqtt::scene::SceneConfig;
use crate::service::hass::{availability_topic, oneclick_topic};
use std::collections::BTreeSet;
use std::path::PathBuf;
use uuid::Uuid;

pub const AVAILABILITY_TOPIC: &str = "gv2mqtt/oneclick/availability";

pub fn unique_id(name: &str) -> String {
    format!(
        "gv2mqtt-one-click-{}",
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()).simple()
    )
}

pub fn scene_config(name: &str) -> SceneConfig {
    SceneConfig {
        base: EntityConfig {
            availability_topic: String::new(),
            availability: vec![
                AvailabilityEntry { topic: availability_topic() },
                AvailabilityEntry { topic: AVAILABILITY_TOPIC.to_string() },
            ],
            availability_mode: Some("all".to_string()),
            name: Some(name.to_string()),
            entity_category: None,
            origin: Origin::default(),
            device: Device::this_service(),
            unique_id: unique_id(name),
            device_class: None,
            icon: None,
        },
        command_topic: oneclick_topic(),
        payload_on: name.to_string(),
    }
}

pub fn preserve_topic(topic: &str, complete: bool, preserve_topology: bool) -> bool {
    preserve_topology
        || (!complete && topic.contains("/scene/gv2mqtt-one-click-"))
        || (!complete && topic.ends_with("-dreamViewToggle/config"))
}

/// Return the durable index and confirmed removals for this registration.
pub fn reconcile_discovery(
    prior: &[String],
    current: &BTreeSet<String>,
    complete: bool,
    preserve_topology: bool,
) -> (BTreeSet<String>, Vec<String>) {
    let mut kept = current.clone();
    let mut removed = vec![];
    for topic in prior {
        if kept.contains(topic) {
            continue;
        }
        if preserve_topic(topic, complete, preserve_topology) {
            kept.insert(topic.clone());
        } else {
            removed.push(topic.clone());
        }
    }
    (kept, removed)
}

fn names_path() -> PathBuf {
    let mut path = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("gv2mqtt-one-click-names.json");
    path
}

pub fn load_names() -> BTreeSet<String> {
    std::fs::read(names_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_names(names: &BTreeSet<String>) -> anyhow::Result<()> {
    let path = names_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
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
        assert!(!preserve_topic(
            "homeassistant/light/device/config",
            false,
            false
        ));
    }

    #[test]
    fn repeated_failed_syncs_keep_the_index_until_a_confirmed_deletion() {
        let old = "homeassistant/scene/gv2mqtt-one-click-old/config".to_string();
        let current = "homeassistant/scene/gv2mqtt-one-click-current/config".to_string();
        let stale_light = "homeassistant/light/old/config".to_string();
        let now = BTreeSet::from([current.clone()]);
        let prior = vec![old.clone(), current.clone(), stale_light.clone()];
        let (first, removed) = reconcile_discovery(&prior, &now, false, false);
        assert_eq!(removed, vec![stale_light]);
        assert!(first.contains(&old));
        // The returned index is exactly what is saved and loaded on restart.
        let (second, removed) =
            reconcile_discovery(&first.into_iter().collect::<Vec<_>>(), &now, false, false);
        assert!(removed.is_empty());
        assert!(second.contains(&old));
        let (third, removed) =
            reconcile_discovery(&second.into_iter().collect::<Vec<_>>(), &now, true, false);
        assert_eq!(removed, vec![old]);
        assert_eq!(third, now);
    }

    #[test]
    fn scene_identity_and_availability_are_stable() {
        let name = "One-Click: Home: Evening";
        let config = scene_config(name);
        assert_eq!(config.base.unique_id, unique_id(name));
        assert_eq!(config.command_topic, oneclick_topic());
        assert_eq!(config.payload_on, name);
        assert_eq!(config.base.availability.len(), 2);
        assert_eq!(config.base.availability[1].topic, AVAILABILITY_TOPIC);
        assert_eq!(config.base.availability_mode.as_deref(), Some("all"));
    }
}
