use crate::{config::MAX_LEVEL, model::*, utils::*};

impl Tier {
    pub(crate) const ALL: [Tier; 5] = [Tier::Normal, Tier::Elite, Tier::Dungeon, Tier::Raid, Tier::WorldBoss];
    pub(crate) fn name(self) -> &'static str { ["Normal", "Elite", "Dungeon", "Raid", "World Boss"][self as usize] }
    pub(crate) fn unlock(self) -> u32 { [1, 5, 10, 20, 40][self as usize] }
    pub(crate) fn mult(self) -> u32 { [1, 2, 4, 8, 16][self as usize] }
    pub(crate) fn loot(self) -> u32 { [25, 50, 100, 100, 100][self as usize] }
}

/// Hero level ±5, additionally limited to the zone's level range.
pub(crate) fn quest_level_bounds(hero_level: u32, zone: Option<&Zone>) -> (u32, u32) {
    let mut min = hero_level.saturating_sub(5).max(1);
    let mut max = hero_level.saturating_add(5).min(MAX_LEVEL);
    if let Some(z) = zone {
        max = max.min(z.hi);
        min = min.max(z.lo);
    }
    (min.min(max), max)
}

impl Quest {
    pub(crate) fn display(&self) -> String {
        let what = match self.qk {
            QKind::Kill => format!("Kill {}× {}", self.goal, self.target),
            QKind::Gather => format!("Gather {}× {}", self.goal, self.target),
        };
        let base = if self.title.is_empty() { what } else { format!("{} — {what}", self.title) };
        if self.chain { format!("{base} (Part {})", self.part) } else { base }
    }
}

pub(crate) fn gather_item(cat: usize) -> String {
    match cat {
        1 => ps(&["Copper Ore", "Tin Ore", "Iron Ore", "Mithril Ore"]),
        2 => ps(&["Peacebloom", "Silverleaf", "Briarthorn", "Kingsblood"]),
        3 => ps(&["Ruined Leather Scraps", "Light Hide", "Thick Hide"]),
        4 => ps(&["Raw Brightscale Fish", "Raw Slitherskin Mackerel", "Oily Blackmouth"]),
        _ => ps(&["Runed Relic Fragment", "Glowing Ember", "Ancient Scroll", "Crystal Shard", "Wolf Meat", "Linen Cloth"]),
    }.to_string()
}

/// Creature rank for a quest tier.
pub(crate) fn rank_for(tier: Tier) -> &'static str {
    match tier {
        Tier::Normal => if rnd(100) < 35 { "Swarmer" } else { "Normal" },
        Tier::Elite => "Elite",
        Tier::Dungeon => "Rare",
        Tier::Raid => "Rare-Elite",
        Tier::WorldBoss => "Boss",
    }
}

/// A random (type, name) creature living in the zone.
pub(crate) fn zone_creature(zone: Option<&Zone>) -> (usize, String) {
    zone.filter(|z| !z.creatures.is_empty())
        .map(|z| pick(&z.creatures).clone())
        .unwrap_or((10, "Wild Creatures".to_string()))
}

/// Kill-quest target (with rank prefix) and its creature type.
pub(crate) fn kill_target(zone: Option<&Zone>, tier: Tier) -> (String, usize) {
    let (t, creature) = zone_creature(zone);
    let name = match rank_for(tier) { "Normal" => creature, rank => format!("{rank} {creature}") };
    (name, t)
}

/// One fight during a quest action.
pub(crate) struct Encounter { pub(crate) name: String, pub(crate) ctype: usize, pub(crate) level: u32 }

/// Higher-level chance depends on the zone: enemy > contested > own faction.
pub(crate) fn roll_level(rel: Rel, hero_level: u32) -> u32 {
    let l = if rnd(100) < rel.high_chance() { hero_level + 1 + rnd(3) } else { hero_level.saturating_sub(rnd(3)) };
    l.clamp(1, MAX_LEVEL)
}

pub(crate) fn encounter_for(q: &Quest, zone: Option<&Zone>, faction: &str, hero_level: u32) -> Encounter {
    let (ctype, name) = match (q.qk, q.ctype) {
        (QKind::Kill, Some(t)) => (t, q.target.clone()),
        _ => zone_creature(zone),
    };
    let rel = zone.map_or(Rel::Same, |z| z.relation(faction));
    Encounter { name, ctype, level: roll_level(rel, hero_level) }
}