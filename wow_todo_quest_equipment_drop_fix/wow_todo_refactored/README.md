# MAH's World of Todo — Refactored Source

This is the refactored `src/` tree for the existing `mah-wow-todo` Cargo project.

## Recent fixes

- Removed duplicate `Msg` type alias.
- Restored cross-module ability/class visibility and lookup imports.
- Restored shared name-generation constants used by `utils.rs`.
- Fixed `gtk::glib` imports in `main.rs` and `persistence.rs`.
- Fixed shared GTK helper imports (`pad`, `set_bar`, `set_options`) in UI submodules.
- Fixed the `Tier` import to use the domain model.
- Fixed the PvP faction dropdown borrow-after-move issue.
- Keeps the requested quest Kill/Gather selector, zone sorting, PvP faction filtering/level sorting, and Character-page abilities layout.

## Important

This bundle intentionally does **not** replace your existing `Cargo.toml`, `Cargo.lock`, or assets/data files. Replace your project's `src/` directory with this one, then run:

```bash
cargo fmt
cargo check
cargo run --release
```


## Recent gameplay rules

- Looted equipment is never auto-equipped. New equipment goes to the bag first; equip it manually from the Equipment tab.
- Quest creation includes a Quest Lv selector. The available levels are limited to five levels below through five levels above the current character level, clamped to levels 1-90.
- Performing a quest action consumes HP, mana, and stamina based on the quest level relative to the character level. Higher-level quests apply greater resource pressure; lower-level quests apply less.
