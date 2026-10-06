use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{data::items::SLOTS, items::qspan, model::*, utils::*};
use super::Ui;

impl Ui {
    pub(crate) fn refresh_blacksmith(self: &Rc<Self>) {
        while let Some(c) = self.smith_list.first_child() { self.smith_list.remove(&c); }
        let Some(a) = self.active.get() else { return };
        let s = self.save.borrow();
        let Some(h) = s.heroes.get(a) else { return };
        let total = h.repair_total();
        self.smith_lbl.set_text(&format!("💰 You have {} gold · repairing everything costs {total}g", h.gold));
        self.smith_all.set_label(&format!("🔨 Repair all ({total}g)"));
        self.smith_all.set_sensitive(total > 0 && h.gold >= total);

        let title = gtk::Label::new(None); title.set_markup("<b>Equipped</b>"); title.set_xalign(0.0);
        self.smith_list.append(&title);
        for (i, slot) in SLOTS.iter().enumerate() {
            if let Some(it) = &h.gear[i] { self.smith_list.append(&self.smith_row(slot, it, h.gold, false, i)); }
        }
        let damaged: Vec<(usize, &Item)> = h.bag.iter().enumerate().filter(|(_, it)| it.durability < 100).collect();
        if !damaged.is_empty() {
            let t = gtk::Label::new(None); t.set_markup("<b>Damaged items in bag</b>"); t.set_xalign(0.0);
            self.smith_list.append(&t);
            for (i, it) in damaged { self.smith_list.append(&self.smith_row("Bag", it, h.gold, true, i)); }
        }
    }

    fn smith_row(self: &Rc<Self>, title: &str, it: &Item, gold: u32, bag: bool, idx: usize) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let lbl = gtk::Label::new(None);
        lbl.set_xalign(0.0); lbl.set_hexpand(true);
        lbl.set_markup(&format!("<b>{}</b>: {}\n<small>ilvl {} · Durability {}/100</small>",
            glib::markup_escape_text(title), qspan(it.quality, it.name.as_str()), it.ilvl, it.durability));
        let cost = it.repair_cost();
        let btn = gtk::Button::with_label(&if cost == 0 { "OK".to_string() } else { format!("Repair {cost}g") });
        btn.set_valign(gtk::Align::Center);
        btn.set_sensitive(cost > 0 && gold >= cost);
        { let u = self.clone(); btn.connect_clicked(move |_| u.repair(bag, idx)); }
        row.append(&lbl); row.append(&btn);
        row
    }

    pub(crate) fn repair(self: &Rc<Self>, bag: bool, i: usize) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let gold = h.gold;
            let it = if bag { h.bag.get_mut(i) } else { h.gear.get_mut(i).and_then(|o| o.as_mut()) };
            let Some(it) = it else { return };
            let (cost, name) = (it.repair_cost(), it.name.clone());
            if cost == 0 { return; }
            if gold < cost {
                msgs.push(m("System", format!("💸 Not enough gold — repairing {name} costs {cost}g.")));
            } else {
                it.durability = 100;
                h.gold -= cost;
                msgs.push(m("Loot", format!("🔨 Repaired {name} for {cost} gold")));
            }
        }
        self.finish(msgs);
    }

    pub(crate) fn repair_all(self: &Rc<Self>) {
        let Some(a) = self.active.get() else { return };
        let mut msgs: Vec<Msg> = vec![];
        {
            let mut s = self.save.borrow_mut();
            let Some(h) = s.heroes.get_mut(a) else { return };
            let total = h.repair_total();
            if total == 0 {
                msgs.push(m("System", "Nothing needs repairing."));
            } else if h.gold < total {
                msgs.push(m("System", format!("💸 Not enough gold — repairing everything costs {total}g.")));
            } else {
                for it in h.gear.iter_mut().flatten().chain(h.bag.iter_mut()) { it.durability = 100; }
                h.gold -= total;
                msgs.push(m("Loot", format!("🔨 Repaired all equipment for {total} gold")));
            }
        }
        self.finish(msgs);
    }
}