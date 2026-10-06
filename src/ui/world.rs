
use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{config::*, model::*, utils::*, world::*};
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
            let mut open: Vec<&Zone> = self.zones.iter()
                .filter(|z| z.open_to(&h.faction, h.level) && z.matches_kind(kind)
                            && (q.is_empty() || z.name.to_lowercase().contains(&q)))
                .collect();
            let total = open.len();
            // Always list the lowest-level zones first. For equal minimum levels,
            // narrower ranges come before broad ranges (e.g. 1-10 before 1-90).
            open.sort_by(|a, b| (a.lo, a.hi, &a.name).cmp(&(b.lo, b.hi, &b.name)));
            open.truncate(ZONE_ROWS);
            self.zcount.set_text(&format!(
                "{} of {} zones available to you (level {}, {}) — showing {} zones, lowest level first. Dungeon zones: +25% rewards, Raid zones: +50%.",
                total, self.zones.len(), h.level, h.faction, open.len()));
            for z in open.iter() {
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
            let name = gtk::Label::new(None);
            name.set_xalign(0.0);
            name.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(z.name.as_str())));
            let sub = gtk::Label::new(Some(&format!(
                "Lv {} · {} · {}", z.level_text(), z.terr, if z.inst.is_empty() { "Open world" } else { z.inst.as_str() })));
            sub.set_xalign(0.0); sub.add_css_class("dim-label");
            info.append(&name); info.append(&sub);
            let here = hero.zone == z.name && hero.zone_inst == z.inst;
            let btn = gtk::Button::with_label(if here { "📍 Here" } else { "Travel" });
            btn.set_sensitive(!here);
            btn.set_valign(gtk::Align::Center);
            { let u = self.clone(); let z = z.clone(); btn.connect_clicked(move |_| u.travel(&z)); }
            row.append(&info); row.append(&btn);
            row
        }

        pub(crate) fn travel(self: &Rc<Self>, z: &Zone) {
            let Some(a) = self.active.get() else { return };
            {
                let mut s = self.save.borrow_mut();
                let Some(h) = s.heroes.get_mut(a) else { return };
                h.zone = z.name.clone();
                h.zone_inst = z.inst.clone();
            }
            let b = inst_bonus(&z.inst);
            let kind = if z.inst.is_empty() { z.terr.as_str() } else { z.inst.as_str() };
            let extra = if b > 0 { format!(" — +{b}% quest rewards") } else { String::new() };
            self.finish(vec![m("Travel", format!("🗺 Traveled to {} ({kind}){extra}", z.name))]);
        }

        // ----- quests
}

