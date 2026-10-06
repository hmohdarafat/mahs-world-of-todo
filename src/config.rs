//! Application-wide configuration and gameplay constants.

pub(crate) const APP_TITLE: &str = "MAH's World of Todo";
pub(crate) const MAX_LEVEL: u32 = 90;
pub(crate) const MAX_LOG: usize = 1000;
pub(crate) const ZONE_ROWS: usize = 80;
pub(crate) const PVP_SHOWN: usize = 40;
pub(crate) const PROF_LEVEL: u32 = 5;
pub(crate) const GATHER_MAX: usize = 4; // CATS[1..=4] are gathering, the rest are crafting
pub(crate) const BAG_MAX: usize = 30;
pub(crate) const MAX_QGOAL: u32 = 15; // max kills / gathers per quest
pub(crate) const GATHER_CHANCE: u32 = 55; // % chance a gather press yields the item
pub(crate) const GATHER_DROP: u32 = 25; // % chance a gather press also drops a potion
pub(crate) const STEP_DROP: u32 = 35; // % chance a finished kill step drops a potion
pub(crate) const POT_PCT: u32 = 40; // potions restore this % of the bar
pub(crate) const STORE_SIZE: usize = 10;

pub(crate) const FACTIONS: [&str; 2] = ["Horde", "Alliance"];
pub(crate) const HORDE: [&str; 6] = ["Orc", "Undead (Forsaken)", "Tauren", "Troll", "Blood Elf", "Goblin"];
pub(crate) const ALLIANCE: [&str; 6] = ["Human", "Dwarf", "Night Elf", "Gnome", "Draenei", "Worgen"];
pub(crate) const NEUTRAL: [&str; 2] = ["Pandaren", "Dracthyr"];
pub(crate) const CATS: [&str; 14] = [
    "Adventure", "Mining", "Herbalism", "Skinning", "Fishing", "Blacksmithing", "Alchemy",
    "Engineering", "Enchanting", "Tailoring", "Leatherworking", "Jewelcrafting", "Inscription", "Cooking",
];
pub(crate) const LOG_FILTERS: [&str; 8] = ["All", "Quest", "Loot", "Level", "Achievement", "PvP", "Travel", "System"];
pub(crate) const PVP_FACTIONS: [&str; 3] = ["All", "Alliance", "Horde"];
pub(crate) const ZONE_KINDS: [&str; 8] = [
    "All", "Zones", "Dungeons", "Raids", "Battlegrounds", "Arenas", "Cities / Sanctuaries", "World PvP",
];
pub(crate) const REALM_A: [&str; 12] = [
    "Storm", "Dark", "Silver", "Iron", "Blood", "Frost", "Ember", "Shadow", "Thunder", "Golden", "Moon", "Star",
];
pub(crate) const REALM_B: [&str; 12] = [
    "crest", "wind", "hollow", "spire", "fall", "vale", "reach", "moor", "forge", "haven", "gate", "watch",
];
pub(crate) const SYL1: [&str; 12] = ["Ka", "Mor", "Thal", "Zul", "Bren", "Gor", "Ael", "Shi", "Vor", "Tal", "Dra", "Nym"];
pub(crate) const SYL2: [&str; 10] = ["ga", "di", "we", "ra", "io", "tha", "mar", "ri", "lo", "za"];
pub(crate) const SYL3: [&str; 8] = ["n", "x", "th", "s", "k", "r", "l", "nd"];

pub(crate) const MOBS: &[&str] = &[
    "Kobolds", "Murlocs", "Gnolls", "Defias Bandits", "Timber Wolves", "Harpies", "Forest Trolls", "Skeletons",
    "Ghouls", "Giant Spiders", "Wild Boars", "Ogres", "Imps", "Wraiths", "Scorpids",
];

// potions: (name, icon)
pub(crate) const POT: [(&str, &str); 3] = [("Health Potion", "🧪"), ("Mana Potion", "🔷"), ("Stamina Potion", "⚡")];
