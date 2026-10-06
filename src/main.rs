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
use K::{Absorb, Buff, Dmg, Dot, Guard, Heal, Kick, Leech, Stun, Util};
use R::{Free, Mana, Stam};

const APP_TITLE: &str = "MAH's World of Todo";
const MAX_LEVEL: u32 = 90;
const MAX_LOG: usize = 1000;
const ZONE_ROWS: usize = 80;
const PVP_SHOWN: usize = 40;
const PROF_LEVEL: u32 = 5;
const GATHER_MAX: usize = 4; // CATS[1..=4] are gathering, the rest are crafting
const BAG_MAX: usize = 30;
const MAX_QGOAL: u32 = 15; // max kills / gathers per quest
const GATHER_CHANCE: u32 = 55; // % chance a gather press yields the item
const GATHER_DROP: u32 = 25; // % chance a gather press also drops a potion
const STEP_DROP: u32 = 35; // % chance a finished kill step drops a potion
const POT_PCT: u32 = 40; // potions restore this % of the bar
const STORE_SIZE: usize = 10;

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

const MOBS: &[&str] = &[
    "Kobolds", "Murlocs", "Gnolls", "Defias Bandits", "Timber Wolves", "Harpies", "Forest Trolls", "Skeletons",
    "Ghouls", "Giant Spiders", "Wild Boars", "Ogres", "Imps", "Wraiths", "Scorpids",
];

// potions: (name, icon)
const POT: [(&str, &str); 3] = [("Health Potion", "🧪"), ("Mana Potion", "🔷"), ("Stamina Potion", "⚡")];

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
const QSELL: [u32; 7] = [1, 2, 3, 5, 8, 25, 0]; // vendor sell factor
const QBUY: [u32; 7] = [40, 60, 100, 250, 600, 1500, 0]; // store price multiplier (%)
const Q_LEGENDARY: usize = 5;
const Q_HEIRLOOM: usize = 6;

const QUALITY_GUIDE: &str = "Poor (grey) — low-level Normal quests; vendor trash, just sell it.\n\
Common (white) — Normal quests and vendors; sell or discard.\n\
Uncommon (green) — any quest, crafting, the Store and random drops; always has an \"of the …\" stat suffix.\n\
Rare (blue) — crafting, quests, the Store (level 20+), Dungeon/Raid tiers and random drops.\n\
Epic (purple) — high-end crafting, Dungeon/Raid/World Boss tiers, rare Store stock (level 40+).\n\
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

// ---------- abilities ----------
// K = what the ability does in combat, R = which bar it spends.
//   Dmg    p = % of your ATK dealt as damage
//   Dot    p = % of your ATK dealt in total over 3 rounds
//   Leech  p = % of ATK as damage, you heal 40% of it
//   Heal   p = % of your max HP restored (x1.4 for Healer specs)
//   Absorb p = % of your max HP as a damage shield
//   Stun   p = % of ATK as damage (0 = pure crowd control); target loses its next turn
//   Kick   p = % of ATK as damage; target's next action is a basic attack
//   Buff   p = % bonus damage for 3 rounds
//   Guard  p = % less damage taken for 3 rounds
//   Util   no combat effect (travel, utility, out-of-combat buffs)
// c = cost in % of the max bar, cd = rounds between uses.
// The percentages are scaled approximations of the retail tooltip coefficients, not exact values.
#[derive(Clone, Copy, PartialEq)]
enum K { Dmg, Dot, Leech, Heal, Absorb, Stun, Kick, Buff, Guard, Util }
#[derive(Clone, Copy, PartialEq)]
enum R { Mana, Stam, Free }

struct Ab {
    l: u32,
    n: &'static str,
    k: K,
    p: u32,
    cd: u32,
    r: R,
    c: u32,
    sp: &'static [&'static str], // empty = every spec
}

const ALL: &[&str] = &[];

