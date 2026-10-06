//! Domain types shared by game systems and the GTK layer.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) enum Wt { Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive }
pub(crate) const ALL_WT: [Wt; 12] = [Wt::Axe, Wt::Bow, Wt::Crossbow, Wt::Dagger, Wt::Fist, Wt::Gun, Wt::Mace, Wt::Polearm, Wt::Staff, Wt::Sword, Wt::Wand, Wt::Warglaive];
impl Wt {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Axe => "Axe", Self::Bow => "Bow", Self::Crossbow => "Crossbow", Self::Dagger => "Dagger", Self::Fist => "Fist Weapon",
            Self::Gun => "Gun", Self::Mace => "Mace", Self::Polearm => "Polearm", Self::Staff => "Staff", Self::Sword => "Sword",
            Self::Wand => "Wand", Self::Warglaive => "Warglaive",
        }
    }
    pub(crate) fn one(self) -> bool { matches!(self, Self::Axe | Self::Dagger | Self::Fist | Self::Mace | Self::Sword | Self::Wand | Self::Warglaive) }
    pub(crate) fn two(self) -> bool { matches!(self, Self::Axe | Self::Bow | Self::Crossbow | Self::Gun | Self::Mace | Self::Polearm | Self::Staff | Self::Sword) }
    pub(crate) fn ranged(self) -> bool { matches!(self, Self::Bow | Self::Crossbow | Self::Gun) }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub(crate) enum Hands { #[default] One, Main, Off, Two }
impl Hands {
    pub(crate) fn name(self) -> &'static str {
        match self { Self::One => "One-Hand", Self::Main => "Main Hand", Self::Off => "Off Hand", Self::Two => "Two-Hand" }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub(crate) enum Kind { #[default] Cosmetic, Cloth, Leather, Mail, Plate, Cloak, Necklace, Ring, Trinket, Shield, Held, Weapon }
impl Kind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Cosmetic => "Cosmetic", Self::Cloth => "Cloth", Self::Leather => "Leather", Self::Mail => "Mail",
            Self::Plate => "Plate", Self::Cloak => "Cloak", Self::Necklace => "Necklace", Self::Ring => "Ring",
            Self::Trinket => "Trinket", Self::Shield => "Shield", Self::Held => "Held In Off-hand", Self::Weapon => "Weapon",
        }
    }
    pub(crate) fn armor_idx(self) -> Option<usize> {
        match self { Self::Cloth => Some(0), Self::Leather => Some(1), Self::Mail => Some(2), Self::Plate => Some(3), _ => None }
    }
}
pub(crate) const ARMOR: [Kind; 4] = [Kind::Cloth, Kind::Leather, Kind::Mail, Kind::Plate];
pub(crate) fn armor_kind(s: &str) -> Kind { match s { "Cloth" => Kind::Cloth, "Leather" => Kind::Leather, "Mail" => Kind::Mail, _ => Kind::Plate } }

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum K { Dmg, Dot, Leech, Heal, Absorb, Stun, Kick, Buff, Guard, Util }
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum R { Mana, Stam, Free }

pub(crate) struct Ab {
    pub(crate) l: u32, pub(crate) n: &'static str, pub(crate) k: K, pub(crate) p: u32,
    pub(crate) cd: u32, pub(crate) r: R, pub(crate) c: u32, pub(crate) sp: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Role { Tank, Healer, Melee, Ranged }
impl Role {
    pub(crate) fn name(self) -> &'static str { match self { Self::Tank => "Tank", Self::Healer => "Healer", Self::Melee => "Melee DPS", Self::Ranged => "Ranged DPS" } }
}

pub(crate) struct Class {
    pub(crate) name: &'static str, pub(crate) armor: &'static str,
    pub(crate) specs: &'static [(&'static str, Role)], pub(crate) abilities: &'static [Ab],
    pub(crate) weapons: &'static [Wt], pub(crate) dual: bool, pub(crate) shield: bool, pub(crate) held: bool,
}

