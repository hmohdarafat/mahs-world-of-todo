use crate::{config::*, data::{abilities::*, classes::*, items::*}, items::*, model::*, utils::*};

impl Hero {
    pub(crate) fn new(name: String, guild: String, realm: &Realm, faction: &str, race: &str, class: usize, spec: usize) -> Self {
        let mut h = Hero {
            name, guild, realm: realm.name.clone(), realm_tier: realm.tier,
            faction: faction.into(), race: race.into(), class, spec,
            level: 1, xp: 0, gold: 0, talents: 0, done: 0, honor: 0, wins: 0, losses: 0,
            zone: start_zone(race).into(), zone_inst: String::new(),
            gear: vec![None; SLOTS.len()], prof: vec![0; CATS.len() - 1],
            achievements: vec![], quests: vec![], log: vec![], bag: vec![],
            hp: 0, mana: 0, sta: 0, pots: [0; 3],
        };
        h.restore_all();
        h
    }

    pub(crate) fn need(&self) -> u32 { 100 + self.level * 50 }

    pub(crate) fn two_now(&self) -> bool {
        self.gear.get(S_MAIN).and_then(|o| o.as_ref()).map_or(false, |i| i.is_two())
    }

    pub(crate) fn fighter(&self) -> Fighter {
        Fighter {
            name: self.name.clone(), faction: self.faction.clone(), race: self.race.clone(),
            class: self.class, spec: self.spec, level: self.level, gear: self.gear.clone(),
            talents: self.talents, cur: Some((self.hp.max(1), self.mana, self.sta)),
        }
    }

    /// (max hp, max mana, max stamina)
    pub(crate) fn maxes(&self) -> (u32, u32, u32) {
        let s = self.fighter().stats();
        (s.hp, s.mana, s.sta)
    }

    pub(crate) fn restore_all(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = a; self.mana = b; self.sta = c;
    }

    pub(crate) fn clamp_res(&mut self) {
        let (a, b, c) = self.maxes();
        self.hp = self.hp.min(a).max(1);
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

    /// Equip into the best slot. Ok(displaced items) or Err(item) if not equipped.
    pub(crate) fn place(&mut self, it: Item, force: bool) -> Result<Vec<Item>, Item> {
        let cands = candidates(&it, self.class, self.two_now());
        if cands.is_empty() { return Err(it); }
        let (class, spec) = (self.class, self.spec);
        let pw = |o: &Option<Item>| o.as_ref().map_or(-1i64, |i| item_power(i, class, spec) as i64);
        let target = cands.iter().copied().min_by_key(|&s| pw(&self.gear[s])).unwrap_or(S_MAIN);
        let two = it.is_two();
        let old_p = if two {
            let (a, b) = (pw(&self.gear[S_MAIN]), pw(&self.gear[S_OFF]));
            if a < 0 && b < 0 { -1 } else { a.max(0) + b.max(0) }
        } else {
            pw(&self.gear[target])
        };
        if !force && item_power(&it, class, spec) as i64 <= old_p { return Err(it); }
        let mut out = vec![];
        if let Some(o) = self.gear[target].take() { out.push(o); }
        if two {
            if let Some(o) = self.gear[S_OFF].take() { out.push(o); }
        }
        self.gear[target] = Some(it);
        Ok(out)
    }

    pub(crate) fn is_upgrade(&self, it: &Item) -> bool {
        let cands = candidates(it, self.class, self.two_now());
        if cands.is_empty() { return false; }
        let p = item_power(it, self.class, self.spec);
        cands.iter().any(|&s| self.gear[s].as_ref().map_or(true, |o| item_power(o, self.class, self.spec) < p))
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

    /// Apply the resource cost of performing one action on a quest.
    /// Quest level is compared with the character level: same-level quests use
    /// the baseline cost; higher-level quests hurt more, and lower-level quests
    /// hurt less.
    pub(crate) fn apply_quest_cost(&mut self, q: &Quest, msgs: &mut Vec<Msg>) {
        let (max_hp, max_mana, max_sta) = self.maxes();
        let diff = q.quest_level as i32 - self.level as i32;
        let pressure = (100 + diff * 15).clamp(40, 175) as u32;

        let hp_loss = ((max_hp as u64 * 6 * pressure as u64) / 10_000).max(1) as u32;
        let mana_loss = ((max_mana as u64 * 3 * pressure as u64) / 10_000).max(1) as u32;
        let sta_loss = ((max_sta as u64 * 4 * pressure as u64) / 10_000).max(1) as u32;

        self.hp = self.hp.saturating_sub(hp_loss).max(1);
        self.mana = self.mana.saturating_sub(mana_loss);
        self.sta = self.sta.saturating_sub(sta_loss);

        msgs.push(m("Quest", format!(
            "⚠ Quest Lv {} vs character Lv {}: -{} HP, -{} mana, -{} stamina",
            q.quest_level, self.level, hp_loss, mana_loss, sta_loss
        )));
    }

    pub(crate) fn loot_one(&mut self, q: &Quest, msgs: &mut Vec<Msg>) {
        let qual = roll_quality(self.level, q.tier, q.cat, q.bonus);
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

        self.xp += xp; self.gold += gold; self.done += 1;
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
        if self.done % 20 == 0 {
            let it = drop_item(self.class, self.spec, self.level, Q_HEIRLOOM);
            msgs.push(m("Loot", "🏺 Heirloom cache unlocked (every 20 quests)!"));
            self.receive(it, &mut msgs);
        }

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
        } else {
            self.regen(15);
        }
        self.check_achievements(&mut msgs);
        msgs
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

// ---------- combat ----------
