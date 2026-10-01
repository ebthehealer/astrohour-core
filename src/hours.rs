//! Planetary hour calculation using the Chaldean system.
//!
//! Algorithm:
//! 1. Get sunrise and sunset for the date via [`crate::solar::sunrise_sunset`].
//! 2. Divide day arc (sunrise→sunset) into 12 equal segments = 12 day hours.
//! 3. Get next sunrise (date+1) to find night arc (sunset→next sunrise).
//! 4. Divide night arc into 12 equal segments = 12 night hours.
//! 5. Hour 0 ruler = day ruler (weekday planet). Each subsequent hour
//!    advances one step in the Chaldean sequence (mod 7).

use chrono::{Local, NaiveDate, Datelike};
use crate::types::{Location, Planet, PlanetaryHour};
use crate::solar::sunrise_sunset;

/// Returns the traditional planetary ruler for the given weekday.
///
/// # Example
/// ```
/// use astrohour_core::{Planet, hours::day_ruler};
/// use chrono::NaiveDate;
///
/// let thursday = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(); // a Thursday
/// assert_eq!(day_ruler(thursday), Planet::Jupiter);
/// ```
pub fn day_ruler(date: NaiveDate) -> Planet {
    Planet::weekday_ruler(date.weekday())
}

/// Compute all 24 planetary hours for the given date and location.
///
/// Indices 0–11 are day hours (sunrise → sunset).
/// Indices 12–23 are night hours (sunset → next sunrise).
///
/// # Example
/// ```
/// use astrohour_core::{Location, hours::planetary_hours, Planet};
/// use chrono::NaiveDate;
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 };
/// // 2025-01-02 is a Thursday → Jupiter day
/// let date = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
/// let hours = planetary_hours(date, loc);
/// assert_eq!(hours.len(), 24);
/// // Hour 0 (first day hour) on Jupiter day = Jupiter
/// assert_eq!(hours[0].planet, Planet::Jupiter);
/// assert!(hours[0].is_day);
/// assert!(!hours[12].is_day);
/// ```
pub fn planetary_hours(date: NaiveDate, loc: Location) -> Vec<PlanetaryHour> {
    let (sunrise, sunset) = sunrise_sunset(date, loc);
    let next_date = date.succ_opt().expect("date overflow");
    let (next_sunrise, _) = sunrise_sunset(next_date, loc);

    let day_arc = sunset.signed_duration_since(sunrise);
    let night_arc = next_sunrise.signed_duration_since(sunset);

    let ruler = day_ruler(date);
    let start_idx = ruler.chaldean_index();

    let mut hours = Vec::with_capacity(24);

    // Day hours: 0–11
    for i in 0u8..12 {
        let planet_idx = (start_idx + i as usize) % 7;
        let planet = Planet::CHALDEAN[planet_idx];
        let start = sunrise + day_arc * i as i32 / 12;
        let end = sunrise + day_arc * (i as i32 + 1) / 12;
        hours.push(PlanetaryHour {
            index: i,
            planet,
            start,
            end,
            is_day: true,
            sequence: i + 1,
        });
    }

    // Night hours: 12–23
    for i in 0u8..12 {
        let planet_idx = (start_idx + 12 + i as usize) % 7;
        let planet = Planet::CHALDEAN[planet_idx];
        let start = sunset + night_arc * i as i32 / 12;
        let end = sunset + night_arc * (i as i32 + 1) / 12;
        hours.push(PlanetaryHour {
            index: 12 + i,
            planet,
            start,
            end,
            is_day: false,
            sequence: i + 1,
        });
    }

    hours
}

/// Find the planetary hour that contains `now` for the given location.
///
/// Returns the matching hour, or `None` if the hours array is somehow empty.
///
/// # Example
/// ```no_run
/// use astrohour_core::{Location, hours::current_hour};
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 };
/// let hour = current_hour(loc);
/// println!("Current hour: {} {}", hour.planet.symbol(), hour.planet.name());
/// ```
pub fn current_hour(loc: Location) -> PlanetaryHour {
    let now = Local::now();
    let date = now.date_naive();
    let hours = planetary_hours(date, loc);

    // Check today's hours first
    if let Some(h) = hours.iter().find(|h| now >= h.start && now < h.end) {
        return h.clone();
    }

    // Before today's sunrise: use yesterday's night hours
    let yesterday = date.pred_opt().expect("date underflow");
    let prev_hours = planetary_hours(yesterday, loc);
    if let Some(h) = prev_hours.iter().find(|h| now >= h.start && now < h.end) {
        return h.clone();
    }

    // Fallback: first hour of today (shouldn't happen)
    hours.into_iter().next().expect("hours must not be empty")
}

/// Minutes remaining in the current planetary hour.
///
/// # Example
/// ```no_run
/// use astrohour_core::{Location, hours::minutes_remaining};
///
/// let loc = Location { lat: 40.7128, lon: -74.0060 };
/// let mins = minutes_remaining(loc);
/// assert!(mins >= 0.0 && mins <= 120.0); // a planetary hour is typically 40–80 min
/// ```
pub fn minutes_remaining(loc: Location) -> f64 {
    let now = Local::now();
    let hour = current_hour(loc);
    hour.minutes_remaining_from(now)
}
