use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{config::*, data::{classes::CLASSES, creatures::*}, model::*, utils::*};
use super::{Ui, pad};

impl Ui {
        pub(crate) fn refresh_logs(self: &Rc<Self>) {
            let Some(a) = self.active.get() else { return };
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(a) else { return };
            let filt = LOG_FILTERS[(self.log_filter.selected() as usize).min(LOG_FILTERS.len() - 1)];
            let text = h.log.iter().rev()
                .filter(|e| filt == "All" || e.cat == filt)
                .map(|e| format!("[{}] [{}] {}", e.time, e.cat, e.msg))
                .collect::<Vec<_>>().join("\n");
            self.log_view.buffer().set_text(&text);
        }

        pub(crate) fn refresh_zones(self: &Rc<Self>) {
            while let Some(c) = self.zlist.first_child() { self.zlist.remove(&c); }
            let Some(a) = self.active.get() else { return };
            let s = self.save.borrow();
            let Some(h) = s.heroes.get(a) else { return };
            let q = self.zsearch.text().to_lowercase();
            let kind = self.zkind.selected();
            // every zone is always listed, regardless of level or faction
            let mut shown: Vec<&Zone> = h.zones.iter()
                .filter(|z| z.matches_kind(kind)
                            && (q.is_empty()
                                || z.name.to_lowercase().contains(&q)
                                || z.creature_text().to_lowercase().contains(&q)))
                .collect();
            let total = shown.len();
            shown.sort_by(|a, b| (a.lo, a.hi, &a.terr, &a.name).cmp(&(b.lo, b.hi, &b.terr, &b.name)));
            shown.truncate(ZONE_ROWS);
            self.zcount.set_text(&format!(
                "{} zones shown (3 per level range, lowest first). 🔒 zones unlock at their minimum level. \
                 Enemy-faction zones give +50% XP and the most higher-level creatures, contested +25%, your own faction's +0%. \
                 Each zone holds at most 3 creature types (green = advantage, yellow = average, red = disadvantage for your class).",
                total));
            for z in shown.iter() {
                let fr = gtk::Frame::new(None);
                fr.set_child(Some(&self.zone_row(z, h)));
                self.zlist.append(&fr);
            }
        }

        pub(crate) fn zone_row(self: &Rc<Self>, z: &Zone, hero: &Hero) -> gtk::Box {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            pad(&row, 6);
            let info = gtk::Box::new(gtk::Orientation::Vertical, 2);
            info.set_hexpand(true);
            info.set_tooltip_text(Some(&z.type_info()));
            let rel = z.relation(&hero.faction);
            let name = gtk::Label::new(None);
            name.set_xalign(0.0);
            name.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(z.name.as_str())));
            let sub = gtk::Label::new(Some(&format!(
                "Lv {} · {} ({}) · +{}% XP · {}% chance of higher-level creatures",
                z.level_text(), rel.name(), z.terr, rel.xp_bonus(), rel.high_chance())));
            sub.set_xalign(0.0); sub.add_css_class("dim-label");
            let cls = CLASSES[hero.class].name;
            let tm = z.types.iter().map(|&t| {
                let t = t.min(CTYPES.len() - 1);
                format!("<span foreground='{}'>{}</span>", matchup(cls, t).color(), CTYPES[t])
            }).collect::<Vec<_>>().join(" · ");
            let types = gtk::Label::new(None);
            types.set_xalign(0.0); types.set_wrap(true);
            types.set_markup(&format!("<b>Creature types:</b> {tm}"));
            let crea = gtk::Label::new(Some(&format!("Creatures: {}", z.creature_text())));
            crea.set_xalign(0.0); crea.set_wrap(true); crea.add_css_class("dim-label");
            info.append(&name); info.append(&sub); info.append(&types); info.append(&crea);
            let here = hero.zone == z.name;
            let open = z.open_to(hero.level);
            let btn = gtk::Button::with_label(if here { "📍 Here" } else if open { "Travel" } else { "🔒 Locked" });
            btn.set_sensitive(!here && open);
            if !open { btn.set_tooltip_text(Some(&format!("Unlocks at level {}", z.lo))); }
            btn.set_valign(gtk::Align::Center);
            { let u = self.clone(); let z = z.clone(); btn.connect_clicked(move |_| u.travel(&z)); }
            row.append(&info); row.append(&btn);
            row
        }

        pub(crate) fn travel(self: &Rc<Self>, z: &Zone) {
            let Some(a) = self.active.get() else { return };
            let rel = {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                h.zone = z.name.clone();
                h.zone_inst.clear();
                z.relation(&h.faction)
            };
            self.finish(vec![m("Travel", format!(
                "🗺 Traveled to {} ({}, +{}% XP) — creatures: {}",
                z.name, rel.name(), rel.xp_bonus(), z.type_text()))]);
        }

        // ----- quests
}