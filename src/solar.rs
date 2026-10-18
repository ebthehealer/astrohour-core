//! Sunrise and sunset calculation via solunatus (NOAA-accurate).

use chrono::{Datelike, DateTime, Local, NaiveDate, TimeZone};
use solunatus::prelude::*;
use solunatus::astro::Location as SolLocation;
pub use solunatus::prelude::{calculate_moonrise, calculate_moonset};

use crate::types::Location;

/// Calculate sunrise and sunset for a given date and location.
///
/// Returns local `DateTime<Local>` for both events.
///
/// # Example
/// ```
/// use astrohour_core::{Location, solar::sunrise_sunset};
/// use chrono::NaiveDate;
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 }; // New York
/// let date = NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(); // summer solstice
/// let (rise, set) = sunrise_sunset(date, loc);
/// assert!(rise < set);
/// ```
pub fn sunrise_sunset(date: NaiveDate, loc: Location) -> (DateTime<Local>, DateTime<Local>) {
    let sol_loc = SolLocation::new(loc.lat, loc.lon)
        .expect("invalid lat/lon");

    // Build a DateTime<Local> at noon on the requested date so solunatus
    // anchors its search to the right calendar day in local time.
    let noon_local = Local
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
        .single()
        .expect("invalid date");

    let rise = calculate_sunrise(&sol_loc, &noon_local)
        .expect("no sunrise (polar day/night not yet handled)");
    let set = calculate_sunset(&sol_loc, &noon_local)
        .expect("no sunset (polar day/night not yet handled)");

    (rise, set)
}
