use crate::{
    config::*,
    data::{abilities::*, classes::CLASSES, creatures::*, items::*},
    items::*, model::*, quests::Encounter,
    stats::{derive_stats, GearAgg},
    utils::*,
};

impl Hero {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(name: String, guild: String, realm: &Realm, faction: &str, race: &str, class: usize, spec: usize, zone: String, zones: Vec<Zone>) -> Self {
        let mut h = Hero {
            name, guild, realm: realm.name.clone(), realm_tier: realm.tier,
            faction: faction.into(), race: race.into(), class, spec,
            level: 1, xp: 0, gold: 0, talents: 0, done: 0, honor: 0, wins: 0, losses: 0,
            zone, zone_inst: String::new(), zones,
            gear: vec![None; SLOTS.len()], prof: vec![0; CATS.len() - 1],
            achievements: vec![], quests: vec![], log: vec![], bag: vec![],
            hp: 0, mana: 0, sta: 0, pots: [0; 3],
            v2: true,
        };
        h.restore_all();
        h
    }

    pub(crate) fn need(&self) -> u32 { 100 + self.level * 50 }

    pub(crate) fn zone_named(&self, name: &str) -> Option<&Zone> { self.zones.iter().find(|z| z.name == name) }
    pub(crate) fn zone_ref(&self) -> Option<&Zone> { self.zone_named(&self.zone) }

    pub(crate) fn two_now(&self) -> bool {
        self.gear.get(S_MAIN).and_then(|o| o.as_ref()).is_some_and(|i| i.is_two())
    }

    pub(crate) fn fighter(&self) -> Fighter {
        Fighter {
            name: self.name.clone(), faction: self.faction.clone(), race: self.race.clone(),
            class: self.class, spec: self.spec, level: self.level, gear: self.gear.clone(),
            talents: self.talents, cur: Some((self.hp.max(1), self.mana, self.sta)),
        }
    }

    /// Derived stats with some slots removed and/or an extra item added. No cloning.
    pub(crate) fn stats_with(&self, drop: &[usize], add: Option<&Item>) -> Stats {
        let items = self.gear.iter().enumerate()
            .filter(|(i, _)| !drop.contains(i))
            .filter_map(|(_, o)| o.as_ref())
            .chain(add);
        derive_stats(self.class, self.spec, self.level, self.talents, &GearAgg::of(items))
    }

    pub(crate) fn stats(&self) -> Stats { self.stats_with(&[], None) }

    /// (max hp, max mana, max stamina)
    pub(crate) fn maxes(&self) -> (u32, u32, u32) {
        let s = self.stats();
        (s.hp, s.mana, s.sta)
    }

    pub(crate) fn restore_all(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = a; self.mana = b; self.sta = c;
    }

    pub(crate) fn clamp_res(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = self.hp.min(a);
        self.mana = self.mana.min(b);
        self.sta = self.sta.min(c);
    }

    pub(crate) fn regen(&mut self, pct: u32) {
        let (a, b, c) = self.maxes();
        self.hp = (self.hp + a * pct / 100).min(a);
        self.mana = (self.mana + b * pct / 100).min(b);
        self.sta = (self.sta + c * pct / 100).min(c);
    }

    pub(crate) fn add_log(&mut self, msgs: Vec<Msg>) {
        let t = now();
        for (c, s) in msgs {
            self.log.push(LogEntry { time: t.clone(), cat: c.into(), msg: s });
        }
        let n = self.log.len();
        if n > MAX_LOG { self.log.drain(..n - MAX_LOG); }
    }

    fn power_at(&self, slot: usize) -> i64 {
        self.gear[slot].as_ref().map_or(-1, |i| item_power(i, self.class, self.spec) as i64)
    }

    /// The candidate slot currently holding the weakest item (empty counts weakest).
    fn best_slot(&self, cands: &[usize]) -> usize {
        cands.iter().copied().min_by_key(|&s| self.power_at(s)).unwrap_or(S_MAIN)
    }

