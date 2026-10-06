use gtk::{glib, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::Rc,
};
use Role::{Healer, Melee, Ranged, Tank};
use Wt::{Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive};

const APP_TITLE: &str = "MAH's World of Todo";
const MAX_LEVEL: u32 = 90;
const MAX_LOG: usize = 1000;
const ZONE_ROWS: usize = 80;
const PVP_SHOWN: usize = 40;
const PROF_LEVEL: u32 = 5;
const GATHER_MAX: usize = 4; // CATS[1..=4] are gathering, the rest are crafting
const BAG_MAX: usize = 30;

const FACTIONS: [&str; 2] = ["Horde", "Alliance"];
const HORDE: [&str; 6] = ["Orc", "Undead (Forsaken)", "Tauren", "Troll", "Blood Elf", "Goblin"];
const ALLIANCE: [&str; 6] = ["Human", "Dwarf", "Night Elf", "Gnome", "Draenei", "Worgen"];
const NEUTRAL: [&str; 2] = ["Pandaren", "Dracthyr"];
const CATS: [&str; 14] = [
    "Adventure", "Mining", "Herbalism", "Skinning", "Fishing", "Blacksmithing", "Alchemy",
    "Engineering", "Enchanting", "Tailoring", "Leatherworking", "Jewelcrafting", "Inscription", "Cooking",
];
const LOG_FILTERS: [&str; 8] = ["All", "Quest", "Loot", "Level", "Achievement", "PvP", "Travel", "System"];
const ZONE_KINDS: [&str; 8] = [
    "All", "Zones", "Dungeons", "Raids", "Battlegrounds", "Arenas", "Cities / Sanctuaries", "World PvP",
];
const REALM_A: [&str; 12] = [
    "Storm", "Dark", "Silver", "Iron", "Blood", "Frost", "Ember", "Shadow", "Thunder", "Golden", "Moon", "Star",
];
const REALM_B: [&str; 12] = [
    "crest", "wind", "hollow", "spire", "fall", "vale", "reach", "moor", "forge", "haven", "gate", "watch",
];
const SYL1: [&str; 12] = ["Ka", "Mor", "Thal", "Zul", "Bren", "Gor", "Ael", "Shi", "Vor", "Tal", "Dra", "Nym"];
const SYL2: [&str; 10] = ["ga", "di", "we", "ra", "io", "tha", "mar", "ri", "lo", "za"];
const SYL3: [&str; 8] = ["n", "x", "th", "s", "k", "r", "l", "nd"];

// ---------- equipment tables ----------
const SLOTS: [&str; 16] = [
    "Head", "Shoulders", "Chest", "Wrist", "Hands", "Waist", "Legs", "Feet", "Cloak", "Necklace",
    "Ring 1", "Ring 2", "Trinket 1", "Trinket 2", "Main Hand", "Off Hand",
];
const S_MAIN: usize = 14;
const S_OFF: usize = 15;

const QUALITY: [&str; 7] = ["Poor", "Common", "Uncommon", "Rare", "Epic", "Legendary", "Heirloom"];
const QCOL: [&str; 7] = ["#9d9d9d", "#ffffff", "#1eff00", "#0070dd", "#a335ee", "#ff8000", "#00ccff"];
const QMULT: [u32; 7] = [80, 100, 112, 128, 150, 190, 140]; // power multiplier (%)
const QILVL: [i32; 7] = [-2, 0, 2, 5, 9, 15, 6]; // item level offset
const QSTAT: [u32; 7] = [0, 0, 30, 34, 38, 44, 34]; // bonus stat size (% of ilvl)
const QSELL: [u32; 7] = [1, 2, 3, 5, 8, 25, 0]; // vendor value factor
const Q_LEGENDARY: usize = 5;
const Q_HEIRLOOM: usize = 6;

const QUALITY_GUIDE: &str = "Poor (grey) — low-level Normal quests; vendor trash, just sell it.\n\
Common (white) — Normal quests and vendors; sell or discard.\n\
Uncommon (green) — any quest, crafting and random drops; always has an \"of the …\" stat suffix.\n\
Rare (blue) — crafting, quests, Dungeon/Raid tiers and random drops (generally level 20+).\n\
Epic (purple) — high-end crafting, Dungeon/Raid/World Boss tiers, rare drops (generally level 40+).\n\
Legendary (orange) — level 40+ Raid / World Boss quests only, ~1.5–3% per drop; unique proper names.\n\
Heirloom (cyan) — a cache every 20th completed quest; item level scales with your level.\n\
Cosmetic — appearance-only armor with no stats; worn only in empty slots.";

const STAT_NAMES: [&str; 8] = [
    "Stamina", "Strength", "Agility", "Intellect", "Haste", "Critical Strike", "Mastery", "Versatility",
];
const SUFFIXES: [(&str, [usize; 2]); 8] = [
    ("of the Bear", [0, 1]),
    ("of the Eagle", [0, 3]),
    ("of the Tiger", [2, 1]),
    ("of the Gorilla", [1, 3]),
    ("of the Feverflare", [4, 6]),
    ("of the Peerless", [5, 6]),
    ("of the Harmonious", [7, 6]),
    ("of the Aurora", [4, 7]),
];

// weapon nouns
const N_DAGGER: &[&str] = &["Dagger", "Kris", "Stiletto", "Dirk", "Shard", "Tooth", "Blade"];
const N_SWORD1: &[&str] = &["Sword", "Saber", "Blade", "Rapier", "Cutlass", "Scimitar", "Cleaver"];
const N_SWORD2: &[&str] = &["Greatsword", "Claymore", "Longsword", "Zweihander", "Blade"];
const N_AXE1: &[&str] = &["Axe", "Hatchet", "Cleaver", "Chopper", "Hackblade"];
const N_AXE2: &[&str] = &["Greataxe", "Battleaxe", "Decapitator", "Chopper", "Hew", "Cleaver"];
const N_MACE1: &[&str] = &["Mace", "Scepter", "Hammer", "Cudgel", "Truncheon", "Club", "Bludgeon"];
const N_MACE2: &[&str] = &["Warhammer", "Greatmace", "Mallet", "Maul", "Smasher"];
const N_FIST: &[&str] = &["Claws", "Fist", "Handblades", "Knuckles", "Talon", "Grasp"];
const N_POLE: &[&str] = &["Polearm", "Halberd", "Spear", "Glaive", "Trident", "Scythe", "Pike"];
const N_STAFF: &[&str] = &["Staff", "Spire", "Rod", "Greatstaff", "Cane", "Pillar", "Stave"];
const N_BOW: &[&str] = &["Longbow", "Recurve", "Greatbow", "Composite Bow", "Bow"];
const N_XBOW: &[&str] = &["Crossbow", "Arbalest", "Repeater", "Heavy Crossbow"];
const N_GUN: &[&str] = &["Rifle", "Musket", "Blunderbuss", "Shotgun", "Carabine", "Hand-Cannon"];
const N_WAND: &[&str] = &["Wand", "Baton", "Rod", "Scepter", "Branch"];
const N_GLAIVE: &[&str] = &["Warglaive", "Twin Glaive", "Felglaive"];
const N_SHIELD: &[&str] = &["Shield", "Bulwark", "Aegis", "Greatshield", "Barricade", "Defender"];
const N_HELD: &[&str] = &["Tome", "Orb", "Grimoire", "Lantern", "Vessel", "Branch", "Talisman", "Icon"];
const N_CLOAK: &[&str] = &["Cloak", "Cape", "Drape", "Shroud", "Greatcloak"];
const N_NECK: &[&str] = &["Necklace", "Pendant", "Amulet", "Choker", "Locket", "Medallion"];
const N_RING: &[&str] = &["Ring", "Band", "Signet", "Loop", "Seal"];
const N_TRINKET: &[&str] = &["Charm", "Totem", "Idol", "Relic", "Figurine", "Fetish", "Badge", "Orb"];

// armor nouns: [slot][Cloth, Leather, Mail, Plate]
const ARMOR_NOUNS: [[&[&str]; 4]; 8] = [
    // Head
    [&["Cowl", "Hood", "Crown", "Circlet", "Cap", "Hat"],
     &["Helm", "Mask", "Headpiece", "Blindfold", "Guise"],
     &["Helm", "Coif", "Headguard", "Greathelm"],
     &["Helm", "Greathelm", "Faceguard", "Crown", "Casque"]],
    // Shoulders
    [&["Mantle", "Amice", "Shoulderpads", "Epaulets"],
     &["Spaulders", "Shoulderpads", "Pauldrons"],
     &["Spaulders", "Epaulets", "Shoulderguards"],
     &["Pauldrons", "Spaulders", "Shoulderplates"]],
    // Chest
    [&["Robe", "Vestments", "Tunic", "Raiment"],
     &["Vest", "Tunic", "Harness", "Jerkin"],
     &["Hauberk", "Chainmail", "Breastplate", "Tunic"],
     &["Breastplate", "Cuirass", "Chestguard", "Armor"]],
    // Wrist
    [&["Bracers", "Cuffs", "Bindings", "Wraps"],
     &["Bracers", "Armwraps", "Wristguards"],
     &["Bracers", "Armguards", "Wristguards"],
     &["Bracers", "Vambraces", "Armplates"]],
    // Hands
    [&["Gloves", "Mitts", "Handwraps"],
     &["Gloves", "Grips", "Handguards"],
     &["Gauntlets", "Gloves", "Handguards"],
     &["Gauntlets", "Fists", "Handguards"]],
    // Waist
    [&["Cord", "Belt", "Sash", "Cinch"],
     &["Belt", "Girdle", "Strap", "Rope Belt"],
     &["Belt", "Girdle", "Waistguard"],
     &["Girdle", "Belt", "Greatbelt", "Waistplate"]],
    // Legs
    [&["Pants", "Leggings", "Trousers", "Kilt", "Skirt"],
     &["Leggings", "Breeches", "Pants"],
     &["Legguards", "Chainleggings", "Greaves"],
     &["Legplates", "Greaves", "Legguards"]],
    // Feet
    [&["Boots", "Slippers", "Sandals", "Footpads"],
     &["Boots", "Footpads", "Treads", "Moccasins"],
     &["Boots", "Greaves", "Sabatons", "Footguards"],
     &["Sabatons", "Boots", "Stompers", "Warboots"]],
];

// materials (generic = Common/Uncommon/Rare, special = possessive Epic names)
const M_CLOTH: &[&str] = &["Linen", "Woolen", "Silk", "Mageweave", "Runecloth", "Netherweave", "Frostweave", "Embersilk"];
const X_CLOTH: &[&str] = &["Shadowsilk", "Moonweave", "Starthread", "Voidcloth", "Dreamweave"];
const M_LEATHER: &[&str] = &["Tanned Leather", "Sylvan Leather", "Cured Leather", "Hardened Leather", "Rugged Leather", "Knothide", "Drakehide", "Wildhide"];
const X_LEATHER: &[&str] = &["Dragonscale", "Emberleather", "Wyrmhide", "Stormhide", "Nightstalker"];
const M_MAIL: &[&str] = &["Ringed", "Linked Chain", "Scaled", "Bronze Chain", "Mithril Chain", "Heavy Chain", "Brigandine", "Riveted"];
const X_MAIL: &[&str] = &["Dragonmail", "Dreadmail", "Frostlink", "Sunmail", "Stormforged"];
const M_PLATE: &[&str] = &["Tempered Steel", "Forged Iron", "Bronze", "Mithril", "Thorium", "Truesilver", "Obsidian", "Adamantite", "Saronite"];
const X_PLATE: &[&str] = &["Dreadsteel", "Titansteel", "Sunforged", "Voidforged", "Starmetal"];
const M_METAL: &[&str] = &["Copper", "Bronze", "Iron", "Steel", "Mithril", "Thorium", "Obsidian", "Cobalt", "Truesilver"];
const X_METAL: &[&str] = &["Dreadsteel", "Frostforged", "Voidtouched", "Sunforged", "Bloodforged"];
const M_WOOD: &[&str] = &["Oak", "Ashwood", "Ironwood", "Ebony", "Yew", "Pliable", "Gnarled", "Spellwoven"];
const X_WOOD: &[&str] = &["Ancient", "Sunbloom", "Wyrmwood", "Nightbranch", "Moonwood"];
const M_TECH: &[&str] = &["Gnomish", "Goblin", "Brass", "Steel-barreled", "Tinkered", "Engineered", "Iron-bound"];
const X_TECH: &[&str] = &["Overclocked", "Explosive", "Masterwork", "Prototype"];
const M_JEWEL: &[&str] = &["Copper", "Silver", "Gold", "Jade", "Moonstone", "Citrine", "Sapphire", "Ruby", "Onyx"];
const X_JEWEL: &[&str] = &["Dragon-eye", "Starfire", "Voidstone", "Sunstone", "Bloodgem"];
const M_TRINKET: &[&str] = &["Carved", "Ancient", "Glowing", "Shimmering", "Runed", "Tarnished"];
const X_TRINKET: &[&str] = &["Primal", "Arcane", "Eldritch", "Void-touched"];
const M_SHIELD: &[&str] = &["Wooden", "Bronze", "Iron", "Steel", "Mithril", "Thorium", "Reinforced"];
const M_HELD: &[&str] = &["Arcane", "Etched", "Gilded", "Ancient", "Crystalline", "Leatherbound"];
const X_HELD: &[&str] = &["Eldritch", "Runic", "Starlit", "Voidbound"];

