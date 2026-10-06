
//! Ability definitions and class/spec ability lookup helpers.

use crate::{data::classes::CLASSES, model::*};
use crate::model::{K::{Absorb, Buff, Dmg, Dot, Guard, Heal, Kick, Leech, Stun, Util}, R::{Free, Mana, Stam}};

const ALL: &[&str] = &[];

#[allow(clippy::too_many_arguments)]
const fn ab(l: u32, n: &'static str, k: K, p: u32, cd: u32, r: R, c: u32, sp: &'static [&'static str]) -> Ab {
    Ab { l, n, k, p, cd, r, c, sp }
}
const fn ut(l: u32, n: &'static str) -> Ab {
    Ab { l, n, k: Util, p: 0, cd: 0, r: Free, c: 0, sp: ALL }
}

pub(crate) const AB_DK: &[Ab] = &[
    ab(1, "Death Strike", Leech, 85, 1, Stam, 12, ALL),
    ab(1, "Rune Strike", Dmg, 95, 1, Stam, 10, ALL),
    ab(1, "Marrowrend", Dmg, 110, 2, Stam, 14, &["Blood"]),
    ab(1, "Blood Boil", Dot, 150, 3, Stam, 14, &["Blood", "Unholy"]),
    ab(1, "Howling Blast", Dmg, 120, 2, Mana, 12, &["Frost"]),
    ab(2, "Death Grip", Stun, 35, 6, Stam, 10, ALL),
    ab(3, "Mind Freeze", Kick, 40, 4, Stam, 8, ALL),
    ab(4, "Raise Dead", Dot, 135, 5, Mana, 15, ALL),
    ab(5, "Anti-Magic Shell", Absorb, 30, 6, Mana, 12, ALL),
    ab(7, "Icebound Fortitude", Guard, 40, 8, Stam, 10, ALL),
    ab(10, "Death and Decay", Dot, 180, 4, Mana, 16, ALL),
    ut(10, "Path of Frost"),
    ut(13, "Death's Advance"),
    ab(19, "Empower Rune Weapon", Buff, 30, 10, Free, 0, ALL),
];

pub(crate) const AB_DH: &[Ab] = &[
    ab(1, "Demon's Bite", Dmg, 90, 1, Stam, 8, &["Havoc", "Devourer"]),
    ab(1, "Shear", Dmg, 90, 1, Stam, 8, &["Vengeance"]),
    ab(1, "Chaos Strike", Dmg, 120, 2, Stam, 14, &["Havoc", "Devourer"]),
    ab(1, "Soul Cleave", Leech, 125, 2, Stam, 16, &["Vengeance"]),
    ab(2, "Fel Rush", Dmg, 75, 3, Stam, 8, &["Havoc", "Devourer"]),
    ab(2, "Infernal Strike", Dmg, 75, 3, Stam, 8, &["Vengeance"]),
    ab(3, "Disrupt", Kick, 30, 4, Stam, 6, ALL),
    ab(4, "Immolation Aura", Dot, 120, 4, Mana, 10, ALL),
    ab(5, "Blur", Guard, 35, 7, Stam, 10, &["Havoc", "Vengeance"]),
    ab(5, "Fiendish Overhaul", Guard, 35, 7, Stam, 10, &["Devourer"]),
    ab(8, "Consume Magic", Dmg, 60, 3, Mana, 8, ALL),
    ab(10, "Metamorphosis", Buff, 40, 10, Free, 0, ALL),
    ut(10, "Glide"),
    ut(12, "Spectral Sight"),
    ab(19, "Chaos Nova", Stun, 70, 7, Mana, 14, ALL),
];

pub(crate) const AB_DRUID: &[Ab] = &[
    ab(1, "Wrath", Dmg, 100, 1, Mana, 6, ALL),
    ab(1, "Moonfire", Dot, 130, 2, Mana, 8, ALL),
    ab(2, "Regrowth", Heal, 22, 2, Mana, 12, ALL),
    ab(2, "Rejuvenation", Heal, 26, 2, Mana, 10, ALL),
    ab(3, "Bear Form", Guard, 25, 8, Stam, 5, ALL),
    ab(3, "Growl", Kick, 20, 5, Stam, 5, ALL),
    ab(3, "Mangle", Dmg, 105, 1, Stam, 12, &["Feral", "Guardian"]),
    ab(4, "Cat Form", Buff, 15, 8, Stam, 5, ALL),
    ab(4, "Rake", Dot, 140, 2, Stam, 10, &["Feral"]),
    ab(4, "Shred", Dmg, 115, 1, Stam, 12, &["Feral"]),
    ut(6, "Travel Form"),
    ut(6, "Dash"),
    ab(8, "Barkskin", Guard, 30, 8, Mana, 8, ALL),
    ab(10, "Entangling Roots", Stun, 50, 7, Mana, 10, ALL),
    ab(10, "Mark of the Wild", Buff, 10, 12, Mana, 8, ALL),
    ut(10, "Teleport: Moonglade"),
    ut(13, "Revive / Rebirth"),
    ab(19, "Stampeding Roar", Buff, 20, 10, Stam, 5, ALL),
];

pub(crate) const AB_EVOKER: &[Ab] = &[
    ab(1, "Living Flame", Dmg, 105, 1, Mana, 6, &["Augmentation", "Devastation"]),
    ab(1, "Living Flame", Heal, 20, 1, Mana, 8, &["Preservation"]),
    ab(1, "Azure Strike", Dmg, 80, 1, Mana, 4, ALL),
    ab(2, "Emerald Blossom", Heal, 28, 3, Mana, 14, ALL),
    ab(3, "Disintegrate", Dot, 170, 3, Mana, 14, ALL),
    ab(4, "Fire Breath", Dmg, 160, 3, Mana, 16, ALL),
    ab(5, "Wing Buffeting", Stun, 40, 6, Mana, 8, ALL),
    ab(8, "Blessing of the Bronze", Buff, 12, 12, Mana, 8, ALL),
    ut(10, "Soar"),
    ut(10, "Skyriding"),
    ab(10, "Deep Breath", Dmg, 200, 6, Mana, 18, ALL),
    ut(13, "Rescue"),
    ab(19, "Time Dilation", Guard, 35, 8, Mana, 10, ALL),
];

pub(crate) const AB_HUNTER: &[Ab] = &[
    ab(1, "Arcane Shot", Dmg, 95, 1, Stam, 8, &["Marksmanship", "Survival"]),
    ab(1, "Cobra Shot", Dmg, 95, 1, Stam, 8, &["Beast Mastery"]),
    ab(1, "Auto Shot", Dmg, 85, 1, Free, 0, ALL),
    ab(2, "Steady Shot", Dmg, 100, 1, Stam, 6, ALL),
    ab(3, "Kill Command", Dmg, 140, 2, Stam, 12, &["Beast Mastery"]),
    ab(3, "Aimed Shot", Dmg, 170, 3, Stam, 16, &["Marksmanship"]),
    ab(3, "Raptor Strike", Dmg, 120, 1, Stam, 10, &["Survival"]),
    ut(4, "Call Pet"),
    ut(4, "Revive Pet"),
    ab(4, "Mend Pet", Heal, 12, 4, Mana, 10, ALL),
    ut(5, "Disengage"),
    ab(7, "Wing Clip", Stun, 35, 6, Stam, 6, &["Survival"]),
    ab(7, "Freezing Trap", Stun, 25, 8, Mana, 10, &["Beast Mastery", "Marksmanship"]),
    ab(8, "Exhilaration", Heal, 30, 8, Free, 0, ALL),
    ab(10, "Aspect of the Turtle", Guard, 50, 12, Free, 0, ALL),
    ut(10, "Tame Beast"),
    ut(13, "Feign Death"),
    ab(19, "Tar Trap", Dot, 90, 6, Mana, 10, ALL),
];

pub(crate) const AB_MAGE: &[Ab] = &[
    ab(1, "Frostbolt", Dmg, 110, 1, Mana, 8, ALL),
    ab(2, "Fire Blast", Dmg, 120, 2, Mana, 8, ALL),
    ab(3, "Frost Nova", Stun, 55, 6, Mana, 10, ALL),
    ut(4, "Blink"),
    ut(5, "Conjure Refreshment"),
    ab(6, "Arcane Explosion", Dmg, 100, 1, Mana, 9, ALL),
    ab(7, "Counterspell", Kick, 40, 5, Mana, 6, ALL),
    ab(8, "Arcane Intellect", Buff, 10, 12, Mana, 6, ALL),
    ut(9, "Slow Fall"),
    ab(10, "Polymorph", Stun, 0, 8, Mana, 10, ALL),
    ab(16, "Invisibility", Guard, 40, 10, Mana, 8, ALL),
    ab(18, "Cone of Cold", Dmg, 150, 3, Mana, 14, ALL),
    ut(21, "Teleport"),
    ut(24, "Portal"),
    ab(49, "Time Warp", Buff, 30, 14, Mana, 10, ALL),
];

pub(crate) const AB_MONK: &[Ab] = &[
    ab(1, "Tiger Palm", Dmg, 90, 1, Stam, 8, ALL),
    ab(2, "Blackout Kick", Dmg, 115, 1, Stam, 12, ALL),
    ut(3, "Roll"),
    ab(4, "Vivify", Heal, 24, 2, Mana, 12, ALL),
    ab(5, "Touch of Death", Dmg, 220, 8, Stam, 16, ALL),
    ab(7, "Spear Hand Strike", Kick, 35, 4, Stam, 6, ALL),
    ab(8, "Fortifying Brew", Guard, 35, 9, Stam, 10, ALL),
    ab(10, "Crackling Jade Lightning", Dot, 130, 2, Mana, 10, ALL),
    ut(10, "Mystic Touch"),
    ut(13, "Transcendence"),
    ab(19, "Leg Sweep", Stun, 50, 7, Stam, 12, ALL),
];

pub(crate) const AB_PALADIN: &[Ab] = &[
    ab(1, "Crusader Strike", Dmg, 100, 1, Stam, 8, ALL),
    ab(1, "Judgment", Dmg, 125, 2, Mana, 8, ALL),
    ab(2, "Flash of Light", Heal, 22, 2, Mana, 12, ALL),
    ab(3, "Shield of the Righteous", Dmg, 130, 2, Stam, 12, &["Protection"]),
    ab(3, "Word of Glory", Heal, 30, 3, Mana, 10, &["Holy", "Retribution"]),
    ab(4, "Consecration", Dot, 140, 4, Mana, 12, ALL),
    ab(5, "Hand of Reckoning", Kick, 25, 5, Stam, 5, ALL),
    ab(7, "Rebuke", Kick, 40, 4, Stam, 6, ALL),
    ab(8, "Divine Protection", Guard, 30, 8, Mana, 8, ALL),
    ab(10, "Divine Shield", Guard, 80, 14, Free, 0, ALL),
    ab(10, "Lay on Hands", Heal, 70, 16, Free, 0, ALL),
    ab(10, "Devotion Aura", Guard, 12, 12, Mana, 6, ALL),
    ab(13, "Blessing of Protection", Absorb, 25, 9, Mana, 10, ALL),
    ab(19, "Hammer of Justice", Stun, 60, 7, Mana, 10, ALL),
];

pub(crate) const AB_PRIEST: &[Ab] = &[
    ab(1, "Smite", Dmg, 100, 1, Mana, 6, ALL),
    ab(1, "Shadow Word: Pain", Dot, 140, 2, Mana, 8, ALL),
    ab(2, "Flash Heal", Heal, 25, 2, Mana, 12, ALL),
    ab(3, "Power Word: Shield", Absorb, 22, 3, Mana, 10, ALL),
    ab(4, "Renew", Heal, 20, 2, Mana, 8, ALL),
    ab(5, "Mind Blast", Dmg, 150, 2, Mana, 12, ALL),
    ab(7, "Psychic Scream", Stun, 25, 8, Mana, 12, ALL),
    ab(8, "Desperate Prayer", Heal, 35, 8, Mana, 8, ALL),
    ab(10, "Power Word: Fortitude", Buff, 10, 12, Mana, 8, ALL),
    ut(10, "Leap of Faith"),
    ab(13, "Fade", Guard, 30, 8, Mana, 8, ALL),
    ut(19, "Mass Dispel"),
];

pub(crate) const AB_ROGUE: &[Ab] = &[
    ab(1, "Sinister Strike", Dmg, 100, 1, Stam, 8, ALL),
    ab(1, "Eviscerate", Dmg, 170, 3, Stam, 16, ALL),
    ab(2, "Stealth", Buff, 25, 10, Free, 0, ALL),
    ab(3, "Slice and Dice", Buff, 30, 8, Stam, 8, &["Assassination", "Outlaw"]),
    ab(3, "Ambush", Dmg, 160, 5, Stam, 10, &["Subtlety"]),
    ab(4, "Kick", Kick, 40, 4, Stam, 6, ALL),
    ab(5, "Kidney Shot", Stun, 50, 7, Stam, 12, ALL),
    ab(7, "Cheap Shot", Stun, 40, 8, Stam, 10, ALL),
    ab(8, "Crimson Vial", Heal, 18, 5, Stam, 8, ALL),
    ab(10, "Vanish", Guard, 60, 12, Free, 0, ALL),
    ut(10, "Sprint"),
    ab(13, "Cloak of Shadows", Guard, 40, 10, Free, 0, ALL),
    ab(19, "Blind", Stun, 0, 9, Stam, 8, ALL),
];

pub(crate) const AB_SHAMAN: &[Ab] = &[
    ab(1, "Lightning Bolt", Dmg, 105, 1, Mana, 6, ALL),
    ab(1, "Primal Strike", Dmg, 105, 1, Stam, 8, &["Enhancement"]),
    ab(2, "Healing Surge", Heal, 24, 2, Mana, 12, ALL),
    ab(3, "Flame Shock", Dot, 140, 2, Mana, 8, ALL),
    ab(4, "Lava Burst", Dmg, 150, 3, Mana, 12, &["Elemental"]),
    ab(4, "Chain Lightning", Dmg, 130, 2, Mana, 12, &["Enhancement", "Restoration"]),
    ut(5, "Ghost Wolf"),
    ab(7, "Wind Shear", Kick, 40, 4, Mana, 6, ALL),
    ab(8, "Astral Shift", Guard, 35, 9, Free, 0, ALL),
    ab(10, "Skyfury", Buff, 12, 12, Mana, 8, ALL),
    ab(10, "Bloodlust", Buff, 35, 14, Free, 0, ALL), // shown as Heroism for Alliance
    ab(13, "Capacitor Totem", Stun, 30, 9, Mana, 10, ALL),
    ut(19, "Earthbind Totem"),
];

pub(crate) const AB_WARLOCK: &[Ab] = &[
    ab(1, "Shadow Bolt", Dmg, 110, 1, Mana, 8, &["Affliction", "Demonology"]),
    ab(1, "Incinerate", Dmg, 115, 1, Mana, 8, &["Destruction"]),
    ab(1, "Curse of Agony", Dot, 150, 2, Mana, 8, ALL),
    ab(2, "Summon Imp", Dot, 110, 5, Mana, 10, ALL),
    ab(3, "Corruption", Dot, 145, 2, Mana, 8, &["Affliction"]),
    ab(3, "Drain Life", Leech, 120, 2, Mana, 10, &["Demonology", "Destruction"]),
    ab(4, "Create Healthstone", Heal, 25, 10, Free, 0, ALL),
    ab(5, "Fear", Stun, 0, 8, Mana, 10, ALL),
    ab(7, "Spell Lock", Kick, 40, 4, Mana, 6, ALL),
    ut(7, "Felhunter"),
    ab(8, "Unending Resolve", Guard, 35, 9, Free, 0, ALL),
    ab(10, "Summon Voidwalker", Guard, 20, 8, Mana, 10, ALL),
    ut(10, "Soulstone"),
    ut(13, "Ritual of Summoning"),
    ab(19, "Shadowfury", Stun, 70, 7, Mana, 14, ALL),
];

pub(crate) const AB_WARRIOR: &[Ab] = &[
    ab(1, "Slam", Dmg, 110, 1, Stam, 10, ALL),
    ab(1, "Charge", Stun, 40, 6, Free, 0, ALL),
    ab(2, "Shield Slam", Dmg, 140, 2, Stam, 12, &["Protection"]),
    ab(2, "Bloodthirst", Leech, 130, 2, Stam, 12, &["Fury"]),
    ab(2, "Mortal Strike", Dmg, 150, 3, Stam, 14, &["Arms"]),
    ab(3, "Taunt", Kick, 15, 5, Stam, 5, ALL),
    ab(3, "Victory Rush", Leech, 100, 3, Free, 0, ALL),
    ab(4, "Execute", Dmg, 190, 4, Stam, 16, ALL),
    ab(5, "Whirlwind", Dmg, 125, 2, Stam, 14, ALL),
    ab(7, "Pummel", Kick, 40, 4, Stam, 6, ALL),
    ab(8, "Shield Wall", Guard, 45, 10, Free, 0, &["Protection"]),
    ab(8, "Enraged Regeneration", Heal, 30, 9, Free, 0, &["Arms", "Fury"]),
    ab(10, "Battle Shout", Buff, 10, 12, Free, 0, ALL),
    ab(10, "Heroic Leap", Dmg, 90, 5, Stam, 8, ALL),
    ab(13, "Spell Reflection", Guard, 40, 9, Free, 0, ALL),
    ab(19, "Rallying Cry", Absorb, 20, 12, Free, 0, ALL),
];

pub(crate) fn spec_ok(a: &Ab, spec: &str) -> bool { a.sp.is_empty() || a.sp.iter().any(|s| *s == spec) }

/// Abilities this class/spec knows at `level`.
pub(crate) fn known(class: usize, spec: usize, level: u32) -> Vec<&'static Ab> {
    let list: &'static [Ab] = CLASSES[class].abilities;
    let sn = CLASSES[class].specs[spec].0;
    list.iter().filter(|a| a.l <= level && spec_ok(a, sn)).collect()
}

