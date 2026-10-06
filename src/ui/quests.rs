use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{config::*, data::classes::*, model::*, persistence::*, quests::*, utils::*};
use super::{Ui, pad};

impl Ui {
        pub(crate) fn quest_row(self: &Rc<Self>, i: usize, q: &Quest) -> gtk::Box {
            let outer = gtk::Box::new(gtk::Orientation::Vertical, 6);
            pad(&outer, 8);
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let info = gtk::Box::new(gtk::Orientation::Vertical, 4);
            info.set_hexpand(true);
            let t = gtk::Label::new(None);
            t.set_xalign(0.0);
            t.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(q.display().as_str())));
            let kind_txt = match q.qk { QKind::Kill => "⚔ Kill quest", QKind::Gather => "🌿 Gather quest" };
            let sub = gtk::Label::new(Some(&format!(
                "{} · Quest Lv {} · {} · {} · 📍 {} · from {}", kind_txt, q.quest_level, q.tier.name(), CATS[q.cat], q.zone, q.giver)));
            sub.set_xalign(0.0); sub.add_css_class("dim-label");
            let bar = gtk::ProgressBar::new();
            bar.set_show_text(true);
            bar.set_fraction((q.progress as f64 / q.goal.max(1) as f64).min(1.0));
            let btxt = match q.qk {
                QKind::Kill => format!("{}/{} kills", q.progress, q.goal),
                QKind::Gather => format!("{}/{} gathered · {} attempts", q.progress, q.goal, q.tries),
            };
            bar.set_text(Some(&btxt));
            info.append(&t); info.append(&sub); info.append(&bar);

            let done = q.progress >= q.goal;
            let act = gtk::Button::with_label(if done { "Turn in" } else { "⚔ +1" });
            if done { act.add_css_class("suggested-action"); }
            if q.qk == QKind::Kill && !done {
                act.set_label("Turn in");
                act.set_sensitive(false);
                act.set_tooltip_text(Some("Finish every kill below first."));
            }
            act.set_valign(gtk::Align::Center);
            { let u = self.clone(); act.connect_clicked(move |_| u.act(i)); }
            let del = gtk::Button::with_label("Abandon");
            del.set_valign(gtk::Align::Center);
            { let u = self.clone(); del.connect_clicked(move |_| u.abandon(i)); }
            row.append(&info); row.append(&act); row.append(&del);
            outer.append(&row);
            if q.qk == QKind::Kill {
                for (si, s) in q.steps.iter().enumerate() {
                    outer.append(&self.step_row(i, si, s));
                }
            }
            outer
        }

        pub(crate) fn step_row(self: &Rc<Self>, qi: usize, si: usize, s: &Step) -> gtk::Box {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_margin_start(12);
            let e = gtk::Entry::new();
            e.set_hexpand(true);
            e.set_placeholder_text(Some(&format!("Kill {} — what is this task?", si + 1)));
            e.set_text(&s.text);
            if s.done { e.set_sensitive(false); }
            { let u = self.clone(); e.connect_changed(move |en| u.edit_step(qi, si, en.text().to_string())); }
            row.append(&e);
            if s.done {
                row.append(&gtk::Label::new(Some("✔ done")));
            } else {
                let ok = gtk::Button::with_label("✔ Complete");
                { let u = self.clone(); ok.connect_clicked(move |_| u.step_done(qi, si)); }
                let no = gtk::Button::with_label("✖ Abandon");
                { let u = self.clone(); no.connect_clicked(move |_| u.step_abandon(qi, si)); }
                row.append(&ok); row.append(&no);
            }
            row
        }

        pub(crate) fn edit_step(self: &Rc<Self>, qi: usize, si: usize, text: String) {
            let Some(a) = self.active.get() else { return };
            let mut s = self.save.borrow_mut();
            if let Some(st) = s.heroes.get_mut(a)
                .and_then(|h| h.quests.get_mut(qi))
                .and_then(|q| q.steps.get_mut(si))
            {
                st.text = text;
            }
            persist(&s);
        }

        pub(crate) fn step_done(self: &Rc<Self>, qi: usize, si: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let Some(q) = h.quests.get_mut(qi) else { return };
                if si >= q.steps.len() || q.steps[si].done { return; }
                q.steps[si].done = true;
                q.progress += 1;
                let txt = q.steps[si].text.trim().to_string();
                let label = if txt.is_empty() { format!("target {}", si + 1) } else { txt };
                let (prog, goal, giver, disp) = (q.progress, q.goal, q.giver.clone(), q.display());
                msgs.push(m("Quest", format!("✔ {disp}: {label} — {prog}/{goal} kills")));
                if prog >= goal {
                    msgs.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
                }
                let quest_snapshot = h.quests[qi].clone();
                let enc = encounter_for(&quest_snapshot, h.zone_named(&quest_snapshot.zone), &h.faction, h.level);                h.step_drop(&mut msgs, STEP_DROP);
                h.apply_quest_cost(&quest_snapshot, &enc, &mut msgs);
            }
            self.finish(msgs);
        }

        pub(crate) fn step_abandon(self: &Rc<Self>, qi: usize, si: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let Some(q) = h.quests.get_mut(qi) else { return };
                if si >= q.steps.len() { return; }
                let st = q.steps.remove(si);
                if st.done { q.progress = q.progress.saturating_sub(1); }
                q.goal = q.steps.len() as u32;
                let (empty, prog, goal, giver) = (q.goal == 0, q.progress, q.goal, q.giver.clone());
                if empty {
                    h.quests.remove(qi);
                    msgs.push(m("Quest", "✖ Last target abandoned — quest removed."));
                } else {
                    msgs.push(m("Quest", format!("✖ Target {} abandoned ({prog}/{goal} kills left to do)", si + 1)));
                    if prog >= goal {
                        msgs.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
                    }
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn add_quest(self: &Rc<Self>) {
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
                    let zref = h.zone_ref();
                    let (min_level, max_level) = quest_level_bounds(h.level, zref);
                    let quest_level = (min_level + self.quest_level.selected()).clamp(min_level, max_level);
                    let giver = pick(&npcs(&h.faction)).to_string();
                    let qk = match self.qkind.selected() {
                        1 => QKind::Gather,
                        _ => QKind::Kill,
                    };
                    let g = (self.goal.value() as u32).clamp(1, MAX_QGOAL);
                    let (goal, target, steps, ctype) = match qk {
                        QKind::Kill => { let (t, ct) = kill_target(zref, tier); (g, t, make_steps(g), Some(ct)) }
                        QKind::Gather => (g, gather_item(ci), vec![], None),
                    };
                    let bonus = zref.map_or(0, |z| z.relation(&h.faction).xp_bonus());
                    let q = Quest {
                        title, giver: giver.clone(), tier, cat: ci, goal, progress: 0,
                        chain: self.chain.is_active(), part: 1, quest_level,
                        zone: h.zone.clone(), bonus,
                        qk, target, steps, tries: 0, ctype,
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
                    self.qkind.set_selected(0);
                    self.finish(vec![msg]);
                }
                Err(e) => self.finish(vec![m("System", e)]),
            }
        }

        /// Gather quests: ⚔ +1 tries to find the item (zone creatures ambush you). Any quest: turn in when complete.
        pub(crate) fn act(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            let msgs = {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                if i >= h.quests.len() { return; }
                let ready = h.quests[i].progress >= h.quests[i].goal;
                if !ready {
                    if h.quests[i].qk != QKind::Gather { return; }
                    let got = rnd(100) < GATHER_CHANCE;
                    let (disp, p, g, tgt, giver) = {
                        let q = &mut h.quests[i];
                        q.tries += 1;
                        if got { q.progress += 1; }
                        (q.display(), q.progress, q.goal, q.target.clone(), q.giver.clone())
                    };
                    let snap = h.quests[i].clone();
                    let enc = encounter_for(&snap, self.zones.iter().find(|z| z.name == snap.zone), &h.faction, h.level);
                    let mut v = vec![m("Quest", format!("⚔ {} (Lv {}) ambushes you while you search for {tgt}!", enc.name, enc.level))];
                    v.push(if got {
                        m("Quest", format!("⚔ {disp} — found {tgt} ({p}/{g})"))
                    } else {
                        m("Quest", format!("💨 {disp} — nothing this time ({p}/{g})"))
                    });
                    if got && p >= g {
                        v.push(m("Quest", format!("Objective complete! Return to {giver} and turn in the quest.")));
                    }
                    h.step_drop(&mut v, GATHER_DROP);
                    h.apply_quest_cost(&snap, &enc, &mut v);
                    v
                } else {
                    let q = h.quests.remove(i);
                    let mut v = h.turn_in(&q);
                    if q.chain {
                        let g = (q.goal + (q.goal / 4).max(1)).min(MAX_QGOAL);
                        let steps = if q.qk == QKind::Kill { make_steps(g) } else { vec![] };
                        let next = Quest { part: q.part + 1, goal: g, progress: 0, steps, tries: 0, ..q.clone() };
                        v.push(m("Quest", format!("🔗 Quest chain continues: {}", next.display())));
                        h.quests.push(next);
                    }
                    v
                }
            };
            self.finish(msgs);
        }

        pub(crate) fn abandon(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                if i < h.quests.len() { h.quests.remove(i); }
            }
            self.finish(vec![m("Quest", "Quest abandoned.")]);
        }

        // ----- PvP
}