    /// Equip into the best slot. Ok(displaced items) or Err(item) if not equipped.
    pub(crate) fn place(&mut self, it: Item, force: bool) -> Result<Vec<Item>, Item> {
        let cands = candidates(&it, self.class, self.two_now());
        if cands.is_empty() { return Err(it); }
        let target = self.best_slot(&cands);
        let two = it.is_two();
        if !force {
            let old_p = if two {
                let (a, b) = (self.power_at(S_MAIN), self.power_at(S_OFF));
                if a < 0 && b < 0 { -1 } else { a.max(0) + b.max(0) }
            } else {
                self.power_at(target)
            };
            if item_power(&it, self.class, self.spec) as i64 <= old_p { return Err(it); }
        }
        let mut out = vec![];
        if let Some(o) = self.gear[target].take() { out.push(o); }
        if two && let Some(o) = self.gear[S_OFF].take() { out.push(o); }
        self.gear[target] = Some(it);
        Ok(out)
    }

    pub(crate) fn is_upgrade(&self, it: &Item) -> bool {
        let cands = candidates(it, self.class, self.two_now());
        if cands.is_empty() { return false; }
        let p = item_power(it, self.class, self.spec);
        cands.iter().any(|&s| self.gear[s].as_ref().is_none_or(|o| item_power(o, self.class, self.spec) < p))
    }

    fn stat_delta_text(before: Stats, after: Stats) -> String {
        let delta = |a: u32, b: u32| -> i64 { b as i64 - a as i64 };
        let parts = [
            ("HP", delta(before.hp, after.hp)),
            ("ATK", delta(before.atk, after.atk)),
            ("DEF", delta(before.def, after.def)),
            ("Crit", delta(before.crit, after.crit)),
            ("Mana", delta(before.mana, after.mana)),
            ("Stamina", delta(before.sta, after.sta)),
            ("Heal", delta(before.heal, after.heal)),
        ];
        let s = parts.iter()
            .filter(|(_, v)| *v != 0)
            .map(|(name, v)| if *name == "Crit" { format!("{name} {v:+}%") } else { format!("{name}{v:+}") })
            .collect::<Vec<_>>()
            .join(" · ");
        if s.is_empty() { "No derived-stat change".to_string() } else { s }
    }

    /// Exact contribution of an equipped item to the character's current derived stats.
    pub(crate) fn equipped_item_impact(&self, slot: usize) -> String {
        let Some(it) = self.gear.get(slot).and_then(|x| x.as_ref()) else { return "No item equipped".to_string(); };
        let both = [S_MAIN, S_OFF];
        let one = [slot];
        let drop: &[usize] = if it.is_two() { &both } else { &one };
        Self::stat_delta_text(self.stats_with(drop, None), self.stats())
    }

    /// Exact contribution of all equipped gear to the current derived stats.
    pub(crate) fn equipped_gear_impact(&self) -> String {
        let naked = derive_stats(self.class, self.spec, self.level, self.talents, &GearAgg::default());
        Self::stat_delta_text(naked, self.stats())
    }

    /// Project the item's exact derived-stat change if the player equips it now.
    pub(crate) fn bag_item_impact(&self, it: &Item) -> String {
        let cands = candidates(it, self.class, self.two_now());
        if cands.is_empty() { return "Not equippable by this class/setup".to_string(); }
        let target = self.best_slot(&cands);
        let both = [S_MAIN, S_OFF];
        let one = [target];
        let drop: &[usize] = if it.is_two() { &both } else { &one };
        Self::stat_delta_text(self.stats(), self.stats_with(drop, Some(it)))
    }

    /// (attack change, defense change) if `it` were equipped now. None = can't be equipped.
    pub(crate) fn equip_delta(&self, it: &Item) -> Option<(i64, i64)> {
        let cands = candidates(it, self.class, self.two_now());
        if cands.is_empty() { return None; }
        let target = self.best_slot(&cands);
        let both = [S_MAIN, S_OFF];
        let one = [target];
        let drop: &[usize] = if it.is_two() { &both } else { &one };
        let (b, a) = (self.stats(), self.stats_with(drop, Some(it)));
        Some((a.atk as i64 - b.atk as i64, a.def as i64 - b.def as i64))
    }

