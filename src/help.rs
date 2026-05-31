// pub const DASH: &str = "\u{2014}";
pub const MAIN_SCREEN: &str = "(q) quit /(s) save withuot quit /\
(l) load / (n) new / (e) edit / (d) delete /\
(↑↓) select / (←→) navigate";
pub const EDIT_SCREEN: &str =
    "(Esc) cancel / (Tab) switch fields / (↑↓) change type / (Enter on type field) save";
pub const SAVE_EXIT: &str =
    "(Enter/s) save / (q) discard / (Esc) cancel \u{2014} type filename or $project_alias";

pub const QUIT_EXIT: &str =
    "(Enter/s) save & quit / (q) quit now / (Esc) cancel \u{2014} type filename or $project_alias";

pub const LOADING: &str =
"(Enter) load / (Esc) cancel \u{2014} type filename or $project_alias";
pub const DELETING: &str = "Type 'y' then (Enter) to confirm / (Esc) cancel";
