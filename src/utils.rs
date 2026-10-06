
use gtk::glib;
use crate::{config::*, data::items::*};

// ---------- helpers ----------
pub(crate) type Msg = (&'static str, String);
pub(crate) fn m(c: &'static str, s: impl Into<String>) -> Msg { (c, s.into()) }
pub(crate) fn rnd(n: u32) -> u32 { glib::random_int_range(0, n as i32) as u32 }
pub(crate) fn pick<T>(a: &[T]) -> &T { &a[rnd(a.len() as u32) as usize] }
pub(crate) fn ps(a: &[&'static str]) -> &'static str { a[rnd(a.len() as u32) as usize] }
pub(crate) fn esc(s: &str) -> String { glib::markup_escape_text(s).to_string() }
pub(crate) fn now() -> String {
    glib::DateTime::now_local().ok()
        .and_then(|d| d.format("%Y-%m-%d %H:%M:%S").ok())
        .map(|s| s.to_string()).unwrap_or_default()
}
pub(crate) fn commas(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out
}
pub(crate) fn player_name() -> String { format!("{}{}{}", pick(&SYL1), pick(&SYL2), pick(&SYL3)) }
pub(crate) fn npc_name() -> String { format!("{}{}{}", ps(&SYL1), ps(&SYL2), ps(NPC_END)) }
pub(crate) fn boss_name() -> String { format!("{}{}{}", ps(&SYL1), ps(&SYL2), ps(BOSS_END)) }
pub(crate) fn elven_name() -> String { format!("{}'{}", ps(ELF_A), ps(ELF_B)) }
pub(crate) fn compound_name() -> String { format!("{}{}", ps(CMP_A), ps(CMP_B)) }

