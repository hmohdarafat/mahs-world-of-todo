//! GTK presentation layer. UI modules only coordinate widgets and invoke game services.

use std::{cell::{Cell, RefCell}, rc::Rc};
use gtk::{glib, prelude::*};

use crate::{
    config::*, data::{creatures::CTYPES, items::QUALITY_GUIDE}, model::*, persistence::*, utils::*, world::*,
};

mod common;
mod create;
mod dashboard;
mod inventory;
mod pvp;
mod quests;
mod selection;
mod world;
mod blacksmith;

use create::create_screen;

pub(crate) use common::*;

const CSS: &str = "progressbar.hp-bar progress { background: #c0392b; } \
     progressbar.mana-bar progress { background: #2e86de; } \
     progressbar.sta-bar progress { background: #d4a017; } \
     frame.gear-slot { padding: 6px; border-radius: 8px; background: #101722; min-width: 145px; } \
     frame.gear-slot label { color: #e6edf3; } \
     frame.gear-empty { border: 2px solid #30363d; } \
     frame.gear-quality-0 { border: 2px solid #9d9d9d; background: rgba(157,157,157,0.08); } \
     frame.gear-quality-1 { border: 2px solid #ffffff; background: rgba(255,255,255,0.06); } \
     frame.gear-quality-2 { border: 2px solid #1eff00; background: rgba(30,255,0,0.08); } \
     frame.gear-quality-3 { border: 2px solid #0070dd; background: rgba(0,112,221,0.10); } \
     frame.gear-quality-4 { border: 2px solid #a335ee; background: rgba(163,53,238,0.12); } \
     frame.gear-quality-5 { border: 2px solid #ff8000; background: rgba(255,128,0,0.12); } \
     frame.gear-quality-6 { border: 2px solid #00ccff; background: rgba(0,204,255,0.12); }";

pub(crate) struct Ui {
    pub(crate) save: RefCell<Save>,
    pub(crate) active: Cell<Option<usize>>,
    pub(crate) confirm_del: Cell<Option<usize>>,
    pub(crate) realms: Vec<Realm>,
    pub(crate) opps: RefCell<Vec<Fighter>>,
    pub(crate) store: RefCell<Vec<Item>>,
    pub(crate) store_key: Cell<(usize, u32)>,
    pub(crate) honor_stock: RefCell<Vec<Item>>,
    pub(crate) honor_key: Cell<(usize, u32)>,
    /// Bit per notebook page: set = page content is stale and must be rebuilt when shown.
    pub(crate) dirty: Cell<u16>,
    pub(crate) save_pending: Cell<bool>,
    pub(crate) stack: gtk::Stack,
    pub(crate) nb: gtk::Notebook,
    pub(crate) sel_list: gtk::Box,
    pub(crate) title: gtk::Label, xp_bar: gtk::ProgressBar, stats: gtk::Label,
    pub(crate) hp_bar: gtk::ProgressBar, mana_bar: gtk::ProgressBar, sta_bar: gtk::ProgressBar,
    pub(crate) pot_btn: [gtk::Button; 3],
    pub(crate) giver: gtk::Label, entry: gtk::Entry, goal: gtk::SpinButton,
    pub(crate) qkind: gtk::DropDown, quest_level: gtk::DropDown, tier: gtk::DropDown, cat: gtk::DropDown, chain: gtk::CheckButton,
    pub(crate) qlist: gtk::Box, status: gtk::Label,
    pub(crate) zsearch: gtk::Entry, zkind: gtk::DropDown, zcount: gtk::Label, zlist: gtk::Box,
    pub(crate) pvp_head: gtk::Label, pvp_faction: gtk::DropDown, pvp_result: gtk::Label, pvp_list: gtk::Box,
    pub(crate) log_filter: gtk::DropDown, log_view: gtk::TextView,
    pub(crate) sheet: gtk::Label, ab_list: gtk::Box,
    pub(crate) eq_sum: gtk::Label, eq_paperdoll: gtk::Box, eq_list: gtk::Box, bag_head: gtk::Label, bag_list: gtk::Box,
    pub(crate) store_gold: gtk::Label, store_pots: gtk::Box, store_list: gtk::Box,
    pub(crate) honor_lbl: gtk::Label, honor_list: gtk::Box,
    pub(crate) smith_lbl: gtk::Label, smith_all: gtk::Button, smith_list: gtk::Box,
}