// name parts
const POOR_ADJ: &[&str] = &["Worn", "Cracked", "Frayed", "Rusty", "Crude", "Tattered", "Battered", "Chipped", "Rotting", "Splintered", "Shoddy", "Ragged"];
const COMMON_ADJ: &[&str] = &["Plain", "Simple", "Standard", "Sturdy", "Apprentice's", "Recruit's", "Journeyman's"];
const RARE_ADJ: &[&str] = &["Reinforced", "Fine", "Superior", "Masterwork", "Gleaming", "Polished", "Runic", "Gladiator's"];
const THEME_ADJ: &[&str] = &["Desecrated", "Vengeful", "Cataclysmic", "Dreadful", "Corrupted", "Radiant", "Merciless", "Eternal", "Shattered", "Sinister", "Hallowed", "Abyssal", "Wrathful", "Spectral"];
const HEIR_ADJ: &[&str] = &["Weathered", "Burnished", "Ancestral", "Timeworn", "Gilded", "Venerable"];
const COSM_ADJ: &[&str] = &["Festive", "Ornate", "Tournament", "Brewfest", "Winter Veil", "Lunar", "Midsummer", "Gilded"];
const GROUP_ADJ: &[&str] = &["Fallen", "Burning", "Silent", "Frozen", "Crimson", "Ashen", "Forsaken", "Shattered"];
const GROUP_NOUN: &[&str] = &["Vanguard", "Legion", "Covenant", "Brotherhood", "Conclave", "Wardens", "Vigil", "Host"];
const LEG_ADJ: &[&str] = &["Blessed", "Cursed", "Hallowed", "Ancient", "Eternal", "Burning", "Sundered", "Undying"];
const LEG_PART: &[&str] = &["Hand", "Heart", "Fang", "Eye", "Voice", "Wrath", "Shadow", "Soul"];
const CMP_A: &[&str] = &["Gore", "Doom", "Frost", "Blood", "Storm", "Soul", "Night", "Wraith", "Dread", "Grim", "Ember", "Void", "Thunder", "Ash"];
const CMP_B: &[&str] = &["howl", "hammer", "bane", "fang", "reaver", "song", "edge", "scream", "brand", "render", "cleaver", "fury", "wrath", "tongue"];
const WIND_A: &[&str] = &["Wind", "Storm", "Dawn", "Night", "Star", "Flame", "Frost", "Soul"];
const WIND_B: &[&str] = &["seeker", "bringer", "breaker", "caller", "walker", "render", "warden", "bearer"];
const NPC_END: &[&str] = &["ia", "ra", "na", "wen", "dor", "mar", "thia", "gar"];
const BOSS_END: &[&str] = &["gor", "thar", "nos", "zul", "rax", "goth", "dun", "mar"];
const ELF_A: &[&str] = &["Felo", "Aela", "Thala", "Noro", "Vyra", "Lora", "Sili", "Kaela"];
const ELF_B: &[&str] = &["melorn", "thas", "dorei", "anar", "vanis", "thalas", "nore", "estra"];

// ---------- item types ----------
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
enum Wt { Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive }
const ALL_WT: [Wt; 12] = [Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive];

impl Wt {
    fn name(self) -> &'static str {
        match self {
            Axe => "Axe", Bow => "Bow", Crossbow => "Crossbow", Dagger => "Dagger", Fist => "Fist Weapon",
            Gun => "Gun", Mace => "Mace", Polearm => "Polearm", Staff => "Staff", Sword => "Sword",
            Wand => "Wand", Warglaive => "Warglaive",
        }
    }
    fn one(self) -> bool { matches!(self, Axe | Dagger | Fist | Mace | Sword | Wand | Warglaive) }
    fn two(self) -> bool { matches!(self, Axe | Bow | Crossbow | Gun | Mace | Polearm | Staff | Sword) }
    fn ranged(self) -> bool { matches!(self, Bow | Crossbow | Gun) }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
enum Hands { #[default] One, Main, Off, Two }
impl Hands {
    fn name(self) -> &'static str {
        match self { Self::One => "One-Hand", Self::Main => "Main Hand", Self::Off => "Off Hand", Self::Two => "Two-Hand" }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
enum Kind { #[default] Cosmetic, Cloth, Leather, Mail, Plate, Cloak, Necklace, Ring, Trinket, Shield, Held, Weapon }
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Cosmetic => "Cosmetic", Self::Cloth => "Cloth", Self::Leather => "Leather", Self::Mail => "Mail",
            Self::Plate => "Plate", Self::Cloak => "Cloak", Self::Necklace => "Necklace", Self::Ring => "Ring",
            Self::Trinket => "Trinket", Self::Shield => "Shield", Self::Held => "Held In Off-hand",
            Self::Weapon => "Weapon",
        }
    }
    fn armor_idx(self) -> Option<usize> {
        match self { Self::Cloth => Some(0), Self::Leather => Some(1), Self::Mail => Some(2), Self::Plate => Some(3), _ => None }
    }
}
const ARMOR: [Kind; 4] = [Kind::Cloth, Kind::Leather, Kind::Mail, Kind::Plate];
fn armor_kind(s: &str) -> Kind {
    match s { "Cloth" => Kind::Cloth, "Leather" => Kind::Leather, "Mail" => Kind::Mail, _ => Kind::Plate }
}

// ---------- classes ----------
#[derive(Clone, Copy, PartialEq)]
enum Role { Tank, Healer, Melee, Ranged }
impl Role {
    fn name(self) -> &'static str {
        match self { Tank => "Tank", Healer => "Healer", Melee => "Melee DPS", Ranged => "Ranged DPS" }
    }
}

struct Class {
    name: &'static str,
    armor: &'static str,
    specs: &'static [(&'static str, Role)],
    abilities: [&'static str; 5],
    weapons: &'static [Wt],
    dual: bool,
    shield: bool,
    held: bool,
}

const CLASSES: [Class; 13] = [
    Class { name: "Death Knight", armor: "Plate", specs: &[("Blood", Tank), ("Frost", Melee), ("Unholy", Melee)],
            abilities: ["Death Strike", "Obliterate", "Death and Decay", "Anti-Magic Shell", "Death Grip"],
            weapons: &[Axe, Mace, Sword, Polearm], dual: true, shield: false, held: false },
    Class { name: "Demon Hunter", armor: "Leather", specs: &[("Havoc", Melee), ("Vengeance", Tank), ("Devourer", Melee)],
            abilities: ["Demon's Bite", "Eye Beam", "Metamorphosis", "Fel Rush", "Immolation Aura"],
            weapons: &[Fist, Sword, Axe, Warglaive], dual: true, shield: false, held: false },
    Class { name: "Druid", armor: "Leather", specs: &[("Balance", Ranged), ("Feral", Melee), ("Guardian", Tank), ("Restoration", Healer)],
            abilities: ["Moonfire", "Rejuvenation", "Rake", "Bear Form", "Starfire"],
            weapons: &[Dagger, Fist, Mace, Polearm, Staff], dual: false, shield: false, held: true },
    Class { name: "Evoker", armor: "Mail", specs: &[("Augmentation", Ranged), ("Devastation", Ranged), ("Preservation", Healer)],
            abilities: ["Living Flame", "Fire Breath", "Azure Strike", "Emerald Blossom", "Hover"],
            weapons: &[Dagger, Fist, Axe, Mace, Sword, Staff], dual: false, shield: false, held: true },
    Class { name: "Hunter", armor: "Mail", specs: &[("Beast Mastery", Ranged), ("Marksmanship", Ranged), ("Survival", Melee)],
            abilities: ["Arcane Shot", "Kill Command", "Multi-Shot", "Aspect of the Cheetah", "Concussive Shot"],
            weapons: &[Bow, Crossbow, Gun, Axe, Dagger, Fist, Polearm, Staff, Sword], dual: false, shield: false, held: false },
    Class { name: "Mage", armor: "Cloth", specs: &[("Arcane", Ranged), ("Fire", Ranged), ("Frost", Ranged)],
            abilities: ["Frostbolt", "Fire Blast", "Frost Nova", "Blink", "Polymorph"],
            weapons: &[Dagger, Sword, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Monk", armor: "Leather", specs: &[("Brewmaster", Tank), ("Mistweaver", Healer), ("Windwalker", Melee)],
            abilities: ["Tiger Palm", "Blackout Kick", "Vivify", "Roll", "Spinning Crane Kick"],
            weapons: &[Fist, Mace, Sword, Axe, Polearm, Staff], dual: true, shield: false, held: false },
    Class { name: "Paladin", armor: "Plate", specs: &[("Holy", Healer), ("Protection", Tank), ("Retribution", Melee)],
            abilities: ["Crusader Strike", "Holy Light", "Judgment", "Divine Shield", "Hammer of Justice"],
            weapons: &[Axe, Mace, Sword, Polearm], dual: false, shield: true, held: false },
    Class { name: "Priest", armor: "Cloth", specs: &[("Discipline", Healer), ("Holy", Healer), ("Shadow", Ranged)],
            abilities: ["Smite", "Power Word: Shield", "Flash Heal", "Shadow Word: Pain", "Mind Blast"],
            weapons: &[Dagger, Mace, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Rogue", armor: "Leather", specs: &[("Assassination", Melee), ("Outlaw", Melee), ("Subtlety", Melee)],
            abilities: ["Sinister Strike", "Stealth", "Eviscerate", "Evasion", "Kick"],
            weapons: &[Dagger, Fist, Mace, Sword, Axe], dual: true, shield: false, held: false },
    Class { name: "Shaman", armor: "Mail", specs: &[("Elemental", Ranged), ("Enhancement", Melee), ("Restoration", Healer)],
            abilities: ["Lightning Bolt", "Healing Wave", "Flame Shock", "Earth Shock", "Ghost Wolf"],
            weapons: &[Axe, Dagger, Fist, Mace, Staff], dual: true, shield: true, held: true },
    Class { name: "Warlock", armor: "Cloth", specs: &[("Affliction", Ranged), ("Demonology", Ranged), ("Destruction", Ranged)],
            abilities: ["Shadow Bolt", "Corruption", "Immolate", "Fear", "Drain Life"],
            weapons: &[Dagger, Sword, Staff, Wand], dual: false, shield: false, held: true },
    Class { name: "Warrior", armor: "Plate", specs: &[("Arms", Melee), ("Fury", Melee), ("Protection", Tank)],
            abilities: ["Charge", "Rend", "Thunder Clap", "Hamstring", "Shield Bash"],
            weapons: &[Axe, Dagger, Fist, Mace, Polearm, Staff, Sword], dual: true, shield: true, held: false },
];

fn spec_names(c: usize) -> Vec<&'static str> { CLASSES[c].specs.iter().map(|s| s.0).collect() }

fn offhand_text(c: &Class) -> String {
    let mut v: Vec<&str> = vec![];
    if c.shield { v.push("Shield"); }
    if c.dual { v.push("Dual wield"); }
    if c.held { v.push("Held off-hand"); }
    if v.is_empty() { "No off-hand".to_string() } else { v.join(" · ") }
}

fn races(faction: &str) -> Vec<&'static str> {
    let base = if faction == "Horde" { HORDE } else { ALLIANCE };
    base.iter().chain(NEUTRAL.iter()).copied().collect()
}

fn start_zone(race: &str) -> &'static str {
    match race {
        "Human" => "Elwynn Forest",
        "Dwarf" | "Gnome" => "Dun Morogh",
        "Night Elf" => "Teldrassil",
        "Draenei" => "Azuremyst Isle",
        "Worgen" => "Gilneas",
        "Orc" | "Troll" => "Durotar",
        "Undead (Forsaken)" => "Tirisfal Glades",
        "Tauren" => "Mulgore",
        "Blood Elf" => "Eversong Woods",
        "Goblin" => "Kezan",
        "Pandaren" => "The Wandering Isle",
        _ => "The Forbidden Reach",
    }
}

fn npcs(f: &str) -> [&'static str; 3] {
    if f == "Horde" { ["Gornek", "Zureetha Fargaze", "Kaltunk"] }
    else { ["Marshal McBride", "Deputy Willem", "Llane Beshere"] }
}

fn cat_kind(i: usize) -> &'static str {
    match i { 0 => "general task", 1..=GATHER_MAX => "gathering", _ => "crafting" }
}
fn fcol(f: &str) -> &'static str { if f == "Horde" { "#e0453a" } else { "#4a8fe7" } }

