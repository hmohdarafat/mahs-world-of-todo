//! Single source of truth for derived character stats.

use crate::{
    data::{abilities::spec_ok, classes::CLASSES, items::{QMULT, STAT_NAMES}},
    items::primary,
    model::*,
};

impl Role {
    /// (HP %, attack %, defense %) multipliers.
    pub(crate) fn mults(self) -> (u32, u32, u32) {
        match self {
            Self::Tank => (150, 70, 150),
            Self::Healer => (110, 60, 100),
            Self::Melee => (110, 120, 90),
            Self::Ranged => (90, 125, 80),
        }
    }
}

impl Class {
    pub(crate) fn armor_mult(&self) -> u32 {
        match self.armor { "Plate" => 130, "Mail" => 115, "Leather" => 100, _ => 85 }
    }
}

impl Item {
    /// Raw gear power of one item (shared by stats, item_power and the breakdown).
    pub(crate) fn gear_power(&self) -> u32 {
        let w = if self.is_two() { 3 } else { 2 };
        self.ilvl * QMULT[self.quality.min(6)] / 100 * w / 2
    }
}

pub(crate) fn known_count(class: usize, spec: usize, level: u32) -> u32 {
    let sn = CLASSES[class].specs[spec].0;
    CLASSES[class].abilities.iter().filter(|a| a.l <= level && spec_ok(a, sn)).count() as u32
}

/// Aggregated numbers from a set of equipped items.
#[derive(Clone, Copy, Default)]
pub(crate) struct GearAgg {
    pub(crate) gp: u32,
    pub(crate) qsum: u32,
    pub(crate) shield: u32,
    pub(crate) tot: [u32; 8],
}

impl GearAgg {
    pub(crate) fn of<'a>(items: impl Iterator<Item = &'a Item>) -> Self {
        let mut a = Self::default();
        for it in items.filter(|i| i.kind != Kind::Cosmetic) {
            a.gp += it.gear_power();
            a.qsum += it.quality.min(5) as u32;
            if it.kind == Kind::Shield { a.shield += it.ilvl / 2; }
            for &(s, v) in &it.stats { a.tot[s.min(7)] += v; }
        }
        a
    }
}

/// Every additive term of every derived stat.
#[derive(Clone, Copy)]
pub(crate) struct Terms {
    pub(crate) role: Role,
    pub(crate) prim: usize,
    pub(crate) gs: u32,
    pub(crate) hp_m: u32, pub(crate) atk_m: u32, pub(crate) def_m: u32, pub(crate) armor: u32,
    pub(crate) hp_core: u32, pub(crate) hp_sta: u32, pub(crate) hp_vers: u32, pub(crate) hp: u32,
    pub(crate) atk_core: u32, pub(crate) atk_ab: u32, pub(crate) atk_tal: u32, pub(crate) atk_prim: u32,
    pub(crate) off_raw: u32, pub(crate) atk_off: u32, pub(crate) atk_haste: u32, pub(crate) atk_vers: u32,
    pub(crate) atk: u32,
    pub(crate) def_core: u32, pub(crate) def_mastery: u32, pub(crate) def_shield: u32, pub(crate) def: u32,
    pub(crate) crit_base: u32, pub(crate) crit_tal: u32, pub(crate) crit_q: u32, pub(crate) crit_rating: u32,
    pub(crate) crit: u32,
    pub(crate) mana_core: u32, pub(crate) mana_int: u32, pub(crate) mana: u32,
    pub(crate) sta_core: u32, pub(crate) sta_bonus: u32, pub(crate) sta: u32,
    pub(crate) heal: u32,
}

impl Terms {
    pub(crate) fn stats(&self) -> Stats {
        Stats {
            hp: self.hp, atk: self.atk, def: self.def, crit: self.crit, heal: self.heal,
            mana: self.mana, sta: self.sta,
            hmul: if self.role == Role::Healer { 140 } else { 100 },
        }
    }
}

pub(crate) fn terms(class: usize, spec: usize, level: u32, talents: u32, g: &GearAgg) -> Terms {
    let c = &CLASSES[class];
    let role = c.specs[spec].1;
    let prim = primary(class, role);
    let t = &g.tot;
    let gs = g.gp * 6 / 14;
    let (hp_m, atk_m, def_m) = role.mults();
    let armor = c.armor_mult();
    let off_raw = t[1] + t[2] + t[3] - t[prim];

    let hp_core = (120 + level * 25 + gs * 4) * hp_m / 100;
    let (hp_sta, hp_vers) = (t[0] * 5, t[7] * 2);
    let hp = hp_core + hp_sta + hp_vers;

    let atk_core = (10 + level * 3 + gs) * atk_m / 100;
    let atk_ab = known_count(class, spec, level) * 3;
    let atk_tal = talents * 4;
    let atk_prim = t[prim];
    let atk_off = off_raw / 4;
    let (atk_haste, atk_vers) = (t[4] / 2, t[7] / 3);
    let atk = atk_core + atk_ab + atk_tal + atk_prim + atk_off + atk_haste + atk_vers;

    let def_core = (level + gs / 2) * def_m / 100 * armor / 100;
    let def_mastery = t[6] / 2;
    let def_shield = g.shield;
    let def = def_core + def_mastery + def_shield;

    let (crit_base, crit_tal, crit_q, crit_rating) = (5, talents, g.qsum / 3, t[5] / 6);
    let crit = (crit_base + crit_tal + crit_q + crit_rating).min(40);

    let (mana_core, mana_int) = (60 + level * 6, t[3] * 2);
    let (sta_core, sta_bonus) = (60 + level * 4, t[0] / 2);

    Terms {
        role, prim, gs, hp_m, atk_m, def_m, armor,
        hp_core, hp_sta, hp_vers, hp,
        atk_core, atk_ab, atk_tal, atk_prim, off_raw, atk_off, atk_haste, atk_vers, atk,
        def_core, def_mastery, def_shield, def,
        crit_base, crit_tal, crit_q, crit_rating, crit,
        mana_core, mana_int, mana: mana_core + mana_int,
        sta_core, sta_bonus, sta: sta_core + sta_bonus,
        heal: if role == Role::Healer { hp * 6 / 100 } else { 0 },
    }
}

pub(crate) fn derive_stats(class: usize, spec: usize, level: u32, talents: u32, g: &GearAgg) -> Stats {
    terms(class, spec, level, talents, g).stats()
}

/// What one raw gear stat does for this class/spec.
pub(crate) fn stat_effect(class: usize, spec: usize, s: usize, v: u32) -> String {
    let prim = primary(class, CLASSES[class].specs[spec].1);
    let effect = match s {
        0 => format!("HP +{} · Stamina +{}", v * 5, v / 2),
        1..=3 => {
            let atk = if s == prim { v } else { v / 4 };
            let mut out = format!("Attack +{atk}");
            if s == 3 { out.push_str(&format!(" · Mana +{}", v * 2)); }
            out
        }
        4 => format!("Attack +{}", v / 2),
        5 => format!("Crit +{}%", v / 6),
        6 => format!("Defense +{}", v / 2),
        7 => format!("HP +{} · Attack +{}", v * 2, v / 3),
        _ => String::new(),
    };
    format!("+{v} {} → {}", STAT_NAMES[s.min(7)], effect)
}