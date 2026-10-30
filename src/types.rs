//! Core data types for astrohour-core.
//!
//! All types derive Serialize/Deserialize for Tauri IPC compatibility.

use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

// ── Location ──────────────────────────────────────────────────────────────────

/// Geographic location used for all astronomical calculations.
///
/// # Example
/// ```
/// use astrohour_core::Location;
/// let loc = Location { lat: 51.5074, lon: -0.1278 }; // London
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Latitude in degrees, positive = North
    pub lat: f64,
    /// Longitude in degrees, positive = East
    pub lon: f64,
}

// ── Planet ────────────────────────────────────────────────────────────────────

/// The seven classical planets in Chaldean order.
///
/// Chaldean order (slowest to fastest): Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon.
/// This order is used for assigning planetary hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Planet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
}

impl Planet {
    /// Unicode symbol for the planet.
    ///
    /// # Example
    /// ```
    /// use astrohour_core::Planet;
    /// assert_eq!(Planet::Jupiter.symbol(), '\u{2643}'); // ♃
    /// ```
    pub fn symbol(&self) -> char {
        match self {
            Planet::Sun => '\u{2609}',     // ☉
            Planet::Moon => '\u{263D}',    // ☽
            Planet::Mars => '\u{2642}',    // ♂
            Planet::Mercury => '\u{263F}', // ☿
            Planet::Jupiter => '\u{2643}', // ♃
            Planet::Venus => '\u{2640}',   // ♀
            Planet::Saturn => '\u{2644}',  // ♄
        }
    }

    /// Full English name of the planet.
    pub fn name(&self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mars => "Mars",
            Planet::Mercury => "Mercury",
            Planet::Jupiter => "Jupiter",
            Planet::Venus => "Venus",
            Planet::Saturn => "Saturn",
        }
    }

    /// Emoji representation (used in Android widget where Unicode symbols may not render).
    pub fn emoji(&self) -> &'static str {
        match self {
            Planet::Sun => "☀️",
            Planet::Moon => "🌙",
            Planet::Mars => "🔴",
            Planet::Mercury => "🟡",
            Planet::Jupiter => "🟣",
            Planet::Venus => "🟢",
            Planet::Saturn => "⚫",
        }
    }

    /// The Chaldean sequence as a fixed array (slowest to fastest).
    /// Index 0 = Saturn, index 6 = Moon.
    pub const CHALDEAN: [Planet; 7] = [
        Planet::Saturn,
        Planet::Jupiter,
        Planet::Mars,
        Planet::Sun,
        Planet::Venus,
        Planet::Mercury,
        Planet::Moon,
    ];

    /// Index of this planet in the Chaldean sequence.
    pub fn chaldean_index(&self) -> usize {
        Self::CHALDEAN
            .iter()
            .position(|p| p == self)
            .expect("all planets are in CHALDEAN")
    }

    /// The traditional day-of-week ruler for each weekday.
    /// Sunday = Sun, Monday = Moon, Tuesday = Mars, Wednesday = Mercury,
    /// Thursday = Jupiter, Friday = Venus, Saturday = Saturn.
    ///
    /// # Example
    /// ```
    /// use astrohour_core::Planet;
    /// use chrono::Weekday;
    /// assert_eq!(Planet::weekday_ruler(Weekday::Thu), Planet::Jupiter);
    /// ```
    pub fn weekday_ruler(weekday: chrono::Weekday) -> Planet {
        match weekday {
            chrono::Weekday::Sun => Planet::Sun,
            chrono::Weekday::Mon => Planet::Moon,
            chrono::Weekday::Tue => Planet::Mars,
            chrono::Weekday::Wed => Planet::Mercury,
            chrono::Weekday::Thu => Planet::Jupiter,
            chrono::Weekday::Fri => Planet::Venus,
            chrono::Weekday::Sat => Planet::Saturn,
        }
    }
}

// ── PlanetaryHour ─────────────────────────────────────────────────────────────

/// A single planetary hour bracket.
///
/// Hours 0–11 are day hours (sunrise → sunset, split into 12 equal parts).
/// Hours 12–23 are night hours (sunset → next sunrise, split into 12 equal parts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryHour {
    /// Index 0–23 (0 = first day hour, 12 = first night hour)
    pub index: u8,
    /// Ruling planet for this hour
    pub planet: Planet,
    /// Start time (local)
    pub start: DateTime<Local>,
    /// End time (local)
    pub end: DateTime<Local>,
    /// true = day hour, false = night hour
    pub is_day: bool,
    /// 1–12 within the day or night half
    pub sequence: u8,
}

impl PlanetaryHour {
    /// Duration of this hour in minutes.
    pub fn duration_minutes(&self) -> f64 {
        self.end.signed_duration_since(self.start).num_seconds() as f64 / 60.0
    }

    /// Minutes remaining from `now` until the end of this hour.
    /// Returns 0.0 if now is after the end.
    pub fn minutes_remaining_from(&self, now: DateTime<Local>) -> f64 {
        let secs = self.end.signed_duration_since(now).num_seconds();
        if secs < 0 { 0.0 } else { secs as f64 / 60.0 }
    }
}