#[allow(clippy::too_many_arguments)]
const fn ab(l: u32, n: &'static str, k: K, p: u32, cd: u32, r: R, c: u32, sp: &'static [&'static str]) -> Ab {
    Ab { l, n, k, p, cd, r, c, sp }
}
const fn ut(l: u32, n: &'static str) -> Ab {
    Ab { l, n, k: Util, p: 0, cd: 0, r: Free, c: 0, sp: ALL }
}

const AB_DK: &[Ab] = &[
    ab(1, "Death Strike", Leech, 85, 1, Stam, 12, ALL),
    ab(1, "Rune Strike", Dmg, 95, 1, Stam, 10, ALL),
    ab(1, "Marrowrend", Dmg, 110, 2, Stam, 14, &["Blood"]),
    ab(1, "Blood Boil", Dot, 150, 3, Stam, 14, &["Blood", "Unholy"]),
    ab(1, "Howling Blast", Dmg, 120, 2, Mana, 12, &["Frost"]),
    ab(2, "Death Grip", Stun, 35, 6, Stam, 10, ALL),
    ab(3, "Mind Freeze", Kick, 40, 4, Stam, 8, ALL),
    ab(4, "Raise Dead", Dot, 135, 5, Mana, 15, ALL),
    ab(5, "Anti-Magic Shell", Absorb, 30, 6, Mana, 12, ALL),
    ab(7, "Icebound Fortitude", Guard, 40, 8, Stam, 10, ALL),
    ab(10, "Death and Decay", Dot, 180, 4, Mana, 16, ALL),
    ut(10, "Path of Frost"),
    ut(13, "Death's Advance"),
    ab(19, "Empower Rune Weapon", Buff, 30, 10, Free, 0, ALL),
];

const AB_DH: &[Ab] = &[
    ab(1, "Demon's Bite", Dmg, 90, 1, Stam, 8, &["Havoc", "Devourer"]),
    ab(1, "Shear", Dmg, 90, 1, Stam, 8, &["Vengeance"]),
    ab(1, "Chaos Strike", Dmg, 120, 2, Stam, 14, &["Havoc", "Devourer"]),
    ab(1, "Soul Cleave", Leech, 125, 2, Stam, 16, &["Vengeance"]),
    ab(2, "Fel Rush", Dmg, 75, 3, Stam, 8, &["Havoc", "Devourer"]),
    ab(2, "Infernal Strike", Dmg, 75, 3, Stam, 8, &["Vengeance"]),
    ab(3, "Disrupt", Kick, 30, 4, Stam, 6, ALL),
    ab(4, "Immolation Aura", Dot, 120, 4, Mana, 10, ALL),
    ab(5, "Blur", Guard, 35, 7, Stam, 10, &["Havoc", "Vengeance"]),
    ab(5, "Fiendish Overhaul", Guard, 35, 7, Stam, 10, &["Devourer"]),
    ab(8, "Consume Magic", Dmg, 60, 3, Mana, 8, ALL),
    ab(10, "Metamorphosis", Buff, 40, 10, Free, 0, ALL),
    ut(10, "Glide"),
    ut(12, "Spectral Sight"),
    ab(19, "Chaos Nova", Stun, 70, 7, Mana, 14, ALL),
];

const AB_DRUID: &[Ab] = &[
    ab(1, "Wrath", Dmg, 100, 1, Mana, 6, ALL),
    ab(1, "Moonfire", Dot, 130, 2, Mana, 8, ALL),
    ab(2, "Regrowth", Heal, 22, 2, Mana, 12, ALL),
    ab(2, "Rejuvenation", Heal, 26, 2, Mana, 10, ALL),
    ab(3, "Bear Form", Guard, 25, 8, Stam, 5, ALL),
    ab(3, "Growl", Kick, 20, 5, Stam, 5, ALL),
    ab(3, "Mangle", Dmg, 105, 1, Stam, 12, &["Feral", "Guardian"]),
    ab(4, "Cat Form", Buff, 15, 8, Stam, 5, ALL),
    ab(4, "Rake", Dot, 140, 2, Stam, 10, &["Feral"]),
    ab(4, "Shred", Dmg, 115, 1, Stam, 12, &["Feral"]),
    ut(6, "Travel Form"),
    ut(6, "Dash"),
    ab(8, "Barkskin", Guard, 30, 8, Mana, 8, ALL),
    ab(10, "Entangling Roots", Stun, 50, 7, Mana, 10, ALL),
    ab(10, "Mark of the Wild", Buff, 10, 12, Mana, 8, ALL),
    ut(10, "Teleport: Moonglade"),
    ut(13, "Revive / Rebirth"),
    ab(19, "Stampeding Roar", Buff, 20, 10, Stam, 5, ALL),
];

const AB_EVOKER: &[Ab] = &[
    ab(1, "Living Flame", Dmg, 105, 1, Mana, 6, &["Augmentation", "Devastation"]),
    ab(1, "Living Flame", Heal, 20, 1, Mana, 8, &["Preservation"]),
    ab(1, "Azure Strike", Dmg, 80, 1, Mana, 4, ALL),
    ab(2, "Emerald Blossom", Heal, 28, 3, Mana, 14, ALL),
    ab(3, "Disintegrate", Dot, 170, 3, Mana, 14, ALL),
    ab(4, "Fire Breath", Dmg, 160, 3, Mana, 16, ALL),
    ab(5, "Wing Buffeting", Stun, 40, 6, Mana, 8, ALL),
    ab(8, "Blessing of the Bronze", Buff, 12, 12, Mana, 8, ALL),
    ut(10, "Soar"),
    ut(10, "Skyriding"),
    ab(10, "Deep Breath", Dmg, 200, 6, Mana, 18, ALL),
    ut(13, "Rescue"),
    ab(19, "Time Dilation", Guard, 35, 8, Mana, 10, ALL),
];

const AB_HUNTER: &[Ab] = &[
    ab(1, "Arcane Shot", Dmg, 95, 1, Stam, 8, &["Marksmanship", "Survival"]),
    ab(1, "Cobra Shot", Dmg, 95, 1, Stam, 8, &["Beast Mastery"]),
    ab(1, "Auto Shot", Dmg, 85, 1, Free, 0, ALL),
    ab(2, "Steady Shot", Dmg, 100, 1, Stam, 6, ALL),
    ab(3, "Kill Command", Dmg, 140, 2, Stam, 12, &["Beast Mastery"]),
    ab(3, "Aimed Shot", Dmg, 170, 3, Stam, 16, &["Marksmanship"]),
    ab(3, "Raptor Strike", Dmg, 120, 1, Stam, 10, &["Survival"]),
    ut(4, "Call Pet"),
    ut(4, "Revive Pet"),
    ab(4, "Mend Pet", Heal, 12, 4, Mana, 10, ALL),
    ut(5, "Disengage"),
    ab(7, "Wing Clip", Stun, 35, 6, Stam, 6, &["Survival"]),
    ab(7, "Freezing Trap", Stun, 25, 8, Mana, 10, &["Beast Mastery", "Marksmanship"]),
    ab(8, "Exhilaration", Heal, 30, 8, Free, 0, ALL),
    ab(10, "Aspect of the Turtle", Guard, 50, 12, Free, 0, ALL),
    ut(10, "Tame Beast"),
    ut(13, "Feign Death"),
    ab(19, "Tar Trap", Dot, 90, 6, Mana, 10, ALL),
];

const AB_MAGE: &[Ab] = &[
    ab(1, "Frostbolt", Dmg, 110, 1, Mana, 8, ALL),
    ab(2, "Fire Blast", Dmg, 120, 2, Mana, 8, ALL),
    ab(3, "Frost Nova", Stun, 55, 6, Mana, 10, ALL),
    ut(4, "Blink"),
    ut(5, "Conjure Refreshment"),
    ab(6, "Arcane Explosion", Dmg, 100, 1, Mana, 9, ALL),
    ab(7, "Counterspell", Kick, 40, 5, Mana, 6, ALL),
    ab(8, "Arcane Intellect", Buff, 10, 12, Mana, 6, ALL),
    ut(9, "Slow Fall"),
    ab(10, "Polymorph", Stun, 0, 8, Mana, 10, ALL),
    ab(16, "Invisibility", Guard, 40, 10, Mana, 8, ALL),
    ab(18, "Cone of Cold", Dmg, 150, 3, Mana, 14, ALL),
    ut(21, "Teleport"),
    ut(24, "Portal"),
    ab(49, "Time Warp", Buff, 30, 14, Mana, 10, ALL),
];

const AB_MONK: &[Ab] = &[
    ab(1, "Tiger Palm", Dmg, 90, 1, Stam, 8, ALL),
    ab(2, "Blackout Kick", Dmg, 115, 1, Stam, 12, ALL),
    ut(3, "Roll"),
    ab(4, "Vivify", Heal, 24, 2, Mana, 12, ALL),
    ab(5, "Touch of Death", Dmg, 220, 8, Stam, 16, ALL),
    ab(7, "Spear Hand Strike", Kick, 35, 4, Stam, 6, ALL),
    ab(8, "Fortifying Brew", Guard, 35, 9, Stam, 10, ALL),
    ab(10, "Crackling Jade Lightning", Dot, 130, 2, Mana, 10, ALL),
    ut(10, "Mystic Touch"),
    ut(13, "Transcendence"),
    ab(19, "Leg Sweep", Stun, 50, 7, Stam, 12, ALL),
];

const AB_PALADIN: &[Ab] = &[
    ab(1, "Crusader Strike", Dmg, 100, 1, Stam, 8, ALL),
    ab(1, "Judgment", Dmg, 125, 2, Mana, 8, ALL),
    ab(2, "Flash of Light", Heal, 22, 2, Mana, 12, ALL),
    ab(3, "Shield of the Righteous", Dmg, 130, 2, Stam, 12, &["Protection"]),
    ab(3, "Word of Glory", Heal, 30, 3, Mana, 10, &["Holy", "Retribution"]),
    ab(4, "Consecration", Dot, 140, 4, Mana, 12, ALL),
    ab(5, "Hand of Reckoning", Kick, 25, 5, Stam, 5, ALL),
    ab(7, "Rebuke", Kick, 40, 4, Stam, 6, ALL),
    ab(8, "Divine Protection", Guard, 30, 8, Mana, 8, ALL),
    ab(10, "Divine Shield", Guard, 80, 14, Free, 0, ALL),
    ab(10, "Lay on Hands", Heal, 70, 16, Free, 0, ALL),
    ab(10, "Devotion Aura", Guard, 12, 12, Mana, 6, ALL),
    ab(13, "Blessing of Protection", Absorb, 25, 9, Mana, 10, ALL),
    ab(19, "Hammer of Justice", Stun, 60, 7, Mana, 10, ALL),
];

const AB_PRIEST: &[Ab] = &[
    ab(1, "Smite", Dmg, 100, 1, Mana, 6, ALL),
    ab(1, "Shadow Word: Pain", Dot, 140, 2, Mana, 8, ALL),
    ab(2, "Flash Heal", Heal, 25, 2, Mana, 12, ALL),
    ab(3, "Power Word: Shield", Absorb, 22, 3, Mana, 10, ALL),
    ab(4, "Renew", Heal, 20, 2, Mana, 8, ALL),
    ab(5, "Mind Blast", Dmg, 150, 2, Mana, 12, ALL),
    ab(7, "Psychic Scream", Stun, 25, 8, Mana, 12, ALL),
    ab(8, "Desperate Prayer", Heal, 35, 8, Mana, 8, ALL),
    ab(10, "Power Word: Fortitude", Buff, 10, 12, Mana, 8, ALL),
    ut(10, "Leap of Faith"),
    ab(13, "Fade", Guard, 30, 8, Mana, 8, ALL),
    ut(19, "Mass Dispel"),
];

const AB_ROGUE: &[Ab] = &[
    ab(1, "Sinister Strike", Dmg, 100, 1, Stam, 8, ALL),
    ab(1, "Eviscerate", Dmg, 170, 3, Stam, 16, ALL),
    ab(2, "Stealth", Buff, 25, 10, Free, 0, ALL),
    ab(3, "Slice and Dice", Buff, 30, 8, Stam, 8, &["Assassination", "Outlaw"]),
    ab(3, "Ambush", Dmg, 160, 5, Stam, 10, &["Subtlety"]),
    ab(4, "Kick", Kick, 40, 4, Stam, 6, ALL),
    ab(5, "Kidney Shot", Stun, 50, 7, Stam, 12, ALL),
    ab(7, "Cheap Shot", Stun, 40, 8, Stam, 10, ALL),
    ab(8, "Crimson Vial", Heal, 18, 5, Stam, 8, ALL),
    ab(10, "Vanish", Guard, 60, 12, Free, 0, ALL),
    ut(10, "Sprint"),
    ab(13, "Cloak of Shadows", Guard, 40, 10, Free, 0, ALL),
    ab(19, "Blind", Stun, 0, 9, Stam, 8, ALL),
];

const AB_SHAMAN: &[Ab] = &[
    ab(1, "Lightning Bolt", Dmg, 105, 1, Mana, 6, ALL),
    ab(1, "Primal Strike", Dmg, 105, 1, Stam, 8, &["Enhancement"]),
    ab(2, "Healing Surge", Heal, 24, 2, Mana, 12, ALL),
    ab(3, "Flame Shock", Dot, 140, 2, Mana, 8, ALL),
    ab(4, "Lava Burst", Dmg, 150, 3, Mana, 12, &["Elemental"]),
    ab(4, "Chain Lightning", Dmg, 130, 2, Mana, 12, &["Enhancement", "Restoration"]),
    ut(5, "Ghost Wolf"),
    ab(7, "Wind Shear", Kick, 40, 4, Mana, 6, ALL),
    ab(8, "Astral Shift", Guard, 35, 9, Free, 0, ALL),
    ab(10, "Skyfury", Buff, 12, 12, Mana, 8, ALL),
    ab(10, "Bloodlust", Buff, 35, 14, Free, 0, ALL), // shown as Heroism for Alliance
    ab(13, "Capacitor Totem", Stun, 30, 9, Mana, 10, ALL),
    ut(19, "Earthbind Totem"),
];

const AB_WARLOCK: &[Ab] = &[
    ab(1, "Shadow Bolt", Dmg, 110, 1, Mana, 8, &["Affliction", "Demonology"]),
    ab(1, "Incinerate", Dmg, 115, 1, Mana, 8, &["Destruction"]),
    ab(1, "Curse of Agony", Dot, 150, 2, Mana, 8, ALL),
    ab(2, "Summon Imp", Dot, 110, 5, Mana, 10, ALL),
    ab(3, "Corruption", Dot, 145, 2, Mana, 8, &["Affliction"]),
    ab(3, "Drain Life", Leech, 120, 2, Mana, 10, &["Demonology", "Destruction"]),
    ab(4, "Create Healthstone", Heal, 25, 10, Free, 0, ALL),
    ab(5, "Fear", Stun, 0, 8, Mana, 10, ALL),
    ab(7, "Spell Lock", Kick, 40, 4, Mana, 6, ALL),
    ut(7, "Felhunter"),
    ab(8, "Unending Resolve", Guard, 35, 9, Free, 0, ALL),
    ab(10, "Summon Voidwalker", Guard, 20, 8, Mana, 10, ALL),
    ut(10, "Soulstone"),
    ut(13, "Ritual of Summoning"),
    ab(19, "Shadowfury", Stun, 70, 7, Mana, 14, ALL),
];

const AB_WARRIOR: &[Ab] = &[
    ab(1, "Slam", Dmg, 110, 1, Stam, 10, ALL),
    ab(1, "Charge", Stun, 40, 6, Free, 0, ALL),
    ab(2, "Shield Slam", Dmg, 140, 2, Stam, 12, &["Protection"]),
    ab(2, "Bloodthirst", Leech, 130, 2, Stam, 12, &["Fury"]),
    ab(2, "Mortal Strike", Dmg, 150, 3, Stam, 14, &["Arms"]),
    ab(3, "Taunt", Kick, 15, 5, Stam, 5, ALL),
    ab(3, "Victory Rush", Leech, 100, 3, Free, 0, ALL),
    ab(4, "Execute", Dmg, 190, 4, Stam, 16, ALL),
    ab(5, "Whirlwind", Dmg, 125, 2, Stam, 14, ALL),
    ab(7, "Pummel", Kick, 40, 4, Stam, 6, ALL),
    ab(8, "Shield Wall", Guard, 45, 10, Free, 0, &["Protection"]),
    ab(8, "Enraged Regeneration", Heal, 30, 9, Free, 0, &["Arms", "Fury"]),
    ab(10, "Battle Shout", Buff, 10, 12, Free, 0, ALL),
    ab(10, "Heroic Leap", Dmg, 90, 5, Stam, 8, ALL),
    ab(13, "Spell Reflection", Guard, 40, 9, Free, 0, ALL),
    ab(19, "Rallying Cry", Absorb, 20, 12, Free, 0, ALL),
];

fn spec_ok(a: &Ab, spec: &str) -> bool { a.sp.is_empty() || a.sp.iter().any(|s| *s == spec) }

/// Abilities this class/spec knows at `level`.
fn known(class: usize, spec: usize, level: u32) -> Vec<&'static Ab> {
    let list: &'static [Ab] = CLASSES[class].abilities;
    let sn = CLASSES[class].specs[spec].0;
    list.iter().filter(|a| a.l <= level && spec_ok(a, sn)).collect()
}

