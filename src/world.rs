
use crate::{config::*, model::*, utils::*};
use std::collections::HashSet;
use std::path::PathBuf;

// ---------- realms ----------
impl Realm {
    pub(crate) fn label(&self) -> String {
        format!("{} — {} players online ({})", self.name, commas(self.pop), ["High", "Medium", "Low"][self.tier])
    }
}
pub(crate) fn make_realms() -> Vec<Realm> {
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
pub(crate) fn inst_bonus(inst: &str) -> u32 { match inst { "Raid" => 50, "Dungeon" => 25, _ => 0 } }

impl Zone {
    pub(crate) fn level_text(&self) -> String {
        if self.lo == self.hi { self.lo.to_string() } else { format!("{}-{}", self.lo, self.hi) }
    }
    pub(crate) fn open_to(&self, faction: &str, level: u32) -> bool {
        (self.terr != "Alliance" || faction == "Alliance")
            && (self.terr != "Horde" || faction == "Horde")
            && level >= self.lo
    }
    pub(crate) fn matches_kind(&self, k: u32) -> bool {
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

pub(crate) fn parse_level(s: &str) -> Option<(u32, u32)> {
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

pub(crate) fn parse_line(line: &str) -> Option<Zone> {
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

pub(crate) fn junk(n: &str) -> bool {
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

pub(crate) fn parse_zones(raw: &str) -> Vec<Zone> {
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

pub(crate) fn load_zones() -> Vec<Zone> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("zones.tsv");
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| FALLBACK_ZONES.to_string());
    let z = parse_zones(&raw);
    if z.is_empty() { parse_zones(FALLBACK_ZONES) } else { z }
}

// ---------- quests ----------

