use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{config::*, data::{classes::*, items::*}, items::*, model::*, utils::*};
use super::Ui;

impl Ui {
        pub(crate) fn refresh_equipment(self: &Rc<Self>) {
            while let Some(c) = self.eq_list.first_child() { self.eq_list.remove(&c); }
            while let Some(c) = self.bag_list.first_child() { self.bag_list.remove(&c); }
            let Some(a) = self.active.get() else { return };
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(a) else { return };
            let f = h.fighter();
            let cls = &CLASSES[h.class];
            let weapons: Vec<&str> = cls.weapons.iter().map(|w| w.name()).collect();
            let filled = h.gear.iter().flatten().count();
            self.eq_sum.set_text(&format!(
                "Avg ilvl {} · {} · {}/{} slots filled\n{} armor · {}\nWeapons: {}",
                f.ilvl_avg(), f.setup(), filled, SLOTS.len(), cls.armor, offhand_text(cls), weapons.join(", ")));

            for (i, slot) in SLOTS.iter().enumerate() {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let lbl = gtk::Label::new(None);
                lbl.set_xalign(0.0); lbl.set_hexpand(true);
                match &h.gear[i] {
                    Some(it) => {
                        let tl = it.type_label();
                        let si = it.stats_inline();
                        lbl.set_markup(&format!(
                            "<b>{slot}</b>: {}\n<small>ilvl {} · {} · {}</small>",
                            qspan(it.quality, it.name.as_str()), it.ilvl,
                            glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
                        lbl.set_tooltip_text(Some(&it.tip()));
                        let btn = gtk::Button::with_label("Unequip");
                        btn.set_valign(gtk::Align::Center);
                        { let u = self.clone(); btn.connect_clicked(move |_| u.unequip(i)); }
                        row.append(&lbl); row.append(&btn);
                    }
                    None => {
                        lbl.set_markup(&format!("<b>{slot}</b>: —"));
                        lbl.add_css_class("dim-label");
                        row.append(&lbl);
                    }
                }
                self.eq_list.append(&row);
            }

            self.bag_head.set_text(&format!("🎒 Bag ({}/{})", h.bag.len(), BAG_MAX));
            if h.bag.is_empty() { self.bag_list.append(&gtk::Label::new(Some("Bag is empty."))); }
            let two_now = h.two_now();
            let mut idx: Vec<usize> = (0..h.bag.len()).collect();
            idx.sort_by_key(|&i| std::cmp::Reverse(h.bag[i].quality));
            for i in idx {
                let it = &h.bag[i];
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let lbl = gtk::Label::new(None);
                lbl.set_xalign(0.0); lbl.set_hexpand(true);
                let tl = it.type_label();
                let si = it.stats_inline();
                lbl.set_markup(&format!(
                    "{}\n<small>ilvl {} · {} · {}</small>",
                    qspan(it.quality, it.name.as_str()), it.ilvl,
                    glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
                lbl.set_tooltip_text(Some(&it.tip()));
                let can = !candidates(it, h.class, two_now).is_empty();
                let eq = gtk::Button::with_label("Equip");
                eq.set_valign(gtk::Align::Center);
                eq.set_sensitive(can);
                if !can { eq.set_tooltip_text(Some("Your class can't use this, or a two-handed weapon blocks the off-hand.")); }
                { let u = self.clone(); eq.connect_clicked(move |_| u.equip(i)); }
                let v = it.sell_value();
                let sell = gtk::Button::with_label(&if v == 0 { "Bound".to_string() } else { format!("Sell {v}g") });
                sell.set_valign(gtk::Align::Center);
                sell.set_sensitive(v > 0);
                { let u = self.clone(); sell.connect_clicked(move |_| u.sell(i)); }
                row.append(&lbl); row.append(&eq); row.append(&sell);
                self.bag_list.append(&row);
            }
        }

        // ----- store
        pub(crate) fn refresh_store(self: &Rc<Self>) {
            while let Some(c) = self.store_pots.first_child() { self.store_pots.remove(&c); }
            while let Some(c) = self.store_list.first_child() { self.store_list.remove(&c); }
            let Some(a) = self.active.get() else { return };
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(a) else { return };
            if self.store_key.get() != (a, h.level) {
                *self.store.borrow_mut() = gen_store(h.class, h.spec, h.level);
                self.store_key.set((a, h.level));
            }
            self.store_gold.set_text(&format!("💰 You have {} gold · bag {}/{}", h.gold, h.bag.len(), BAG_MAX));

            let pp = potion_price(h.level);
            for k in 0..3 {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let lbl = gtk::Label::new(Some(&format!(
                    "{} {} — restores {POT_PCT}% · you have {}", POT[k].1, POT[k].0, h.pots[k])));
                lbl.set_xalign(0.0); lbl.set_hexpand(true);
                let b = gtk::Button::with_label(&format!("Buy {pp}g"));
                b.set_sensitive(h.gold >= pp);
                { let u = self.clone(); b.connect_clicked(move |_| u.buy_potion(k)); }
                row.append(&lbl); row.append(&b);
                self.store_pots.append(&row);
            }

            let stock = self.store.borrow();
            if stock.is_empty() { self.store_list.append(&gtk::Label::new(Some("Sold out — press Restock."))); }
            for (i, it) in stock.iter().enumerate() {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let lbl = gtk::Label::new(None);
                lbl.set_xalign(0.0); lbl.set_hexpand(true);
                let tl = it.type_label();
                let si = it.stats_inline();
                let up = if h.is_upgrade(it) { "<span foreground='#3fb950'>▲</span> " } else { "" };
                lbl.set_markup(&format!(
                    "{up}{}\n<small>ilvl {} · {} · {}</small>",
                    qspan(it.quality, it.name.as_str()), it.ilvl,
                    glib::markup_escape_text(tl.as_str()), glib::markup_escape_text(si.as_str())));
                lbl.set_tooltip_text(Some(&it.tip()));
                let price = it.buy_price();
                let b = gtk::Button::with_label(&format!("Buy {price}g"));
                b.set_valign(gtk::Align::Center);
                b.set_sensitive(h.gold >= price);
                { let u = self.clone(); b.connect_clicked(move |_| u.buy(i)); }
                row.append(&lbl); row.append(&b);
                self.store_list.append(&row);
            }
        }

        pub(crate) fn restock(self: &Rc<Self>) {
            self.store_key.set((usize::MAX, 0));
            self.refresh_store();
        }

        pub(crate) fn buy(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            let it = {
                let st = self.store.borrow();
                match st.get(i) { Some(x) => x.clone(), None => return }
            };
            let price = it.buy_price();
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                if h.gold < price {
                    msgs.push(m("System", format!("💸 Not enough gold — {} costs {price}g.", it.name)));
                } else if h.bag.len() >= BAG_MAX {
                    msgs.push(m("Loot", "🎒 Bag is full — sell something first."));
                } else {
                    h.gold -= price;
                    msgs.push(m("Loot", format!("🛒 Bought {} for {price} gold", it.label())));
                    h.bag.push(it);
                    self.store.borrow_mut().remove(i);
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn buy_potion(self: &Rc<Self>, k: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let price = potion_price(h.level);
                if h.gold < price {
                    msgs.push(m("System", format!("💸 Not enough gold — a {} costs {price}g.", POT[k].0)));
                } else {
                    h.gold -= price;
                    h.pots[k] += 1;
                    msgs.push(m("Loot", format!("🛒 Bought a {} for {price} gold", POT[k].0)));
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn drink(self: &Rc<Self>, k: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let mx = h.maxes();
                let max = [mx.0, mx.1, mx.2][k];
                let cur = [h.hp, h.mana, h.sta][k];
                if h.pots[k] == 0 {
                    msgs.push(m("System", format!("You have no {}.", POT[k].0)));
                } else if cur >= max {
                    msgs.push(m("System", "That bar is already full."));
                } else {
                    let newv = (cur + max * POT_PCT / 100).min(max);
                    match k { 0 => h.hp = newv, 1 => h.mana = newv, _ => h.sta = newv }
                    h.pots[k] -= 1;
                    msgs.push(m("Loot", format!("{} Drank a {}: +{}", POT[k].1, POT[k].0, newv - cur)));
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn equip(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                if i >= h.bag.len() { return; }
                let it = h.bag.remove(i);
                let label = it.label();
                match h.place(it, true) {
                    Ok(old) => {
                        msgs.push(m("Loot", format!("🛡 Equipped {label}")));
                        for o in old { h.stash(o, &mut msgs); }
                        h.check_achievements(&mut msgs);
                    }
                    Err(it) => {
                        msgs.push(m("Loot", format!("❌ Can't equip {label}")));
                        let at = i.min(h.bag.len());
                        h.bag.insert(at, it);
                    }
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn unequip(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                if h.bag.len() >= BAG_MAX {
                    msgs.push(m("Loot", "🎒 Bag is full — sell something first."));
                } else if let Some(it) = h.gear.get_mut(i).and_then(|o| o.take()) {
                    msgs.push(m("Loot", format!("📦 Unequipped {}", it.label())));
                    h.bag.push(it);
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn sell(self: &Rc<Self>, i: usize) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let Some(it) = h.bag.get(i) else { return };
                let v = it.sell_value();
                if v == 0 {
                    msgs.push(m("Loot", format!("🔒 {} is soulbound and can't be sold.", it.name)));
                } else {
                    let it = h.bag.remove(i);
                    h.gold += v;
                    msgs.push(m("Loot", format!("💰 Sold {} for {v} gold", it.label())));
                }
            }
            self.finish(msgs);
        }

        pub(crate) fn sell_junk(self: &Rc<Self>) {
            let Some(a) = self.active.get() else { return };
            let mut msgs: Vec<Msg> = vec![];
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                let (mut total, mut n) = (0u32, 0u32);
                let mut keep: Vec<Item> = vec![];
                for it in std::mem::take(&mut h.bag) {
                    let v = it.sell_value();
                    if it.quality <= 2 && v > 0 { total += v; n += 1; } else { keep.push(it); }
                }
                h.bag = keep;
                h.gold += total;
                msgs.push(m("Loot", format!("💰 Sold {n} junk items for {total} gold")));
            }
            self.finish(msgs);
        }

}