    pub(crate) fn stash(&mut self, it: Item, msgs: &mut Vec<Msg>) {
        if self.bag.len() < BAG_MAX || it.quality == Q_HEIRLOOM {
            self.bag.push(it);
        } else {
            let g = it.sell_value();
            self.gold += g;
            msgs.push(m("Loot", format!("🪙 Bag full — sold {} for {g} gold", it.label())));
        }
    }

    pub(crate) fn receive(&mut self, it: Item, msgs: &mut Vec<Msg>) {
        let label = it.label();
        let before = self.bag.len();
        self.stash(it, msgs);
        if self.bag.len() > before {
            msgs.push(m("Loot", format!("🎒 New equipment added to your bag: {label}")));
        }
    }

    /// Resource-loss multiplier (in %) for a quest `diff` levels above (+) or below (-) the hero.
    /// Above the hero it grows ×1.7 per level, so even +1 hurts and +4 is brutal.
    pub(crate) fn level_pressure(diff: i32) -> u32 {
        if diff > 0 {
            let mut p = 100u32;
            for _ in 0..diff.min(6) { p = p * 17 / 10; }
            p
        } else {
            (100 + diff * 12).clamp(40, 100) as u32
        }
    }

    /// Apply the resource cost of performing one action on a quest.
    pub(crate) fn apply_quest_cost(&mut self, q: &Quest, enc: &Encounter, msgs: &mut Vec<Msg>) {
        let (max_hp, max_mana, max_sta) = self.maxes();
        let lvl = self.level as i32;
        // quest level difference, plus any extra danger from a higher-level creature
        let diff = (q.quest_level as i32 - lvl) + (enc.level as i32 - lvl).max(0);
        let mut pressure = Self::level_pressure(diff);
        let mu = matchup(CLASSES[self.class].name, enc.ctype);
        pressure = match mu {
            Matchup::Easy => pressure * 80 / 100,
            Matchup::Hard => pressure * 125 / 100,
            Matchup::Average => pressure,
        };

        let hp_loss = ((max_hp as u64 * 6 * pressure as u64) / 10_000).max(1) as u32;
        let mana_loss = ((max_mana as u64 * 3 * pressure as u64) / 10_000).max(1) as u32;
        let sta_loss = ((max_sta as u64 * 4 * pressure as u64) / 10_000).max(1) as u32;

        self.hp = self.hp.saturating_sub(hp_loss);
        self.mana = self.mana.saturating_sub(mana_loss);
        self.sta = self.sta.saturating_sub(sta_loss);

        msgs.push(m("Quest", format!(
            "⚠ {} (Lv {}, {}) — {} matchup · level difference {:+} → ×{:.1} · -{} HP, -{} mana, -{} stamina",
            enc.name, enc.level, CTYPES[enc.ctype.min(CTYPES.len() - 1)], mu.label(),
            diff, pressure as f64 / 100.0, hp_loss, mana_loss, sta_loss
        )));
        if self.hp == 0 { self.wear_gear(10, msgs); }
    }

    pub(crate) fn loot_one(&mut self, q: &Quest, msgs: &mut Vec<Msg>) {
        let qual = roll_quality(self.level, q.tier, q.cat, 0);
        let (cl, sp, lv) = (self.class, self.spec, self.level);
        let it = match rnd(100) {
            0..=5 => unusable_item(cl, sp, lv, qual),
            6..=11 => cosmetic_item(cl, sp, lv),
            _ => drop_item(cl, sp, lv, qual),
        };
        self.receive(it, msgs);
    }

