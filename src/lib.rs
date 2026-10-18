//! # astrohour-core
//!
//! Standalone astronomical calculation library for AstroHour.
//! No Tauri dependencies. Every public function is independently testable.
//!
//! ## Modules
//! - [`types`] — core data types (Planet, PlanetaryHour, MoonPhase, DaySchedule, etc.)
//! - [`solar`] — sunrise/sunset via solunatus
//! - [`hours`] — Chaldean planetary hour calculation
//! - [`lunar`] — moon phase, lunar day, elongation
//! - [`display`] — status bar text formatting

pub mod types;
pub mod solar;
pub mod hours;
pub mod lunar;
pub mod display;

// Re-export the main public API at crate root for convenience
pub use types::{
    Location, Planet, PlanetaryHour, MoonPhase, Phase, DaySchedule, DisplayMode,
};
pub use solar::sunrise_sunset;
pub use hours::{planetary_hours, current_hour, minutes_remaining, day_ruler};
pub use lunar::{sun_moon_elongation, lunar_day, moon_phase};
pub use display::status_bar_text;

use chrono::NaiveDate;

/// Compute the complete day schedule for a given date and location.
///
/// This is the primary entry point for the AstroHour app. It assembles
/// all 24 planetary hours, moon phase data, sunrise/sunset, and the
/// planetary day ruler into a single structure.
///
/// # Example
/// ```
/// use astrohour_core::{day_schedule, Location};
/// use chrono::NaiveDate;
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 }; // New York
/// let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
/// let schedule = day_schedule(date, loc);
/// assert_eq!(schedule.hours.len(), 24);
/// assert!(schedule.sunrise < schedule.sunset);
/// ```
pub fn day_schedule(date: NaiveDate, loc: Location) -> DaySchedule {
    use chrono::{Local, TimeZone, Datelike};
    use solunatus::astro::Location as SolLocation;

    let (sunrise, sunset) = solar::sunrise_sunset(date, loc);
    let ruler = hours::day_ruler(date);
    let all_hours = hours::planetary_hours(date, loc);
    let now_utc = chrono::Utc::now();
    let moon = lunar::moon_phase(now_utc);

    // Moonrise / moonset via solunatus (same crate used for solar)
    let sol_loc = SolLocation::new(loc.lat, loc.lon).expect("invalid lat/lon");
    let noon_local = Local
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
        .single()
        .expect("invalid date");
    let moonrise = solar::calculate_moonrise(&sol_loc, &noon_local);
    let moonset  = solar::calculate_moonset(&sol_loc, &noon_local);

    DaySchedule {
        date,
        location: loc,
        planetary_day: ruler,
        sunrise,
        sunset,
        moonrise,
        moonset,
        hours: all_hours,
        moon,
    }
}
