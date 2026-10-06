use std::rc::Rc;
use gtk::{glib, prelude::*};
use crate::{config::*, data::{abilities::*, classes::*, creatures::matchup_markup, items::SLOTS}, model::*, persistence::*, quests::*, utils::*};
use super::{Ui, pad, set_bar, set_options};

impl Ui {
        pub(crate) fn finish(self: &Rc<Self>, msgs: Vec<Msg>) {
            if let Some(a) = self.active.get() {
                if let Some(h) = self.save.borrow_mut().heroes.get_mut(a) {
                    h.clamp_res();
                    h.add_log(msgs);
                }
            }
            persist(&self.save.borrow());
            self.refresh();
        }

        pub(crate) fn clear_logs(self: &Rc<Self>) {
            if let Some(a) = self.active.get() {
                if let Some(h) = self.save.borrow_mut().heroes.get_mut(a) { h.log.clear(); }
            }
            self.finish(vec![m("System", "🧹 Logs cleared.")]);
        }

        pub(crate) fn refresh(self: &Rc<Self>) {
            let Some(a) = self.active.get() else { return };
            {
                let s = self.save.borrow();
                let Some(h) = s.heroes.get(a) else { return };
                let c = &CLASSES[h.class];
                let (spec_name, role) = c.specs[h.spec];
                let f = h.fighter();
                let st = f.stats();

                let guild = if h.guild.is_empty() { String::new() }
                            else { format!(" · &lt;{}&gt;", glib::markup_escape_text(h.guild.as_str())) };
                self.title.set_markup(&format!(
                    "<span size='x-large' weight='bold'>{}</span>{guild}\nLevel {} {} {} ({} · {}) · <span foreground='{}'>{}</span> · {}\n📍 {}",
                    glib::markup_escape_text(h.name.as_str()), h.level, h.race, c.name, spec_name, role.name(),
                    fcol(&h.faction), h.faction, glib::markup_escape_text(h.realm.as_str()),
                    glib::markup_escape_text(h.zone.as_str())));

                self.xp_bar.set_fraction((h.xp as f64 / h.need() as f64).min(1.0));
                self.xp_bar.set_text(Some(&format!("{} / {} XP", h.xp, h.need())));
                set_bar(&self.hp_bar, "❤ HP", h.hp, st.hp);
                set_bar(&self.mana_bar, "🔷 Mana", h.mana, st.mana);
                set_bar(&self.sta_bar, "⚡ Stamina", h.sta, st.sta);
                for k in 0..3 {
                    let short = POT[k].0.split(' ').next().unwrap_or("");
                    self.pot_btn[k].set_label(&format!("{} {} ×{}", POT[k].1, short, h.pots[k]));
                    self.pot_btn[k].set_sensitive(h.pots[k] > 0);
                    self.pot_btn[k].set_tooltip_text(Some(&format!("Drink a {}: restores {POT_PCT}% of the bar", POT[k].0)));
                }
                self.stats.set_text(&format!("💰 {} gold   ✨ Talents: {}   🎖 Honor: {}", h.gold, h.talents, h.honor));

                let zref = h.zone_ref();
                let zinfo = zref.map_or(String::new(), |z| {
                    let r = z.relation(&h.faction);
                    format!(" (Lv {} · {} · +{}% XP · {}% chance of higher-level creatures)",
                            z.level_text(), r.name(), r.xp_bonus(), r.high_chance())
                });
                self.giver.set_text(&format!("Find an NPC in {}{zinfo} and accept a quest:", h.zone));
                let (quest_min, quest_max) = quest_level_bounds(h.level, zref);
                let previous_level = self.quest_level.model()
                    .and_then(|m| m.downcast::<gtk::StringList>().ok())
                    .and_then(|sl| sl.string(self.quest_level.selected()).map(|v| v.as_str().trim_start_matches("Lv ").parse::<u32>().ok()).flatten())
                    .unwrap_or(h.level);
                let qlv: Vec<String> = (quest_min..=quest_max).map(|lv| format!("Lv {lv}")).collect();
                set_options(&self.quest_level, &qlv);
                let keep_level = previous_level.clamp(quest_min, quest_max);
                self.quest_level.set_selected((keep_level - quest_min) as u32);
                let tl: Vec<String> = Tier::ALL.iter().map(|t| {
                    if t.unlock() <= h.level { format!("{} · x{} rewards", t.name(), t.mult()) }
                    else { format!("{} 🔒 Lv {}", t.name(), t.unlock()) }
                }).collect();
                set_options(&self.tier, &tl);
                let cl: Vec<String> = CATS.iter().enumerate().map(|(i, n)| {
                    if i == 0 || h.level >= PROF_LEVEL { format!("{n} ({})", cat_kind(i)) }
                    else { format!("{n} 🔒 Lv {PROF_LEVEL}") }
                }).collect();
                set_options(&self.cat, &cl);

                while let Some(ch) = self.qlist.first_child() { self.qlist.remove(&ch); }
                if h.quests.is_empty() { self.qlist.append(&gtk::Label::new(Some("No active quests."))); }
                for (i, q) in h.quests.iter().enumerate() {
                    let fr = gtk::Frame::new(None);
                    fr.set_child(Some(&self.quest_row(i, q)));
                    self.qlist.append(&fr);
                }
                let tail: Vec<&str> = h.log.iter().rev().take(3).map(|e| e.msg.as_str()).collect();
                self.status.set_text(&tail.into_iter().rev().collect::<Vec<_>>().join("\n"));

                let pop = self.realms.get(h.realm_tier).map_or(0, |r| r.pop);
                let pvp_faction = PVP_FACTIONS[(self.pvp_faction.selected() as usize).min(PVP_FACTIONS.len() - 1)];
                let shown = self.opps.borrow().iter().filter(|f| pvp_faction == "All" || f.faction == pvp_faction).count();
                self.pvp_head.set_text(&format!(
                    "🌐 Realm {} — {} players online (showing {} nearby, lowest level first · faction: {})\nYou: Lv {} · HP {}/{} · Mana {}/{} · Stamina {}/{} · ATK {} · DEF {} · Crit {}% · {}W-{}L · {} honor",
                    h.realm, commas(pop), shown, pvp_faction, h.level, h.hp, st.hp, h.mana, st.mana, h.sta, st.sta,
                    st.atk, st.def, st.crit, h.wins, h.losses, h.honor));

                let mut t = String::new();
                t += &format!("<b>Combat overview</b>\nHP {}/{} · Mana {}/{} · Stamina {}/{}\n", h.hp, st.hp, h.mana, st.mana, h.sta, st.sta);
                t += &format!("ATK {} · DEF {} · Crit {}% · Regen {}/round · avg ilvl {}\n", st.atk, st.def, st.crit, st.heal, f.ilvl_avg());
                t += &format!("{} armor · {} · {}\n\n", c.armor, role.name(), f.setup());
                t += &matchup_markup(c.name);
                t += "\n\n";
                t += &f.stat_breakdown();
                t += "\n<b>Exact equipment contribution by slot</b>\n";
                for (i, slot) in SLOTS.iter().enumerate() {
                    match &h.gear[i] {
                        Some(it) if it.kind != Kind::Cosmetic => {
                            let impact = h.equipped_item_impact(i);
                            let raw = it.stats_inline();
                            t += &format!(
                                "  {slot}: {} — {} — Final-stat contribution: {}\n",
                                esc(it.name.as_str()), esc(&raw), esc(&impact)
                            );
                        }
                        Some(it) => t += &format!("  {slot}: {} — cosmetic, no stat contribution\n", esc(it.name.as_str())),
                        None => t += &format!("  {slot}: empty\n"),
                    }
                }
                t += "\n";
                t += "<b>Professions</b>\n";
                let profs: Vec<String> = h.prof.iter().enumerate().filter(|(_, p)| **p > 0)
                    .map(|(i, p)| format!("{}: {}", CATS[i + 1], p)).collect();
                t += &if profs.is_empty() { "—".to_string() } else { profs.join("\n") };
                t += "\n\n<b>Achievements</b>\n";
                t += &if h.achievements.is_empty() { "—".to_string() } else { h.achievements.join("\n") };
                self.sheet.set_markup(&t);
                self.refresh_abilities(h, &st);
            }
            self.refresh_equipment();
            self.refresh_store();
            self.refresh_honor_store();
            self.refresh_blacksmith();
            self.refresh_zones();
            self.refresh_logs();
        }