// ---------- helpers ----------
type Msg = (&'static str, String);
fn m(c: &'static str, s: impl Into<String>) -> Msg { (c, s.into()) }
fn rnd(n: u32) -> u32 { glib::random_int_range(0, n as i32) as u32 }
fn pick<T>(a: &[T]) -> &T { &a[rnd(a.len() as u32) as usize] }
fn ps(a: &[&'static str]) -> &'static str { a[rnd(a.len() as u32) as usize] }
fn now() -> String {
    glib::DateTime::now_local().ok()
        .and_then(|d| d.format("%Y-%m-%d %H:%M:%S").ok())
        .map(|s| s.to_string()).unwrap_or_default()
}
fn commas(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out
}
fn player_name() -> String { format!("{}{}{}", pick(&SYL1), pick(&SYL2), pick(&SYL3)) }
fn npc_name() -> String { format!("{}{}{}", ps(&SYL1), ps(&SYL2), ps(NPC_END)) }
fn boss_name() -> String { format!("{}{}{}", ps(&SYL1), ps(&SYL2), ps(BOSS_END)) }
fn elven_name() -> String { format!("{}'{}", ps(ELF_A), ps(ELF_B)) }
fn compound_name() -> String { format!("{}{}", ps(CMP_A), ps(CMP_B)) }

// ---------- realms ----------
struct Realm { name: String, pop: u32, tier: usize }
impl Realm {
    fn label(&self) -> String {
        format!("{} — {} players online ({})", self.name, commas(self.pop), ["High", "Medium", "Low"][self.tier])
    }
}
fn make_realms() -> Vec<Realm> {
    let mins = [5000u32, 2000, 500];
    let spread = [3000u32, 1500, 500];
    let mut names: Vec<String> = vec![];
    while names.len() < 3 {
        let n = format!("{}{}", pick(&REALM_A), pick(&REALM_B));
        if !names.contains(&n) { names.push(n); }
    }
    names.into_iter().enumerate()
        .map(|(i, name)| Realm { name, pop: mins[i] + rnd(spread[i]), tier: i }).collect()
}

// ---------- zones (from src/zones.tsv) ----------
#[derive(Clone)]
struct Zone { name: String, lo: u32, hi: u32, terr: String, inst: String }

fn inst_bonus(inst: &str) -> u32 { match inst { "Raid" => 50, "Dungeon" => 25, _ => 0 } }

impl Zone {
    fn level_text(&self) -> String {
        if self.lo == self.hi { self.lo.to_string() } else { format!("{}-{}", self.lo, self.hi) }
    }
    fn open_to(&self, faction: &str, level: u32) -> bool {
        (self.terr != "Alliance" || faction == "Alliance")
            && (self.terr != "Horde" || faction == "Horde")
            && level >= self.lo
    }
    fn matches_kind(&self, k: u32) -> bool {
        match k {
            1 => self.inst.is_empty() && matches!(self.terr.as_str(), "Alliance" | "Horde" | "Contested"),
            2 => self.inst == "Dungeon",
            3 => self.inst == "Raid",
            4 => self.inst == "Battleground",
            5 => self.inst == "Arena",
            6 => self.terr == "Sanctuary",
            7 => self.terr == "World PvP",
            _ => true,
        }
    }
}

const FALLBACK_ZONES: &str = "Elwynn Forest\t1 - 30\tAlliance\t\nDurotar\t1 - 30\tHorde\t\nStranglethorn Vale\t10 - 30\tContested\t\nThe Deadmines\t7 - 30\tAlliance\tDungeon\nMolten Core\t30\tContested\tRaid\nWarsong Gulch\t60 - 90\tPvP\tBattleground\nDalaran\t1 - 90\tSanctuary\t";

fn parse_level(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    if s.is_empty() { return Some((1, MAX_LEVEL)); }
    let mut it = s.split('-').map(|p| p.trim().parse::<u32>());
    match (it.next(), it.next()) {
        (Some(Ok(a)), None) => Some((a, a)),
        (Some(Ok(a)), Some(Ok(b))) => Some((a, b)),
        _ => None,
    }
}

fn strip_tail<'a>(t: &'a [&'a str], known: &[&str]) -> (&'a [&'a str], String) {
    for k in known {
        let w: Vec<&str> = k.split(' ').collect();
        if t.len() > w.len() && t[t.len() - w.len()..] == w[..] {
            return (&t[..t.len() - w.len()], k.to_string());
        }
    }
    (t, String::new())
}

fn parse_line(line: &str) -> Option<Zone> {
    let line = line.trim_end();
    if line.trim().is_empty() || line.starts_with("Name,") || line.starts_with("Name\t") { return None; }
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() >= 3 {
        let (lo, hi) = parse_level(f[1])?;
        return Some(Zone {
            name: f[0].trim().to_string(), lo, hi,
            terr: f[2].trim().to_string(),
            inst: f.get(3).map_or("", |s| s.trim()).to_string(),
        });
    }
    // fallback when tabs were turned into spaces: parse from the right
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let (t, inst) = strip_tail(&tokens, &["Artifact Acquisition", "Class Hall", "Battleground", "Dungeon", "Raid", "Arena", "Transit"]);
    let (t, terr) = strip_tail(t, &["World PvP", "Contested", "Sanctuary", "Alliance", "Horde", "PvP"]);
    if terr.is_empty() { return None; }
    let n = t.len();
    let (t, lo, hi) = if n >= 4 && t[n - 2] == "-" {
        match (t[n - 3].parse::<u32>(), t[n - 1].parse::<u32>()) {
            (Ok(a), Ok(b)) => (&t[..n - 3], a, b),
            _ => (t, 1, MAX_LEVEL),
        }
    } else if n >= 2 {
        match t[n - 1].parse::<u32>() { Ok(a) => (&t[..n - 1], a, a), Err(_) => (t, 1, MAX_LEVEL) }
    } else { (t, 1, MAX_LEVEL) };
    if t.is_empty() { return None; }
    Some(Zone { name: t.join(" "), lo, hi, terr, inst })
}

fn junk(n: &str) -> bool {
    let l = n.to_lowercase();
    n.is_empty()
        || n.starts_with(|c: char| c.is_ascii_digit())
        || n.contains('[')
        || [
            "test", " dev", "dev ", "devland", "do not", "dnt", "(copy", "delete me", "unused", "xxold", "zzold",
            "zz_", "placeholder", "playground", "prototype", "smoketest", "demoarea", "demo area", "dummy",
            "internal only", "blockout", "doodad", "hackaton", "blizzcon", "gm island", "crapopolis", "marie lazar",
            "doug land", "pocket dimension", "null space", "allied - ", "boost experience", "env art", "fx - ",
            "_dev", "dreg_", "prop hunt", "version)", "small battleground", "flight bounds", "qa and",
        ].iter().any(|p| l.contains(p))
}

fn parse_zones(raw: &str) -> Vec<Zone> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in raw.lines() {
        let Some(z) = parse_line(line) else { continue };
        if junk(&z.name) { continue; }
        let key = (z.name.clone(), z.lo, z.hi, z.terr.clone(), z.inst.clone());
        if seen.insert(key) { out.push(z); }
    }
    out.sort_by(|a, b| (a.lo, &a.name).cmp(&(b.lo, &b.name)));
    out
}

fn load_zones() -> Vec<Zone> {
    let z = parse_zones(include_str!("zones.tsv"));
    if z.is_empty() { parse_zones(FALLBACK_ZONES) } else { z }
}

// ---------- quests ----------
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
enum Tier { Normal, Elite, Dungeon, Raid, WorldBoss }
impl Tier {
    const ALL: [Tier; 5] = [Tier::Normal, Tier::Elite, Tier::Dungeon, Tier::Raid, Tier::WorldBoss];
    fn name(self) -> &'static str { ["Normal", "Elite", "Dungeon", "Raid", "World Boss"][self as usize] }
    fn unlock(self) -> u32 { [1, 5, 10, 20, 40][self as usize] }
    fn mult(self) -> u32 { [1, 2, 4, 8, 16][self as usize] }
    fn loot(self) -> u32 { [25, 50, 100, 100, 100][self as usize] }
}

#[derive(Serialize, Deserialize, Clone)]
struct Quest {
    title: String, giver: String, tier: Tier, cat: usize, goal: u32, progress: u32,
    chain: bool, part: u32,
    #[serde(default)] zone: String,
    #[serde(default)] bonus: u32,
}
impl Quest {
    fn display(&self) -> String {
        if self.chain { format!("{} (Part {})", self.title, self.part) } else { self.title.clone() }
    }
}

// ---------- items ----------
#[derive(Serialize, Deserialize, Clone)]
struct Item {
    name: String,
    #[serde(alias = "rarity")]
    quality: usize,
    ilvl: u32,
    #[serde(default)] slot: usize,
    #[serde(default)] kind: Kind,
    #[serde(default)] wt: Option<Wt>,
    #[serde(default)] hands: Hands,
    #[serde(default)] stats: Vec<(usize, u32)>,
}

fn qspan(q: usize, text: &str) -> String {
    let t = glib::markup_escape_text(text);
    if q == 1 { t.to_string() } else { format!("<span foreground='{}'>{}</span>", QCOL[q.min(6)], t) }
}

impl Item {
    fn is_two(&self) -> bool { self.kind == Kind::Weapon && self.hands == Hands::Two }

    fn type_label(&self) -> String {
        match self.kind {
            Kind::Weapon => format!("{} {}", self.hands.name(), self.wt.map_or("Weapon", |w| w.name())),
            Kind::Shield => "Shield".to_string(),
            Kind::Held => "Held In Off-hand".to_string(),
            Kind::Cosmetic => format!("Cosmetic {}", SLOTS[self.slot.min(7)]),
            Kind::Cloth | Kind::Leather | Kind::Mail | Kind::Plate => {
                format!("{} {}", self.kind.name(), SLOTS[self.slot.min(7)])
            }
            k => k.name().to_string(),
        }
    }

    fn label(&self) -> String {
        format!("[{}] {} (ilvl {} · {})", QUALITY[self.quality.min(6)], self.name, self.ilvl, self.type_label())
    }

    fn sell_value(&self) -> u32 {
        let f = QSELL[self.quality.min(6)];
        if f == 0 { 0 } else { (self.ilvl * f / 3).max(1) }
    }

    fn stats_inline(&self) -> String {
        if self.kind == Kind::Cosmetic { return "appearance only".to_string(); }
        if self.stats.is_empty() { return "no bonus stats".to_string(); }
        self.stats.iter().map(|&(s, v)| format!("+{v} {}", STAT_NAMES[s.min(7)])).collect::<Vec<_>>().join(", ")
    }

    fn tip(&self) -> String {
        let body = if self.kind == Kind::Cosmetic { "Appearance only — no stats".to_string() }
            else if self.stats.is_empty() { "No bonus stats".to_string() }
            else {
                self.stats.iter().map(|&(s, v)| format!("+{v} {}", STAT_NAMES[s.min(7)]))
                    .collect::<Vec<_>>().join("\n")
            };
        let sell = if self.sell_value() == 0 { "Soulbound — can't be sold".to_string() }
                   else { format!("Sells for {} gold", self.sell_value()) };
        format!("{}\n[{}] · ilvl {} · {}\n{}\n{}",
                self.name, QUALITY[self.quality.min(6)], self.ilvl, self.type_label(), body, sell)
    }
}

fn primary(class: usize, role: Role) -> usize {
    match CLASSES[class].name {
        "Mage" | "Priest" | "Warlock" | "Evoker" => 3,
        "Death Knight" | "Warrior" => 1,
        "Paladin" => if role == Healer { 3 } else { 1 },
        "Druid" | "Monk" | "Shaman" => if matches!(role, Healer | Ranged) { 3 } else { 2 },
        _ => 2,
    }
}

fn stat_value(s: usize, v: u32, prim: usize) -> u32 {
    match s { 0 => v, 1..=3 => if s == prim { v } else { v / 4 }, _ => v / 2 }
}

fn item_power(it: &Item, class: usize, spec: usize) -> u32 {
    if it.kind == Kind::Cosmetic { return 0; }
    let prim = primary(class, CLASSES[class].specs[spec].1);
    let w = if it.is_two() { 3 } else { 2 };
    let mut p = it.ilvl * QMULT[it.quality.min(6)] / 100 * w / 2;
    for &(s, v) in &it.stats { p += stat_value(s, v, prim); }
    p
}

/// Which equipment slots could this item go into for the class?
fn candidates(it: &Item, class: usize, two_now: bool) -> Vec<usize> {
    let c = &CLASSES[class];
    match it.kind {
        Kind::Cosmetic => vec![it.slot.min(7)],
        Kind::Cloth | Kind::Leather | Kind::Mail | Kind::Plate => {
            if it.kind.name() == c.armor { vec![it.slot.min(7)] } else { vec![] }
        }
        Kind::Cloak => vec![8],
        Kind::Necklace => vec![9],
        Kind::Ring => vec![10, 11],
        Kind::Trinket => vec![12, 13],
        Kind::Shield => if c.shield && !two_now { vec![S_OFF] } else { vec![] },
        Kind::Held => if c.held && !two_now { vec![S_OFF] } else { vec![] },
        Kind::Weapon => {
            let Some(w) = it.wt else { return vec![] };
            if !c.weapons.contains(&w) { return vec![]; }
            match it.hands {
                Hands::Two | Hands::Main => vec![S_MAIN],
                Hands::Off => if c.dual && !two_now { vec![S_OFF] } else { vec![] },
                Hands::One => {
                    let mut v = vec![S_MAIN];
                    if c.dual && !two_now { v.push(S_OFF); }
                    v
                }
            }
        }
    }
}

fn usable(it: &Item, class: usize) -> bool { !candidates(it, class, false).is_empty() }

// ---------- random item generation ----------
fn weapon_nouns(w: Wt, two: bool) -> &'static [&'static str] {
    match (w, two) {
        (Dagger, _) => N_DAGGER,
        (Sword, false) => N_SWORD1,
        (Sword, true) => N_SWORD2,
        (Axe, false) => N_AXE1,
        (Axe, true) => N_AXE2,
        (Mace, false) => N_MACE1,
        (Mace, true) => N_MACE2,
        (Fist, _) => N_FIST,
        (Polearm, _) => N_POLE,
        (Staff, _) => N_STAFF,
        (Bow, _) => N_BOW,
        (Crossbow, _) => N_XBOW,
        (Gun, _) => N_GUN,
        (Wand, _) => N_WAND,
        (Warglaive, _) => N_GLAIVE,
    }
}

fn mats(kind: Kind, wt: Option<Wt>) -> (&'static [&'static str], &'static [&'static str]) {
    match kind {
        Kind::Cloth | Kind::Cloak | Kind::Cosmetic => (M_CLOTH, X_CLOTH),
        Kind::Leather => (M_LEATHER, X_LEATHER),
        Kind::Mail => (M_MAIL, X_MAIL),
        Kind::Plate => (M_PLATE, X_PLATE),
        Kind::Necklace | Kind::Ring => (M_JEWEL, X_JEWEL),
        Kind::Trinket => (M_TRINKET, X_TRINKET),
        Kind::Shield => (M_SHIELD, X_METAL),
        Kind::Held => (M_HELD, X_HELD),
        Kind::Weapon => match wt {
            Some(Bow | Staff | Wand) => (M_WOOD, X_WOOD),
            Some(Gun | Crossbow) => (M_TECH, X_TECH),
            _ => (M_METAL, X_METAL),
        },
    }
}

fn noun_for(look: Kind, slot: usize, wt: Option<Wt>, hands: Hands, ai: usize) -> &'static str {
    let list: &[&'static str] = match look {
        Kind::Cloth | Kind::Leather | Kind::Mail | Kind::Plate | Kind::Cosmetic => {
            ARMOR_NOUNS[slot.min(7)][ai.min(3)]
        }
        Kind::Cloak => N_CLOAK,
        Kind::Necklace => N_NECK,
        Kind::Ring => N_RING,
        Kind::Trinket => N_TRINKET,
        Kind::Shield => N_SHIELD,
        Kind::Held => N_HELD,
        Kind::Weapon => weapon_nouns(wt.unwrap_or(Sword), hands == Hands::Two),
    };
    ps(list)
}