fn aname(a: &Ab, faction: &str) -> &'static str {
    if a.n == "Bloodlust" && faction == "Alliance" { "Heroism" } else { a.n }
}

fn kname(k: K) -> &'static str {
    match k {
        Dmg => "Damage", Dot => "Damage over time", Leech => "Drain", Heal => "Heal", Absorb => "Shield",
        Stun => "Stun", Kick => "Interrupt", Buff => "Buff", Guard => "Defensive", Util => "Utility",
    }
}

fn kcol(k: K) -> &'static str {
    match k {
        Dmg | Dot | Leech => "#e0453a",
        Heal | Absorb => "#3fb950",
        Stun | Kick => "#d29922",
        Buff | Guard => "#4a8fe7",
        Util => "#8b949e",
    }
}

fn ab_effect(a: &Ab, st: &Stats) -> String {
    let dmg = st.atk * a.p / 100;
    let eff = match a.k {
        Dmg => format!("Deals ~{dmg} damage"),
        Dot => format!("~{}/round for 3 rounds (~{} total)", dmg / 3, dmg / 3 * 3),
        Leech => format!("Deals ~{dmg} damage, heals you for ~{}", dmg * 40 / 100),
        Heal => format!("Heals ~{} HP", st.hp * a.p / 100 * st.hmul / 100),
        Absorb => format!("Absorbs ~{} damage", st.hp * a.p / 100),
        Stun => if a.p > 0 { format!("Deals ~{dmg} and stuns for 1 round") } else { "Incapacitates the target for 1 round".to_string() },
        Kick => format!("Deals ~{dmg} and interrupts (target's next action is a basic attack)"),
        Buff => format!("+{}% damage for 3 rounds", a.p),
        Guard => format!("-{}% damage taken for 3 rounds", a.p),
        Util => return "Utility — no combat effect".to_string(),
    };
    let cost = match a.r {
        Free => "free".to_string(),
        Mana => format!("{}% mana (~{})", a.c, st.mana * a.c / 100),
        Stam => format!("{}% stamina (~{})", a.c, st.sta * a.c / 100),
    };
    let cd = if a.cd >= 2 { format!(" · cooldown {} rounds", a.cd) } else { String::new() };
    format!("{eff} · cost {cost}{cd}")
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
    abilities: &'static [Ab],
    weapons: &'static [Wt],
    dual: bool,
    shield: bool,
    held: bool,
}