        pub(crate) fn refresh_abilities(&self, h: &Hero, st: &Stats) {
            while let Some(c) = self.ab_list.first_child() { self.ab_list.remove(&c); }
            let cls = &CLASSES[h.class];
            let spec_name = cls.specs[h.spec].0;
            let (have, locked): (Vec<&Ab>, Vec<&Ab>) = cls.abilities.iter()
                .filter(|a| spec_ok(a, spec_name))
                .partition(|a| a.l <= h.level);
            let head = gtk::Label::new(None);
            head.set_xalign(0.0);
            head.set_markup(&format!("<b>Abilities</b> — {} learned (damage/heal numbers use your current stats)", have.len()));
            self.ab_list.append(&head);
            for a in have {
                let lbl = gtk::Label::new(None);
                lbl.set_xalign(0.0); lbl.set_wrap(true);
                lbl.set_markup(&format!(
                    "<span foreground='{}'><b>{}</b></span>  <small>Lv {} · {}</small>\n<small>{}</small>",
                    kcol(a.k), esc(aname(a, &h.faction)), a.l, kname(a.k), esc(&ab_effect(a, st))));
                pad(&lbl, 6);
                let fr = gtk::Frame::new(None);
                fr.set_child(Some(&lbl));
                self.ab_list.append(&fr);
            }
            if !locked.is_empty() {
                let txt: Vec<String> = locked.iter().map(|a| format!("Lv {} {}", a.l, aname(a, &h.faction))).collect();
                let l = gtk::Label::new(Some(&format!("🔒 Upcoming: {}", txt.join(" · "))));
                l.set_xalign(0.0); l.set_wrap(true); l.add_css_class("dim-label");
                self.ab_list.append(&l);
            }
        }

}