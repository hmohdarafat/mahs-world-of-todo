use std::rc::Rc;
use gtk::prelude::*;
use crate::{config::*, data::{classes::*, creatures::matchup_markup}, model::*, persistence::*, utils::*, world::{all_zone_names, generate_zones, starting_zone}};
use super::{common::labeled, Ui};

pub(crate) fn create_screen(ui: &Rc<Ui>) -> gtk::Box {

        let b = gtk::Box::new(gtk::Orientation::Vertical, 10);
        b.set_halign(gtk::Align::Center); b.set_valign(gtk::Align::Center); b.set_width_request(480);
        let head = gtk::Label::new(None);
        head.set_markup("<span size='xx-large' weight='bold'>Who am I?</span>");

        let labels: Vec<String> = ui.realms.iter().map(|r| r.label()).collect();
        let lrefs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
        let realm = gtk::DropDown::from_strings(&lrefs);
        let faction = gtk::DropDown::from_strings(&FACTIONS);
        let race = gtk::DropDown::from_strings(&races("Horde"));
        let class_names: Vec<&str> = CLASSES.iter().map(|c| c.name).collect();
        let class = gtk::DropDown::from_strings(&class_names);
        let spec = gtk::DropDown::from_strings(&spec_names(0));
        let name = gtk::Entry::new(); name.set_placeholder_text(Some("Character name"));
        let guild = gtk::Entry::new(); guild.set_placeholder_text(Some("Guild (optional)"));
        let info = gtk::Label::new(None);
        let mu = gtk::Label::new(None); mu.set_xalign(0.0); mu.set_wrap(true);
        info.set_xalign(0.0); info.set_wrap(true); info.add_css_class("dim-label");

        let upd: Rc<dyn Fn()> = {
        let (c, s, i, mu2) = (class.clone(), spec.clone(), info.clone(), mu.clone());            Rc::new(move || {
                let cl = &CLASSES[(c.selected() as usize).min(CLASSES.len() - 1)];
                let si = (s.selected() as usize).min(cl.specs.len() - 1);
                let mut roles: Vec<&str> = vec![];
                for (_, r) in cl.specs {
                    if !roles.contains(&r.name()) { roles.push(r.name()); }
                }
                let weapons: Vec<&str> = cl.weapons.iter().map(|w| w.name()).collect();
                i.set_text(&format!(
                    "{} armor · {} · weapons: {}\nclass roles: {} · selected spec role: {}",
                    cl.armor, offhand_text(cl), weapons.join(", "), roles.join(", "), cl.specs[si].1.name()));
                    mu2.set_markup(&matchup_markup(cl.name));
            })
        };
        {
            let r = race.clone();
            faction.connect_selected_notify(move |d| {
                let fac = FACTIONS[(d.selected() as usize).min(FACTIONS.len() - 1)];
                r.set_model(Some(&gtk::StringList::new(&races(fac))));
                r.set_selected(0);
            });
        }
        {
            let (sp, u) = (spec.clone(), upd.clone());
            class.connect_selected_notify(move |d| {
                let ci = (d.selected() as usize).min(CLASSES.len() - 1);
                sp.set_model(Some(&gtk::StringList::new(&spec_names(ci))));
                sp.set_selected(0);
                u();
            });
        }
        { let u = upd.clone(); spec.connect_selected_notify(move |_| u()); }
        upd();

        let go = gtk::Button::with_label("Enter the world");
        go.add_css_class("suggested-action");
        let back = gtk::Button::with_label("← Back to characters");

        b.append(&head);
        b.append(&labeled("1. Choose realm (population)", &realm));
        b.append(&labeled("2. Choose faction", &faction));
        b.append(&labeled("3. Choose race (faction races + Pandaren, Dracthyr)", &race));
        b.append(&labeled("4. Choose class", &class));
        b.append(&labeled("5. Choose specialization", &spec));
        b.append(&info);
        b.append(&mu);
        b.append(&labeled("6. Create identity", &name));
        b.append(&guild);
        b.append(&go);
        b.append(&back);

        { let u = ui.clone(); back.connect_clicked(move |_| u.show_select()); }
        let u = ui.clone();
        go.connect_clicked(move |_| {
            name.remove_css_class("error");
            let n = name.text().trim().to_string();
            let dup = u.save.borrow().heroes.iter().any(|h| h.name.eq_ignore_ascii_case(&n));
            if n.is_empty() || dup { name.add_css_class("error"); return; }
            let ri = (realm.selected() as usize).min(u.realms.len() - 1);
            let fac = FACTIONS[(faction.selected() as usize).min(FACTIONS.len() - 1)];
            let rl = races(fac);
            let race_name = rl[(race.selected() as usize).min(rl.len() - 1)];
            let ci = (class.selected() as usize).min(CLASSES.len() - 1);
            let si = (spec.selected() as usize).min(CLASSES[ci].specs.len() - 1);
            let zones = generate_zones(&all_zone_names(&u.save.borrow().heroes));
            let zone = starting_zone(&zones, fac);
            let hero = Hero::new(n.clone(), guild.text().trim().to_string(), &u.realms[ri], fac, race_name, ci, si, zone, zones);
            let idx = {
                let mut s = u.save.borrow_mut();
                s.heroes.push(hero);
                s.heroes.len() - 1
            };
            persist(&u.save.borrow());
            name.set_text("");
            u.enter(idx);
            u.finish(vec![m("System", format!("🎉 {n} created! Find an NPC and accept your first quest."))]);
        });
        b
    }