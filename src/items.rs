use gtk::glib;
use crate::{config::*, data::{classes::*, items::*}, model::*, utils::*};
use crate::model::{Role::{Healer, Ranged, Tank}, Wt::{Axe, Bow, Crossbow, Dagger, Fist, Gun, Mace, Polearm, Staff, Sword, Wand, Warglaive}};

pub(crate) fn qspan(q: usize, text: &str) -> String {
    let t = glib::markup_escape_text(text);
    if q == 1 { t.to_string() } else { format!("<span foreground='{}'>{}</span>", QCOL[q.min(6)], t) }
}

impl Item {

    pub(crate) fn repair_cost(&self) -> u32 {
        let missing = 100u32.saturating_sub(self.durability);
        if missing == 0 { 0 } else { ((self.ilvl * 3 + 10) * missing / 100).max(1) }
    }

    pub(crate) fn is_two(&self) -> bool { self.kind == Kind::Weapon && self.hands == Hands::Two }

    pub(crate) fn type_label(&self) -> String {
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

    pub(crate) fn label(&self) -> String {
        format!("[{}] {} (ilvl {} · {})", QUALITY[self.quality.min(6)], self.name, self.ilvl, self.type_label())
    }

    pub(crate) fn sell_value(&self) -> u32 {
        let f = QSELL[self.quality.min(6)];
        if f == 0 { 0 } else { (self.ilvl * f / 3).max(1) }
    }

    /// Store price: grows with item level and quality, always well above the sell value.
    pub(crate) fn buy_price(&self) -> u32 {
        let il = self.ilvl;
        let base = il * 4 + il * il / 4 + 10;
        (base * QBUY[self.quality.min(6)] / 100).max(5)
    }

    /// Honor store price: grows with item level; Epic and better cost double.
    pub(crate) fn honor_price(&self) -> u32 {
        (20 + self.ilvl * 3) * if self.quality >= 4 { 2 } else { 1 }
    }

    pub(crate) fn stats_inline(&self) -> String {
        if self.kind == Kind::Cosmetic { return "appearance only".to_string(); }
        if self.stats.is_empty() { return "no bonus stats".to_string(); }
        self.stats.iter().map(|&(s, v)| format!("+{v} {}", STAT_NAMES[s.min(7)])).collect::<Vec<_>>().join(", ")
    }

    /// Recover the named suffix for old saves that predate the explicit suffix field.
    pub(crate) fn suffix_index(&self) -> Option<usize> {
        self.suffix.or_else(|| {
            SUFFIXES.iter().position(|(name, _)| self.name.ends_with(name))
        })
    }

    pub(crate) fn affix_text(&self) -> String {
        match self.suffix_index() {
            Some(i) => format!("Suffix: {}", SUFFIXES[i].0),
            None => "No stat-bearing named suffix".to_string(),
        }
    }

    /// Explain exactly what each raw gear stat changes in the current class/spec.
    pub(crate) fn stat_impact_text(&self, class: usize, spec: usize) -> String {
        let prim = primary(class, CLASSES[class].specs[spec].1);
        self.stats.iter().map(|&(s, v)| {
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
        }).collect::<Vec<_>>().join("\n")
    }

    pub(crate) fn tip(&self) -> String {
        let body = if self.kind == Kind::Cosmetic { "Appearance only — no stats".to_string() }
            else if self.stats.is_empty() { "No bonus stats".to_string() }
            else {
                self.stats.iter().map(|&(s, v)| format!("+{v} {}", STAT_NAMES[s.min(7)]))
                    .collect::<Vec<_>>().join("\n")
            };
        let sell = if self.sell_value() == 0 { "Soulbound — can't be sold".to_string() }
                else { format!("Sells for {} gold", self.sell_value()) };
        format!("{}\n[{}] · ilvl {} · {}\n{}\n{}\nDurability {}/100\n{}",
                self.name, QUALITY[self.quality.min(6)], self.ilvl, self.type_label(), self.affix_text(), body,
                self.durability, sell)
    }
}

pub(crate) fn primary(class: usize, role: Role) -> usize {
    match CLASSES[class].name {
        "Mage" | "Priest" | "Warlock" | "Evoker" => 3,
        "Death Knight" | "Warrior" => 1,
        "Paladin" => if role == Healer { 3 } else { 1 },
        "Druid" | "Monk" | "Shaman" => if matches!(role, Healer | Ranged) { 3 } else { 2 },
        _ => 2,
    }
}

pub(crate) fn stat_value(s: usize, v: u32, prim: usize) -> u32 {
    match s { 0 => v, 1..=3 => if s == prim { v } else { v / 4 }, _ => v / 2 }
}

pub(crate) fn item_power(it: &Item, class: usize, spec: usize) -> u32 {
    if it.kind == Kind::Cosmetic { return 0; }
    let prim = primary(class, CLASSES[class].specs[spec].1);
    let w = if it.is_two() { 3 } else { 2 };
    let mut p = it.ilvl * QMULT[it.quality.min(6)] / 100 * w / 2;
    for &(s, v) in &it.stats { p += stat_value(s, v, prim); }
    p
}

/// Which equipment slots could this item go into for the class?
pub(crate) fn candidates(it: &Item, class: usize, two_now: bool) -> Vec<usize> {
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

// ---------- random item generation ----------
pub(crate) fn weapon_nouns(w: Wt, two: bool) -> &'static [&'static str] {
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

pub(crate) fn mats(kind: Kind, wt: Option<Wt>) -> (&'static [&'static str], &'static [&'static str]) {
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

pub(crate) fn noun_for(look: Kind, slot: usize, wt: Option<Wt>, hands: Hands, ai: usize) -> &'static str {
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

pub(crate) fn legend_name(noun: &str) -> String {
    match rnd(3) {
        0 => format!("{}, {} {noun} of the {}{}", compound_name(), ps(LEG_ADJ), ps(WIND_A), ps(WIND_B)),
        1 => format!("{}, {} of {}", compound_name(), ps(LEG_PART), boss_name()),
        _ => elven_name(),
    }
}

/// Dynamic name generator. Returns (name, optional stat-suffix index).
pub(crate) fn gen_name(q: usize, kind: Kind, slot: usize, wt: Option<Wt>, hands: Hands, ai: usize) -> (String, Option<usize>) {
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

pub(crate) fn roll_stats(q: usize, ilvl: u32, class: usize, spec: usize, suffix: Option<usize>) -> Vec<(usize, u32)> {
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
pub(crate) fn build(class: usize, spec: usize, level: u32, kind: Kind, slot: usize, wt: Option<Wt>, hands: Hands, q: usize) -> Item {
    let ai = if kind == Kind::Cosmetic { rnd(4) as usize } else { kind.armor_idx().unwrap_or(0) };
    let ilvl = if kind == Kind::Cosmetic { 1 }
        else if q == Q_HEIRLOOM { level + 6 }
        else { (level as i32 + 2 + QILVL[q.min(6)] + rnd(3) as i32).max(1) as u32 };
    let (name, suffix) = gen_name(q, kind, slot, wt, hands, ai);
    let stats = if kind == Kind::Cosmetic { vec![] } else { roll_stats(q, ilvl, class, spec, suffix) };
    Item { name, quality: q, ilvl, slot, kind, wt, hands, stats, suffix, durability: 100 }
}

pub(crate) fn pick_weapon(class: usize, spec: usize) -> (Wt, Hands) {
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

pub(crate) fn pick_offhand(class: usize, spec: usize) -> Option<(Kind, Option<Wt>, Hands)> {
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
pub(crate) fn make_item(class: usize, spec: usize, level: u32, cat: usize, q: usize) -> Option<Item> {
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

pub(crate) fn random_cat() -> usize {
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

pub(crate) fn drop_item(class: usize, spec: usize, level: u32, q: usize) -> Item {
    loop {
        if let Some(it) = make_item(class, spec, level, random_cat(), q) { return it; }
    }
}

pub(crate) fn cosmetic_item(class: usize, spec: usize, level: u32) -> Item {
    build(class, spec, level, Kind::Cosmetic, rnd(8) as usize, None, Hands::One, 1 + rnd(4) as usize)
}

/// Armor or weapon the class cannot use (vendor fodder).
pub(crate) fn unusable_item(class: usize, spec: usize, level: u32, q: usize) -> Item {
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

pub(crate) fn roll_quality(level: u32, tier: Tier, cat: usize, bonus: u32) -> usize {
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

pub(crate) fn npc_quality(level: u32) -> usize {
    if rnd(100) < 4 { return Q_HEIRLOOM; }
    let r = rnd(100);
    let mut q = if r < 15 { 0 } else if r < 40 { 1 } else if r < 70 { 2 } else if r < 90 { 3 } else if r < 99 { 4 } else { 5 };
    if q >= 4 && level < 40 { q = 3; }
    if q == 3 && level < 20 { q = 2; }
    q
}

pub(crate) fn build_gear(class: usize, spec: usize, level: u32) -> Vec<Option<Item>> {
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
pub(crate) fn store_quality(level: u32) -> usize {
    let r = rnd(100);
    if level >= 40 && r >= 94 { 4 }
    else if level >= 20 && r >= 70 { 3 }
    else if r >= 35 { 2 }
    else { 1 }
}

/// Items for sale: usable by the class, ilvl right around the hero's level.
pub(crate) fn gen_store(class: usize, spec: usize, level: u32) -> Vec<Item> {
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

/// Honor store stock: Rare gear below level 40, Epic from level 40, usable by the class.
pub(crate) fn gen_honor_store(class: usize, spec: usize, level: u32) -> Vec<Item> {
    let q = if level >= 40 { 4 } else { 3 };
    let mut v: Vec<Item> = vec![];
    let mut tries = 0;
    while v.len() < STORE_SIZE && tries < 300 {
        tries += 1;
        if let Some(it) = make_item(class, spec, level + 2, random_cat(), q) { v.push(it); }
    }
    v.sort_by_key(|i| (i.slot, std::cmp::Reverse(i.quality)));
    v
}

pub(crate) fn potion_price(level: u32) -> u32 { 6 + level * 2 }