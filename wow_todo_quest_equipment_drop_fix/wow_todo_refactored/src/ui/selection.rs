use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{data::classes::*, persistence::persist, utils::*};
use super::{Ui, pad};

impl Ui {
        pub(crate) fn refresh_select(self: &Rc<Self>) {
            while let Some(c) = self.sel_list.first_child() { self.sel_list.remove(&c); }
            let s = self.save.borrow();
            if s.heroes.is_empty() {
                self.sel_list.append(&gtk::Label::new(Some("No saved characters yet — create one!")));
            }
            for (i, hero) in s.heroes.iter().enumerate() {
                let c = &CLASSES[hero.class];
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                pad(&row, 8);
                let info = gtk::Label::new(None);
                info.set_xalign(0.0); info.set_hexpand(true);
                info.set_markup(&format!(
                    "<b>{}</b>\nLevel {} {} {} ({}) · <span foreground='{}'>{}</span> · {}\n📍 {} · 💰 {} · 🏆 {} achievements",
                    glib::markup_escape_text(hero.name.as_str()), hero.level, hero.race, c.name, c.specs[hero.spec].0,
                    fcol(&hero.faction), hero.faction, glib::markup_escape_text(hero.realm.as_str()),
                    glib::markup_escape_text(hero.zone.as_str()), hero.gold, hero.achievements.len()));
                let play = gtk::Button::with_label("▶ Play");
                play.add_css_class("suggested-action"); play.set_valign(gtk::Align::Center);
                let confirming = self.confirm_del.get() == Some(i);
                let del = gtk::Button::with_label(if confirming { "Confirm delete?" } else { "Delete" });
                if confirming { del.add_css_class("destructive-action"); }
                del.set_valign(gtk::Align::Center);
                { let u = self.clone(); play.connect_clicked(move |_| u.enter(i)); }
                { let u = self.clone(); del.connect_clicked(move |_| u.delete_click(i)); }
                row.append(&info); row.append(&play); row.append(&del);
                let f = gtk::Frame::new(None);
                f.set_child(Some(&row));
                self.sel_list.append(&f);
            }
        }

        pub(crate) fn delete_click(self: &Rc<Self>, i: usize) {
            if self.confirm_del.get() == Some(i) {
                self.confirm_del.set(None);
                {
                    let mut s = self.save.borrow_mut();
                    if i < s.heroes.len() { s.heroes.remove(i); }
                }
                persist(&self.save.borrow());
            } else {
                self.confirm_del.set(Some(i));
            }
            self.refresh_select();
        }

        pub(crate) fn enter(self: &Rc<Self>, i: usize) {
            self.confirm_del.set(None);
            self.store_key.set((usize::MAX, 0));
            let (realm, pop) = {
                let s = self.save.borrow();
                let Some(h) = s.heroes.get(i) else { return };
                (h.realm.clone(), self.realms.get(h.realm_tier).map_or(0, |r| r.pop))
            };
            self.active.set(Some(i));
            self.new_opponents();
            self.stack.set_visible_child_name("game");
            self.finish(vec![m("System", format!("🔑 Entered realm {realm} ({} players online)", commas(pop)))]);
        }

}