pub(crate) struct Realm { pub(crate) name: String, pub(crate) pop: u32, pub(crate) tier: usize }
#[derive(Clone)]
pub(crate) struct Zone { pub(crate) name: String, pub(crate) lo: u32, pub(crate) hi: u32, pub(crate) terr: String, pub(crate) inst: String }

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) enum Tier { Normal, Elite, Dungeon, Raid, WorldBoss }
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub(crate) enum QKind { #[default] Kill, Gather }
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Step { pub(crate) text: String, pub(crate) done: bool }
pub(crate) fn make_steps(n: u32) -> Vec<Step> { (0..n).map(|_| Step { text: String::new(), done: false }).collect() }

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Quest {
    pub(crate) title: String, pub(crate) giver: String, pub(crate) tier: Tier, pub(crate) cat: usize, pub(crate) goal: u32, pub(crate) progress: u32,
    pub(crate) chain: bool, pub(crate) part: u32,
    #[serde(default)] pub(crate) zone: String, #[serde(default)] pub(crate) bonus: u32, #[serde(default)] pub(crate) qk: QKind,
    #[serde(default)] pub(crate) quest_level: u32,
    #[serde(default)] pub(crate) target: String, #[serde(default)] pub(crate) steps: Vec<Step>, #[serde(default)] pub(crate) tries: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Item {
    pub(crate) name: String, #[serde(alias = "rarity")] pub(crate) quality: usize, pub(crate) ilvl: u32,
    #[serde(default)] pub(crate) slot: usize, #[serde(default)] pub(crate) kind: Kind, #[serde(default)] pub(crate) wt: Option<Wt>,
    #[serde(default)] pub(crate) hands: Hands, #[serde(default)] pub(crate) stats: Vec<(usize, u32)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct LogEntry { pub(crate) time: String, pub(crate) cat: String, pub(crate) msg: String }

#[derive(Serialize, Deserialize)]
pub(crate) struct Hero {
    pub(crate) name: String, pub(crate) guild: String, pub(crate) realm: String, pub(crate) realm_tier: usize,
    pub(crate) faction: String, pub(crate) race: String, pub(crate) class: usize, pub(crate) spec: usize,
    pub(crate) level: u32, pub(crate) xp: u32, pub(crate) gold: u32, pub(crate) talents: u32, pub(crate) done: u32,
    pub(crate) honor: u32, pub(crate) wins: u32, pub(crate) losses: u32, pub(crate) zone: String, pub(crate) zone_inst: String,
    pub(crate) gear: Vec<Option<Item>>, pub(crate) prof: Vec<u32>, pub(crate) achievements: Vec<String>, pub(crate) quests: Vec<Quest>, pub(crate) log: Vec<LogEntry>,
    #[serde(default)] pub(crate) bag: Vec<Item>, #[serde(default)] pub(crate) hp: u32, #[serde(default)] pub(crate) mana: u32,
    #[serde(default)] pub(crate) sta: u32, #[serde(default)] pub(crate) pots: [u32; 3],
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Save { pub(crate) heroes: Vec<Hero> }

#[derive(Clone, Copy)]
pub(crate) struct Stats { pub(crate) hp: u32, pub(crate) atk: u32, pub(crate) def: u32, pub(crate) crit: u32, pub(crate) heal: u32, pub(crate) mana: u32, pub(crate) sta: u32, pub(crate) hmul: u32 }

#[derive(Clone)]
pub(crate) struct Fighter {
    pub(crate) name: String, pub(crate) faction: String, pub(crate) race: String, pub(crate) class: usize, pub(crate) spec: usize, pub(crate) level: u32,
    pub(crate) gear: Vec<Option<Item>>, pub(crate) talents: u32, pub(crate) cur: Option<(u32, u32, u32)>,
}

/// One fighter's live state during a duel.
pub(crate) struct Side {
    pub(crate) st: Stats, pub(crate) hp: i64, pub(crate) mana: u32, pub(crate) sta: u32, pub(crate) abs: Vec<&'static Ab>,
    pub(crate) cds: Vec<u32>, pub(crate) absorb: i64, pub(crate) buff: (u32, u32), pub(crate) guard: (u32, u32),
    pub(crate) dot: (u32, u32), pub(crate) stunned: bool, pub(crate) kicked: bool,
}

pub(crate) struct Duel { pub(crate) won: bool, pub(crate) lines: Vec<String>, pub(crate) hp: u32, pub(crate) mana: u32, pub(crate) sta: u32 }
