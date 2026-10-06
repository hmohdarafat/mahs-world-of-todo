use crate::{config::*, data::creatures::*, model::*, utils::*};
use std::collections::HashSet;

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

// ---------- zones ----------
impl Rel {
    pub(crate) fn name(self) -> &'static str {
        match self { Self::Same => "Your faction", Self::Contested => "Contested", Self::Opposite => "Enemy faction" }
    }
    /// % bonus XP on quests accepted in this zone.
    pub(crate) fn xp_bonus(self) -> u32 { [0, 25, 50][self as usize] }
    /// % chance an encounter is higher level than the hero.
    pub(crate) fn high_chance(self) -> u32 { [10, 25, 40][self as usize] }
}

impl Zone {
    pub(crate) fn level_text(&self) -> String {
        if self.lo == self.hi { self.lo.to_string() } else { format!("{}-{}", self.lo, self.hi) }
    }
    pub(crate) fn open_to(&self, level: u32) -> bool { level >= self.lo }
    pub(crate) fn relation(&self, faction: &str) -> Rel {
        if self.terr == "Contested" { Rel::Contested }
        else if self.terr == faction { Rel::Same }
        else { Rel::Opposite }
    }
    /// k = 0: every zone; k >= 1: zones containing creature type k - 1.
    pub(crate) fn matches_kind(&self, k: u32) -> bool { k == 0 || self.types.contains(&(k as usize - 1)) }
    pub(crate) fn type_text(&self) -> String {
        self.types.iter().map(|&t| CTYPES[t.min(CTYPES.len() - 1)]).collect::<Vec<_>>().join(" · ")
    }
    pub(crate) fn creature_text(&self) -> String {
        self.creatures.iter()
            .map(|(t, n)| format!("{n} ({})", CTYPES[(*t).min(CTYPES.len() - 1)]))
            .collect::<Vec<_>>().join(", ")
    }
    pub(crate) fn type_info(&self) -> String {
        self.types.iter().map(|&t| {
            let t = t.min(CTYPES.len() - 1);
            format!("{}: {}", CTYPES[t], CTYPE_INFO[t])
        }).collect::<Vec<_>>().join("\n")
    }
}

/// 1-3 distinct creature types, 2 creatures per type.
fn roll_creatures() -> (Vec<usize>, Vec<(usize, String)>) {
    let n = 1 + rnd(3) as usize;
    let mut types: Vec<usize> = vec![];
    while types.len() < n {
        let t = rnd(CTYPES.len() as u32) as usize;
        if !types.contains(&t) { types.push(t); }
    }
    types.sort();
    let mut creatures = vec![];
    for &t in &types {
        let pool: &[&str] = CREATURES[t];
        let want = 2.min(pool.len());
        let mut picked: Vec<&str> = vec![];
        while picked.len() < want {
            let c = ps(pool);
            if !picked.contains(&c) { picked.push(c); }
        }
        creatures.extend(picked.into_iter().map(|c| (t, c.to_string())));
    }
    (types, creatures)
}

/// 1-10, then 10-15, 15-20 ... up to MAX_LEVEL.
pub(crate) fn zone_bands() -> Vec<(u32, u32)> {
    let mut v = vec![(1, 10)];
    let mut lo = 10;
    while lo < MAX_LEVEL { v.push((lo, (lo + 5).min(MAX_LEVEL))); lo += 5; }
    v
}

/// 3 zones (Alliance, Horde, Contested) for every level band.
pub(crate) fn generate_zones() -> Vec<Zone> {
    let mut used: HashSet<String> = HashSet::new();
    let mut out = vec![];
    for (lo, hi) in zone_bands() {
        for terr in ZONE_TERR {
            let name = loop {
                let n = format!("{}{}", pick(&ZONE_A), pick(&ZONE_B));
                if used.insert(n.clone()) { break n; }
            };
            let (types, creatures) = roll_creatures();
            out.push(Zone { name, lo, hi, terr: terr.into(), types, creatures });
        }
    }
    out
}

pub(crate) fn starting_zone(zones: &[Zone], faction: &str) -> String {
    zones.iter().find(|z| z.terr == faction).or_else(|| zones.first())
        .map_or_else(String::new, |z| z.name.clone())
}

/// Best zone for this hero: same-faction zone at their level, else any open zone.
pub(crate) fn fallback_zone(zones: &[Zone], faction: &str, level: u32) -> String {
    zones.iter().filter(|z| z.open_to(level) && z.terr == faction).max_by_key(|z| z.lo)
        .or_else(|| zones.iter().filter(|z| z.open_to(level)).max_by_key(|z| z.lo))
        .or_else(|| zones.first())
        .map_or_else(String::new, |z| z.name.clone())
}