pub(crate) fn aname(a: &Ab, faction: &str) -> &'static str {
    if a.n == "Bloodlust" && faction == "Alliance" { "Heroism" } else { a.n }
}

pub(crate) fn kname(k: K) -> &'static str {
    match k {
        Dmg => "Damage", Dot => "Damage over time", Leech => "Drain", Heal => "Heal", Absorb => "Shield",
        Stun => "Stun", Kick => "Interrupt", Buff => "Buff", Guard => "Defensive", Util => "Utility",
    }
}

pub(crate) fn kcol(k: K) -> &'static str {
    match k {
        Dmg | Dot | Leech => "#e0453a",
        Heal | Absorb => "#3fb950",
        Stun | Kick => "#d29922",
        Buff | Guard => "#4a8fe7",
        Util => "#8b949e",
    }
}

pub(crate) fn ab_effect(a: &Ab, st: &Stats) -> String {
    let dmg = st.atk * a.p / 100;
    let eff = match a.k {
        Dmg => format!("Deals ~{dmg} damage"),
        Dot => format!("~{}/round for 3 rounds (~{} total)", dmg / 3, dmg / 3 * 3),
        Leech => format!("Deals ~{dmg} damage, heals you for ~{}", dmg * 40 / 100),
        Heal => format!("Heals ~{} HP", st.hp * a.p / 100 * st.hmul / 100),
        Absorb => format!("Absorbs ~{} damage", st.hp * a.p / 100),
        Stun => if a.p > 0 { format!("Deals ~{dmg} and stuns for 1 round") } else { "Incapacitates the target for 1 round".to_string() },
        Kick => format!("Deals ~{dmg} and interrupts (target's next action is a basic attack)"),
        Buff => format!("+{}% damage for 3 rounds", a.p),
        Guard => format!("-{}% damage taken for 3 rounds", a.p),
        Util => return "Utility — no combat effect".to_string(),
    };
    let cost = match a.r {
        Free => "free".to_string(),
        Mana => format!("{}% mana (~{})", a.c, st.mana * a.c / 100),
        Stam => format!("{}% stamina (~{})", a.c, st.sta * a.c / 100),
    };
    let cd = if a.cd >= 2 { format!(" · cooldown {} rounds", a.cd) } else { String::new() };
    format!("{eff} · cost {cost}{cd}")
}