const CLASSES: [Class; 13] = [
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
fn esc(s: &str) -> String { glib::markup_escape_text(s).to_string() }
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
enum QKind { #[default] Kill, Gather }

/// One kill target of a Kill quest: a to-do line you type in yourself.
#[derive(Serialize, Deserialize, Clone)]
struct Step { text: String, done: bool }

fn make_steps(n: u32) -> Vec<Step> { (0..n).map(|_| Step { text: String::new(), done: false }).collect() }

#[derive(Serialize, Deserialize, Clone)]
struct Quest {
    title: String, giver: String, tier: Tier, cat: usize, goal: u32, progress: u32,
    chain: bool, part: u32,
    #[serde(default)] zone: String,
    #[serde(default)] bonus: u32,
    #[serde(default)] qk: QKind,
    #[serde(default)] target: String, // mob type (Kill) or item (Gather)
    #[serde(default)] steps: Vec<Step>, // Kill quests only
    #[serde(default)] tries: u32, // Gather attempts
}
impl Quest {
    fn display(&self) -> String {
        let what = match self.qk {
            QKind::Kill => format!("Kill {}× {}", self.goal, self.target),
            QKind::Gather => format!("Gather {}× {}", self.goal, self.target),
        };
        let base = if self.title.is_empty() { what } else { format!("{} — {what}", self.title) };
        if self.chain { format!("{base} (Part {})", self.part) } else { base }
    }
}

fn gather_item(cat: usize) -> String {
    match cat {
        1 => ps(&["Copper Ore", "Tin Ore", "Iron Ore", "Mithril Ore"]),
        2 => ps(&["Peacebloom", "Silverleaf", "Briarthorn", "Kingsblood"]),
        3 => ps(&["Ruined Leather Scraps", "Light Hide", "Thick Hide"]),
        4 => ps(&["Raw Brightscale Fish", "Raw Slitherskin Mackerel", "Oily Blackmouth"]),
        _ => ps(&["Runed Relic Fragment", "Glowing Ember", "Ancient Scroll", "Crystal Shard", "Wolf Meat", "Linen Cloth"]),
    }.to_string()
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

    /// Store price: grows with item level and quality, always well above the sell value.
    fn buy_price(&self) -> u32 {
        let il = self.ilvl;
        let base = il * 4 + il * il / 4 + 10;
        (base * QBUY[self.quality.min(6)] / 100).max(5)
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
    let unit = (ilvl * QSTAT[q.min(6)] / 100) as i32;
    // every stat lands 1-2 points above or below the base value
    let val = || -> u32 {
        let off = [-2i32, -1, 1, 2][rnd(4) as usize];
        (unit + off).max(1) as u32
    };
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

// ---------- store ----------
fn store_quality(level: u32) -> usize {
    let r = rnd(100);
    if level >= 40 && r >= 94 { 4 }
    else if level >= 20 && r >= 70 { 3 }
    else if r >= 35 { 2 }
    else { 1 }
}

/// Items for sale: usable by the class, ilvl right around the hero's level.
fn gen_store(class: usize, spec: usize, level: u32) -> Vec<Item> {
    let base = level.saturating_sub(2).max(1);
    let mut v: Vec<Item> = vec![];
    let mut tries = 0;
    while v.len() < STORE_SIZE && tries < 300 {
        tries += 1;
        if let Some(it) = make_item(class, spec, base, random_cat(), store_quality(level)) { v.push(it); }
    }
    v.sort_by_key(|i| (i.slot, std::cmp::Reverse(i.quality)));
    v
}

fn potion_price(level: u32) -> u32 { 6 + level * 2 }

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
    gear: Vec<Option<Item>>, prof: Vec<u32>, achievements: Vec<String>,
    quests: Vec<Quest>, log: Vec<LogEntry>,
    #[serde(default)] bag: Vec<Item>,
    #[serde(default)] hp: u32,
    #[serde(default)] mana: u32,
    #[serde(default)] sta: u32,
    #[serde(default)] pots: [u32; 3],
}

#[derive(Serialize, Deserialize, Default)]
struct Save { heroes: Vec<Hero> }

impl Hero {
    fn new(name: String, guild: String, realm: &Realm, faction: &str, race: &str, class: usize, spec: usize) -> Self {
        let mut h = Hero {
            name, guild, realm: realm.name.clone(), realm_tier: realm.tier,
            faction: faction.into(), race: race.into(), class, spec,
            level: 1, xp: 0, gold: 0, talents: 0, done: 0, honor: 0, wins: 0, losses: 0,
            zone: start_zone(race).into(), zone_inst: String::new(),
            gear: vec![None; SLOTS.len()], prof: vec![0; CATS.len() - 1],
            achievements: vec![], quests: vec![], log: vec![], bag: vec![],
            hp: 0, mana: 0, sta: 0, pots: [0; 3],
        };
        h.restore_all();
        h
    }

    fn need(&self) -> u32 { 100 + self.level * 50 }

    fn two_now(&self) -> bool {
        self.gear.get(S_MAIN).and_then(|o| o.as_ref()).map_or(false, |i| i.is_two())
    }

    fn fighter(&self) -> Fighter {
        Fighter {
            name: self.name.clone(), faction: self.faction.clone(), race: self.race.clone(),
            class: self.class, spec: self.spec, level: self.level, gear: self.gear.clone(),
            talents: self.talents, cur: Some((self.hp.max(1), self.mana, self.sta)),
        }
    }

    /// (max hp, max mana, max stamina)
    fn maxes(&self) -> (u32, u32, u32) {
        let s = self.fighter().stats();
        (s.hp, s.mana, s.sta)
    }

    fn restore_all(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = a; self.mana = b; self.sta = c;
    }

    fn clamp_res(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = self.hp.min(a).max(1);
        self.mana = self.mana.min(b);
        self.sta = self.sta.min(c);
    }

    fn regen(&mut self, pct: u32) {
        let (a, b, c) = self.maxes();
        self.hp = (self.hp + a * pct / 100).min(a);
        self.mana = (self.mana + b * pct / 100).min(b);
        self.sta = (self.sta + c * pct / 100).min(c);
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

    fn is_upgrade(&self, it: &Item) -> bool {
        let cands = candidates(it, self.class, self.two_now());
        if cands.is_empty() { return false; }
        let p = item_power(it, self.class, self.spec);
        cands.iter().any(|&s| self.gear[s].as_ref().map_or(true, |o| item_power(o, self.class, self.spec) < p))
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

    /// Each finished quest step / gather attempt has a chance to drop one random potion (0 or 1).
    fn step_drop(&mut self, msgs: &mut Vec<Msg>, chance: u32) {
        if rnd(100) < chance {
            let k = rnd(3) as usize;
            self.pots[k] += 1;
            msgs.push(m("Loot", format!("{} Found a {}!", POT[k].1, POT[k].0)));
        }
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
        // gather quests pay +1% per item gathered on top of the linear scaling
        let extra = if q.qk == QKind::Gather { q.goal } else { 0 };
        let pct = 100 + q.bonus + extra;
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
        if extra > 0 { msgs.push(m("Quest", format!("🌿 Big haul bonus: +{extra}%"))); }
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

        let before = self.level;
        while self.xp >= self.need() && self.level < MAX_LEVEL {
            let n = self.need();
            self.xp -= n;
            self.level += 1;
            self.refresh_heirlooms();
            msgs.push(m("Level", format!("⬆ LEVEL UP! You are now level {}", self.level)));
            let fresh: Vec<&'static Ab> = known(self.class, self.spec, self.level)
                .into_iter().filter(|a| a.l == self.level).collect();
            if fresh.is_empty() {
                self.talents += 1;
                msgs.push(m("Level", "✨ New talent point"));
            } else {
                for a in fresh {
                    msgs.push(m("Level", format!("📖 New ability: {} ({})", aname(a, &self.faction), kname(a.k))));
                }
            }
            for t in Tier::ALL {
                if t.unlock() == self.level { msgs.push(m("Level", format!("🔓 {} quests unlocked", t.name()))); }
            }
            if self.level == PROF_LEVEL { msgs.push(m("Level", "🔓 Professions unlocked")); }
        }
        if self.level > before {
            self.restore_all();
            msgs.push(m("Level", "💚 HP, mana and stamina fully restored"));
        } else {
            self.regen(15);
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
struct Stats { hp: u32, atk: u32, def: u32, crit: u32, heal: u32, mana: u32, sta: u32, hmul: u32 }

#[derive(Clone)]
struct Fighter {
    name: String, faction: String, race: String, class: usize, spec: usize, level: u32,
    gear: Vec<Option<Item>>, talents: u32,
    cur: Option<(u32, u32, u32)>, // current hp/mana/stamina (None = full)
}

impl Fighter {
    fn role(&self) -> Role { CLASSES[self.class].specs[self.spec].1 }

    fn abilities(&self) -> Vec<&'static Ab> { known(self.class, self.spec, self.level) }

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
                + self.abilities().len() as u32 * 3 + self.talents * 4
                + tot[prim] + off_prim / 4 + tot[4] / 2 + tot[7] / 3,
            def: (self.level + g / 2) * def_m / 100 * armor / 100 + tot[6] / 2 + shield_def,
            crit: (5 + self.talents + qsum / 3 + tot[5] / 6).min(40),
            heal: if role == Healer { hp * 6 / 100 } else { 0 },
            mana: 60 + self.level * 6 + tot[3] * 2,
            sta: 60 + self.level * 4 + tot[0] / 2,
            hmul: if role == Healer { 140 } else { 100 },
        }
    }

    fn summary(&self) -> String {
        let c = &CLASSES[self.class];
        let s = self.stats();
        format!("Lv {} {} {} ({}) · {} · {} · ilvl {} · HP {} ATK {} DEF {} · {} abilities",
                self.level, self.race, c.name, c.specs[self.spec].0, self.role().name(), c.armor,
                self.ilvl_avg(), s.hp, s.atk, s.def, self.abilities().len())
    }

    fn detail(&self) -> String {
        let ab: Vec<&str> = self.abilities().iter().map(|a| aname(a, &self.faction)).collect();
        let ab = if ab.is_empty() { "—".to_string() } else { ab.join(", ") };
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

/// One fighter's live state during a duel.
struct Side {
    st: Stats,
    hp: i64,
    mana: u32,
    sta: u32,
    abs: Vec<&'static Ab>,
    cds: Vec<u32>,
    absorb: i64,
    buff: (u32, u32),  // (rounds left, % bonus damage)
    guard: (u32, u32), // (rounds left, % less damage taken)
    dot: (u32, u32),   // (rounds left, damage per round)
    stunned: bool,
    kicked: bool,
}

impl Side {
    fn new(f: &Fighter) -> Self {
        let st = f.stats();
        let (hp, mana, sta) = f.cur
            .map_or((st.hp, st.mana, st.sta), |(h, mn, s)| (h.min(st.hp), mn.min(st.mana), s.min(st.sta)));
        let abs = f.abilities();
        let n = abs.len();
        Side {
            st, hp: hp.max(1) as i64, mana, sta, abs, cds: vec![0; n], absorb: 0,
            buff: (0, 0), guard: (0, 0), dot: (0, 0), stunned: false, kicked: false,
        }
    }
}

struct Duel { won: bool, lines: Vec<String>, hp: u32, mana: u32, sta: u32 }

fn cost_of(a: &Ab, s: &Side) -> u32 {
    match a.r { Mana => s.st.mana * a.c / 100, Stam => s.st.sta * a.c / 100, Free => 0 }
}

fn hit(me: &Side, foe: &Side, pct: u32) -> (i64, bool) {
    let mut base = me.st.atk * pct / 100 * (85 + rnd(31)) / 100;
    if me.buff.0 > 0 { base = base * (100 + me.buff.1) / 100; }
    let mut dmg = (base * 100 / (100 + foe.st.def / 2)).max(1);
    let crit = rnd(100) < me.st.crit;
    if crit { dmg *= 2; }
    if foe.guard.0 > 0 { dmg = (dmg * (100 - foe.guard.1.min(90)) / 100).max(1); }
    (dmg as i64, crit)
}

fn take(s: &mut Side, dmg: i64) {
    let a = dmg.min(s.absorb);
    s.absorb -= a;
    s.hp -= dmg - a;
}

fn crit_tag(c: bool) -> &'static str { if c { " CRIT" } else { "" } }

/// Pick which ability to use this turn (None = basic attack).
fn choose(me: &Side, foe: &Side) -> Option<usize> {
    let maxhp = me.st.hp as i64;
    let low = me.hp * 100 < maxhp * 50;
    let mut opts: Vec<(usize, u32)> = vec![];
    for (i, a) in me.abs.iter().enumerate() {
        if a.k == Util || me.cds[i] > 0 { continue; }
        let have = match a.r { Mana => me.mana, Stam => me.sta, Free => u32::MAX };
        if have < cost_of(a, me) { continue; }
        let ok = match a.k {
            Heal => me.hp * 100 < maxhp * 75,
            Absorb => me.absorb == 0 && me.hp * 100 < maxhp * 90,
            Guard => me.guard.0 == 0 && me.hp * 100 < maxhp * 70,
            Buff => me.buff.0 == 0,
            Stun => !foe.stunned,
            Kick => !foe.kicked,
            Dot => foe.dot.0 == 0,
            _ => true,
        };
        if !ok { continue; }
        let w = match a.k {
            Heal | Absorb | Guard => if low { 300 } else { 100 },
            _ => a.p.max(40),
        };
        opts.push((i, w));
    }
    if opts.is_empty() { return None; }
    let total: u32 = opts.iter().map(|o| o.1).sum();
    let mut r = rnd(total);
    for (i, w) in &opts {
        if r < *w { return Some(*i); }
        r -= *w;
    }
    Some(opts[0].0)
}

/// One fighter's action. Returns the log text for it.
fn turn(f: &Fighter, g: &Fighter, me: &mut Side, foe: &mut Side) -> String {
    let nm = f.name.as_str();
    for c in me.cds.iter_mut() { if *c > 0 { *c -= 1; } }
    let mut pre = String::new();
    if me.dot.0 > 0 {
        let d = me.dot.1 as i64;
        take(me, d);
        me.dot.0 -= 1;
        if me.hp <= 0 { return format!("{nm} is finished off by damage over time ({d})"); }
        pre = format!("[-{d} dot] ");
    }
    if me.stunned {
        me.stunned = false;
        return format!("{pre}{nm} is stunned and loses the turn");
    }
    let interrupted = me.kicked;
    me.kicked = false;
    let choice = if interrupted { None } else { choose(me, foe) };

    let body = match choice {
        None => {
            let (d, c) = hit(me, foe, 100);
            take(foe, d);
            me.mana = (me.mana + me.st.mana * 5 / 100).min(me.st.mana);
            me.sta = (me.sta + me.st.sta * 5 / 100).min(me.st.sta);
            let verb = if interrupted { "is interrupted and attacks" } else { "attacks" };
            format!("{nm} {verb} → {d}{}", crit_tag(c))
        }
        Some(i) => {
            let a: &'static Ab = me.abs[i];
            let cost = cost_of(a, me);
            match a.r {
                Mana => me.mana -= cost.min(me.mana),
                Stam => me.sta -= cost.min(me.sta),
                Free => {}
            }
            me.cds[i] = a.cd;
            let an = aname(a, &f.faction);
            let verb = if a.r == Mana { "casts" } else { "uses" };
            match a.k {
                Dmg => {
                    let (d, c) = hit(me, foe, a.p);
                    take(foe, d);
                    format!("{nm} {verb} {an} → {d}{}", crit_tag(c))
                }
                Dot => {
                    let tick = (me.st.atk * a.p / 100 / 3).max(1);
                    foe.dot = (3, tick);
                    format!("{nm} {verb} {an} → {} suffers {tick}/round for 3 rounds", g.name)
                }
                Leech => {
                    let (d, c) = hit(me, foe, a.p);
                    take(foe, d);
                    let h = (d * 40 / 100).max(1);
                    me.hp = (me.hp + h).min(me.st.hp as i64);
                    format!("{nm} {verb} {an} → {d}{} (+{h} HP)", crit_tag(c))
                }
                Heal => {
                    let amt = (me.st.hp * a.p / 100 * me.st.hmul / 100 * (90 + rnd(21)) / 100) as i64;
                    me.hp = (me.hp + amt).min(me.st.hp as i64);
                    format!("{nm} {verb} {an} → heals {amt} HP")
                }
                Absorb => {
                    let amt = (me.st.hp * a.p / 100) as i64;
                    me.absorb += amt;
                    format!("{nm} {verb} {an} → shield absorbs {amt}")
                }
                Stun => {
                    foe.stunned = true;
                    if a.p > 0 {
                        let (d, c) = hit(me, foe, a.p);
                        take(foe, d);
                        format!("{nm} {verb} {an} → {d}{} · {} is stunned", crit_tag(c), g.name)
                    } else {
                        format!("{nm} {verb} {an} → {} is incapacitated", g.name)
                    }
                }
                Kick => {
                    let (d, c) = hit(me, foe, a.p);
                    take(foe, d);
                    foe.kicked = true;
                    format!("{nm} {verb} {an} → {d}{} · {} is interrupted", crit_tag(c), g.name)
                }
                Buff => {
                    me.buff = (4, a.p);
                    format!("{nm} {verb} {an} → +{}% damage", a.p)
                }
                Guard => {
                    me.guard = (4, a.p);
                    format!("{nm} {verb} {an} → -{}% damage taken", a.p)
                }
                Util => String::new(),
            }
        }
    };
    format!("{pre}{body}")
}

fn tick(s: &mut Side) {
    if s.buff.0 > 0 { s.buff.0 -= 1; }
    if s.guard.0 > 0 { s.guard.0 -= 1; }
}

fn end_round(s: &mut Side) {
    s.mana = (s.mana + s.st.mana * 5 / 100).min(s.st.mana);
    s.sta = (s.sta + s.st.sta * 5 / 100).min(s.st.sta);
    s.hp = (s.hp + s.st.heal as i64).min(s.st.hp as i64);
}

fn fight(a: &Fighter, b: &Fighter) -> Duel {
    let (mut sa, mut sb) = (Side::new(a), Side::new(b));
    let mut lines = vec![];
    let mut result: Option<bool> = None;
    for r in 1..=40 {
        let ta = turn(a, b, &mut sa, &mut sb);
        tick(&mut sa);
        let mut line = format!("R{r}: {ta}");
        if sa.hp <= 0 { lines.push(line); result = Some(false); break; }
        if sb.hp <= 0 { lines.push(line); result = Some(true); break; }
        let tb = turn(b, a, &mut sb, &mut sa);
        tick(&mut sb);
        line += &format!(" | {tb}");
        lines.push(line);
        if sa.hp <= 0 { result = Some(false); break; }
        if sb.hp <= 0 { result = Some(true); break; }
        end_round(&mut sa);
        end_round(&mut sb);
    }
    let won = result.unwrap_or_else(|| sa.hp * 1000 / sa.st.hp as i64 >= sb.hp * 1000 / sb.st.hp as i64);
    Duel { won, lines, hp: sa.hp.max(0) as u32, mana: sa.mana, sta: sa.sta }
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
    Fighter {
        name: player_name(), faction: faction.into(), race: pick(&pool).to_string(), class, spec, level, gear,
        talents: rnd(level / 3 + 1), cur: None,
    }
}

// ---------- persistence ----------
fn save_path() -> PathBuf {
    let d = glib::user_data_dir().join("wow-todo");
    let _ = std::fs::create_dir_all(&d);
    d.join("characters.json")
}

/// Upgrade old saves: old gear system, kill-quest steps, resource bars.
fn migrate(s: &mut Save) {
    let map: [usize; 6] = [0, 1, 2, 6, S_MAIN, 12];
    for h in &mut s.heroes {
        for q in &mut h.quests {
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
        if h.hp == 0 { h.restore_all(); } else { h.clamp_res(); }
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
fn bar(class: &str) -> gtk::ProgressBar {
    let b = gtk::ProgressBar::new();
    b.set_show_text(true);
    b.set_hexpand(true);
    b.add_css_class(class);
    b
}
fn set_bar(b: &gtk::ProgressBar, label: &str, cur: u32, max: u32) {
    b.set_fraction((cur as f64 / max.max(1) as f64).min(1.0));
    b.set_text(Some(&format!("{label} {cur} / {max}")));
}

struct Ui {
    save: RefCell<Save>,
    active: Cell<Option<usize>>,
    confirm_del: Cell<Option<usize>>,
    realms: Vec<Realm>,
    zones: Vec<Zone>,
    opps: RefCell<Vec<Fighter>>,
    store: RefCell<Vec<Item>>,
    store_key: Cell<(usize, u32)>,
    stack: gtk::Stack,
    sel_list: gtk::Box,
    title: gtk::Label, xp_bar: gtk::ProgressBar, stats: gtk::Label,
    hp_bar: gtk::ProgressBar, mana_bar: gtk::ProgressBar, sta_bar: gtk::ProgressBar,
    pot_btn: [gtk::Button; 3],
    giver: gtk::Label, entry: gtk::Entry, goal: gtk::SpinButton,
    tier: gtk::DropDown, cat: gtk::DropDown, chain: gtk::CheckButton,
    qlist: gtk::Box, status: gtk::Label,
    zsearch: gtk::Entry, zkind: gtk::DropDown, zcount: gtk::Label, zlist: gtk::Box,
    pvp_head: gtk::Label, pvp_result: gtk::Label, pvp_list: gtk::Box,
    log_filter: gtk::DropDown, log_view: gtk::TextView,
    sheet: gtk::Label, ab_list: gtk::Box,
    eq_sum: gtk::Label, eq_list: gtk::Box, bag_head: gtk::Label, bag_list: gtk::Box,
    store_gold: gtk::Label, store_pots: gtk::Box, store_list: gtk::Box,
}

impl Ui {
    fn build(app: &gtk::Application) -> Rc<Self> {
        let ho = gtk::Orientation::Horizontal;
        let ve = gtk::Orientation::Vertical;

        // ----- colours for the resource bars
        let css = gtk::CssProvider::new();
        css.load_from_data(
            "progressbar.hp-bar progress { background: #c0392b; } \
             progressbar.mana-bar progress { background: #2e86de; } \
             progressbar.sta-bar progress { background: #d4a017; }",
        );
        if let Some(d) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(&d, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

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
        let hp_bar = bar("hp-bar");
        let mana_bar = bar("mana-bar");
        let sta_bar = bar("sta-bar");
        let bars = gtk::Box::new(ho, 8);
        bars.append(&hp_bar); bars.append(&mana_bar); bars.append(&sta_bar);
        let pot_btn = [gtk::Button::new(), gtk::Button::new(), gtk::Button::new()];
        let potrow = gtk::Box::new(ho, 8);
        potrow.append(&gtk::Label::new(Some("Potions:")));
        for b in &pot_btn { potrow.append(b); }
        let stats = gtk::Label::new(None); stats.set_xalign(0.0); stats.set_wrap(true);

        // ----- quests tab
        let giver = gtk::Label::new(None); giver.set_xalign(0.0);
        let entry = gtk::Entry::new(); entry.set_hexpand(true);
        entry.set_placeholder_text(Some("e.g. Write report / Clean the kitchen"));
        let goal = gtk::SpinButton::with_range(1.0, MAX_QGOAL as f64, 1.0);
        goal.set_tooltip_text(Some("Kill quests: how many mobs to kill — each kill becomes its own to-do line.\nGather quests ignore this: the amount (3-15) is rolled for you."));
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
        add_row.append(&labeled("Kill count", &goal));
        add_row.append(&labeled("Difficulty", &tier));
        add_row.append(&labeled("Profession", &cat));
        add_row.append(&labeled("Chain", &chain));
        add_row.append(&add_btn);
        let hint = gtk::Label::new(Some(
            "Each quest is randomly a KILL quest (kill X mobs — every kill is a line you fill in and mark done or abandon) \
             or a GATHER quest (press ⚔ +1: you may or may not find the item; the amount, 3-15, is rolled for you, bigger hauls pay more). \
             Every step has a chance to drop a health, mana or stamina potion.",
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
        zcount.set_wrap(true);
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

        // ----- store tab
        let store_gold = gtk::Label::new(None); store_gold.set_xalign(0.0);
        let store_hint = gtk::Label::new(Some(
            "Stock is generated around your level and usable by your class. Bought gear goes to your bag — equip it from the Equipment tab. \
             ▲ marks an upgrade over what you wear.",
        ));
        store_hint.set_xalign(0.0); store_hint.set_wrap(true); store_hint.add_css_class("dim-label");
        let pot_title = gtk::Label::new(None); pot_title.set_markup("<b>Potions</b>"); pot_title.set_xalign(0.0);
        let store_pots = gtk::Box::new(ve, 4);
        let st_title = gtk::Label::new(None); st_title.set_markup("<b>Equipment for sale</b>");
        st_title.set_xalign(0.0); st_title.set_hexpand(true);
        let restock = gtk::Button::with_label("🔄 Restock");
        let st_row = gtk::Box::new(ho, 8);
        st_row.append(&st_title); st_row.append(&restock);
        let store_list = gtk::Box::new(ve, 4);
        let store_inner = gtk::Box::new(ve, 8);
        pad(&store_inner, 10);
        store_inner.append(&store_gold); store_inner.append(&store_hint); store_inner.append(&pot_title);
        store_inner.append(&store_pots); store_inner.append(&st_row); store_inner.append(&store_list);
        let store_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&store_inner).build();

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

        // ----- character tab (stats + abilities)
        let sheet = gtk::Label::new(None); sheet.set_xalign(0.0); sheet.set_yalign(0.0); sheet.set_wrap(true);
        let ab_list = gtk::Box::new(ve, 6);
        let sheet_box = gtk::Box::new(ve, 10);
        pad(&sheet_box, 10);
        sheet_box.append(&sheet); sheet_box.append(&ab_list);
        let sheet_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&sheet_box).build();

        let nb = gtk::Notebook::new();
        nb.set_vexpand(true);
        nb.append_page(&quests_page, Some(&gtk::Label::new(Some("📜 Quests"))));
        nb.append_page(&zones_page, Some(&gtk::Label::new(Some("🗺 Zones"))));
        nb.append_page(&pvp_page, Some(&gtk::Label::new(Some("⚔ Realm PvP"))));
        nb.append_page(&eq_scroll, Some(&gtk::Label::new(Some("🎒 Equipment"))));
        nb.append_page(&store_scroll, Some(&gtk::Label::new(Some("🏪 Store"))));
        nb.append_page(&sheet_scroll, Some(&gtk::Label::new(Some("🧙 Character"))));
        nb.append_page(&logs_page, Some(&gtk::Label::new(Some("📋 Logs"))));

        let game = gtk::Box::new(ve, 10);
        pad(&game, 12);
        game.append(&head); game.append(&xp_bar); game.append(&bars); game.append(&potrow);
        game.append(&stats); game.append(&nb);

        let stack = gtk::Stack::new();
        stack.add_named(&select, Some("select"));
        stack.add_named(&game, Some("game"));
        let win = gtk::ApplicationWindow::builder().application(app)
            .title(APP_TITLE).default_width(1000).default_height(900).build();
        win.set_child(Some(&stack));

        let ui = Rc::new(Ui {
            save: RefCell::new(load()), active: Cell::new(None), confirm_del: Cell::new(None),
            realms: make_realms(), zones: load_zones(), opps: RefCell::new(vec![]),
            store: RefCell::new(vec![]), store_key: Cell::new((usize::MAX, 0)),
            stack, sel_list, title, xp_bar, stats, hp_bar, mana_bar, sta_bar, pot_btn,
            giver, entry, goal, tier, cat, chain, qlist, status,
            zsearch, zkind, zcount, zlist, pvp_head, pvp_result, pvp_list, log_filter, log_view, sheet, ab_list,
            eq_sum, eq_list, bag_head, bag_list, store_gold, store_pots, store_list,
        });

        { let u = ui.clone(); sel_new.connect_clicked(move |_| u.show_create()); }
        { let u = ui.clone(); add_btn.connect_clicked(move |_| u.add_quest()); }
        { let u = ui.clone(); ui.entry.connect_activate(move |_| u.add_quest()); }
        { let u = ui.clone(); save_btn.connect_clicked(move |_| u.finish(vec![m("System", "💾 Character saved.")])); }
        { let u = ui.clone(); switch_btn.connect_clicked(move |_| u.show_select()); }
        { let u = ui.clone(); pvp_refresh.connect_clicked(move |_| u.new_opponents()); }
        { let u = ui.clone(); log_clear.connect_clicked(move |_| u.clear_logs()); }
        { let u = ui.clone(); sell_junk.connect_clicked(move |_| u.sell_junk()); }
        { let u = ui.clone(); restock.connect_clicked(move |_| u.restock()); }
        { let u = ui.clone(); ui.zsearch.connect_changed(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.zkind.connect_selected_notify(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.log_filter.connect_selected_notify(move |_| u.refresh_logs()); }
        for (k, b) in ui.pot_btn.iter().enumerate() {
            let u = ui.clone();
            b.connect_clicked(move |_| u.drink(k));
        }

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
        self.store_key.set((usize::MAX, 0));
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
        self.store_key.set((usize::MAX, 0));
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
            if let Some(h) = self.save.borrow_mut().heroes.get_mut(a) {
                h.clamp_res();
                h.add_log(msgs);
            }
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
            set_bar(&self.hp_bar, "❤ HP", h.hp, st.hp);
            set_bar(&self.mana_bar, "🔷 Mana", h.mana, st.mana);
            set_bar(&self.sta_bar, "⚡ Stamina", h.sta, st.sta);
            for k in 0..3 {
                let short = POT[k].0.split(' ').next().unwrap_or("");
                self.pot_btn[k].set_label(&format!("{} {} ×{}", POT[k].1, short, h.pots[k]));
                self.pot_btn[k].set_sensitive(h.pots[k] > 0);
                self.pot_btn[k].set_tooltip_text(Some(&format!("Drink a {}: restores {POT_PCT}% of the bar", POT[k].0)));
            }
            self.stats.set_text(&format!("💰 {} gold   ✨ Talents: {}   🎖 Honor: {}", h.gold, h.talents, h.honor));

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
                "🌐 Realm {} — {} players online (showing {} nearby, lowest level first)\nYou: Lv {} · HP {}/{} · Mana {}/{} · Stamina {}/{} · ATK {} · DEF {} · Crit {}% · {}W-{}L · {} honor",
                h.realm, commas(pop), PVP_SHOWN, h.level, h.hp, st.hp, h.mana, st.mana, h.sta, st.sta,
                st.atk, st.def, st.crit, h.wins, h.losses, h.honor));

            let mut t = String::from("<b>Combat stats</b>\n");
            t += &format!("HP {}/{} · Mana {}/{} · Stamina {}/{}\n", h.hp, st.hp, h.mana, st.mana, h.sta, st.sta);
            t += &format!("ATK {} · DEF {} · Crit {}% · Regen {}/round · avg ilvl {}\n", st.atk, st.def, st.crit, st.heal, f.ilvl_avg());
            t += &format!("{} armor · {} · {} (see the Equipment tab)\n\n", c.armor, role.name(), f.setup());
            t += "<b>Professions</b>\n";
            let profs: Vec<String> = h.prof.iter().enumerate().filter(|(_, p)| **p > 0)
                .map(|(i, p)| format!("{}: {}", CATS[i + 1], p)).collect();
            t += &if profs.is_empty() { "—".to_string() } else { profs.join("\n") };
            t += "\n\n<b>Achievements</b>\n";
            t += &if h.achievements.is_empty() { "—".to_string() } else { h.achievements.join("\n") };
            self.sheet.set_markup(&t);
            self.refresh_abilities(h, &st);
        }
        self.refresh_equipment();
        self.refresh_store();
        self.refresh_zones();
        self.refresh_logs();
    }

    fn refresh_abilities(&self, h: &Hero, st: &Stats) {
        while let Some(c) = self.ab_list.first_child() { self.ab_list.remove(&c); }
        let cls = &CLASSES[h.class];
        let spec_name = cls.specs[h.spec].0;
        let (have, locked): (Vec<&Ab>, Vec<&Ab>) = cls.abilities.iter()
            .filter(|a| spec_ok(a, spec_name))
            .partition(|a| a.l <= h.level);
        let head = gtk::Label::new(None);
        head.set_xalign(0.0);
        head.set_markup(&format!("<b>Abilities</b> — {} learned (damage/heal numbers use your current stats)", have.len()));
        self.ab_list.append(&head);
        for a in have {
            let lbl = gtk::Label::new(None);
            lbl.set_xalign(0.0); lbl.set_wrap(true);
            lbl.set_markup(&format!(
                "<span foreground='{}'><b>{}</b></span>  <small>Lv {} · {}</small>\n<small>{}</small>",
                kcol(a.k), esc(aname(a, &h.faction)), a.l, kname(a.k), esc(&ab_effect(a, st))));
            pad(&lbl, 6);
            let fr = gtk::Frame::new(None);
            fr.set_child(Some(&lbl));
            self.ab_list.append(&fr);
        }
        if !locked.is_empty() {
            let txt: Vec<String> = locked.iter().map(|a| format!("Lv {} {}", a.l, aname(a, &h.faction))).collect();
            let l = gtk::Label::new(Some(&format!("🔒 Upcoming: {}", txt.join(" · "))));
            l.set_xalign(0.0); l.set_wrap(true); l.add_css_class("dim-label");
            self.ab_list.append(&l);
        }
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

    // ----- store
    fn refresh_store(self: &Rc<Self>) {
        while let Some(c) = self.store_pots.first_child() { self.store_pots.remove(&c); }
        while let Some(c) = self.store_list.first_child() { self.store_list.remove(&c); }
        let Some(a) = self.active.get() else { return };
        let s = self.save.borrow();
        let Some(h) = s.heroes.get(a) else { return };
        if self.store_key.get() != (a, h.level) {
            *self.store.borrow_mut() = gen_store(h.class, h.spec, h.level);
            self.store_key.set((a, h.level));
        }
        self.store_gold.set_text(&format!("💰 You have {} gold · bag {}/{}", h.gold, h.bag.len(), BAG_MAX));

        let pp = potion_price(h.level);
        for k in 0..3 {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::new(Some(&format!(
                "{} {} — restores {POT_PCT}% · you have {}", POT[k].1, POT[k].0, h.pots[k])));
            lbl.set_xalign(0.0); lbl.set_hexpand(true);
            let b = gtk::Button::with_label(&format!("Buy {pp}g"));
            b.set_sensitive(h.gold >= pp);
            { let u = self.clone(); b.connect_clicked(move |_| u.buy_potion(k)); }
            row.append(&lbl); row.append(&b);
            self.store_pots.append(&row);
        }

        let stock = self.store.borrow();
        if stock.is_empty() { self.store_list.append(&gtk::Label::new(Some("Sold out — press Restock."))); }
        for (i, it) in stock.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let lbl = gtk::Label::new(None);
            lbl.set_xalign(0.0); lbl.set_hexpand(true);
            let tl = it.type_label();
            let si = it.stats_inline();
            let up = if h.is_upgrade(it) { "<span foreground='#3fb950'>▲</span> " } else { "" };
            lbl.set_markup(&format!(
                "{up}{}\n<small>ilvl {} · {} · {}</small>",
                qspan(it.quality, it.name.as_str()), it.ilvl,
                glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
            lbl.set_tooltip_text(Some(&it.tip()));
            let price = it.buy_price();
            let b = gtk::Button::with_label(&format!("Buy {price}g"));
            b.set_valign(gtk::Align::Center);
            b.set_sensitive(h.gold >= price);
            { let u = self.clone(); b.connect_clicked(move |_| u.buy(i)); }
            row.append(&lbl); row.append(&b);
            self.store_list.append(&row);
        }
    }

    fn restock(self: &Rc<Self>) {
        self.store_key.set((usize::MAX, 0));
        self.refresh_store();
    }

    fn buy(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let it = {
            let st = self.store.borrow();
            match st.get(i) { Some(x) => x.clone(), None => return }
        };
        let price = it.buy_price();
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if h.gold < price {
                msgs.push(m("System", format!("💸 Not enough gold — {} costs {price}g.", it.name)));
            } else if h.bag.len() >= BAG_MAX {
                msgs.push(m("Loot", "🎒 Bag is full — sell something first."));
            } else {
                h.gold -= price;
                msgs.push(m("Loot", format!("🛒 Bought {} for {price} gold", it.label())));
                h.bag.push(it);
                self.store.borrow_mut().remove(i);
            }
        }
        self.finish(msgs);
    }

    fn buy_potion(self: &Rc<Self>, k: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let price = potion_price(h.level);
            if h.gold < price {
                msgs.push(m("System", format!("💸 Not enough gold — a {} costs {price}g.", POT[k].0)));
            } else {
                h.gold -= price;
                h.pots[k] += 1;
                msgs.push(m("Loot", format!("🛒 Bought a {} for {price} gold", POT[k].0)));
            }
        }
        self.finish(msgs);
    }

    fn drink(self: &Rc<Self>, k: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let mx = h.maxes();
            let max = [mx.0, mx.1, mx.2][k];
            let cur = [h.hp, h.mana, h.sta][k];
            if h.pots[k] == 0 {
                msgs.push(m("System", format!("You have no {}.", POT[k].0)));
            } else if cur >= max {
                msgs.push(m("System", "That bar is already full."));
            } else {
                let newv = (cur + max * POT_PCT / 100).min(max);
                match k { 0 => h.hp = newv, 1 => h.mana = newv, _ => h.sta = newv }
                h.pots[k] -= 1;
                msgs.push(m("Loot", format!("{} Drank a {}: +{}", POT[k].1, POT[k].0, newv - cur)));
            }
        }
        self.finish(msgs);
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
        let mut open: Vec<&Zone> = self.zones.iter()
            .filter(|z| z.open_to(&h.faction, h.level) && z.matches_kind(kind)
                        && (q.is_empty() || z.name.to_lowercase().contains(&q)))
            .collect();
        let total = open.len();
        // keep the zones closest to the hero's level, then list them lowest level first
        let lvl = h.level;
        let dist = |z: &Zone| if lvl > z.hi { lvl - z.hi } else { 0 };
        open.sort_by_key(|z| (dist(z), lvl - z.lo, z.name.clone()));
        open.truncate(ZONE_ROWS);
        open.sort_by(|x, y| (x.lo, &x.name).cmp(&(y.lo, &y.name)));
        self.zcount.set_text(&format!(
            "{} of {} zones available to you (level {}, {}) — showing the {} closest to your level, lowest first. Dungeon zones: +25% rewards, Raid zones: +50%.",
            total, self.zones.len(), h.level, h.faction, open.len()));
        for z in open.iter() {
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

    // ----- quests
    fn quest_row(self: &Rc<Self>, i: usize, q: &Quest) -> gtk::Box {
        let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
        pad(&outer, 8);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let info = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info.set_hexpand(true);
        let t = gtk::Label::new(None);
        t.set_xalign(0.0);
        t.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(q.display().as_str())));
        let kind_txt = match q.qk { QKind::Kill => "⚔ Kill quest", QKind::Gather => "🌿 Gather quest" };
        let sub = gtk::Label::new(Some(&format!(
            "{} · {} · {} · 📍 {} · from {}", kind_txt, q.tier.name(), CATS[q.cat], q.zone, q.giver)));
        sub.set_xalign(0.0); sub.add_css_class("dim-label");
        let bar = gtk::ProgressBar::new();
        bar.set_show_text(true);
        bar.set_fraction((q.progress as f64 / q.goal.max(1) as f64).min(1.0));
        let btxt = match q.qk {
            QKind::Kill => format!("{}/{} kills", q.progress, q.goal),
            QKind::Gather => format!("{}/{} gathered · {} attempts", q.progress, q.goal, q.tries),
        };
        bar.set_text(Some(&btxt));
        info.append(&t); info.append(&sub); info.append(&bar);

        let done = q.progress >= q.goal;
        let act = gtk::Button::with_label(if done { "Turn in" } else { "⚔ +1" });
        if done { act.add_css_class("suggested-action"); }
        if q.qk == QKind::Kill && !done {
            act.set_label("Turn in");
            act.set_sensitive(false);
            act.set_tooltip_text(Some("Finish every kill below first."));
        }
        act.set_valign(gtk::Align::Center);
        { let u = self.clone(); act.connect_clicked(move |_| u.act(i)); }
        let del = gtk::Button::with_label("Abandon");
        del.set_valign(gtk::Align::Center);
        { let u = self.clone(); del.connect_clicked(move |_| u.abandon(i)); }
        row.append(&info); row.append(&act); row.append(&del);
        outer.append(&row);
        if q.qk == QKind::Kill {
            for (si, s) in q.steps.iter().enumerate() {
                outer.append(&self.step_row(i, si, s));
            }
        }
        outer
    }

    fn step_row(self: &Rc<Self>, qi: usize, si: usize, s: &Step) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_start(12);
        let e = gtk::Entry::new();
        e.set_hexpand(true);
        e.set_placeholder_text(Some(&format!("Kill {} — what is this task?", si + 1)));
        e.set_text(&s.text);
        if s.done { e.set_sensitive(false); }
        { let u = self.clone(); e.connect_changed(move |en| u.edit_step(qi, si, en.text().to_string())); }
        row.append(&e);
        if s.done {
            row.append(&gtk::Label::new(Some("✔ done")));
        } else {
            let ok = gtk::Button::with_label("✔ Complete");
            { let u = self.clone(); ok.connect_clicked(move |_| u.step_done(qi, si)); }
            let no = gtk::Button::with_label("✖ Abandon");
            { let u = self.clone(); no.connect_clicked(move |_| u.step_abandon(qi, si)); }
            row.append(&ok); row.append(&no);
        }
        row
    }

    fn edit_step(self: &Rc<Self>, qi: usize, si: usize, text: String) {
        let Some(a) = self.active.get() else { return };
        let mut s = self.save.borrow_mut();
        if let Some(st) = s.heroes.get_mut(a)
            .and_then(|h| h.quests.get_mut(qi))
            .and_then(|q| q.steps.get_mut(si))
        {
            st.text = text;
        }
        persist(&s);
    }

    fn step_done(self: &Rc<Self>, qi: usize, si: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let Some(q) = h.quests.get_mut(qi) else { return };
            if si >= q.steps.len() || q.steps[si].done { return; }
            q.steps[si].done = true;
            q.progress += 1;
            let txt = q.steps[si].text.trim().to_string();
            let label = if txt.is_empty() { format!("target {}", si + 1) } else { txt };
            let (prog, goal, giver, disp) = (q.progress, q.goal, q.giver.clone(), q.display());
            msgs.push(m("Quest", format!("✔ {disp}: {label} — {prog}/{goal} kills")));
            if prog >= goal {
                msgs.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
            }
            h.step_drop(&mut msgs, STEP_DROP);
            h.regen(4);
        }
        self.finish(msgs);
    }

    fn step_abandon(self: &Rc<Self>, qi: usize, si: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let Some(q) = h.quests.get_mut(qi) else { return };
            if si >= q.steps.len() { return; }
            let st = q.steps.remove(si);
            if st.done { q.progress = q.progress.saturating_sub(1); }
            q.goal = q.steps.len() as u32;
            let (empty, prog, goal, giver) = (q.goal == 0, q.progress, q.goal, q.giver.clone());
            if empty {
                h.quests.remove(qi);
                msgs.push(m("Quest", "✖ Last target abandoned — quest removed."));
            } else {
                msgs.push(m("Quest", format!("✖ Target {} abandoned ({prog}/{goal} kills left to do)", si + 1)));
                if prog >= goal {
                    msgs.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
                }
            }
        }
        self.finish(msgs);
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
                let qk = if rnd(2) == 0 { QKind::Kill } else { QKind::Gather };
                let (goal, target, steps) = match qk {
                    QKind::Kill => {
                        let g = (self.goal.value() as u32).clamp(1, MAX_QGOAL);
                        (g, ps(MOBS).to_string(), make_steps(g))
                    }
                    QKind::Gather => (3 + rnd(MAX_QGOAL - 2), gather_item(ci), vec![]),
                };
                let q = Quest {
                    title, giver: giver.clone(), tier, cat: ci, goal, progress: 0,
                    chain: self.chain.is_active(), part: 1,
                    zone: h.zone.clone(), bonus: inst_bonus(&h.zone_inst),
                    qk, target, steps, tries: 0,
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

    /// Gather quests: ⚔ +1 tries to find the item. Any quest: turn in when complete.
    fn act(self: &Rc<Self>, i: usize) {
        let Some(a) = self.active.get() else { return };
        let msgs = {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            if i >= h.quests.len() { return; }
            let ready = h.quests[i].progress >= h.quests[i].goal;
            if !ready {
                if h.quests[i].qk != QKind::Gather { return; }
                let got = rnd(100) < GATHER_CHANCE;
                let (disp, p, g, tgt, giver) = {
                    let q = &mut h.quests[i];
                    q.tries += 1;
                    if got { q.progress += 1; }
                    (q.display(), q.progress, q.goal, q.target.clone(), q.giver.clone())
                };
                let mut v = vec![if got {
                    m("Quest", format!("⚔ {disp} — found {tgt} ({p}/{g})"))
                } else {
                    m("Quest", format!("💨 {disp} — nothing this time ({p}/{g})"))
                }];
                if got && p >= g {
                    v.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
                }
                h.step_drop(&mut v, GATHER_DROP);
                h.regen(4);
                v
            } else {
                let q = h.quests.remove(i);
                let mut v = h.turn_in(&q);
                if q.chain {
                    let g = (q.goal + (q.goal / 4).max(1)).min(MAX_QGOAL);
                    let steps = if q.qk == QKind::Kill { make_steps(g) } else { vec![] };
                    let next = Quest { part: q.part + 1, goal: g, progress: 0, steps, tries: 0, ..q.clone() };
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
        let mut v: Vec<Fighter> = (0..PVP_SHOWN).map(|_| gen_player(lvl)).collect();
        v.sort_by_key(|f| f.level);
        *self.opps.borrow_mut() = v;
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
            btn.set_tooltip_text(Some("Duel this player with your current gear, abilities, HP, mana and stamina"));
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
            let me = h.fighter();
            let d = fight(&me, &opp);
            let rounds = d.lines.len();
            let mut msgs: Vec<Msg> = vec![m("PvP", format!(
                "⚔ Duel: {} (Lv {}, {}) vs {} (Lv {}, {})",
                me.name, me.level, CLASSES[me.class].specs[me.spec].0,
                opp.name, opp.level, CLASSES[opp.class].specs[opp.spec].0))];
            msgs.extend(d.lines.into_iter().map(|l| m("PvP", l)));
            h.mana = d.mana;
            h.sta = d.sta;
            let text;
            if d.won {
                let honor = 10 + opp.level / 5;
                let gold = opp.level * 2 + rnd(10);
                h.hp = d.hp.max(1);
                h.wins += 1; h.honor += honor; h.gold += gold;
                text = format!("🏆 Victory vs {} (Lv {}) in {rounds} rounds: +{honor} honor, +{gold} gold", opp.name, opp.level);
            } else {
                h.hp = (h.maxes().0 / 10).max(1);
                h.losses += 1; h.honor += 1;
                text = format!("💀 Defeat vs {} (Lv {}) after {rounds} rounds: +1 honor — you wake at 10% HP", opp.name, opp.level);
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