//! Status bar text formatting.

use crate::types::{DisplayMode, Location};
use crate::hours::{current_hour, minutes_remaining};

/// Generate the status bar string for the given location and display mode.
///
/// # Outputs by mode
/// - `IconOnly`  → `"♃"`
/// - `Short`     → `"♃ Jupiter Hour"`
/// - `Full`      → `"♃ Jupiter Hour (Day) · 34m"`
///
/// # Example
/// ```no_run
/// use astrohour_core::{Location, DisplayMode, display::status_bar_text};
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 };
/// let s = status_bar_text(loc, DisplayMode::Short);
/// // e.g. "♃ Jupiter Hour"
/// assert!(s.contains("Hour"));
/// ```
pub fn status_bar_text(loc: Location, mode: DisplayMode) -> String {
    let hour = current_hour(loc);
    let symbol = hour.planet.symbol();
    let name = hour.planet.name();

    match mode {
        DisplayMode::IconOnly => symbol.to_string(),
        DisplayMode::Short => format!("{symbol} {name} Hour"),
        DisplayMode::Full => {
            let mins = minutes_remaining(loc);
            let day_night = if hour.is_day { "Day" } else { "Night" };
            format!("{symbol} {name} Hour ({day_night}) \u{00b7} {:.0}m", mins)
        }
    }
}
