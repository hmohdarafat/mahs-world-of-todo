use gtk::{glib, prelude::*};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::Rc,
};
use Role::{Healer, Melee, Ranged, Tank};

const APP_TITLE: &str = "MAH's World of Todo";
const MAX_LEVEL: u32 = 90;
const MAX_LOG: usize = 1000;
const ZONE_ROWS: usize = 80;
const PVP_SHOWN: usize = 40;
const PROF_LEVEL: u32 = 5;
const GATHER_MAX: usize = 4; // CATS[1..=4] are gathering, the rest are crafting

const FACTIONS: [&str; 2] = ["Horde", "Alliance"];
const HORDE: [&str; 6] = ["Orc", "Undead (Forsaken)", "Tauren", "Troll", "Blood Elf", "Goblin"];
const ALLIANCE: [&str; 6] = ["Human", "Dwarf", "Night Elf", "Gnome", "Draenei", "Worgen"];
const NEUTRAL: [&str; 2] = ["Pandaren", "Dracthyr"];
const CATS: [&str; 14] = [
    "Adventure", "Mining", "Herbalism", "Skinning", "Fishing", "Blacksmithing", "Alchemy",
    "Engineering", "Enchanting", "Tailoring", "Leatherworking", "Jewelcrafting", "Inscription", "Cooking",
];
const SLOTS: [&str; 6] = ["Head", "Shoulders", "Chest", "Legs", "Weapon", "Trinket"];
const NOUNS: [&str; 6] = ["Helm", "Pauldrons", "Chestguard", "Leggings", "Blade", "Charm"];
const PREFIX: [&str; 4] = ["Worn", "Sturdy", "Runed", "Dragonforged"];
const RARITY: [&str; 4] = ["#9d9d9d", "#1eff00", "#0070dd", "#a335ee"];
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
}

const CLASSES: [Class; 13] = [
    Class { name: "Death Knight", armor: "Plate", specs: &[("Blood", Tank), ("Frost", Melee), ("Unholy", Melee)],
            abilities: ["Death Strike", "Obliterate", "Death and Decay", "Anti-Magic Shell", "Death Grip"] },
    Class { name: "Demon Hunter", armor: "Leather", specs: &[("Havoc", Melee), ("Vengeance", Tank), ("Devourer", Melee)],
            abilities: ["Demon's Bite", "Eye Beam", "Metamorphosis", "Fel Rush", "Immolation Aura"] },
    Class { name: "Druid", armor: "Leather", specs: &[("Balance", Ranged), ("Feral", Melee), ("Guardian", Tank), ("Restoration", Healer)],
            abilities: ["Moonfire", "Rejuvenation", "Rake", "Bear Form", "Starfire"] },
    Class { name: "Evoker", armor: "Mail", specs: &[("Augmentation", Ranged), ("Devastation", Ranged), ("Preservation", Healer)],
            abilities: ["Living Flame", "Fire Breath", "Azure Strike", "Emerald Blossom", "Hover"] },
    Class { name: "Hunter", armor: "Mail", specs: &[("Beast Mastery", Ranged), ("Marksmanship", Ranged), ("Survival", Melee)],
            abilities: ["Arcane Shot", "Kill Command", "Multi-Shot", "Aspect of the Cheetah", "Concussive Shot"] },
    Class { name: "Mage", armor: "Cloth", specs: &[("Arcane", Ranged), ("Fire", Ranged), ("Frost", Ranged)],
            abilities: ["Frostbolt", "Fire Blast", "Frost Nova", "Blink", "Polymorph"] },
    Class { name: "Monk", armor: "Leather", specs: &[("Brewmaster", Tank), ("Mistweaver", Healer), ("Windwalker", Melee)],
            abilities: ["Tiger Palm", "Blackout Kick", "Vivify", "Roll", "Spinning Crane Kick"] },
    Class { name: "Paladin", armor: "Plate", specs: &[("Holy", Healer), ("Protection", Tank), ("Retribution", Melee)],
            abilities: ["Crusader Strike", "Holy Light", "Judgment", "Divine Shield", "Hammer of Justice"] },
    Class { name: "Priest", armor: "Cloth", specs: &[("Discipline", Healer), ("Holy", Healer), ("Shadow", Ranged)],
            abilities: ["Smite", "Power Word: Shield", "Flash Heal", "Shadow Word: Pain", "Mind Blast"] },
    Class { name: "Rogue", armor: "Leather", specs: &[("Assassination", Melee), ("Outlaw", Melee), ("Subtlety", Melee)],
            abilities: ["Sinister Strike", "Stealth", "Eviscerate", "Evasion", "Kick"] },
    Class { name: "Shaman", armor: "Mail", specs: &[("Elemental", Ranged), ("Enhancement", Melee), ("Restoration", Healer)],
            abilities: ["Lightning Bolt", "Healing Wave", "Flame Shock", "Earth Shock", "Ghost Wolf"] },
    Class { name: "Warlock", armor: "Cloth", specs: &[("Affliction", Ranged), ("Demonology", Ranged), ("Destruction", Ranged)],
            abilities: ["Shadow Bolt", "Corruption", "Immolate", "Fear", "Drain Life"] },
    Class { name: "Warrior", armor: "Plate", specs: &[("Arms", Melee), ("Fury", Melee), ("Protection", Tank)],
            abilities: ["Charge", "Rend", "Thunder Clap", "Hamstring", "Shield Bash"] },
];