fn legend_name(noun: &str) -> String {
    match rnd(3) {
        0 => format!("{}, {} {noun} of the {}{}", compound_name(), ps(LEG_ADJ), ps(WIND_A), ps(WIND_B)),
        1 => format!("{}, {} of {}", compound_name(), ps(LEG_PART), boss_name()),
        _ => elven_name(),
    }
}

/// Dynamic name generator. Returns (name, optional stat-suffix index).
fn gen_name(q: usize, kind: Kind, slot: usize, wt: Option<Wt>, hands: Hands, ai: usize) -> (String, Option<usize>) {
    let look = if kind == Kind::Cosmetic { ARMOR[ai.min(3)] } else { kind };
    let noun = noun_for(look, slot, wt, hands, ai);
    let (generic, special) = mats(look, wt);
    let mat = ps(generic);
    let weapon = kind == Kind::Weapon;
    if kind == Kind::Cosmetic {
        return (format!("{} {mat} {noun}", ps(COSM_ADJ)), None);
    }
    match q {
        0 => (format!("{} {noun}", ps(POOR_ADJ)), None),
        1 => {
            let pre = if rnd(100) < 35 { format!("{} ", ps(COMMON_ADJ)) } else { String::new() };
            (format!("{pre}{mat} {noun}"), None)
        }
        2 => {
            let s = rnd(SUFFIXES.len() as u32) as usize;
            (format!("{mat} {noun} {}", SUFFIXES[s].0), Some(s))
        }
        3 => {
            if rnd(100) < 55 {
                let s = rnd(SUFFIXES.len() as u32) as usize;
                (format!("{} {mat} {noun} {}", ps(RARE_ADJ), SUFFIXES[s].0), Some(s))
            } else {
                (format!("{} {mat} {noun}", ps(THEME_ADJ)), None)
            }
        }
        4 => match rnd(100) {
            0..=34 => (format!("{} {mat} {noun}", ps(THEME_ADJ)), None),
            35..=64 => (format!("{}'s {} {noun}", npc_name(), ps(special)), None),
            65..=84 if weapon => (compound_name(), None),
            _ => (format!("{noun} of the {} {}", ps(GROUP_ADJ), ps(GROUP_NOUN)), None),
        },
        5 => (legend_name(noun), None),
        _ => (format!("{} {mat} {noun}", ps(HEIR_ADJ)), None),
    }
}

fn roll_stats(q: usize, ilvl: u32, class: usize, spec: usize, suffix: Option<usize>) -> Vec<(usize, u32)> {
    let unit = ilvl * QSTAT[q.min(6)] / 100;
    let val = || (unit + rnd(3)).max(1);
    if let Some(s) = suffix {
        return SUFFIXES[s].1.iter().map(|&st| (st, val())).collect();
    }
    let n = match q { 0 | 1 => 0, 2 | 3 => 2, 4 => 3, 5 => 4, _ => 2 };
    if n == 0 { return vec![]; }
    let prim = primary(class, CLASSES[class].specs[spec].1);
    let mut stats: Vec<usize> = vec![if q == Q_HEIRLOOM { 0 } else { prim }];
    if q == Q_HEIRLOOM { stats.push(prim); }
    let pool = [0usize, 4, 5, 6, 7];
    while stats.len() < n {
        let s = pool[rnd(5) as usize];
        if !stats.contains(&s) { stats.push(s); }
    }
    stats.into_iter().map(|s| (s, val())).collect()
}

#[allow(clippy::too_many_arguments)]
fn build(class: usize, spec: usize, level: u32, kind: Kind, slot: usize, wt: Option<Wt>, hands: Hands, q: usize) -> Item {
    let ai = if kind == Kind::Cosmetic { rnd(4) as usize } else { kind.armor_idx().unwrap_or(0) };
    let ilvl = if kind == Kind::Cosmetic { 1 }
        else if q == Q_HEIRLOOM { level + 6 }
        else { (level as i32 + 2 + QILVL[q.min(6)] + rnd(3) as i32).max(1) as u32 };
    let (name, suffix) = gen_name(q, kind, slot, wt, hands, ai);
    let stats = if kind == Kind::Cosmetic { vec![] } else { roll_stats(q, ilvl, class, spec, suffix) };
    Item { name, quality: q, ilvl, slot, kind, wt, hands, stats }
}

fn pick_weapon(class: usize, spec: usize) -> (Wt, Hands) {
    let c = &CLASSES[class];
    let role = c.specs[spec].1;
    let mut pool: Vec<Wt> = c.weapons.to_vec();
    if c.name == "Hunter" {
        let want = role == Ranged;
        pool.retain(|w| w.ranged() == want);
    }
    let shield_tank = c.shield && role == Tank;
    if shield_tank {
        let ones: Vec<Wt> = pool.iter().copied().filter(|w| w.one()).collect();
        if !ones.is_empty() { pool = ones; }
    }
    let w = pool[rnd(pool.len() as u32) as usize];
    let one_variant = if rnd(100) < 15 { Hands::Main } else { Hands::One };
    let hands = if w.one() && w.two() {
        if !shield_tank && rnd(2) == 0 { Hands::Two } else { one_variant }
    } else if w.two() {
        Hands::Two
    } else {
        one_variant
    };
    (w, hands)
}

fn pick_offhand(class: usize, spec: usize) -> Option<(Kind, Option<Wt>, Hands)> {
    let c = &CLASSES[class];
    let role = c.specs[spec].1;
    let mut opts: Vec<u8> = vec![];
    if c.dual { opts.push(0); }
    if c.shield { opts.push(1); }
    if c.held { opts.push(2); }
    if c.shield && role == Tank { opts = vec![1]; }
    if opts.is_empty() { return None; }
    Some(match opts[rnd(opts.len() as u32) as usize] {
        0 => {
            let pool: Vec<Wt> = c.weapons.iter().copied().filter(|w| w.one() && !w.ranged() && *w != Wand).collect();
            let w = pool[rnd(pool.len() as u32) as usize];
            (Kind::Weapon, Some(w), if rnd(100) < 25 { Hands::Off } else { Hands::One })
        }
        1 => (Kind::Shield, None, Hands::One),
        _ => (Kind::Held, None, Hands::One),
    })
}

/// cat: 0..=7 body, 8 cloak, 9 necklace, 10/11 ring, 12/13 trinket, 14 weapon, 15 off-hand.
fn make_item(class: usize, spec: usize, level: u32, cat: usize, q: usize) -> Option<Item> {
    let own = armor_kind(CLASSES[class].armor);
    Some(match cat {
        0..=7 => build(class, spec, level, own, cat, None, Hands::One, q),
        8 => build(class, spec, level, Kind::Cloak, 8, None, Hands::One, q),
        9 => build(class, spec, level, Kind::Necklace, 9, None, Hands::One, q),
        10 | 11 => build(class, spec, level, Kind::Ring, 10, None, Hands::One, q),
        12 | 13 => build(class, spec, level, Kind::Trinket, 12, None, Hands::One, q),
        14 => {
            let (w, h) = pick_weapon(class, spec);
            build(class, spec, level, Kind::Weapon, S_MAIN, Some(w), h, q)
        }
        _ => {
            let (k, w, h) = pick_offhand(class, spec)?;
            build(class, spec, level, k, S_OFF, w, h, q)
        }
    })
}

fn random_cat() -> usize {
    match rnd(100) {
        0..=47 => rnd(8) as usize,
        48..=55 => 8,
        56..=63 => 9,
        64..=72 => 10,
        73..=80 => 12,
        81..=91 => S_MAIN,
        _ => S_OFF,
    }
}

fn drop_item(class: usize, spec: usize, level: u32, q: usize) -> Item {
    loop {
        if let Some(it) = make_item(class, spec, level, random_cat(), q) { return it; }
    }
}

fn cosmetic_item(class: usize, spec: usize, level: u32) -> Item {
    build(class, spec, level, Kind::Cosmetic, rnd(8) as usize, None, Hands::One, 1 + rnd(4) as usize)
}

/// Armor or weapon the class cannot use (vendor fodder).
fn unusable_item(class: usize, spec: usize, level: u32, q: usize) -> Item {
    let c = &CLASSES[class];
    let wrong_armor = |_: ()| {
        let own = armor_kind(c.armor);
        let others: Vec<Kind> = ARMOR.iter().copied().filter(|k| *k != own).collect();
        build(class, spec, level, *pick(&others), rnd(8) as usize, None, Hands::One, q)
    };
    if rnd(2) == 0 { return wrong_armor(()); }
    let bad: Vec<Wt> = ALL_WT.iter().copied().filter(|w| !c.weapons.contains(w)).collect();
    if bad.is_empty() { return wrong_armor(()); }
    let w = *pick(&bad);
    let hands = if w.two() && !w.one() { Hands::Two } else { Hands::One };
    build(class, spec, level, Kind::Weapon, S_MAIN, Some(w), hands, q)
}

fn roll_quality(level: u32, tier: Tier, cat: usize, bonus: u32) -> usize {
    let t = tier as u32;
    let high = t >= Tier::Raid as u32;
    let chance = if tier == Tier::WorldBoss { 30 } else { 15 };
    if level >= 40 && (high || bonus >= 50) && rnd(1000) < chance { return Q_LEGENDARY; }
    let craft = if cat > GATHER_MAX { 80 } else { 0 };
    let score = rnd(1000) + t * 80 + bonus * 2 + craft;
    if score >= 1000 && level >= 40 { 4 }
    else if score >= 800 && (level >= 20 || t >= Tier::Dungeon as u32) { 3 }
    else if score >= 480 { 2 }
    else if score >= 240 { 1 }
    else { 0 }
}

fn npc_quality(level: u32) -> usize {
    if rnd(100) < 4 { return Q_HEIRLOOM; }
    let r = rnd(100);
    let mut q = if r < 15 { 0 } else if r < 40 { 1 } else if r < 70 { 2 } else if r < 90 { 3 } else if r < 99 { 4 } else { 5 };
    if q >= 4 && level < 40 { q = 3; }
    if q == 3 && level < 20 { q = 2; }
    q
}

fn build_gear(class: usize, spec: usize, level: u32) -> Vec<Option<Item>> {
    let mut g: Vec<Option<Item>> = vec![None; SLOTS.len()];
    for slot in 0..S_MAIN {
        if rnd(100) < 90 {
            let cat = match slot { 11 => 10, 13 => 12, s => s };
            g[slot] = make_item(class, spec, level, cat, npc_quality(level));
        }
    }
    if let Some(mh) = make_item(class, spec, level, S_MAIN, npc_quality(level)) {
        let two = mh.is_two();
        g[S_MAIN] = Some(mh);
        if !two && rnd(100) < 90 {
            g[S_OFF] = make_item(class, spec, level, S_OFF, npc_quality(level));
        }
    }
    g
}

// ---------- hero ----------
#[derive(Serialize, Deserialize, Clone)]
struct LogEntry { time: String, cat: String, msg: String }

#[derive(Serialize, Deserialize)]
struct Hero {
    name: String, guild: String, realm: String, realm_tier: usize,
    faction: String, race: String, class: usize, spec: usize,
    level: u32, xp: u32, gold: u32, talents: u32, done: u32,
    honor: u32, wins: u32, losses: u32,
    zone: String, zone_inst: String,
    abilities: Vec<String>, gear: Vec<Option<Item>>, prof: Vec<u32>, achievements: Vec<String>,
    quests: Vec<Quest>, log: Vec<LogEntry>,
    #[serde(default)] bag: Vec<Item>,
}

#[derive(Serialize, Deserialize, Default)]
struct Save { heroes: Vec<Hero> }

impl Hero {
    fn new(name: String, guild: String, realm: &Realm, faction: &str, race: &str, class: usize, spec: usize) -> Self {
        Hero {
            name, guild, realm: realm.name.clone(), realm_tier: realm.tier,
            faction: faction.into(), race: race.into(), class, spec,
            level: 1, xp: 0, gold: 0, talents: 0, done: 0, honor: 0, wins: 0, losses: 0,
            zone: start_zone(race).into(), zone_inst: String::new(),
            abilities: vec![], gear: vec![None; SLOTS.len()], prof: vec![0; CATS.len() - 1],
            achievements: vec![], quests: vec![], log: vec![], bag: vec![],
        }
    }

    fn need(&self) -> u32 { 100 + self.level * 50 }

    fn two_now(&self) -> bool {
        self.gear.get(S_MAIN).and_then(|o| o.as_ref()).map_or(false, |i| i.is_two())
    }

    fn fighter(&self) -> Fighter {
        Fighter {
            name: self.name.clone(), faction: self.faction.clone(), race: self.race.clone(),
            class: self.class, spec: self.spec, level: self.level, gear: self.gear.clone(),
            abilities: self.abilities.clone(), talents: self.talents,
        }
    }

    fn add_log(&mut self, msgs: Vec<Msg>) {
        let t = now();
        for (c, s) in msgs {
            self.log.push(LogEntry { time: t.clone(), cat: c.into(), msg: s });
        }
        let n = self.log.len();
        if n > MAX_LOG { self.log.drain(..n - MAX_LOG); }
    }

