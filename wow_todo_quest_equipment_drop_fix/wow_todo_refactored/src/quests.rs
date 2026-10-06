use crate::{config::MAX_LEVEL, model::*, utils::*};

impl Tier {
    pub(crate) const ALL: [Tier; 5] = [Tier::Normal, Tier::Elite, Tier::Dungeon, Tier::Raid, Tier::WorldBoss];
    pub(crate) fn name(self) -> &'static str { ["Normal", "Elite", "Dungeon", "Raid", "World Boss"][self as usize] }
    pub(crate) fn unlock(self) -> u32 { [1, 5, 10, 20, 40][self as usize] }
    pub(crate) fn mult(self) -> u32 { [1, 2, 4, 8, 16][self as usize] }
    pub(crate) fn loot(self) -> u32 { [25, 50, 100, 100, 100][self as usize] }
}

pub(crate) fn quest_level_bounds(hero_level: u32) -> (u32, u32) {
    let min = hero_level.saturating_sub(5).max(1);
    let max = hero_level.saturating_add(5).min(MAX_LEVEL);
    (min, max)
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
