//! Startup identity art (printed before standard log dumps).

/// Block logo + product line. Keep ASCII-only for Windows consoles.
pub const LOGO: &str = r#"
 ______ _______ _    _ ______          _   _
|  ____|__   __| |  | |  ____|   /\   | \ | |
| |__     | |  | |__| | |__     /  \  |  \| |
|  __|    | |  |  __  |  __|   / /\ \ | . ` |
| |____   | |  | |  | | |____ / ____ \| |\  |
|______|  |_|  |_|  |_|______/_/    \_\_| \_|
"#;

/// One-line product identity under the logo.
pub const PRODUCT: &str = "Ethean Lean Consensus Client";

/// Operator-facing slogan (English).
pub const SLOGAN: &str = "Lean Consensus · hash-based sigs · faster finality";
