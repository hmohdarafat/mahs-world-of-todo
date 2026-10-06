//! Class, specialization, race, and role catalogs.

use crate::{config::*, data::abilities::*, model::*};
use crate::model::{Role::{Healer, Melee, Ranged, Tank}, Wt::{Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive}};

pub(crate) const CLASSES: [Class; 13] = [
    Class { name: "Death Knight", armor: "Plate", specs: &[("Blood", Tank), ("Frost", Melee), ("Unholy", Melee)],
            abilities: AB_DK,
            weapons: &[Axe, Mace, Sword, Polearm], dual: true, shield: false, held: false },
    Class { name: "Demon Hunter", armor: "Leather", specs: &[("Havoc", Melee), ("Vengeance", Tank), ("Devourer", Melee)],
            abilities: AB_DH,
            weapons: &[Fist, Sword, Axe, Warglaive], dual: true, shield: false, held: false },
    Class { name: "Druid", armor: "Leather", specs: &[("Balance", Ranged), ("Feral", Melee), ("Guardian", Tank), ("Restoration", Healer)],
            abilities: AB_DRUID,
            weapons: &[Dagger, Fist, Mace, Polearm, Staff], dual: false, shield: false, held: true },
    Class { name: "Evoker", armor: "Mail", specs: &[("Augmentation", Ranged), ("Devastation", Ranged), ("Preservation", Healer)],
            abilities: AB_EVOKER,
            weapons: &[Dagger, Fist, Axe, Mace, Sword, Staff], dual: false, shield: false, held: true },
    Class { name: "Hunter", armor: "Mail", specs: &[("Beast Mastery", Ranged), ("Marksmanship", Ranged), ("Survival", Melee)],
            abilities: AB_HUNTER,
            weapons: &[Bow, Crossbow, Gun, Axe, Dagger, Fist, Polearm, Staff, Sword], dual: false, shield: false, held: false },
    Class { name: "Mage", armor: "Cloth", specs: &[("Arcane", Ranged), ("Fire", Ranged), ("Frost", Ranged)],
            abilities: AB_MAGE,
            weapons: &[Dagger, Sword, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Monk", armor: "Leather", specs: &[("Brewmaster", Tank), ("Mistweaver", Healer), ("Windwalker", Melee)],
            abilities: AB_MONK,
            weapons: &[Fist, Mace, Sword, Axe, Polearm, Staff], dual: true, shield: false, held: false },
    Class { name: "Paladin", armor: "Plate", specs: &[("Holy", Healer), ("Protection", Tank), ("Retribution", Melee)],
            abilities: AB_PALADIN,
            weapons: &[Axe, Mace, Sword, Polearm], dual: false, shield: true, held: false },
    Class { name: "Priest", armor: "Cloth", specs: &[("Discipline", Healer), ("Holy", Healer), ("Shadow", Ranged)],
            abilities: AB_PRIEST,
            weapons: &[Dagger, Mace, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Rogue", armor: "Leather", specs: &[("Assassination", Melee), ("Outlaw", Melee), ("Subtlety", Melee)],
            abilities: AB_ROGUE,
            weapons: &[Dagger, Fist, Mace, Sword, Axe], dual: true, shield: false, held: false },
    Class { name: "Shaman", armor: "Mail", specs: &[("Elemental", Ranged), ("Enhancement", Melee), ("Restoration", Healer)],
            abilities: AB_SHAMAN,
            weapons: &[Axe, Dagger, Fist, Mace, Staff], dual: true, shield: true, held: true },
    Class { name: "Warlock", armor: "Cloth", specs: &[("Affliction", Ranged), ("Demonology", Ranged), ("Destruction", Ranged)],
            abilities: AB_WARLOCK,
            weapons: &[Dagger, Sword, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Warrior", armor: "Plate", specs: &[("Arms", Melee), ("Fury", Melee), ("Protection", Tank)],
            abilities: AB_WARRIOR,
            weapons: &[Axe, Dagger, Fist, Mace, Polearm, Staff, Sword], dual: true, shield: true, held: false },
];

pub(crate) fn spec_names(c: usize) -> Vec<&'static str> { CLASSES[c].specs.iter().map(|s| s.0).collect() }

pub(crate) fn offhand_text(c: &Class) -> String {
    let mut v: Vec<&str> = vec![];
    if c.shield { v.push("Shield"); }
    if c.dual { v.push("Dual wield"); }
    if c.held { v.push("Held off-hand"); }
    if v.is_empty() { "No off-hand".to_string() } else { v.join(" · ") }
}

pub(crate) fn races(faction: &str) -> Vec<&'static str> {
    let base = if faction == "Horde" { HORDE } else { ALLIANCE };
    base.iter().chain(NEUTRAL.iter()).copied().collect()
}

pub(crate) fn npcs(f: &str) -> [&'static str; 3] {
    if f == "Horde" { ["Gornek", "Zureetha Fargaze", "Kaltunk"] }
    else { ["Marshal McBride", "Deputy Willem", "Llane Beshere"] }
}

pub(crate) fn cat_kind(i: usize) -> &'static str {
    match i { 0 => "general task", 1..=GATHER_MAX => "gathering", _ => "crafting" }
}
pub(crate) fn fcol(f: &str) -> &'static str { if f == "Horde" { "#e0453a" } else { "#4a8fe7" } }