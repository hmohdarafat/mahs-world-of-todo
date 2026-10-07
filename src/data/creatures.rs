//! Creature types and the creature pool for each type.

pub(crate) const CTYPES: [&str; 11] = [
    "Humanoid", "Beast", "Demon", "Undead", "Dragonkin", "Elemental",
    "Giant", "Mechanical", "Aberration", "Critter", "Uncategorized",
];
pub(crate) const CTYPE_INFO: [&str; 11] = [
    "Mortals: player races, ogres, troggs, gnolls. Susceptible to Polymorph and Sap.",
    "Wild and tamable animals. Susceptible to Scare Beast and Hibernate.",
    "Burning Legion / Twisting Nether creatures. Susceptible to Banish, Enslave Demon, Turn Evil.",
    "Reanimated beings and the Scourge. Susceptible to Turn Evil, Shackle Undead, Wake of Ashes.",
    "Dragons, whelps and drakes. Susceptible to Hibernate.",
    "Manifestations of nature and magic. Susceptible to Banish.",
    "Massive entities such as sea and mountain giants.",
    "Machines and constructs. Can be disabled with Disable Trap.",
    "Twisted Void creations and Old God minions.",
    "Harmless neutral ambient creatures.",
    "Rare units without a specific tag.",
];
pub(crate) const CREATURES: [&[&str]; 11] = [
    &["Defias Bandits", "Gnolls", "Murlocs", "Ogres", "Troggs", "Forest Trolls", "Kobolds", "Satyrs"],
    &["Timber Wolves", "Giant Spiders", "Wild Boars", "Dire Bears", "Raptors", "Scorpids", "Bats", "Crocolisks"],
    &["Imps", "Felhounds", "Doomguards", "Voidwalkers", "Succubi", "Fel Stalkers"],
    &["Skeletons", "Ghouls", "Wraiths", "Zombies", "Banshees", "Scourge Footmen"],
    &["Red Whelps", "Black Drakes", "Twilight Drakes", "Proto-Drakes", "Dragonspawn"],
    &["Fire Elementals", "Earth Elementals", "Water Elementals", "Air Elementals", "Living Flames", "Storm Spirits"],
    &["Hill Giants", "Sea Giants", "Mountain Giants", "Frost Giants", "Stone Giants"],
    &["Shredders", "Harvest Golems", "Mechanostriders", "Clockwork Gnomes", "Iron Constructs", "Bombots"],
    &["Faceless Ones", "Void Spawn", "Old God Minions", "Qiraji Drones", "Tentacled Horrors"],
    &["Rats", "Squirrels", "Rabbits", "Frogs", "Cockroaches", "Seagulls"],
    &["Wandering Wisps", "Mysterious Shades", "Unmarked Oddities", "Stray Totems"],
];

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Matchup { Easy, Average, Hard }
impl Matchup {
    pub(crate) fn color(self) -> &'static str {
        match self { Self::Easy => "#3fb950", Self::Average => "#d29922", Self::Hard => "#e0453a" }
    }
    pub(crate) fn label(self) -> &'static str {
        match self { Self::Easy => "Advantage (easy)", Self::Average => "Average", Self::Hard => "Disadvantage (hard)" }
    }
}

// classes with an advantage / disadvantage per creature type (same order as CTYPES)
pub(crate) const EASY: [&[&str]; 11] = [
    &["Mage", "Rogue", "Hunter", "Monk"],
    &["Hunter", "Druid", "Warrior"],
    &["Warlock", "Paladin", "Demon Hunter"],
    &["Paladin", "Priest", "Death Knight"],
    &["Druid", "Evoker"],
    &["Warlock", "Shaman"],
    &["Hunter", "Mage"],
    &["Hunter", "Rogue"],
    &["Priest", "Paladin"],
    &[], // Critter: easy for everyone (handled in matchup())
    &[],
];
pub(crate) const HARD: [&[&str]; 11] = [
    &["Death Knight", "Evoker"],
    &["Mage", "Priest", "Demon Hunter"],
    &["Druid", "Monk"],
    &["Rogue", "Hunter"],
    &["Mage", "Warlock", "Shaman"],
    &["Rogue", "Warrior", "Monk"],
    &["Rogue", "Druid", "Paladin"],
    &["Priest", "Druid"],
    &["Mage", "Warlock"],
    &[],
    &[],
];

pub(crate) fn matchup(class: &str, t: usize) -> Matchup {
    let t = t.min(CTYPES.len() - 1);
    if t == 9 || EASY[t].contains(&class) { Matchup::Easy }
    else if HARD[t].contains(&class) { Matchup::Hard }
    else { Matchup::Average }
}

pub(crate) fn matchup_markup(class: &str) -> String {
    let mut g: [Vec<&str>; 3] = [vec![], vec![], vec![]];
    for (t, name) in CTYPES.iter().enumerate() {
        let i = match matchup(class, t) { Matchup::Easy => 0, Matchup::Average => 1, Matchup::Hard => 2 };
        g[i].push(*name);
    }
    let line = |m: Matchup, v: &Vec<&str>| format!(
        "<span foreground='{}'><b>{}</b>: {}</span>", m.color(), m.label(),
        if v.is_empty() { "—".to_string() } else { v.join(", ") });
    format!("<b>Creature matchups</b>\n{}\n{}\n{}",
        line(Matchup::Easy, &g[0]), line(Matchup::Average, &g[1]), line(Matchup::Hard, &g[2]))
}