impl Ui {
    pub(crate) fn build(app: &gtk::Application) -> Rc<Self> {
        let ho = gtk::Orientation::Horizontal;
        let ve = gtk::Orientation::Vertical;

        // ----- styling
        let css = gtk::CssProvider::new();
        css.load_from_data(CSS);
        if let Some(d) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(&d, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

        // ----- character select screen
        let sel_list = gtk::Box::new(ve, 8);
        let sel_scroll = gtk::ScrolledWindow::builder().vexpand(true).min_content_height(320).child(&sel_list).build();
        let sel_new = gtk::Button::with_label("+ Create new character");
        sel_new.add_css_class("suggested-action");
        let sel_title = gtk::Label::new(None);
        sel_title.set_markup(&format!("<span size='xx-large' weight='bold'>{}</span>", glib::markup_escape_text(APP_TITLE)));
        let select = gtk::Box::new(ve, 12);
        pad(&select, 24);
        select.set_halign(gtk::Align::Center);
        select.set_width_request(620);
        select.append(&sel_title);
        select.append(&gtk::Label::new(Some("Choose a character to continue your adventure")));
        select.append(&sel_scroll);
        select.append(&sel_new);

        // ----- game header
        let title = gtk::Label::new(None); title.set_xalign(0.0); title.set_hexpand(true);
        let save_btn = gtk::Button::with_label("💾 Save"); save_btn.set_valign(gtk::Align::Start);
        let switch_btn = gtk::Button::with_label("Switch character"); switch_btn.set_valign(gtk::Align::Start);
        let head = gtk::Box::new(ho, 8);
        head.append(&title); head.append(&save_btn); head.append(&switch_btn);
        let xp_bar = gtk::ProgressBar::new(); xp_bar.set_show_text(true);
        let hp_bar = bar("hp-bar");
        let mana_bar = bar("mana-bar");
        let sta_bar = bar("sta-bar");
        let bars = gtk::Box::new(ho, 8);
        bars.append(&hp_bar); bars.append(&mana_bar); bars.append(&sta_bar);
        let pot_btn = [gtk::Button::new(), gtk::Button::new(), gtk::Button::new()];
        let potrow = gtk::Box::new(ho, 8);
        potrow.append(&gtk::Label::new(Some("Potions:")));
        for b in &pot_btn { potrow.append(b); }
        let stats = gtk::Label::new(None); stats.set_xalign(0.0); stats.set_wrap(true);

        // ----- quests tab
        let giver = gtk::Label::new(None); giver.set_xalign(0.0);
        let entry = gtk::Entry::new(); entry.set_hexpand(true);
        entry.set_placeholder_text(Some("e.g. Write report / Clean the kitchen"));
        let goal = gtk::SpinButton::with_range(1.0, MAX_QGOAL as f64, 1.0);
        goal.set_tooltip_text(Some("How many times the objective must be completed (1-15). Kill quests create one task per kill; gather quests count successful gathers."));
        let qkind = gtk::DropDown::from_strings(&["Kill", "Gather"]);
        let quest_level = gtk::DropDown::from_strings(&["Lv 1", "Lv 2", "Lv 3", "Lv 4", "Lv 5", "Lv 6"]);
        quest_level.set_tooltip_text(Some("Quest level is limited to 5 levels below/above your current character level."));
        qkind.set_tooltip_text(Some("Choose the quest objective type. Kill targets and gather ambushes use creatures from the zone you are in."));
        let tier = gtk::DropDown::from_strings(&["Normal"]);
        tier.set_tooltip_text(Some("Difficulty = how big the task is. It also sets the creature rank: Normal/Swarmer, Elite, Rare, Rare-Elite, Boss."));
        let cat = gtk::DropDown::from_strings(&["Adventure"]);
        cat.set_tooltip_text(Some("Profession = what kind of task this is. Completing it levels that skill (unlocks at level 5)."));
        let chain = gtk::CheckButton::with_label("Follow-up");
        chain.set_tooltip_text(Some("Turning in spawns a slightly harder follow-up quest"));
        let add_btn = gtk::Button::with_label("Accept quest");
        add_btn.add_css_class("suggested-action");
        add_btn.set_valign(gtk::Align::End);
        let add_row = gtk::Box::new(ho, 8);
        add_row.append(&labeled("Quest", &entry));
        add_row.append(&labeled("Type", &qkind));
        add_row.append(&labeled("Quest Lv", &quest_level));
        add_row.append(&labeled("Goal", &goal));
        add_row.append(&labeled("Difficulty", &tier));
        add_row.append(&labeled("Profession", &cat));
        add_row.append(&labeled("Chain", &chain));
        add_row.append(&add_btn);
        let hint = gtk::Label::new(Some(
            "Choose KILL (kill X creatures from your current zone — every kill is a line you fill in and mark done or abandon) \
             or GATHER (press ⚔ +1: each attempt may or may not find the item, and zone creatures ambush you while you search). \
             The Goal field controls the required count. \
             Every step has a chance to drop a health, mana or stamina potion. Quest Lv stays within ±5 of your character level; \
             higher-level quests consume more HP, mana and stamina when performed.",
        ));
        hint.set_xalign(0.0); hint.set_wrap(true); hint.add_css_class("dim-label");
        let qlist = gtk::Box::new(ve, 6);
        let qscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&qlist).build();
        let status = gtk::Label::new(None); status.set_xalign(0.0); status.set_wrap(true);
        let quests_page = gtk::Box::new(ve, 8);
        pad(&quests_page, 10);
        for w in [giver.upcast_ref::<gtk::Widget>(), add_row.upcast_ref(), hint.upcast_ref(), qscroll.upcast_ref(), status.upcast_ref()] {
            quests_page.append(w);
        }

        // ----- zones tab
        let zsearch = gtk::Entry::new(); zsearch.set_hexpand(true);
        zsearch.set_placeholder_text(Some("Search zones or creatures…"));
        let mut kinds: Vec<&str> = vec!["All creature types"];
        kinds.extend(CTYPES);
        let zkind = gtk::DropDown::from_strings(&kinds);
        let zcount = gtk::Label::new(None); zcount.set_xalign(0.0); zcount.add_css_class("dim-label");
        zcount.set_wrap(true);
        let zlist = gtk::Box::new(ve, 4);
        let zscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&zlist).build();
        let zrow = gtk::Box::new(ho, 8);
        zrow.append(&zsearch); zrow.append(&zkind);
        let zones_page = gtk::Box::new(ve, 8);
        pad(&zones_page, 10);
        zones_page.append(&zrow); zones_page.append(&zcount); zones_page.append(&zscroll);

