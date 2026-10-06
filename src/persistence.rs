use gtk::glib;
use std::path::PathBuf;
use crate::{
    config::{MAX_LEVEL, ZONE_TERR}, data::{classes::*, items::*}, items::make_item, model::*, utils::*,
    world::{fallback_zone, generate_zones, zone_bands},
};

pub(crate) fn save_path() -> PathBuf {
    let d = glib::user_data_dir().join("wow-todo");
    let _ = std::fs::create_dir_all(&d);
    d.join("characters.json")
}

/// Upgrade old saves: old gear system, kill-quest steps, resource bars, removed zones.
pub(crate) fn migrate(s: &mut Save) {
    let map: [usize; 6] = [0, 1, 2, 6, S_MAIN, 12];
    let zones = &s.zones;
    for h in &mut s.heroes {
        for q in &mut h.quests {
            if q.quest_level == 0 {
                q.quest_level = h.level.clamp(1, MAX_LEVEL);
            }
            if q.target.is_empty() { q.target = "Kobolds".into(); }
            let (g, p) = (q.goal, q.progress);
            if q.qk == QKind::Kill && q.steps.is_empty() && g > 0 {
                q.steps = (0..g).map(|i| Step { text: String::new(), done: i < p }).collect();
            }
        }
        if h.gear.len() != SLOTS.len() {
            let old = std::mem::take(&mut h.gear);
            h.gear = vec![None; SLOTS.len()];
            let class = h.class.min(CLASSES.len() - 1);
            let spec = h.spec.min(CLASSES[class].specs.len() - 1);
            for (i, it) in old.into_iter().enumerate() {
                let (Some(it), Some(&cat)) = (it, map.get(i)) else { continue };
                let q = [0usize, 2, 3, 4][it.quality.min(3)];
                if let Some(mut n) = make_item(class, spec, h.level, cat, q) {
                    n.ilvl = it.ilvl.max(1);
                    h.gear[cat] = Some(n);
                }
            }
            h.log.push(LogEntry {
                time: now(), cat: "System".into(),
                msg: "🔧 Old gear converted to the new equipment system.".into(),
            });
        }
        if !zones.iter().any(|z| z.name == h.zone) {
            h.zone = fallback_zone(zones, &h.faction, h.level);
            h.zone_inst.clear();
        }
        // 0 HP is now a valid state; only old saves (no `v2`) treat 0 as "unset".
        if !h.v2 {
            if h.hp == 0 { h.restore_all(); }
            h.v2 = true;
        }
        h.clamp_res();
    }
}

pub(crate) fn load() -> Save {
    let mut s: Save = std::fs::read_to_string(save_path()).ok()
        .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let regen = s.zones.len() != zone_bands().len() * ZONE_TERR.len();
    if regen { s.zones = generate_zones(); }
    migrate(&mut s);
    if regen { persist(&s); }
    s
}
pub(crate) fn persist(s: &Save) {
    if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write(save_path(), j); }
}