use gtk::prelude::*;

pub(crate) fn pad(w: &impl IsA<gtk::Widget>, n: i32) {
    w.set_margin_top(n); w.set_margin_bottom(n); w.set_margin_start(n); w.set_margin_end(n);
}
pub(crate) fn labeled(text: &str, w: &impl IsA<gtk::Widget>) -> gtk::Box {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.0);
    l.add_css_class("dim-label");
    b.append(&l); b.append(w);
    b
}
/// Remove every child of a box.
pub(crate) fn clear(b: &gtk::Box) {
    while let Some(c) = b.first_child() { b.remove(&c); }
}
/// The dropdown's current value from a parallel slice (index clamped).
pub(crate) fn sel<T: Copy>(dd: &gtk::DropDown, a: &[T]) -> T {
    a[(dd.selected() as usize).min(a.len() - 1)]
}
pub(crate) fn set_options(dd: &gtk::DropDown, items: &[String]) {
    let same = dd.model().and_then(|mo| mo.downcast::<gtk::StringList>().ok()).is_some_and(|sl| {
        sl.n_items() as usize == items.len()
            && items.iter().enumerate()
                .all(|(i, s)| sl.string(i as u32).is_some_and(|g| g.as_str() == s.as_str()))
    });
    if same { return; }
    let sel = dd.selected();
    let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    dd.set_model(Some(&gtk::StringList::new(&refs)));
    dd.set_selected(if sel == gtk::INVALID_LIST_POSITION { 0 } else { sel.min(items.len() as u32 - 1) });
}
pub(crate) fn bar(class: &str) -> gtk::ProgressBar {
    let b = gtk::ProgressBar::new();
    b.set_show_text(true);
    b.set_hexpand(true);
    b.add_css_class(class);
    b
}
pub(crate) fn set_bar(b: &gtk::ProgressBar, label: &str, cur: u32, max: u32) {
    b.set_fraction((cur as f64 / max.max(1) as f64).min(1.0));
    b.set_text(Some(&format!("{label} {cur} / {max}")));
}