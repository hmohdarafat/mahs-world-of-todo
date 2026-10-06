//! Static item naming, quality, and equipment data.

// ---------- equipment tables ----------
pub(crate) const SLOTS: [&str; 16] = [
    "Head", "Shoulders", "Chest", "Wrist", "Hands", "Waist", "Legs", "Feet", "Cloak", "Necklace",
    "Ring 1", "Ring 2", "Trinket 1", "Trinket 2", "Main Hand", "Off Hand",
];
pub(crate) const S_MAIN: usize = 14;
pub(crate) const S_OFF: usize = 15;

pub(crate) const QUALITY: [&str; 7] = ["Poor", "Common", "Uncommon", "Rare", "Epic", "Legendary", "Heirloom"];
pub(crate) const QCOL: [&str; 7] = ["#9d9d9d", "#ffffff", "#1eff00", "#0070dd", "#a335ee", "#ff8000", "#00ccff"];
pub(crate) const QMULT: [u32; 7] = [80, 100, 112, 128, 150, 190, 140]; // power multiplier (%)
pub(crate) const QILVL: [i32; 7] = [-2, 0, 2, 5, 9, 15, 6]; // item level offset
pub(crate) const QSTAT: [u32; 7] = [0, 0, 30, 34, 38, 44, 34]; // bonus stat size (% of ilvl)
pub(crate) const QSELL: [u32; 7] = [1, 2, 3, 5, 8, 25, 0]; // vendor sell factor
pub(crate) const QBUY: [u32; 7] = [40, 60, 100, 250, 600, 1500, 0]; // store price multiplier (%)
pub(crate) const Q_LEGENDARY: usize = 5;
pub(crate) const Q_HEIRLOOM: usize = 6;

pub(crate) const QUALITY_GUIDE: &str = "Poor (grey) — low-level Normal quests; vendor trash, just sell it.\n\
Common (white) — Normal quests and vendors; sell or discard.\n\
Uncommon (green) — any quest, crafting, the Store and random drops; always has an \"of the …\" stat suffix.\n\
Rare (blue) — crafting, quests, the Store (level 20+), Dungeon/Raid tiers and random drops.\n\
Epic (purple) — high-end crafting, Dungeon/Raid/World Boss tiers, rare Store stock (level 40+).\n\
Legendary (orange) — level 40+ Raid / World Boss quests only, ~1.5–3% per drop; unique proper names.\n\
Heirloom (cyan) — a cache every 20th completed quest; item level scales with your level.\n\
Cosmetic — appearance-only armor with no stats; worn only in empty slots.";