        // ----- pvp tab
        let pvp_head = gtk::Label::new(None); pvp_head.set_xalign(0.0); pvp_head.set_wrap(true); pvp_head.set_hexpand(true);
        let pvp_faction = gtk::DropDown::from_strings(&PVP_FACTIONS);
        pvp_faction.set_tooltip_text(Some("Show players from all factions, Alliance only, or Horde only."));
        let pvp_refresh = gtk::Button::with_label("🔄 New players"); pvp_refresh.set_valign(gtk::Align::Start);
        let pvp_result = gtk::Label::new(None); pvp_result.set_xalign(0.0); pvp_result.set_wrap(true);
        let pvp_list = gtk::Box::new(ve, 6);
        let pscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&pvp_list).build();
        let prow = gtk::Box::new(ho, 8);
        prow.append(&pvp_head);
        prow.append(&labeled("Faction", &pvp_faction));
        prow.append(&pvp_refresh);
        let pvp_page = gtk::Box::new(ve, 8);
        pad(&pvp_page, 10);
        pvp_page.append(&prow); pvp_page.append(&pvp_result); pvp_page.append(&pscroll);

        // ----- equipment tab
        let eq_sum = gtk::Label::new(None); eq_sum.set_xalign(0.0); eq_sum.set_wrap(true);
        let eq_paperdoll = gtk::Box::new(ve, 8);
        let eq_list = gtk::Box::new(ve, 4);
        let eq_title = gtk::Label::new(None); eq_title.set_markup("<b>Equipped</b>"); eq_title.set_xalign(0.0);
        let bag_head = gtk::Label::new(None); bag_head.set_xalign(0.0); bag_head.set_hexpand(true);
        let sell_junk = gtk::Button::with_label("💰 Sell junk (Poor–Uncommon)");
        let bag_row = gtk::Box::new(ho, 8);
        bag_row.append(&bag_head); bag_row.append(&sell_junk);
        let bag_list = gtk::Box::new(ve, 4);
        let guide = gtk::Expander::new(Some("Quality guide — where items come from"));
        let guide_lbl = gtk::Label::new(Some(QUALITY_GUIDE));
        guide_lbl.set_xalign(0.0); guide_lbl.set_wrap(true);
        guide.set_child(Some(&guide_lbl));
        let eq_inner = gtk::Box::new(ve, 8);
        pad(&eq_inner, 10);
        eq_inner.append(&eq_sum); eq_inner.append(&guide); eq_inner.append(&eq_paperdoll); eq_inner.append(&eq_title);
        eq_inner.append(&eq_list); eq_inner.append(&bag_row); eq_inner.append(&bag_list);
        let eq_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&eq_inner).build();

        // ----- store tab
        let store_gold = gtk::Label::new(None); store_gold.set_xalign(0.0);
        let store_hint = gtk::Label::new(Some(
            "Stock is generated around your level and usable by your class. Bought gear goes to your bag — equip it from the Equipment tab. \
             ▲ marks an upgrade over what you wear.",
        ));
        store_hint.set_xalign(0.0); store_hint.set_wrap(true); store_hint.add_css_class("dim-label");
        let pot_title = gtk::Label::new(None); pot_title.set_markup("<b>Potions</b>"); pot_title.set_xalign(0.0);
        let store_pots = gtk::Box::new(ve, 4);
        let st_title = gtk::Label::new(None); st_title.set_markup("<b>Equipment for sale</b>");
        st_title.set_xalign(0.0); st_title.set_hexpand(true);
        let restock = gtk::Button::with_label("🔄 Restock");
        let st_row = gtk::Box::new(ho, 8);
        st_row.append(&st_title); st_row.append(&restock);
        let store_list = gtk::Box::new(ve, 4);
        let store_inner = gtk::Box::new(ve, 8);
        pad(&store_inner, 10);
        store_inner.append(&store_gold); store_inner.append(&store_hint); store_inner.append(&pot_title);
        store_inner.append(&store_pots); store_inner.append(&st_row); store_inner.append(&store_list);
        let store_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&store_inner).build();

        // ----- honor store tab
        let honor_lbl = gtk::Label::new(None); honor_lbl.set_xalign(0.0);
        let honor_hint = gtk::Label::new(Some(
            "Earn honor by winning Realm PvP duels (beating the opposite faction ★ gives +1 honor and +50% XP). \
             Stock is Rare gear below level 40 and Epic gear from level 40, usable by your class. \
             Bought gear goes to your bag. ▲ marks an upgrade over what you wear.",
        ));
        honor_hint.set_xalign(0.0); honor_hint.set_wrap(true); honor_hint.add_css_class("dim-label");
        let honor_list = gtk::Box::new(ve, 4);
        let honor_inner = gtk::Box::new(ve, 8);
        pad(&honor_inner, 10);
        honor_inner.append(&honor_lbl); honor_inner.append(&honor_hint); honor_inner.append(&honor_list);
        let honor_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&honor_inner).build();

        // ----- blacksmith tab
        let smith_lbl = gtk::Label::new(None); smith_lbl.set_xalign(0.0); smith_lbl.set_hexpand(true);
        let smith_all = gtk::Button::with_label("🔨 Repair all");
        let smith_hint = gtk::Label::new(Some(
            "Equipment starts at 100/100 durability and loses 10 every time you hit 0 HP. \
            Repair cost scales with item level and missing durability."));
        smith_hint.set_xalign(0.0); smith_hint.set_wrap(true); smith_hint.add_css_class("dim-label");
        let smith_list = gtk::Box::new(ve, 4);
        let smith_top = gtk::Box::new(ho, 8);
        smith_top.append(&smith_lbl); smith_top.append(&smith_all);
        let smith_inner = gtk::Box::new(ve, 8);
        pad(&smith_inner, 10);
        smith_inner.append(&smith_top); smith_inner.append(&smith_hint); smith_inner.append(&smith_list);
        let smith_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&smith_inner).build();

        // ----- logs tab
        let log_filter = gtk::DropDown::from_strings(&LOG_FILTERS);
        let log_clear = gtk::Button::with_label("Clear logs");
        let spacer = gtk::Box::new(ho, 0); spacer.set_hexpand(true);
        let lrow = gtk::Box::new(ho, 8);
        lrow.append(&gtk::Label::new(Some("Filter:")));
        lrow.append(&log_filter); lrow.append(&spacer); lrow.append(&log_clear);
        let log_view = gtk::TextView::new();
        log_view.set_editable(false); log_view.set_cursor_visible(false);
        log_view.set_monospace(true); log_view.set_wrap_mode(gtk::WrapMode::WordChar);
        pad(&log_view, 8);
        let lscroll = gtk::ScrolledWindow::builder().vexpand(true).child(&log_view).build();
        let logs_page = gtk::Box::new(ve, 8);
        pad(&logs_page, 10);
        logs_page.append(&lrow); logs_page.append(&lscroll);

        // ----- character tab (stats + abilities)
        let sheet = gtk::Label::new(None); sheet.set_xalign(0.0); sheet.set_yalign(0.0); sheet.set_wrap(true);
        let ab_list = gtk::Box::new(ve, 6);
        let sheet_box = gtk::Box::new(ve, 10);
        pad(&sheet_box, 10);
        sheet_box.append(&sheet); sheet_box.append(&ab_list);
        let sheet_scroll = gtk::ScrolledWindow::builder().vexpand(true).child(&sheet_box).build();

        // Page order matters: dashboard::refresh_page maps these indices.
        let nb = gtk::Notebook::new();
        nb.set_vexpand(true);
        nb.append_page(&quests_page, Some(&gtk::Label::new(Some("📜 Quests"))));      // 0
        nb.append_page(&zones_page, Some(&gtk::Label::new(Some("🗺 Zones"))));        // 1
        nb.append_page(&pvp_page, Some(&gtk::Label::new(Some("⚔ Realm PvP"))));      // 2
        nb.append_page(&eq_scroll, Some(&gtk::Label::new(Some("🎒 Equipment"))));     // 3
        nb.append_page(&store_scroll, Some(&gtk::Label::new(Some("🏪 Store"))));      // 4
        nb.append_page(&honor_scroll, Some(&gtk::Label::new(Some("🎖 Honor Store")))); // 5
        nb.append_page(&smith_scroll, Some(&gtk::Label::new(Some("🔨 Blacksmith")))); // 6
        nb.append_page(&sheet_scroll, Some(&gtk::Label::new(Some("🧙 Character"))));  // 7
        nb.append_page(&logs_page, Some(&gtk::Label::new(Some("📋 Logs"))));          // 8

        let game = gtk::Box::new(ve, 10);
        pad(&game, 12);
        game.append(&head); game.append(&xp_bar); game.append(&bars); game.append(&potrow);
        game.append(&stats); game.append(&nb);

        let stack = gtk::Stack::new();
        stack.add_named(&select, Some("select"));
        stack.add_named(&game, Some("game"));
        let win = gtk::ApplicationWindow::builder().application(app)
            .title(APP_TITLE).default_width(1000).default_height(900).build();
        win.set_child(Some(&stack));

        let save = load();

        let ui = Rc::new(Ui {
            save: RefCell::new(save), active: Cell::new(None), confirm_del: Cell::new(None),
            realms: make_realms(), opps: RefCell::new(vec![]),
            store: RefCell::new(vec![]), store_key: Cell::new((usize::MAX, 0)),
            honor_stock: RefCell::new(vec![]), honor_key: Cell::new((usize::MAX, 0)),
            dirty: Cell::new(u16::MAX), save_pending: Cell::new(false),
            stack, nb, sel_list, title, xp_bar, stats, hp_bar, mana_bar, sta_bar, pot_btn,
            giver, entry, goal, qkind, quest_level, tier, cat, chain, qlist, status,
            zsearch, zkind, zcount, zlist, pvp_head, pvp_faction, pvp_result, pvp_list, log_filter, log_view, sheet, ab_list,
            eq_sum, eq_paperdoll, eq_list, bag_head, bag_list, store_gold, store_pots, store_list,
            honor_lbl, honor_list, smith_lbl, smith_all, smith_list,
        });

        { let u = ui.clone(); sel_new.connect_clicked(move |_| u.show_create()); }
        { let u = ui.clone(); add_btn.connect_clicked(move |_| u.add_quest()); }
        { let u = ui.clone(); ui.entry.connect_activate(move |_| u.add_quest()); }
        {
            let u = ui.clone();
            save_btn.connect_clicked(move |_| {
                u.finish(vec![m("System", "💾 Character saved.")]);
                persist(&u.save.borrow());
            });
        }
        { let u = ui.clone(); switch_btn.connect_clicked(move |_| u.show_select()); }
        { let u = ui.clone(); pvp_refresh.connect_clicked(move |_| u.new_opponents()); }
        { let u = ui.clone(); let faction_filter = ui.pvp_faction.clone(); faction_filter.connect_selected_notify(move |_| u.new_opponents()); }
        { let u = ui.clone(); log_clear.connect_clicked(move |_| u.clear_logs()); }
        { let u = ui.clone(); sell_junk.connect_clicked(move |_| u.sell_junk()); }
        { let u = ui.clone(); restock.connect_clicked(move |_| u.restock()); }
        { let u = ui.clone(); ui.zsearch.connect_changed(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.zkind.connect_selected_notify(move |_| u.refresh_zones()); }
        { let u = ui.clone(); ui.log_filter.connect_selected_notify(move |_| u.refresh_logs()); }
        { let u = ui.clone(); ui.smith_all.connect_clicked(move |_| u.repair_all()); }
        { let u = ui.clone(); ui.nb.connect_switch_page(move |_, _, p| u.refresh_page(p)); }
        for (k, b) in ui.pot_btn.iter().enumerate() {
            let u = ui.clone();
            b.connect_clicked(move |_| u.drink(k));
        }
        // flush any debounced save when the window closes
        {
            let u = ui.clone();
            win.connect_close_request(move |_| {
                persist(&u.save.borrow());
                glib::Propagation::Proceed
            });
        }

        win.present();
        ui
    }

    pub(crate) fn start(self: &Rc<Self>) {
        self.stack.add_named(&create_screen(self), Some("create"));
        if self.save.borrow().heroes.is_empty() { self.show_create(); } else { self.show_select(); }
    }

    pub(crate) fn show_create(self: &Rc<Self>) { self.stack.set_visible_child_name("create"); }

    pub(crate) fn show_select(self: &Rc<Self>) {
        persist(&self.save.borrow());
        self.active.set(None);
        self.confirm_del.set(None);
        self.store_key.set((usize::MAX, 0));
        self.honor_key.set((usize::MAX, 0));
        self.refresh_select();
        self.stack.set_visible_child_name("select");
    }
}