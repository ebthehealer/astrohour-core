//! Moon phase, lunar day, and Sun–Moon elongation.
//!
//! ## Elongation source
//! `esbat::lunar_phase()` returns the Sun–Moon difference in ecliptic longitudes
//! as degrees in [0, 360), per the Calendrical Calculations algorithms:
//!   0° = New Moon, 90° = First Quarter, 180° = Full Moon, 270° = Last Quarter.
//!
//! `solunatus::lunar_position()` gives illumination fraction (geocentric).

use chrono::{DateTime, Utc};
use solunatus::prelude::*;
use solunatus::astro::Location as SolLocation;
use crate::types::{MoonPhase, Phase};

/// Sun–Moon elongation in degrees (0–360°) at the given UTC instant.
///
/// 0° = New Moon, 90° = First Quarter, 180° = Full Moon, 270° = Last Quarter.
///
/// # Example
/// ```no_run
/// use astrohour_core::lunar::sun_moon_elongation;
/// use chrono::Utc;
///
/// let now = Utc::now();
/// let deg = sun_moon_elongation(now);
/// assert!(deg >= 0.0 && deg < 360.0);
/// ```
pub fn sun_moon_elongation(dt: DateTime<Utc>) -> f64 {
    let deg = esbat::lunar_phase(dt);
    // esbat already clamps to [0, 360)
    deg
}

/// Lunar day number 1–30 using the equal-angle method.
///
/// Formula: `floor(elongation_degrees / 12.0) as u8 + 1`
///
/// Each 12° of elongation ≈ one lunar day in a 30-day synodic month.
///
/// # Example
/// ```no_run
/// use astrohour_core::lunar::lunar_day;
/// use chrono::Utc;
///
/// let day = lunar_day(Utc::now());
/// assert!(day >= 1 && day <= 30);
/// ```
pub fn lunar_day(dt: DateTime<Utc>) -> u8 {
    let deg = sun_moon_elongation(dt);
    ((deg / 12.0).floor() as u8 + 1).clamp(1, 30)
}

/// Current moon phase data: phase, illumination, lunar day, elongation.
///
/// Illumination is geocentric (location-independent) from solunatus.
///
/// # Example
/// ```no_run
/// use astrohour_core::lunar::moon_phase;
/// use chrono::Utc;
///
/// let mp = moon_phase(Utc::now());
/// println!("{} {} · Moon Day {}", mp.phase.emoji(), mp.phase.name(), mp.lunar_day);
/// assert!(mp.illumination >= 0.0 && mp.illumination <= 1.0);
/// ```
pub fn moon_phase(dt: DateTime<Utc>) -> MoonPhase {
    let elongation = sun_moon_elongation(dt);
    let phase = Phase::from_elongation(elongation);
    let day = ((elongation / 12.0).floor() as u8 + 1).clamp(1, 30);

    // Illumination from solunatus (geocentric; prime meridian fine here)
    let sol_loc = SolLocation::new(0.0, 0.0).unwrap();
    let dt_local = dt.with_timezone(&chrono::Local);
    let moon_pos = lunar_position(&sol_loc, &dt_local);

    MoonPhase {
        phase,
        illumination: moon_pos.illumination,
        lunar_day: day,
        elongation_deg: elongation,
    }
}