    /// Equip into the best slot. Ok(displaced items) or Err(item) if not equipped.
    fn place(&mut self, it: Item, force: bool) -> Result<Vec<Item>, Item> {
        let cands = candidates(&it, self.class, self.two_now());
        if cands.is_empty() { return Err(it); }
        let (class, spec) = (self.class, self.spec);
        let pw = |o: &Option<Item>| o.as_ref().map_or(-1i64, |i| item_power(i, class, spec) as i64);
        let target = cands.iter().copied().min_by_key(|&s| pw(&self.gear[s])).unwrap_or(S_MAIN);
        let two = it.is_two();
        let old_p = if two {
            let (a, b) = (pw(&self.gear[S_MAIN]), pw(&self.gear[S_OFF]));
            if a < 0 && b < 0 { -1 } else { a.max(0) + b.max(0) }
        } else {
            pw(&self.gear[target])
        };
        if !force && item_power(&it, class, spec) as i64 <= old_p { return Err(it); }
        let mut out = vec![];
        if let Some(o) = self.gear[target].take() { out.push(o); }
        if two {
            if let Some(o) = self.gear[S_OFF].take() { out.push(o); }
        }
        self.gear[target] = Some(it);
        Ok(out)
    }

    fn stash(&mut self, it: Item, msgs: &mut Vec<Msg>) {
        if self.bag.len() < BAG_MAX || it.quality == Q_HEIRLOOM {
            self.bag.push(it);
        } else {
            let g = it.sell_value();
            self.gold += g;
            msgs.push(m("Loot", format!("🪙 Bag full — sold {} for {g} gold", it.label())));
        }
    }

    fn receive(&mut self, it: Item, msgs: &mut Vec<Msg>) {
        let label = it.label();
        if !usable(&it, self.class) {
            let g = it.sell_value();
            self.gold += g;
            msgs.push(m("Loot", format!("🪙 Can't use {label} — sold for {g} gold")));
            return;
        }
        match self.place(it, false) {
            Ok(old) => {
                msgs.push(m("Loot", format!("🎁 Equipped {label}")));
                for o in old { self.stash(o, msgs); }
            }
            Err(it) => {
                msgs.push(m("Loot", format!("🎒 Bag: {label}")));
                self.stash(it, msgs);
            }
        }
    }

    fn loot_one(&mut self, q: &Quest, msgs: &mut Vec<Msg>) {
        let qual = roll_quality(self.level, q.tier, q.cat, q.bonus);
        let (cl, sp, lv) = (self.class, self.spec, self.level);
        let it = match rnd(100) {
            0..=5 => unusable_item(cl, sp, lv, qual),
            6..=11 => cosmetic_item(cl, sp, lv),
            _ => drop_item(cl, sp, lv, qual),
        };
        self.receive(it, msgs);
    }

    fn refresh_heirlooms(&mut self) {
        let l = self.level + 6;
        for it in self.gear.iter_mut().flatten().chain(self.bag.iter_mut()) {
            if it.quality == Q_HEIRLOOM {
                it.ilvl = l;
                let v = l * QSTAT[Q_HEIRLOOM] / 100 + 1;
                for s in it.stats.iter_mut() { s.1 = v; }
            }
        }
    }

    fn turn_in(&mut self, q: &Quest) -> Vec<Msg> {
        let mult = q.tier.mult();
        let pct = 100 + q.bonus;
        let xp = 25 * q.goal * mult * pct / 100;
        let mut gold = q.goal * mult * 5 + rnd(10);
        let mut loot = q.tier.loot();
        if (1..=GATHER_MAX).contains(&q.cat) { gold += gold / 2; } // gathering sells ore/herbs
        if q.cat > GATHER_MAX { loot += 30; }                      // crafting improves loot
        gold = gold * pct / 100;
        if q.cat > 0 { self.prof[q.cat - 1] += q.goal; }

        self.xp += xp; self.gold += gold; self.done += 1;
        let mut msgs: Vec<Msg> = vec![m("Quest", format!("✔ Quest complete: {} → +{xp} XP, +{gold} gold", q.display()))];
        if q.bonus > 0 { msgs.push(m("Quest", format!("🗺 {} zone bonus: +{}%", q.zone, q.bonus))); }
        if q.cat > 0 { msgs.push(m("Quest", format!("🔨 {} skill +{}", CATS[q.cat], q.goal))); }

        let drops = if matches!(q.tier, Tier::Raid | Tier::WorldBoss) { 2 } else { 1 };
        for _ in 0..drops {
            if rnd(100) < loot { self.loot_one(q, &mut msgs); }
        }
        if self.done % 20 == 0 {
            let it = drop_item(self.class, self.spec, self.level, Q_HEIRLOOM);
            msgs.push(m("Loot", "🏺 Heirloom cache unlocked (every 20 quests)!"));
            self.receive(it, &mut msgs);
        }

        while self.xp >= self.need() && self.level < MAX_LEVEL {
            let n = self.need();
            self.xp -= n;
            self.level += 1;
            self.refresh_heirlooms();
            msgs.push(m("Level", format!("⬆ LEVEL UP! You are now level {}", self.level)));
            let l = self.level as usize;
            let ab = CLASSES[self.class].abilities;
            if l % 2 == 0 && l / 2 - 1 < ab.len() {
                let a = ab[l / 2 - 1].to_string();
                msgs.push(m("Level", format!("📖 New ability: {a}")));
                self.abilities.push(a);
            } else {
                self.talents += 1;
                msgs.push(m("Level", "✨ New talent point"));
            }
            for t in Tier::ALL {
                if t.unlock() == self.level { msgs.push(m("Level", format!("🔓 {} quests unlocked", t.name()))); }
            }
            if self.level == PROF_LEVEL { msgs.push(m("Level", "🔓 Professions unlocked")); }
        }
        self.check_achievements(&mut msgs);
        msgs
    }

    fn check_achievements(&mut self, msgs: &mut Vec<Msg>) {
        let list = [
            ("First Blood - 1 quest", self.done >= 1),
            ("Quest Veteran - 10 quests", self.done >= 10),
            ("Level 5", self.level >= 5),
            ("Level 10", self.level >= 10),
            ("Level 20", self.level >= 20),
            ("Level 40", self.level >= 40),
            ("Moneybags - 100 gold", self.gold >= 100),
            ("Epic Find", self.gear.iter().flatten().any(|i| i.quality >= 4 && i.quality != Q_HEIRLOOM)),
            ("Legendary Find", self.gear.iter().flatten().chain(self.bag.iter()).any(|i| i.quality == Q_LEGENDARY)),
            ("Heirloom Collector", self.gear.iter().flatten().chain(self.bag.iter()).any(|i| i.quality == Q_HEIRLOOM)),
            ("Fully Geared - every slot filled", self.gear.iter().all(|g| g.is_some())),
            ("Master Crafter - 25 skill", self.prof.iter().any(|&p| p >= 25)),
            ("Duelist - 1 PvP win", self.wins >= 1),
            ("Gladiator - 10 PvP wins", self.wins >= 10),
        ];
        for (n, ok) in list {
            if ok && !self.achievements.iter().any(|a| a == n) {
                self.achievements.push(n.to_string());
                msgs.push(m("Achievement", format!("🏆 Achievement: {n}")));
            }
        }
    }
}

// ---------- combat ----------
#[derive(Clone, Copy)]
struct Stats { hp: u32, atk: u32, def: u32, crit: u32, heal: u32 }

#[derive(Clone)]
struct Fighter {
    name: String, faction: String, race: String, class: usize, spec: usize, level: u32,
    gear: Vec<Option<Item>>, abilities: Vec<String>, talents: u32,
}

impl Fighter {
    fn role(&self) -> Role { CLASSES[self.class].specs[self.spec].1 }

    fn ilvl_avg(&self) -> u32 {
        let v: Vec<u32> = self.gear.iter().flatten().filter(|i| i.kind != Kind::Cosmetic).map(|i| i.ilvl).collect();
        if v.is_empty() { 0 } else { v.iter().sum::<u32>() / v.len() as u32 }
    }

    fn setup(&self) -> &'static str {
        match (&self.gear[S_MAIN], &self.gear[S_OFF]) {
            (None, None) => "Unarmed",
            (Some(mh), _) if mh.is_two() => "Two-handed",
            (_, Some(o)) if o.kind == Kind::Weapon => "Dual wield",
            (_, Some(o)) if o.kind == Kind::Shield => "One-hand + Shield",
            (_, Some(_)) => "One-hand + Off-hand",
            (Some(_), None) => "One-handed",
        }
    }

    fn stats(&self) -> Stats {
        let role = self.role();
        let prim = primary(self.class, role);
        let mut gp = 0u32;
        let mut qsum = 0u32;
        let mut tot = [0u32; 8];
        let mut shield_def = 0u32;
        for it in self.gear.iter().flatten() {
            if it.kind == Kind::Cosmetic { continue; }
            let w = if it.is_two() { 3 } else { 2 };
            gp += it.ilvl * QMULT[it.quality.min(6)] / 100 * w / 2;
            qsum += it.quality.min(5) as u32;
            for &(s, v) in &it.stats { tot[s.min(7)] += v; }
            if it.kind == Kind::Shield { shield_def += it.ilvl / 2; }
        }
        let g = gp * 6 / 14;
        let (hp_m, atk_m, def_m) = match role {
            Tank => (150, 70, 150),
            Healer => (110, 60, 100),
            Melee => (110, 120, 90),
            Ranged => (90, 125, 80),
        };
        let armor = match CLASSES[self.class].armor { "Plate" => 130, "Mail" => 115, "Leather" => 100, _ => 85 };
        let off_prim = tot[1] + tot[2] + tot[3] - tot[prim];
        let hp = (120 + self.level * 25 + g * 4) * hp_m / 100 + tot[0] * 5 + tot[7] * 2;
        Stats {
            hp,
            atk: (10 + self.level * 3 + g) * atk_m / 100
                + self.abilities.len() as u32 * 3 + self.talents * 4
                + tot[prim] + off_prim / 4 + tot[4] / 2 + tot[7] / 3,
            def: (self.level + g / 2) * def_m / 100 * armor / 100 + tot[6] / 2 + shield_def,
            crit: (5 + self.talents + qsum / 3 + tot[5] / 6).min(40),
            heal: if role == Healer { hp * 6 / 100 } else { 0 },
        }
    }

    fn strike(&self, me: &Stats, foe: &Stats) -> (u32, bool, String) {
        let name = if self.abilities.is_empty() { "Attack".to_string() } else { pick(&self.abilities).clone() };
        let base = me.atk * (85 + rnd(31)) / 100;
        let mut dmg = (base * 100 / (100 + foe.def / 2)).max(1);
        let crit = rnd(100) < me.crit;
        if crit { dmg *= 2; }
        (dmg, crit, name)
    }

    fn summary(&self) -> String {
        let c = &CLASSES[self.class];
        let s = self.stats();
        format!("Lv {} {} {} ({}) · {} · {} · ilvl {} · HP {} ATK {} DEF {}",
                self.level, self.race, c.name, c.specs[self.spec].0, self.role().name(), c.armor,
                self.ilvl_avg(), s.hp, s.atk, s.def)
    }

    fn detail(&self) -> String {
        let ab = if self.abilities.is_empty() { "—".to_string() } else { self.abilities.join(", ") };
        let mut t = format!("Abilities: {ab}\nTalents: {}\nSetup: {}\n", self.talents, self.setup());
        for (i, slot) in SLOTS.iter().enumerate() {
            match &self.gear[i] {
                Some(it) => t += &format!("{slot}: [{}] {} (ilvl {})\n", QUALITY[it.quality.min(6)], it.name, it.ilvl),
                None => t += &format!("{slot}: —\n"),
            }
        }
        t
    }
}

fn fight(a: &Fighter, b: &Fighter) -> (bool, Vec<String>) {
    let (sa, sb) = (a.stats(), b.stats());
    let (mut ha, mut hb) = (sa.hp as i64, sb.hp as i64);
    let mut lines = vec![];
    for r in 1..=40 {
        let (da, ca, na) = a.strike(&sa, &sb);
        hb -= da as i64;
        let mut line = format!("R{r}: {} {na} → {da}{}", a.name, if ca { " CRIT" } else { "" });
        if hb <= 0 { lines.push(line); return (true, lines); }
        let (db, cb, nb) = b.strike(&sb, &sa);
        ha -= db as i64;
        line += &format!(" | {} {nb} → {db}{}", b.name, if cb { " CRIT" } else { "" });
        lines.push(line);
        if ha <= 0 { return (false, lines); }
        ha = (ha + sa.heal as i64).min(sa.hp as i64);
        hb = (hb + sb.heal as i64).min(sb.hp as i64);
    }
    (ha * 1000 / sa.hp as i64 >= hb * 1000 / sb.hp as i64, lines)
}

fn gen_player(near: u32) -> Fighter {
    let faction = if rnd(2) == 0 { "Horde" } else { "Alliance" };
    let pool = races(faction);
    let level = if rnd(10) < 7 {
        (near as i32 + rnd(17) as i32 - 8).clamp(1, MAX_LEVEL as i32) as u32
    } else {
        1 + rnd(MAX_LEVEL)
    };
    let class = rnd(CLASSES.len() as u32) as usize;
    let spec = rnd(CLASSES[class].specs.len() as u32) as usize;
    let gear = build_gear(class, spec, level);
    let n = 1 + rnd((level / 2).clamp(1, 5)) as usize;
    Fighter {
        name: player_name(), faction: faction.into(), race: pick(&pool).to_string(), class, spec, level, gear,
        abilities: CLASSES[class].abilities[..n].iter().map(|s| s.to_string()).collect(),
        talents: rnd(level / 3 + 1),
    }
}

// ---------- persistence ----------
fn save_path() -> PathBuf {
    let d = glib::user_data_dir().join("wow-todo");
    let _ = std::fs::create_dir_all(&d);
    d.join("characters.json")
}

