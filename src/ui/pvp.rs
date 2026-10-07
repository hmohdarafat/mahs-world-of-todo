use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{combat::*, config::*, data::classes::*, model::*, utils::*};
use super::{clear, pad, sel, Ui};

impl Ui {
    pub(crate) fn new_opponents(self: &Rc<Self>) {
        let lvl = self.active.get()
            .and_then(|a| self.save.borrow().heroes.get(a).map(|h| h.level)).unwrap_or(1);
        let faction = sel(&self.pvp_faction, &PVP_FACTIONS);
        // Keep generating until the selected faction has a full list.
        let mut v: Vec<Fighter> = Vec::with_capacity(PVP_SHOWN);
        while v.len() < PVP_SHOWN {
            let fighter = gen_player(lvl);
            if faction == "All" || fighter.faction == faction {
                v.push(fighter);
            }
        }
        v.sort_by(|a, b| (a.level, a.name.as_str()).cmp(&(b.level, b.name.as_str())));
        *self.opps.borrow_mut() = v;
        self.pvp_result.set_text("");
        self.refresh_pvp();
    }

    pub(crate) fn refresh_pvp(self: &Rc<Self>) {
        clear(&self.pvp_list);
        let faction = sel(&self.pvp_faction, &PVP_FACTIONS);
        let my_faction = self.active.get()
            .and_then(|a| self.save.borrow().heroes.get(a).map(|h| h.faction.clone()))
            .unwrap_or_default();
        let opponents = self.opps.borrow();
        let mut visible: Vec<(usize, &Fighter)> = opponents.iter().enumerate()
            .filter(|(_, f)| faction == "All" || f.faction == faction)
            .collect();
        visible.sort_by(|a, b| (a.1.level, a.1.name.as_str()).cmp(&(b.1.level, b.1.name.as_str())));

        if visible.is_empty() {
            let l = gtk::Label::new(Some("No players left nearby — press “🔄 New players”."));
            l.add_css_class("dim-label");
            self.pvp_list.append(&l);
        }

        for (i, o) in visible.into_iter() {
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
            let enemy = o.faction != my_faction;
            let btn = gtk::Button::with_label(if enemy { "⚔ ★" } else { "⚔" });
            btn.set_tooltip_text(Some(&format!(
                "Duel this player with your current gear, abilities, HP, mana and stamina.{}",
                if enemy { "\n★ Opposite faction: a win gives +50% XP and +1 honor." } else { "" })));
            btn.set_valign(gtk::Align::Center);
            { let u = self.clone(); btn.connect_clicked(move |_| u.duel(i)); }
            row.append(&info); row.append(&btn);
            let fr = gtk::Frame::new(None);
            fr.set_child(Some(&row));
            self.pvp_list.append(&fr);
        }
    }

    pub(crate) fn duel(self: &Rc<Self>, i: usize) {
        let opp = {
            let o = self.opps.borrow();
            match o.get(i) { Some(x) => x.clone(), None => return }
        };
        self.with_hero(|h, msgs| {
            let me = h.fighter();
            let d = fight(&me, &opp);
            // a defeated player leaves the list
            if d.won {
                let mut o = self.opps.borrow_mut();
                if i < o.len() { o.remove(i); }
            }
            let rounds = d.lines.len();
            msgs.push(m("PvP", format!(
                "⚔ Duel: {} (Lv {}, {}) vs {} (Lv {}, {})",
                me.name, me.level, CLASSES[me.class].specs[me.spec].0,
                opp.name, opp.level, CLASSES[opp.class].specs[opp.spec].0)));
            msgs.extend(d.lines.into_iter().map(|l| m("PvP", l)));
            h.mana = d.mana;
            h.sta = d.sta;
            let text;
            let mut xp_gain = 0u32;
            if d.won {
                let enemy = opp.faction != me.faction;
                let honor = 10 + opp.level / 5 + enemy as u32;
                let base_xp = 10 + opp.level * 2 + rnd(10);
                xp_gain = if enemy { base_xp * 3 / 2 } else { base_xp };
                let gold = opp.level * 2 + rnd(10);
                h.hp = d.hp.max(1);
                h.wins += 1; h.honor += honor; h.gold += gold;
                text = format!(
                    "🏆 Victory vs {} (Lv {}{}) in {rounds} rounds: +{honor} honor, +{xp_gain} XP, +{gold} gold",
                    opp.name, opp.level, if enemy { ", enemy faction" } else { "" });
            } else {
                h.hp = (h.maxes().0 / 10).max(1);
                h.losses += 1; h.honor += 1;
                text = format!("💀 Defeat vs {} (Lv {}) after {rounds} rounds: +1 honor — you wake at 10% HP", opp.name, opp.level);
            }
            msgs.push(m("PvP", text.clone()));
            if !d.won { h.wear_gear(10, msgs); }
            if xp_gain > 0 { h.gain_xp(xp_gain, msgs); }
            h.check_achievements(msgs);
            self.pvp_result.set_text(&text);
        });
    }
}