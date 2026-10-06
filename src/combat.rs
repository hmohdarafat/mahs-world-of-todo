use crate::{config::*, data::{abilities::*, classes::*, items::*}, items::*, model::*, utils::*, world::*};
use crate::model::{K::{Absorb, Buff, Dmg, Dot, Guard, Heal, Kick, Leech, Stun, Util}, R::{Free, Mana, Stam}, Role::{Healer, Melee, Ranged, Tank}};

impl Fighter {
    pub(crate) fn role(&self) -> Role { CLASSES[self.class].specs[self.spec].1 }

    pub(crate) fn abilities(&self) -> Vec<&'static Ab> { known(self.class, self.spec, self.level) }

    pub(crate) fn ilvl_avg(&self) -> u32 {
        let v: Vec<u32> = self.gear.iter().flatten().filter(|i| i.kind != Kind::Cosmetic).map(|i| i.ilvl).collect();
        if v.is_empty() { 0 } else { v.iter().sum::<u32>() / v.len() as u32 }
    }

    pub(crate) fn setup(&self) -> &'static str {
        match (&self.gear[S_MAIN], &self.gear[S_OFF]) {
            (None, None) => "Unarmed",
            (Some(mh), _) if mh.is_two() => "Two-handed",
            (_, Some(o)) if o.kind == Kind::Weapon => "Dual wield",
            (_, Some(o)) if o.kind == Kind::Shield => "One-hand + Shield",
            (_, Some(_)) => "One-hand + Off-hand",
            (Some(_), None) => "One-handed",
        }
    }

    pub(crate) fn stats(&self) -> Stats {
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

    pub(crate) fn summary(&self) -> String {
        let c = &CLASSES[self.class];
        let s = self.stats();
        format!("Lv {} {} {} ({}) · {} · {} · ilvl {} · HP {} ATK {} DEF {} · {} abilities",
                self.level, self.race, c.name, c.specs[self.spec].0, self.role().name(), c.armor,
                self.ilvl_avg(), s.hp, s.atk, s.def, self.abilities().len())
    }

    pub(crate) fn detail(&self) -> String {
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

impl Side {
    pub(crate) fn new(f: &Fighter) -> Self {
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

pub(crate) fn cost_of(a: &Ab, s: &Side) -> u32 {
    match a.r { Mana => s.st.mana * a.c / 100, Stam => s.st.sta * a.c / 100, Free => 0 }
}

pub(crate) fn hit(me: &Side, foe: &Side, pct: u32) -> (i64, bool) {
    let mut base = me.st.atk * pct / 100 * (85 + rnd(31)) / 100;
    if me.buff.0 > 0 { base = base * (100 + me.buff.1) / 100; }
    let mut dmg = (base * 100 / (100 + foe.st.def / 2)).max(1);
    let crit = rnd(100) < me.st.crit;
    if crit { dmg *= 2; }
    if foe.guard.0 > 0 { dmg = (dmg * (100 - foe.guard.1.min(90)) / 100).max(1); }
    (dmg as i64, crit)
}

pub(crate) fn take(s: &mut Side, dmg: i64) {
    let a = dmg.min(s.absorb);
    s.absorb -= a;
    s.hp -= dmg - a;
}

pub(crate) fn crit_tag(c: bool) -> &'static str { if c { " CRIT" } else { "" } }

/// Pick which ability to use this turn (None = basic attack).
pub(crate) fn choose(me: &Side, foe: &Side) -> Option<usize> {
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
pub(crate) fn turn(f: &Fighter, g: &Fighter, me: &mut Side, foe: &mut Side) -> String {
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

pub(crate) fn tick(s: &mut Side) {
    if s.buff.0 > 0 { s.buff.0 -= 1; }
    if s.guard.0 > 0 { s.guard.0 -= 1; }
}

pub(crate) fn end_round(s: &mut Side) {
    s.mana = (s.mana + s.st.mana * 5 / 100).min(s.st.mana);
    s.sta = (s.sta + s.st.sta * 5 / 100).min(s.st.sta);
    s.hp = (s.hp + s.st.heal as i64).min(s.st.hp as i64);
}

pub(crate) fn fight(a: &Fighter, b: &Fighter) -> Duel {
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

pub(crate) fn gen_player(near: u32) -> Fighter {
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