/// Convert gear saved by the old 6-slot system into the new equipment system.
fn migrate(s: &mut Save) {
    let map: [usize; 6] = [0, 1, 2, 6, S_MAIN, 12];
    for h in &mut s.heroes {
        if h.gear.len() == SLOTS.len() { continue; }
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
}

fn load() -> Save {
    let mut s: Save = std::fs::read_to_string(save_path()).ok()
        .and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    migrate(&mut s);
    s
}
fn persist(s: &Save) {
    if let Ok(j) = serde_json::to_string_pretty(s) { let _ = std::fs::write(save_path(), j); }
}

// ---------- UI ----------
fn pad(w: &impl IsA<gtk::Widget>, n: i32) {
    w.set_margin_top(n); w.set_margin_bottom(n); w.set_margin_start(n); w.set_margin_end(n);
}
fn labeled(text: &str, w: &impl IsA<gtk::Widget>) -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.0);
    l.add_css_class("dim-label");
    b.append(&l); b.append(w);
    b
}
fn set_options(dd: &gtk::DropDown, items: &[String]) {
    let same = dd.model().and_then(|mo| mo.downcast::<gtk::StringList>().ok()).map_or(false, |sl| {
        sl.n_items() as usize == items.len()
            && items.iter().enumerate()
                .all(|(i, s)| sl.string(i as u32).map_or(false, |g| g.as_str() == s.as_str()))
    });
    if same { return; }
    let sel = dd.selected();
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    dd.set_model(Some(&gtk::StringList::new(&refs)));
    dd.set_selected(if sel == gtk::INVALID_LIST_POSITION { 0 } else { sel.min(items.len() as u32 - 1) });
}

struct Ui {
    save: RefCell<Save>,
    active: Cell<Option<usize>>,
    confirm_del: Cell<Option<usize>>,
    realms: Vec<Realm>,
    zones: Vec<Zone>,
    opps: RefCell<Vec<Fighter>>,
    stack: gtk::Stack,
    sel_list: gtk::Box,
    title: gtk::Label, xp_bar: gtk::ProgressBar, stats: gtk::Label,
    giver: gtk::Label, entry: gtk::Entry, goal: gtk::SpinButton,
    tier: gtk::DropDown, cat: gtk::DropDown, chain: gtk::CheckButton,
    qlist: gtk::Box, status: gtk::Label,
    zsearch: gtk::Entry, zkind: gtk::DropDown, zcount: gtk::Label, zlist: gtk::Box,
    pvp_head: gtk::Label, pvp_result: gtk::Label, pvp_list: gtk::Box,
    log_filter: gtk::DropDown, log_view: gtk::TextView,
    sheet: gtk::Label,
    eq_sum: gtk::Label, eq_list: gtk::Box, bag_head: gtk::Label, bag_list: gtk::Box,
}

impl Ui {
    fn build(app: &gtk::Application) -> Rc<Self> {
        let ho = gtk::Orientation::Horizontal;
        let ve = gtk::Orientation::Vertical;

        // ----- character select screen
        let sel_list = gtk::Box::new(ve, 8);
        let sel_scroll = gtk::ScrolledWindow::builder().vexpand(true).min_content_height(320).child(&sel_list).build();
        let sel_new = gtk::Button::with_label("+ Create new character");
        sel_new.add_css_class("suggested-action");
        let sel_title = gtk::Label::new(None);
        sel_title.set_markup(&format!("<span size='xx-large' weight='bold'>{}</span>", glib::markup_escape_text(APP_TITLE)));
        let select = gtk::Box::new(ve, 12);
        pad(&select, 24);
        select.set_halign(gtk::Align::Center);
        select.set_width_request(620);
        select.append(&sel_title);
        select.append(&gtk::Label::new(Some("Choose a character to continue your adventure")));
        select.append(&sel_scroll);
        select.append(&sel_new);

        // ----- game header
        let title = gtk::Label::new(None); title.set_xalign(0.0); title.set_hexpand(true);
        let save_btn = gtk::Button::with_label("💾 Save"); save_btn.set_valign(gtk::Align::Start);
        let switch_btn = gtk::Button::with_label("Switch character"); switch_btn.set_valign(gtk::Align::Start);
        let head = gtk::Box::new(ho, 8);
        head.append(&title); head.append(&save_btn); head.append(&switch_btn);
        let xp_bar = gtk::ProgressBar::new(); xp_bar.set_show_text(true);
        let stats = gtk::Label::new(None); stats.set_xalign(0.0); stats.set_wrap(true);

        // ----- quests tab
        let giver = gtk::Label::new(None); giver.set_xalign(0.0);
        let entry = gtk::Entry::new(); entry.set_hexpand(true);
        entry.set_placeholder_text(Some("e.g. Kobolds slain / Write report"));
        let goal = gtk::SpinButton::with_range(1.0, 100.0, 1.0);
        goal.set_tooltip_text(Some("How many steps: Kill 10 Kobolds = 10"));
        let tier = gtk::DropDown::from_strings(&["Normal"]);
        tier.set_tooltip_text(Some("Difficulty = how big the task is. Bigger tasks pay more XP, gold and loot and unlock as you level."));
        let cat = gtk::DropDown::from_strings(&["Adventure"]);
        cat.set_tooltip_text(Some("Profession = what kind of task this is. Completing it levels that skill (unlocks at level 5)."));
        let chain = gtk::CheckButton::with_label("Follow-up");
        chain.set_tooltip_text(Some("Turning in spawns a slightly harder follow-up quest"));
        let add_btn = gtk::Button::with_label("Accept quest");
        add_btn.add_css_class("suggested-action");
        add_btn.set_valign(gtk::Align::End);
        let add_row = gtk::Box::new(ho, 8);
        add_row.append(&labeled("Quest", &entry));
        add_row.append(&labeled("Steps", &goal));
        add_row.append(&labeled("Difficulty", &tier));
        add_row.append(&labeled("Profession", &cat));
        add_row.append(&labeled("Chain", &chain));
        add_row.append(&add_btn);
        let hint = gtk::Label::new(Some(
            "Difficulty = task size (Normal · Elite Lv5 · Dungeon Lv10 · Raid Lv20 · World Boss Lv40; more XP/loot). \
             Profession = task type (levels a gathering/crafting skill, unlocks Lv5). Chain = spawns a harder follow-up.",
        ));
        hint.set_xalign(0.0); hint.set_wrap(true); hint.add_css_class("dim-label");
        let qlist = gtk::Box::new(ve, 6);
        let qscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&qlist).build();
        let status = gtk::Label::new(None); status.set_xalign(0.0); status.set_wrap(true);
        let quests_page = gtk::Box::new(ve, 8);
        pad(&quests_page, 10);
        for w in [giver.upcast_ref::<gtk::Widget>(), add_row.upcast_ref(), hint.upcast_ref(), qscroll.upcast_ref(), status.upcast_ref()] {
            quests_page.append(w);
        }