pub(crate) const STAT_NAMES: [&str; 8] = [
    "Stamina", "Strength", "Agility", "Intellect", "Haste", "Critical Strike", "Mastery", "Versatility",
];
pub(crate) const SUFFIXES: [(&str, [usize; 2]); 8] = [
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
pub(crate) const N_DAGGER: &[&str] = &["Dagger", "Kris", "Stiletto", "Dirk", "Shard", "Tooth", "Blade"];
pub(crate) const N_SWORD1: &[&str] = &["Sword", "Saber", "Blade", "Rapier", "Cutlass", "Scimitar", "Cleaver"];
pub(crate) const N_SWORD2: &[&str] = &["Greatsword", "Claymore", "Longsword", "Zweihander", "Blade"];
pub(crate) const N_AXE1: &[&str] = &["Axe", "Hatchet", "Cleaver", "Chopper", "Hackblade"];
pub(crate) const N_AXE2: &[&str] = &["Greataxe", "Battleaxe", "Decapitator", "Chopper", "Hew", "Cleaver"];
pub(crate) const N_MACE1: &[&str] = &["Mace", "Scepter", "Hammer", "Cudgel", "Truncheon", "Club", "Bludgeon"];
pub(crate) const N_MACE2: &[&str] = &["Warhammer", "Greatmace", "Mallet", "Maul", "Smasher"];
pub(crate) const N_FIST: &[&str] = &["Claws", "Fist", "Handblades", "Knuckles", "Talon", "Grasp"];
pub(crate) const N_POLE: &[&str] = &["Polearm", "Halberd", "Spear", "Glaive", "Trident", "Scythe", "Pike"];
pub(crate) const N_STAFF: &[&str] = &["Staff", "Spire", "Rod", "Greatstaff", "Cane", "Pillar", "Stave"];
pub(crate) const N_BOW: &[&str] = &["Longbow", "Recurve", "Greatbow", "Composite Bow", "Bow"];
pub(crate) const N_XBOW: &[&str] = &["Crossbow", "Arbalest", "Repeater", "Heavy Crossbow"];
pub(crate) const N_GUN: &[&str] = &["Rifle", "Musket", "Blunderbuss", "Shotgun", "Carabine", "Hand-Cannon"];
pub(crate) const N_WAND: &[&str] = &["Wand", "Baton", "Rod", "Scepter", "Branch"];
pub(crate) const N_GLAIVE: &[&str] = &["Warglaive", "Twin Glaive", "Felglaive"];
pub(crate) const N_SHIELD: &[&str] = &["Shield", "Bulwark", "Aegis", "Greatshield", "Barricade", "Defender"];
pub(crate) const N_HELD: &[&str] = &["Tome", "Orb", "Grimoire", "Lantern", "Vessel", "Branch", "Talisman", "Icon"];
pub(crate) const N_CLOAK: &[&str] = &["Cloak", "Cape", "Drape", "Shroud", "Greatcloak"];
pub(crate) const N_NECK: &[&str] = &["Necklace", "Pendant", "Amulet", "Choker", "Locket", "Medallion"];
pub(crate) const N_RING: &[&str] = &["Ring", "Band", "Signet", "Loop", "Seal"];
pub(crate) const N_TRINKET: &[&str] = &["Charm", "Totem", "Idol", "Relic", "Figurine", "Fetish", "Badge", "Orb"];

// armor nouns: [slot][Cloth, Leather, Mail, Plate]
pub(crate) const ARMOR_NOUNS: [[&[&str]; 4]; 8] = [
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
pub(crate) const M_CLOTH: &[&str] = &["Linen", "Woolen", "Silk", "Mageweave", "Runecloth", "Netherweave", "Frostweave", "Embersilk"];
pub(crate) const X_CLOTH: &[&str] = &["Shadowsilk", "Moonweave", "Starthread", "Voidcloth", "Dreamweave"];
pub(crate) const M_LEATHER: &[&str] = &["Tanned Leather", "Sylvan Leather", "Cured Leather", "Hardened Leather", "Rugged Leather", "Knothide", "Drakehide", "Wildhide"];
pub(crate) const X_LEATHER: &[&str] = &["Dragonscale", "Emberleather", "Wyrmhide", "Stormhide", "Nightstalker"];
pub(crate) const M_MAIL: &[&str] = &["Ringed", "Linked Chain", "Scaled", "Bronze Chain", "Mithril Chain", "Heavy Chain", "Brigandine", "Riveted"];
pub(crate) const X_MAIL: &[&str] = &["Dragonmail", "Dreadmail", "Frostlink", "Sunmail", "Stormforged"];
pub(crate) const M_PLATE: &[&str] = &["Tempered Steel", "Forged Iron", "Bronze", "Mithril", "Thorium", "Truesilver", "Obsidian", "Adamantite", "Saronite"];
pub(crate) const X_PLATE: &[&str] = &["Dreadsteel", "Titansteel", "Sunforged", "Voidforged", "Starmetal"];
pub(crate) const M_METAL: &[&str] = &["Copper", "Bronze", "Iron", "Steel", "Mithril", "Thorium", "Obsidian", "Cobalt", "Truesilver"];
pub(crate) const X_METAL: &[&str] = &["Dreadsteel", "Frostforged", "Voidtouched", "Sunforged", "Bloodforged"];
pub(crate) const M_WOOD: &[&str] = &["Oak", "Ashwood", "Ironwood", "Ebony", "Yew", "Pliable", "Gnarled", "Spellwoven"];
pub(crate) const X_WOOD: &[&str] = &["Ancient", "Sunbloom", "Wyrmwood", "Nightbranch", "Moonwood"];
pub(crate) const M_TECH: &[&str] = &["Gnomish", "Goblin", "Brass", "Steel-barreled", "Tinkered", "Engineered", "Iron-bound"];
pub(crate) const X_TECH: &[&str] = &["Overclocked", "Explosive", "Masterwork", "Prototype"];
pub(crate) const M_JEWEL: &[&str] = &["Copper", "Silver", "Gold", "Jade", "Moonstone", "Citrine", "Sapphire", "Ruby", "Onyx"];
pub(crate) const X_JEWEL: &[&str] = &["Dragon-eye", "Starfire", "Voidstone", "Sunstone", "Bloodgem"];
pub(crate) const M_TRINKET: &[&str] = &["Carved", "Ancient", "Glowing", "Shimmering", "Runed", "Tarnished"];
pub(crate) const X_TRINKET: &[&str] = &["Primal", "Arcane", "Eldritch", "Void-touched"];
pub(crate) const M_SHIELD: &[&str] = &["Wooden", "Bronze", "Iron", "Steel", "Mithril", "Thorium", "Reinforced"];
pub(crate) const M_HELD: &[&str] = &["Arcane", "Etched", "Gilded", "Ancient", "Crystalline", "Leatherbound"];
pub(crate) const X_HELD: &[&str] = &["Eldritch", "Runic", "Starlit", "Voidbound"];

// name parts
pub(crate) const POOR_ADJ: &[&str] = &["Worn", "Cracked", "Frayed", "Rusty", "Crude", "Tattered", "Battered", "Chipped", "Rotting", "Splintered", "Shoddy", "Ragged"];
pub(crate) const COMMON_ADJ: &[&str] = &["Plain", "Simple", "Standard", "Sturdy", "Apprentice's", "Recruit's", "Journeyman's"];
pub(crate) const RARE_ADJ: &[&str] = &["Reinforced", "Fine", "Superior", "Masterwork", "Gleaming", "Polished", "Runic", "Gladiator's"];
pub(crate) const THEME_ADJ: &[&str] = &["Desecrated", "Vengeful", "Cataclysmic", "Dreadful", "Corrupted", "Radiant", "Merciless", "Eternal", "Shattered", "Sinister", "Hallowed", "Abyssal", "Wrathful", "Spectral"];
pub(crate) const HEIR_ADJ: &[&str] = &["Weathered", "Burnished", "Ancestral", "Timeworn", "Gilded", "Venerable"];
pub(crate) const COSM_ADJ: &[&str] = &["Festive", "Ornate", "Tournament", "Brewfest", "Winter Veil", "Lunar", "Midsummer", "Gilded"];
pub(crate) const GROUP_ADJ: &[&str] = &["Fallen", "Burning", "Silent", "Frozen", "Crimson", "Ashen", "Forsaken", "Shattered"];
pub(crate) const GROUP_NOUN: &[&str] = &["Vanguard", "Legion", "Covenant", "Brotherhood", "Conclave", "Wardens", "Vigil", "Host"];
pub(crate) const LEG_ADJ: &[&str] = &["Blessed", "Cursed", "Hallowed", "Ancient", "Eternal", "Burning", "Sundered", "Undying"];
pub(crate) const LEG_PART: &[&str] = &["Hand", "Heart", "Fang", "Eye", "Voice", "Wrath", "Shadow", "Soul"];
pub(crate) const CMP_A: &[&str] = &["Gore", "Doom", "Frost", "Blood", "Storm", "Soul", "Night", "Wraith", "Dread", "Grim", "Ember", "Void", "Thunder", "Ash"];
pub(crate) const CMP_B: &[&str] = &["howl", "hammer", "bane", "fang", "reaver", "song", "edge", "scream", "brand", "render", "cleaver", "fury", "wrath", "tongue"];
pub(crate) const WIND_A: &[&str] = &["Wind", "Storm", "Dawn", "Night", "Star", "Flame", "Frost", "Soul"];
pub(crate) const WIND_B: &[&str] = &["seeker", "bringer", "breaker", "caller", "walker", "render", "warden", "bearer"];
pub(crate) const NPC_END: &[&str] = &["ia", "ra", "na", "wen", "dor", "mar", "thia", "gar"];
pub(crate) const BOSS_END: &[&str] = &["gor", "thar", "nos", "zul", "rax", "goth", "dun", "mar"];
pub(crate) const ELF_A: &[&str] = &["Felo", "Aela", "Thala", "Noro", "Vyra", "Lora", "Sili", "Kaela"];
pub(crate) const ELF_B: &[&str] = &["melorn", "thas", "dorei", "anar", "vanis", "thalas", "nore", "estra"];