// ── Moon Phase ────────────────────────────────────────────────────────────────

/// The eight traditional lunar phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl Phase {
    /// Display name for the phase.
    pub fn name(&self) -> &'static str {
        match self {
            Phase::NewMoon => "New Moon",
            Phase::WaxingCrescent => "Waxing Crescent",
            Phase::FirstQuarter => "First Quarter",
            Phase::WaxingGibbous => "Waxing Gibbous",
            Phase::FullMoon => "Full Moon",
            Phase::WaningGibbous => "Waning Gibbous",
            Phase::LastQuarter => "Last Quarter",
            Phase::WaningCrescent => "Waning Crescent",
        }
    }

    /// Emoji for the phase.
    pub fn emoji(&self) -> char {
        match self {
            Phase::NewMoon => '\u{1F311}',       // 🌑
            Phase::WaxingCrescent => '\u{1F312}', // 🌒
            Phase::FirstQuarter => '\u{1F313}',   // 🌓
            Phase::WaxingGibbous => '\u{1F314}',  // 🌔
            Phase::FullMoon => '\u{1F315}',       // 🌕
            Phase::WaningGibbous => '\u{1F316}',  // 🌖
            Phase::LastQuarter => '\u{1F317}',    // 🌗
            Phase::WaningCrescent => '\u{1F318}', // 🌘
        }
    }

    /// Derive phase from Sun–Moon elongation (0–360°).
    ///
    /// # Example
    /// ```
    /// use astrohour_core::Phase;
    /// assert_eq!(Phase::from_elongation(0.0), Phase::NewMoon);
    /// assert_eq!(Phase::from_elongation(180.0), Phase::FullMoon);
    /// assert_eq!(Phase::from_elongation(90.0), Phase::FirstQuarter);
    /// ```
    pub fn from_elongation(deg: f64) -> Phase {
        match deg as u32 {
            0..=22 => Phase::NewMoon,
            23..=67 => Phase::WaxingCrescent,
            68..=112 => Phase::FirstQuarter,
            113..=157 => Phase::WaxingGibbous,
            158..=202 => Phase::FullMoon,
            203..=247 => Phase::WaningGibbous,
            248..=292 => Phase::LastQuarter,
            293..=337 => Phase::WaningCrescent,
            _ => Phase::NewMoon, // 338–360 wraps back to new
        }
    }
}

/// Lunar phase data for a specific moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonPhase {
    /// The current phase
    pub phase: Phase,
    /// Illumination fraction 0.0–1.0 (0 = new, 1 = full)
    pub illumination: f64,
    /// Lunar day 1–30 (equal-angle method: floor(elongation / 12) + 1)
    pub lunar_day: u8,
    /// Sun–Moon elongation in degrees (0–360)
    pub elongation_deg: f64,
    /// Moon's geocentric ecliptic longitude in degrees (0–360, tropical)
    pub ecliptic_longitude_deg: f64,
    /// Tropical zodiac sign the Moon is in
    pub sign: ZodiacSign,
}

/// The twelve tropical zodiac signs, 30° each starting at 0° Aries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZodiacSign {
    Aries, Taurus, Gemini, Cancer, Leo, Virgo,
    Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces,
}

impl ZodiacSign {
    /// Sign containing the given ecliptic longitude (any real number of degrees).
    pub fn from_longitude(deg: f64) -> Self {
        use ZodiacSign::*;
        const ALL: [ZodiacSign; 12] = [Aries, Taurus, Gemini, Cancer, Leo, Virgo,
            Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces];
        let i = (deg.rem_euclid(360.0) / 30.0).floor() as usize;
        ALL[i.min(11)]
    }
}

// ── DaySchedule ───────────────────────────────────────────────────────────────

/// Complete astronomical schedule for one day at one location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaySchedule {
    pub date: NaiveDate,
    pub location: Location,
    /// Traditional planetary ruler of the day (Sun=Sunday, Moon=Monday, etc.)
    pub planetary_day: Planet,
    pub sunrise: DateTime<Local>,
    pub sunset: DateTime<Local>,
    /// Moonrise for this date at this location (None = no moonrise, e.g. polar)
    pub moonrise: Option<DateTime<Local>>,
    /// Moonset for this date at this location
    pub moonset: Option<DateTime<Local>>,
    /// All 24 planetary hours: indices 0–11 = day, 12–23 = night
    pub hours: Vec<PlanetaryHour>,
    pub moon: MoonPhase,
}

// ── DisplayMode ───────────────────────────────────────────────────────────────

/// Controls how much text appears in the status bar.
///
/// # Examples
/// - `IconOnly`  → `"♃"`
/// - `Short`     → `"♃ Jupiter Hour"`
/// - `Full`      → `"♃ Jupiter Hour (Day) · 34m"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    IconOnly,
    Short,
    Full,
}