        // ----- zones tab
        let zsearch = gtk::Entry::new(); zsearch.set_hexpand(true);
        zsearch.set_placeholder_text(Some("Search zones…"));
        let zkind = gtk::DropDown::from_strings(&ZONE_KINDS);
        let zcount = gtk::Label::new(None); zcount.set_xalign(0.0); zcount.add_css_class("dim-label");
        let zlist = gtk::Box::new(ve, 4);
        let zscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&zlist).build();
        let zrow = gtk::Box::new(ho, 8);
        zrow.append(&zsearch); zrow.append(&zkind);
        let zones_page = gtk::Box::new(ve, 8);
        pad(&zones_page, 10);
        zones_page.append(&zrow); zones_page.append(&zcount); zones_page.append(&zscroll);

        // ----- pvp tab
        let pvp_head = gtk::Label::new(None); pvp_head.set_xalign(0.0); pvp_head.set_wrap(true); pvp_head.set_hexpand(true);
        let pvp_refresh = gtk::Button::with_label("🔄 New players"); pvp_refresh.set_valign(gtk::Align::Start);
        let pvp_result = gtk::Label::new(None); pvp_result.set_xalign(0.0); pvp_result.set_wrap(true);
        let pvp_list = gtk::Box::new(ve, 6);
        let pscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&pvp_list).build();
        let prow = gtk::Box::new(ho, 8);
        prow.append(&pvp_head); prow.append(&pvp_refresh);
        let pvp_page = gtk::Box::new(ve, 8);
        pad(&pvp_page, 10);
        pvp_page.append(&prow); pvp_page.append(&pvp_result); pvp_page.append(&pscroll);

        // ----- equipment tab
        let eq_sum = gtk::Label::new(None); eq_sum.set_xalign(0.0); eq_sum.set_wrap(true);
        let eq_list = gtk::Box::new(ve, 4);
        let eq_title = gtk::Label::new(None); eq_title.set_markup("<b>Equipped</b>"); eq_title.set_xalign(0.0);
        let bag_head = gtk::Label::new(None); bag_head.set_xalign(0.0); bag_head.set_hexpand(true);
        let sell_junk = gtk::Button::with_label("💰 Sell junk (Poor–Uncommon)");
        let bag_row = gtk::Box::new(ho, 8);
        bag_row.append(&bag_head); bag_row.append(&sell_junk);
        let bag_list = gtk::Box::new(ve, 4);
        let guide = gtk::Expander::new(Some("Quality guide — where items come from"));
        let guide_lbl = gtk::Label::new(Some(QUALITY_GUIDE));
        guide_lbl.set_xalign(0.0); guide_lbl.set_wrap(true);
        guide.set_child(Some(&guide_lbl));
        let eq_inner = gtk::Box::new(ve, 8);
        pad(&eq_inner, 10);
        eq_inner.append(&eq_sum); eq_inner.append(&guide); eq_inner.append(&eq_title);
        eq_inner.append(&eq_list); eq_inner.append(&bag_row); eq_inner.append(&bag_list);
        let eq_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&eq_inner).build();

        // ----- logs tab
        let log_filter = gtk::DropDown::from_strings(&LOG_FILTERS);
        let log_clear = gtk::Button::with_label("Clear logs");
        let spacer = gtk::Box::new(ho, 0); spacer.set_hexpand(true);
        let lrow = gtk::Box::new(ho, 8);
        lrow.append(&gtk::Label::new(Some("Filter:")));
        lrow.append(&log_filter); lrow.append(&spacer); lrow.append(&log_clear);
        let log_view = gtk::TextView::new();
        log_view.set_editable(false); log_view.set_cursor_visible(false);
        log_view.set_monospace(true); log_view.set_wrap_mode(gtk::WrapMode::WordChar);
        pad(&log_view, 8);
        let lscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&log_view).build();
        let logs_page = gtk::Box::new(ve, 8);
        pad(&logs_page, 10);
        logs_page.append(&lrow); logs_page.append(&lscroll);

        // ----- character tab
        let sheet = gtk::Label::new(None); sheet.set_xalign(0.0); sheet.set_yalign(0.0); sheet.set_wrap(true);
        pad(&sheet, 10);
        let sheet_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&sheet).build();

        let nb = gtk::Notebook::new();
        nb.set_vexpand(true);
        nb.append_page(&quests_page, Some(&gtk::Label::new(Some("📜 Quests"))));
        nb.append_page(&zones_page, Some(&gtk::Label::new(Some("🗺 Zones"))));
        nb.append_page(&pvp_page, Some(&gtk::Label::new(Some("⚔ Realm PvP"))));
        nb.append_page(&eq_scroll, Some(&gtk::Label::new(Some("🎒 Equipment"))));
        nb.append_page(&sheet_scroll, Some(&gtk::Label::new(Some("🧙 Character"))));
        nb.append_page(&logs_page, Some(&gtk::Label::new(Some("📋 Logs"))));

        let game = gtk::Box::new(ve, 10);
        pad(&game, 12);
        game.append(&head); game.append(&xp_bar); game.append(&stats); game.append(&nb);

        let stack = gtk::Stack::new();
        stack.add_named(&select, Some("select"));
        stack.add_named(&game, Some("game"));
        let win = gtk::ApplicationWindow::builder().application(app)
            .title(APP_TITLE).default_width(960).default_height(840).build();
        win.set_child(Some(&stack));

        let ui = Rc::new(Ui {
            save: RefCell::new(load()), active: Cell::new(None), confirm_del: Cell::new(None),
            realms: make_realms(), zones: load_zones(), opps: RefCell::new(vec![]),
            stack, sel_list, title, xp_bar, stats, giver, entry, goal, tier, cat, chain, qlist, status,
            zsearch, zkind, zcount, zlist, pvp_head, pvp_result, pvp_list, log_filter, log_view, sheet,
            eq_sum, eq_list, bag_head, bag_list,
        });

        { let u = ui.clone(); sel_new.connect_clicked(move |_| u.show_create()); }
        { let u = ui.clone(); add_btn.connect_clicked(move |_| u.add_quest()); }
        { let u = ui.clone(); ui.entry.connect_activate(move |_| u.add_quest()); }
        { let u = ui.clone(); save_btn.connect_clicked(move |_| u.finish(vec![m("System", "💾 Character saved.")])); }
        { let u = ui.clone(); switch_btn.connect_clicked(move |_| u.show_select()); }
        { let u = ui.clone(); pvp_refresh.connect_clicked(move |_| u.new_opponents()); }
        { let u = ui.clone(); log_clear.connect_clicked(move |_| u.clear_logs()); }
        { let u = ui.clone(); sell_junk.connect_clicked(move |_| u.sell_junk()); }
        { let u = ui.clone(); ui.zsearch.connect_changed(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.zkind.connect_selected_notify(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.log_filter.connect_selected_notify(move |_| u.refresh_logs()); }

        win.present();
        ui
    }

    fn start(self: &Rc<Self>) {
        self.stack.add_named(&create_screen(self), Some("create"));
        if self.save.borrow().heroes.is_empty() { self.show_create(); } else { self.show_select(); }
    }

    fn show_create(self: &Rc<Self>) { self.stack.set_visible_child_name("create"); }

    fn show_select(self: &Rc<Self>) {
        persist(&self.save.borrow());
        self.active.set(None);
        self.confirm_del.set(None);
        self.refresh_select();
        self.stack.set_visible_child_name("select");
    }

    fn refresh_select(self: &Rc<Self>) {
        while let Some(c) = self.sel_list.first_child() { self.sel_list.remove(&c); }
        let s = self.save.borrow();
        if s.heroes.is_empty() {
            self.sel_list.append(&gtk::Label::new(Some("No saved characters yet — create one!")));
        }
        for (i, hero) in s.heroes.iter().enumerate() {
            let c = &CLASSES[hero.class];
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            pad(&row, 8);
            let info = gtk::Label::new(None);
            info.set_xalign(0.0); info.set_hexpand(true);
            info.set_markup(&format!(
                "<b>{}</b>\nLevel {} {} {} ({}) · <span foreground='{}'>{}</span> · {}\n📍 {} · 💰 {} · 🏆 {} achievements",
                glib::markup_escape_text(hero.name.as_str()), hero.level, hero.race, c.name, c.specs[hero.spec].0,
                fcol(&hero.faction), hero.faction, glib::markup_escape_text(hero.realm.as_str()),
                glib::markup_escape_text(hero.zone.as_str()), hero.gold, hero.achievements.len()));
            let play = gtk::Button::with_label("▶ Play");
            play.add_css_class("suggested-action"); play.set_valign(gtk::Align::Center);
            let confirming = self.confirm_del.get() == Some(i);
            let del = gtk::Button::with_label(if confirming { "Confirm delete?" } else { "Delete" });
            if confirming { del.add_css_class("destructive-action"); }
            del.set_valign(gtk::Align::Center);
            { let u = self.clone(); play.connect_clicked(move |_| u.enter(i)); }
            { let u = self.clone(); del.connect_clicked(move |_| u.delete_click(i)); }
            row.append(&info); row.append(&play); row.append(&del);
            let f = gtk::Frame::new(None);
            f.set_child(Some(&row));
            self.sel_list.append(&f);
        }
    }

    fn delete_click(self: &Rc<Self>, i: usize) {
        if self.confirm_del.get() == Some(i) {
            self.confirm_del.set(None);
            {
                let mut s = self.save.borrow_mut();
                if i < s.heroes.len() { s.heroes.remove(i); }
            }
            persist(&self.save.borrow());
        } else {
            self.confirm_del.set(Some(i));
        }
        self.refresh_select();
    }

    fn enter(self: &Rc<Self>, i: usize) {
        self.confirm_del.set(None);
        let (realm, pop) = {
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(i) else { return };
            (h.realm.clone(), self.realms.get(h.realm_tier).map_or(0, |r| r.pop))
        };
        self.active.set(Some(i));
        self.new_opponents();
        self.stack.set_visible_child_name("game");
        self.finish(vec![m("System", format!("🔑 Entered realm {realm} ({} players online)", commas(pop)))]);
    }

    fn finish(self: &Rc<Self>, msgs: Vec<Msg>) {
        if let Some(a) = self.active.get() {
            if let Some(h) = self.save.borrow_mut().heroes.get_mut(a) { h.add_log(msgs); }
        }
        persist(&self.save.borrow());
        self.refresh();
    }

    fn clear_logs(self: &Rc<Self>) {
        if let Some(a) = self.active.get() {
            if let Some(h) = self.save.borrow_mut().heroes.get_mut(a) { h.log.clear(); }
        }
        self.finish(vec![m("System", "🧹 Logs cleared.")]);
    }

    fn refresh(self: &Rc<Self>) {
        let Some(a) = self.active.get() else { return };
        {
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(a) else { return };
            let c = &CLASSES[h.class];
            let (spec_name, role) = c.specs[h.spec];
            let f = h.fighter();
            let st = f.stats();

            let guild = if h.guild.is_empty() { String::new() }
                        else { format!(" · &lt;{}&gt;", glib::markup_escape_text(h.guild.as_str())) };
            self.title.set_markup(&format!(
                "<span size='x-large' weight='bold'>{}</span>{guild}\nLevel {} {} {} ({} · {}) · <span foreground='{}'>{}</span> · {}\n📍 {}",
                glib::markup_escape_text(h.name.as_str()), h.level, h.race, c.name, spec_name, role.name(),
                fcol(&h.faction), h.faction, glib::markup_escape_text(h.realm.as_str()),
                glib::markup_escape_text(h.zone.as_str())));

            self.xp_bar.set_fraction((h.xp as f64 / h.need() as f64).min(1.0));
            self.xp_bar.set_text(Some(&format!("{} / {} XP", h.xp, h.need())));
            let ab = if h.abilities.is_empty() { "—".to_string() } else { h.abilities.join(", ") };
            self.stats.set_text(&format!(
                "💰 {} gold   ✨ Talents: {}   🎖 Honor: {}   📖 Abilities: {}", h.gold, h.talents, h.honor, ab));

            self.giver.set_text(&format!("Find an NPC in {} and accept a quest:", h.zone));
            let tl: Vec<String> = Tier::ALL.iter().map(|t| {
                if t.unlock() <= h.level { format!("{} · x{} rewards", t.name(), t.mult()) }
                else { format!("{} 🔒 Lv {}", t.name(), t.unlock()) }
            }).collect();
            set_options(&self.tier, &tl);
            let cl: Vec<String> = CATS.iter().enumerate().map(|(i, n)| {
                if i == 0 || h.level >= PROF_LEVEL { format!("{n} ({})", cat_kind(i)) }
                else { format!("{n} 🔒 Lv {PROF_LEVEL}") }
            }).collect();
            set_options(&self.cat, &cl);

            while let Some(ch) = self.qlist.first_child() { self.qlist.remove(&ch); }
            if h.quests.is_empty() { self.qlist.append(&gtk::Label::new(Some("No active quests."))); }
            for (i, q) in h.quests.iter().enumerate() {
                let fr = gtk::Frame::new(None);
                fr.set_child(Some(&self.quest_row(i, q)));
                self.qlist.append(&fr);
            }
            let tail: Vec<&str> = h.log.iter().rev().take(3).map(|e| e.msg.as_str()).collect();
            self.status.set_text(&tail.into_iter().rev().collect::<Vec<_>>().join("\n"));

            let pop = self.realms.get(h.realm_tier).map_or(0, |r| r.pop);
            self.pvp_head.set_text(&format!(
                "🌐 Realm {} — {} players online (showing {} nearby)\nYou: Lv {} · HP {} · ATK {} · DEF {} · Crit {}% · Heal {}/round · {}W-{}L · {} honor",
                h.realm, commas(pop), PVP_SHOWN, h.level, st.hp, st.atk, st.def, st.crit, st.heal, h.wins, h.losses, h.honor));

            let mut t = String::from("<b>Combat stats</b>\n");
            t += &format!("HP {} · ATK {} · DEF {} · Crit {}% · Heal {}/round · avg ilvl {}\n", st.hp, st.atk, st.def, st.crit, st.heal, f.ilvl_avg());
            t += &format!("{} armor · {} · {} (see the Equipment tab)\n\n", c.armor, role.name(), f.setup());
            t += "<b>Professions</b>\n";
            let profs: Vec<String> = h.prof.iter().enumerate().filter(|(_, p)| **p > 0)
                .map(|(i, p)| format!("{}: {}", CATS[i + 1], p)).collect();
            t += &if profs.is_empty() { "—".to_string() } else { profs.join("\n") };
            t += "\n\n<b>Achievements</b>\n";
            t += &if h.achievements.is_empty() { "—".to_string() } else { h.achievements.join("\n") };
            self.sheet.set_markup(&t);
        }
        self.refresh_equipment();
        self.refresh_zones();
        self.refresh_logs();
    }

    fn refresh_equipment(self: &Rc<Self>) {
        while let Some(c) = self.eq_list.first_child() { self.eq_list.remove(&c); }
        while let Some(c) = self.bag_list.first_child() { self.bag_list.remove(&c); }
        let Some(a) = self.active.get() else { return };
        let s = self.save.borrow();
        let Some(h) = s.heroes.get(a) else { return };
        let f = h.fighter();
        let cls = &CLASSES[h.class];
        let weapons: Vec<&str> = cls.weapons.iter().map(|w| w.name()).collect();
        let filled = h.gear.iter().flatten().count();
        self.eq_sum.set_text(&format!(
            "Avg ilvl {} · {} · {}/{} slots filled\n{} armor · {}\nWeapons: {}",
            f.ilvl_avg(), f.setup(), filled, SLOTS.len(), cls.armor, offhand_text(cls), weapons.join(", ")));

        for (i, slot) in SLOTS.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::new(None);
            lbl.set_xalign(0.0); lbl.set_hexpand(true);
            match &h.gear[i] {
                Some(it) => {
                    let tl = it.type_label();
                    let si = it.stats_inline();
                    lbl.set_markup(&format!(
                        "<b>{slot}</b>: {}\n<small>ilvl {} · {} · {}</small>",
                        qspan(it.quality, it.name.as_str()), it.ilvl,
                        glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
                    lbl.set_tooltip_text(Some(&it.tip()));
                    let btn = gtk::Button::with_label("Unequip");
                    btn.set_valign(gtk::Align::Center);
                    { let u = self.clone(); btn.connect_clicked(move |_| u.unequip(i)); }
                    row.append(&lbl); row.append(&btn);
                }
                None => {
                    lbl.set_markup(&format!("<b>{slot}</b>: —"));
                    lbl.add_css_class("dim-label");
                    row.append(&lbl);
                }
            }
            self.eq_list.append(&row);
        }

        self.bag_head.set_text(&format!("🎒 Bag ({}/{})", h.bag.len(), BAG_MAX));
        if h.bag.is_empty() { self.bag_list.append(&gtk::Label::new(Some("Bag is empty."))); }
        let two_now = h.two_now();
        let mut idx: Vec<usize> = (0..h.bag.len()).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(h.bag[i].quality));
        for i in idx {
            let it = &h.bag[i];
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::new(None);
            lbl.set_xalign(0.0); lbl.set_hexpand(true);
            let tl = it.type_label();
            let si = it.stats_inline();
            lbl.set_markup(&format!(
                "{}\n<small>ilvl {} · {} · {}</small>",
                qspan(it.quality, it.name.as_str()), it.ilvl,
                glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
            lbl.set_tooltip_text(Some(&it.tip()));
            let can = !candidates(it, h.class, two_now).is_empty();
            let eq = gtk::Button::with_label("Equip");
            eq.set_valign(gtk::Align::Center);
            eq.set_sensitive(can);
            if !can { eq.set_tooltip_text(Some("Your class can't use this, or a two-handed weapon blocks the off-hand.")); }
            { let u = self.clone(); eq.connect_clicked(move |_| u.equip(i)); }
            let v = it.sell_value();
            let sell = gtk::Button::with_label(&if v == 0 { "Bound".to_string() } else { format!("Sell {v}g") });
            sell.set_valign(gtk::Align::Center);
            sell.set_sensitive(v > 0);
            { let u = self.clone(); sell.connect_clicked(move |_| u.sell(i)); }
            row.append(&lbl); row.append(&eq); row.append(&sell);
            self.bag_list.append(&row);
        }
    }

    fn equip(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if i >= h.bag.len() { return; }
            let it = h.bag.remove(i);
            let label = it.label();
            match h.place(it, true) {
                Ok(old) => {
                    msgs.push(m("Loot", format!("🛡 Equipped {label}")));
                    for o in old { h.stash(o, &mut msgs); }
                    h.check_achievements(&mut msgs);
                }
                Err(it) => {
                    msgs.push(m("Loot", format!("❌ Can't equip {label}")));
                    let at = i.min(h.bag.len());
                    h.bag.insert(at, it);
                }
            }
        }
        self.finish(msgs);
    }

    fn unequip(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if h.bag.len() >= BAG_MAX {
                msgs.push(m("Loot", "🎒 Bag is full — sell something first."));
            } else if let Some(it) = h.gear.get_mut(i).and_then(|o| o.take()) {
                msgs.push(m("Loot", format!("📦 Unequipped {}", it.label())));
                h.bag.push(it);
            }
        }
        self.finish(msgs);
    }

    fn sell(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let Some(it) = h.bag.get(i) else { return };
            let v = it.sell_value();
            if v == 0 {
                msgs.push(m("Loot", format!("🔒 {} is soulbound and can't be sold.", it.name)));
            } else {
                let it = h.bag.remove(i);
                h.gold += v;
                msgs.push(m("Loot", format!("💰 Sold {} for {v} gold", it.label())));
            }
        }
        self.finish(msgs);
    }

    fn sell_junk(self: &Rc<Self>) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let (mut total, mut n) = (0u32, 0u32);
            let mut keep: Vec<Item> = vec![];
            for it in std::mem::take(&mut h.bag) {
                let v = it.sell_value();
                if it.quality <= 2 && v > 0 { total += v; n += 1; } else { keep.push(it); }
            }
            h.bag = keep;
            h.gold += total;
            msgs.push(m("Loot", format!("💰 Sold {n} junk items for {total} gold")));
        }
        self.finish(msgs);
    }

    fn refresh_logs(self: &Rc<Self>) {
        let Some(a) = self.active.get() else { return };
        let s = self.save.borrow();
        let Some(h) = s.heroes.get(a) else { return };
        let filt = LOG_FILTERS[(self.log_filter.selected() as usize).min(LOG_FILTERS.len() - 1)];
        let text = h.log.iter().rev()
            .filter(|e| filt == "All" || e.cat == filt)
            .map(|e| format!("[{}] [{}] {}", e.time, e.cat, e.msg))
            .collect::<Vec<_>>().join("\n");
        self.log_view.buffer().set_text(&text);
    }

    fn refresh_zones(self: &Rc<Self>) {
        while let Some(c) = self.zlist.first_child() { self.zlist.remove(&c); }
        let Some(a) = self.active.get() else { return };
        let s = self.save.borrow();
        let Some(h) = s.heroes.get(a) else { return };
        let q = self.zsearch.text().to_lowercase();
        let kind = self.zkind.selected();
        let open: Vec<&Zone> = self.zones.iter()
            .filter(|z| z.open_to(&h.faction, h.level) && z.matches_kind(kind)
                        && (q.is_empty() || z.name.to_lowercase().contains(&q)))
            .collect();
        self.zcount.set_text(&format!(
            "{} of {} zones available to you (level {}, {}) — showing {}. Dungeon zones: +25% rewards, Raid zones: +50%.",
            open.len(), self.zones.len(), h.level, h.faction, open.len().min(ZONE_ROWS)));
        for z in open.iter().take(ZONE_ROWS) {
            let fr = gtk::Frame::new(None);
            fr.set_child(Some(&self.zone_row(z, h)));
            self.zlist.append(&fr);
        }
    }

    fn zone_row(self: &Rc<Self>, z: &Zone, hero: &Hero) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        pad(&row, 6);
        let info = gtk::Box::new(gtk::Orientation::Vertical, 2);
        info.set_hexpand(true);
        let name = gtk::Label::new(None);
        name.set_xalign(0.0);
        name.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(z.name.as_str())));
        let sub = gtk::Label::new(Some(&format!(
            "Lv {} · {} · {}", z.level_text(), z.terr, if z.inst.is_empty() { "Open world" } else { z.inst.as_str() })));
        sub.set_xalign(0.0); sub.add_css_class("dim-label");
        info.append(&name); info.append(&sub);
        let here = hero.zone == z.name && hero.zone_inst == z.inst;
        let btn = gtk::Button::with_label(if here { "📍 Here" } else { "Travel" });
        btn.set_sensitive(!here);
        btn.set_valign(gtk::Align::Center);
        { let u = self.clone(); let z = z.clone(); btn.connect_clicked(move |_| u.travel(&z)); }
        row.append(&info); row.append(&btn);
        row
    }

    fn travel(self: &Rc<Self>, z: &Zone) {
        let Some(a) = self.active.get() else { return };
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            h.zone = z.name.clone();
            h.zone_inst = z.inst.clone();
        }
        let b = inst_bonus(&z.inst);
        let kind = if z.inst.is_empty() { z.terr.as_str() } else { z.inst.as_str() };
        let extra = if b > 0 { format!(" — +{b}% quest rewards") } else { String::new() };
        self.finish(vec![m("Travel", format!("🗺 Traveled to {} ({kind}){extra}", z.name))]);
    }

    fn quest_row(self: &Rc<Self>, i: usize, q: &Quest) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        pad(&row, 8);
        let info = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info.set_hexpand(true);
        let t = gtk::Label::new(None);
        t.set_xalign(0.0);
        t.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(q.display().as_str())));
        let sub = gtk::Label::new(Some(&format!("{} · {} · 📍 {} · from {}", q.tier.name(), CATS[q.cat], q.zone, q.giver)));
        sub.set_xalign(0.0); sub.add_css_class("dim-label");
        let bar = gtk::ProgressBar::new();
        bar.set_show_text(true);
        bar.set_fraction(q.progress as f64 / q.goal as f64);
        bar.set_text(Some(&format!("{}/{}", q.progress, q.goal)));
        info.append(&t); info.append(&sub); info.append(&bar);

        let done = q.progress >= q.goal;
        let act = gtk::Button::with_label(if done { "Turn in" } else { "⚔ +1" });
        if done { act.add_css_class("suggested-action"); }
        act.set_valign(gtk::Align::Center);
        { let u = self.clone(); act.connect_clicked(move |_| u.act(i)); }
        let del = gtk::Button::with_label("Abandon");
        del.set_valign(gtk::Align::Center);
        { let u = self.clone(); del.connect_clicked(move |_| u.abandon(i)); }
        row.append(&info); row.append(&act); row.append(&del);
        row
    }

    fn add_quest(self: &Rc<Self>) {
        let title = self.entry.text().trim().to_string();
        if title.is_empty() { return; }
        let Some(a) = self.active.get() else { return };
        let tier = Tier::ALL[(self.tier.selected() as usize).min(Tier::ALL.len() - 1)];
        let ci = (self.cat.selected() as usize).min(CATS.len() - 1);
        let result: Result<Msg, String> = {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if tier.unlock() > h.level {
                Err(format!("🔒 {} quests unlock at level {}.", tier.name(), tier.unlock()))
            } else if ci > 0 && h.level < PROF_LEVEL {
                Err(format!("🔒 Professions unlock at level {PROF_LEVEL}."))
            } else {
                let giver = pick(&npcs(&h.faction)).to_string();
                let q = Quest {
                    title, giver: giver.clone(), tier, cat: ci, goal: self.goal.value() as u32, progress: 0,
                    chain: self.chain.is_active(), part: 1,
                    zone: h.zone.clone(), bonus: inst_bonus(&h.zone_inst),
                };
                let msg = m("Quest", format!("📜 {giver} ({}): \"{}\" — quest accepted!", q.zone, q.display()));
                h.quests.push(q);
                Ok(msg)
            }
        };
        match result {
            Ok(msg) => {
                self.entry.set_text("");
                self.goal.set_value(1.0);
                self.finish(vec![msg]);
            }
            Err(e) => self.finish(vec![m("System", e)]),
        }
    }

    fn act(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let msgs = {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if i >= h.quests.len() { return; }
            if h.quests[i].progress < h.quests[i].goal {
                let q = &mut h.quests[i];
                q.progress += 1;
                let mut v = vec![m("Quest", format!("⚔ {} — {}/{}", q.display(), q.progress, q.goal))];
                if q.progress == q.goal {
                    v.push(m("Quest", format!("Objective complete! Return to {} and turn in the quest.", q.giver)));
                }
                v
            } else {
                let q = h.quests.remove(i);
                let mut v = h.turn_in(&q);
                if q.chain {
                    let next = Quest { part: q.part + 1, goal: q.goal + (q.goal / 4).max(1), progress: 0, ..q.clone() };
                    v.push(m("Quest", format!("🔗 Quest chain continues: {}", next.display())));
                    h.quests.push(next);
                }
                v
            }
        };
        self.finish(msgs);
    }

    fn abandon(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if i < h.quests.len() { h.quests.remove(i); }
        }
        self.finish(vec![m("Quest", "Quest abandoned.")]);
    }

    // ----- PvP
    fn new_opponents(self: &Rc<Self>) {
        let lvl = self.active.get()
            .and_then(|a| self.save.borrow().heroes.get(a).map(|h| h.level)).unwrap_or(1);
        *self.opps.borrow_mut() = (0..PVP_SHOWN).map(|_| gen_player(lvl)).collect();
        self.pvp_result.set_text("");
        self.refresh_pvp();
    }

    fn refresh_pvp(self: &Rc<Self>) {
        while let Some(c) = self.pvp_list.first_child() { self.pvp_list.remove(&c); }
        for (i, o) in self.opps.borrow().iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            pad(&row, 6);
            let info = gtk::Box::new(gtk::Orientation::Vertical, 2);
            info.set_hexpand(true);
            info.set_tooltip_text(Some(&o.detail()));
            let name = gtk::Label::new(None);
            name.set_xalign(0.0);
            name.set_markup(&format!("<b>{}</b>  <span foreground='{}' size='small'>{}</span>",
                glib::markup_escape_text(o.name.as_str()), fcol(&o.faction), o.faction));
            let sub = gtk::Label::new(Some(&o.summary()));
            sub.set_xalign(0.0); sub.add_css_class("dim-label");
            info.append(&name); info.append(&sub);
            let btn = gtk::Button::with_label("⚔");
            btn.set_tooltip_text(Some("Duel this player with your current gear, abilities and stats"));
            btn.set_valign(gtk::Align::Center);
            { let u = self.clone(); btn.connect_clicked(move |_| u.duel(i)); }
            row.append(&info); row.append(&btn);
            let fr = gtk::Frame::new(None);
            fr.set_child(Some(&row));
            self.pvp_list.append(&fr);
        }
    }

    fn duel(self: &Rc<Self>, i: usize) {
        let opp = {
            let o = self.opps.borrow();
            match o.get(i) { Some(x) => x.clone(), None => return }
        };
        let Some(a) = self.active.get() else { return };
        let (text, msgs) = {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let (won, lines) = fight(&h.fighter(), &opp);
            let rounds = lines.len();
            let mut msgs: Vec<Msg> = lines.into_iter().map(|l| m("PvP", l)).collect();
            let text;
            if won {
                let honor = 10 + opp.level / 5;
                let gold = opp.level * 2 + rnd(10);
                h.wins += 1; h.honor += honor; h.gold += gold;
                text = format!("🏆 Victory vs {} (Lv {}) in {rounds} rounds: +{honor} honor, +{gold} gold", opp.name, opp.level);
            } else {
                h.losses += 1; h.honor += 1;
                text = format!("💀 Defeat vs {} (Lv {}) after {rounds} rounds: +1 honor", opp.name, opp.level);
            }
            msgs.push(m("PvP", text.clone()));
            h.check_achievements(&mut msgs);
            (text, msgs)
        };
        self.pvp_result.set_text(&text);
        self.finish(msgs);
    }
}