fn spec_names(c: usize) -> Vec<&'static str> { CLASSES[c].specs.iter().map(|s| s.0).collect() }

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

// ---------- data ----------
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

#[derive(Serialize, Deserialize, Clone)]
struct Item { name: String, rarity: usize, ilvl: u32 }

fn make_item(level: u32, armor: &str, slot: usize, rarity: usize) -> Item {
    let name = if slot < 4 {
        format!("{} {} {}", PREFIX[rarity], armor, NOUNS[slot])
    } else {
        format!("{} {}", PREFIX[rarity], NOUNS[slot])
    };
    Item { name, rarity, ilvl: level + 2 + rarity as u32 * 3 + rnd(3) }
}

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
            achievements: vec![], quests: vec![], log: vec![],
        }
    }

    fn need(&self) -> u32 { 100 + self.level * 50 }

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

        if rnd(100) < loot {
            let roll = rnd(100) + (mult - 1) * 6;
            let rarity = match roll { 130.. => 3, 100..=129 => 2, 60..=99 => 1, _ => 0 };
            let slot = rnd(SLOTS.len() as u32) as usize;
            let item = make_item(self.level, CLASSES[self.class].armor, slot, rarity);
            let better = self.gear[slot].as_ref().map_or(true, |c| item.ilvl > c.ilvl);
            if better {
                msgs.push(m("Loot", format!("🎁 Loot equipped: {} (ilvl {})", item.name, item.ilvl)));
                self.gear[slot] = Some(item);
            } else {
                self.gold += item.ilvl;
                msgs.push(m("Loot", format!("🎁 Loot sold: {} for {} gold", item.name, item.ilvl)));
            }
        }

        while self.xp >= self.need() && self.level < MAX_LEVEL {
            let n = self.need();
            self.xp -= n;
            self.level += 1;
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
            ("Epic Find", self.gear.iter().flatten().any(|i| i.rarity == 3)),
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
        let v: Vec<u32> = self.gear.iter().flatten().map(|i| i.ilvl).collect();
        if v.is_empty() { 0 } else { v.iter().sum::<u32>() / v.len() as u32 }
    }

    fn stats(&self) -> Stats {
        let g: u32 = self.gear.iter().flatten().map(|i| i.ilvl).sum();
        let rar: u32 = self.gear.iter().flatten().map(|i| i.rarity as u32).sum();
        let (hp_m, atk_m, def_m) = match self.role() {
            Tank => (150, 70, 150),
            Healer => (110, 60, 100),
            Melee => (110, 120, 90),
            Ranged => (90, 125, 80),
        };
        let armor = match CLASSES[self.class].armor { "Plate" => 130, "Mail" => 115, "Leather" => 100, _ => 85 };
        let hp = (120 + self.level * 25 + g * 4) * hp_m / 100;
        Stats {
            hp,
            atk: (10 + self.level * 3 + g) * atk_m / 100 + self.abilities.len() as u32 * 3 + self.talents * 4,
            def: (self.level + g / 2) * def_m / 100 * armor / 100,
            crit: (5 + self.talents + rar).min(40),
            heal: if self.role() == Healer { hp * 6 / 100 } else { 0 },
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
        let mut t = format!("Abilities: {ab}\nTalents: {}\n", self.talents);
        for (i, slot) in SLOTS.iter().enumerate() {
            match &self.gear[i] {
                Some(it) => t += &format!("{slot}: {} (ilvl {})\n", it.name, it.ilvl),
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
    let armor = CLASSES[class].armor;
    let gear: Vec<Option<Item>> = (0..SLOTS.len()).map(|slot| {
        if rnd(100) < 90 {
            let rarity = match rnd(100) { 0..=49 => 0, 50..=79 => 1, 80..=94 => 2, _ => 3 };
            Some(make_item(level, armor, slot, rarity))
        } else { None }
    }).collect();
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
fn load() -> Save {
    std::fs::read_to_string(save_path()).ok()
        .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
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
        nb.append_page(&sheet_scroll, Some(&gtk::Label::new(Some("🧙 Character"))));
        nb.append_page(&logs_page, Some(&gtk::Label::new(Some("📋 Logs"))));

        let game = gtk::Box::new(ve, 10);
        pad(&game, 12);
        game.append(&head); game.append(&xp_bar); game.append(&stats); game.append(&nb);

        let stack = gtk::Stack::new();
        stack.add_named(&select, Some("select"));
        stack.add_named(&game, Some("game"));
        let win = gtk::ApplicationWindow::builder().application(app)
            .title(APP_TITLE).default_width(940).default_height(820).build();
        win.set_child(Some(&stack));

        let ui = Rc::new(Ui {
            save: RefCell::new(load()), active: Cell::new(None), confirm_del: Cell::new(None),
            realms: make_realms(), zones: load_zones(), opps: RefCell::new(vec![]),
            stack, sel_list, title, xp_bar, stats, giver, entry, goal, tier, cat, chain, qlist, status,
            zsearch, zkind, zcount, zlist, pvp_head, pvp_result, pvp_list, log_filter, log_view, sheet,
        });

        { let u = ui.clone(); sel_new.connect_clicked(move |_| u.show_create()); }
        { let u = ui.clone(); add_btn.connect_clicked(move |_| u.add_quest()); }
        { let u = ui.clone(); ui.entry.connect_activate(move |_| u.add_quest()); }
        { let u = ui.clone(); save_btn.connect_clicked(move |_| u.finish(vec![m("System", "💾 Character saved.")])); }
        { let u = ui.clone(); switch_btn.connect_clicked(move |_| u.show_select()); }
        { let u = ui.clone(); pvp_refresh.connect_clicked(move |_| u.new_opponents()); }
        { let u = ui.clone(); log_clear.connect_clicked(move |_| u.clear_logs()); }
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
            t += &format!("{} armor · {}\n\n<b>Gear</b>\n", c.armor, role.name());
            for (i, slot) in SLOTS.iter().enumerate() {
                match &h.gear[i] {
                    Some(it) => t += &format!("{slot}: <span foreground='{}'>{}</span> (ilvl {})\n", RARITY[it.rarity], it.name, it.ilvl),
                    None => t += &format!("{slot}: —\n"),
                }
            }
            t += "\n<b>Professions</b>\n";
            let profs: Vec<String> = h.prof.iter().enumerate().filter(|(_, p)| **p > 0)
                .map(|(i, p)| format!("{}: {}", CATS[i + 1], p)).collect();
            t += &if profs.is_empty() { "—".to_string() } else { profs.join("\n") };
            t += "\n\n<b>Achievements</b>\n";
            t += &if h.achievements.is_empty() { "—".to_string() } else { h.achievements.join("\n") };
            self.sheet.set_markup(&t);
        }
        self.refresh_zones();
        self.refresh_logs();
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
            i.set_text(&format!("{} armor · class roles: {} · selected spec role: {}",
                                cl.armor, roles.join(", "), cl.specs[si].1.name()));
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