    /// Each finished quest step / gather attempt has a chance to drop one random potion (0 or 1).
    pub(crate) fn step_drop(&mut self, msgs: &mut Vec<Msg>, chance: u32) {
        if rnd(100) < chance {
            let k = rnd(3) as usize;
            self.pots[k] += 1;
            msgs.push(m("Loot", format!("{} Found a {}!", POT[k].1, POT[k].0)));
        }
    }

    pub(crate) fn refresh_heirlooms(&mut self) {
        let l = self.level + 6;
        for it in self.gear.iter_mut().flatten().chain(self.bag.iter_mut()) {
            if it.quality == Q_HEIRLOOM {
                it.ilvl = l;
                let v = l * QSTAT[Q_HEIRLOOM] / 100 + 1;
                for s in it.stats.iter_mut() { s.1 = v; }
            }
        }
    }

    /// Adds XP and processes level-ups. Returns true if at least one level was gained.
    pub(crate) fn gain_xp(&mut self, xp: u32, msgs: &mut Vec<Msg>) -> bool {
        self.xp += xp;
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
            true
        } else {
            false
        }
    }

    pub(crate) fn turn_in(&mut self, q: &Quest) -> Vec<Msg> {
        let mult = q.tier.mult();
        // gather quests pay +1% per item gathered on top of the linear scaling
        let extra = if q.qk == QKind::Gather { q.goal } else { 0 };
        let pct = 100 + q.bonus + extra;
        let xp = 25 * q.goal * mult * pct / 100;
        let mut gold = q.goal * mult * 5 + rnd(10);
        if (1..=GATHER_MAX).contains(&q.cat) { gold += gold / 2; } // gathering sells ore/herbs
        gold = gold * pct / 100;
        if q.cat > 0 { self.prof[q.cat - 1] += q.goal; }

        self.gold += gold; self.done += 1;
        let mut msgs: Vec<Msg> = vec![m("Quest", format!("✔ Quest complete: {} → +{xp} XP, +{gold} gold", q.display()))];
        if q.bonus > 0 { msgs.push(m("Quest", format!("🗺 {} zone bonus: +{}%", q.zone, q.bonus))); }
        if extra > 0 { msgs.push(m("Quest", format!("🌿 Big haul bonus: +{extra}%"))); }
        if q.cat > 0 { msgs.push(m("Quest", format!("🔨 {} skill +{}", CATS[q.cat], q.goal))); }

        // Every completed quest awards at least one equipment item.
        // Raid and World Boss quests award two. New equipment always goes to the bag.
        let drops = if matches!(q.tier, Tier::Raid | Tier::WorldBoss) { 2 } else { 1 };
        for _ in 0..drops {
            self.loot_one(q, &mut msgs);
        }
        if self.done.is_multiple_of(20) {
            let it = drop_item(self.class, self.spec, self.level, Q_HEIRLOOM);
            msgs.push(m("Loot", "🏺 Heirloom cache unlocked (every 20 quests)!"));
            self.receive(it, &mut msgs);
        }

        if !self.gain_xp(xp, &mut msgs) { self.regen(15); }
        self.check_achievements(&mut msgs);
        msgs
    }

    pub(crate) fn wear_gear(&mut self, amt: u32, msgs: &mut Vec<Msg>) {
        let (mut n, mut broken) = (0u32, 0u32);
        for it in self.gear.iter_mut().flatten() {
            if it.durability > 0 {
                it.durability = it.durability.saturating_sub(amt);
                n += 1;
                if it.durability == 0 { broken += 1; }
            }
        }
        if n > 0 {
            msgs.push(m("System", format!("💥 0 HP — equipment durability -{amt} on {n} items{}",
                if broken > 0 { format!(" ({broken} broken)") } else { String::new() })));
        }
    }

    pub(crate) fn repair_total(&self) -> u32 {
        self.gear.iter().flatten().chain(self.bag.iter()).map(|i| i.repair_cost()).sum()
    }

    pub(crate) fn check_achievements(&mut self, msgs: &mut Vec<Msg>) {
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