fn create_screen(ui: &Rc<Ui>) -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 10);
    b.set_halign(gtk::Align::Center); b.set_valign(gtk::Align::Center); b.set_width_request(480);
    let head = gtk::Label::new(None);
    head.set_markup("<span size='xx-large' weight='bold'>Who am I?</span>");

    let labels: Vec<String> = ui.realms.iter().map(|r| r.label()).collect();
    let lrefs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
    let realm = gtk::DropDown::from_strings(&lrefs);
    let faction = gtk::DropDown::from_strings(&FACTIONS);
    let race = gtk::DropDown::from_strings(&races("Horde"));
    let class_names: Vec<&str> = CLASSES.iter().map(|c| c.name).collect();
    let class = gtk::DropDown::from_strings(&class_names);
    let spec = gtk::DropDown::from_strings(&spec_names(0));
    let name = gtk::Entry::new(); name.set_placeholder_text(Some("Character name"));
    let guild = gtk::Entry::new(); guild.set_placeholder_text(Some("Guild (optional)"));
    let info = gtk::Label::new(None);
    info.set_xalign(0.0); info.set_wrap(true); info.add_css_class("dim-label");

    let upd: Rc<dyn Fn()> = {
        let (c, s, i) = (class.clone(), spec.clone(), info.clone());
        Rc::new(move || {
            let cl = &CLASSES[(c.selected() as usize).min(CLASSES.len() - 1)];
            let si = (s.selected() as usize).min(cl.specs.len() - 1);
            let mut roles: Vec<&str> = vec![];
            for (_, r) in cl.specs {
                if !roles.contains(&r.name()) { roles.push(r.name()); }
            }
            let weapons: Vec<&str> = cl.weapons.iter().map(|w| w.name()).collect();
            i.set_text(&format!(
                "{} armor · {} · weapons: {}\nclass roles: {} · selected spec role: {}",
                cl.armor, offhand_text(cl), weapons.join(", "), roles.join(", "), cl.specs[si].1.name()));
        })
    };
    {
        let r = race.clone();
        faction.connect_selected_notify(move |d| {
            let fac = FACTIONS[(d.selected() as usize).min(FACTIONS.len() - 1)];
            r.set_model(Some(&gtk::StringList::new(&races(fac))));
            r.set_selected(0);
        });
    }
    {
        let (sp, u) = (spec.clone(), upd.clone());
        class.connect_selected_notify(move |d| {
            let ci = (d.selected() as usize).min(CLASSES.len() - 1);
            sp.set_model(Some(&gtk::StringList::new(&spec_names(ci))));
            sp.set_selected(0);
            u();
        });
    }
    { let u = upd.clone(); spec.connect_selected_notify(move |_| u()); }
    upd();

    let go = gtk::Button::with_label("Enter the world");
    go.add_css_class("suggested-action");
    let back = gtk::Button::with_label("← Back to characters");

    b.append(&head);
    b.append(&labeled("1. Choose realm (population)", &realm));
    b.append(&labeled("2. Choose faction", &faction));
    b.append(&labeled("3. Choose race (faction races + Pandaren, Dracthyr)", &race));
    b.append(&labeled("4. Choose class", &class));
    b.append(&labeled("5. Choose specialization", &spec));
    b.append(&info);
    b.append(&labeled("6. Create identity", &name));
    b.append(&guild);
    b.append(&go);
    b.append(&back);

    { let u = ui.clone(); back.connect_clicked(move |_| u.show_select()); }
    let u = ui.clone();
    go.connect_clicked(move |_| {
        name.remove_css_class("error");
        let n = name.text().trim().to_string();
        let dup = u.save.borrow().heroes.iter().any(|h| h.name.eq_ignore_ascii_case(&n));
        if n.is_empty() || dup { name.add_css_class("error"); return; }
        let ri = (realm.selected() as usize).min(u.realms.len() - 1);
        let fac = FACTIONS[(faction.selected() as usize).min(FACTIONS.len() - 1)];
        let rl = races(fac);
        let race_name = rl[(race.selected() as usize).min(rl.len() - 1)];
        let ci = (class.selected() as usize).min(CLASSES.len() - 1);
        let si = (spec.selected() as usize).min(CLASSES[ci].specs.len() - 1);
        let hero = Hero::new(n.clone(), guild.text().trim().to_string(), &u.realms[ri], fac, race_name, ci, si);
        let idx = {
            let mut s = u.save.borrow_mut();
            s.heroes.push(hero);
            s.heroes.len() - 1
        };
        persist(&u.save.borrow());
        name.set_text("");
        u.enter(idx);
        u.finish(vec![m("System", format!("🎉 {n} created! Find an NPC and accept your first quest."))]);
    });
    b
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder().application_id("dev.example.WowTodo").build();
    app.connect_activate(|app| {
        let ui = Ui::build(app);
        ui.start();
    });
    app.run()
}