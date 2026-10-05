//! astrohour-cli -- JSON interface to astrohour-core for the Noctalia plugin.
//!
//! Usage:
//!   astrohour-cli --lat 40.7128 --lon -74.0060
//!   astrohour-cli --lat 40.7128 --lon -74.0060 --date 2025-06-15
//!
//! Outputs a single JSON object to stdout. Non-zero exit on error.

use astrohour_core::{
    day_schedule, display::status_bar_text,
    Location, DisplayMode,
    hours::{current_hour, minutes_remaining},
};
use chrono::{Local, NaiveDate};
use serde::Serialize;
use std::process;

#[derive(Serialize)]
struct CliOutput {
    date: String,

    // Current planetary hour
    current_planet:    String,  // "Jupiter"
    current_symbol:    String,  // "♃"
    current_emoji:     String,  // "🪐"
    minutes_remaining: i64,
    hour_index:        u8,

    // Day
    day_ruler:        String,
    day_ruler_symbol: String,
    sunrise_unix:     i64,
    sunset_unix:      i64,

    // Moon
    moon_phase:        String,  // "Waxing Crescent"
    moon_emoji:        String,  // "🌒"
    moon_illumination: f64,
    lunar_day:         u8,

    // All 24 hours
    hours: Vec<CliHour>,

    // Pre-formatted bar text
    bar_icon:  String,  // "♃"
    bar_short: String,  // "♃ Jupiter Hour"
    bar_full:  String,  // "♃ Jupiter Hour (Day) · 38m"
}

#[derive(Serialize)]
struct CliHour {
    index:      u8,
    planet:     String,
    symbol:     String,
    emoji:      String,
    start_unix: i64,
    end_unix:   i64,
    is_day:     bool,
    is_current: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut lat:  Option<f64>       = None;
    let mut lon:  Option<f64>       = None;
    let mut date: Option<NaiveDate> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--lat"  => { i += 1; lat  = args.get(i).and_then(|v| v.parse().ok()); }
            "--lon"  => { i += 1; lon  = args.get(i).and_then(|v| v.parse().ok()); }
            "--date" => {
                i += 1;
                date = args.get(i)
                    .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok());
            }
            "--help" | "-h" => {
                eprintln!("Usage: astrohour-cli --lat <lat> --lon <lon> [--date YYYY-MM-DD]");
                process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    let lat = match lat {
        Some(v) => v,
        None => { eprintln!("error: --lat required"); process::exit(1); }
    };
    let lon = match lon {
        Some(v) => v,
        None => { eprintln!("error: --lon required"); process::exit(1); }
    };

    let loc  = Location { lat, lon };
    let date = date.unwrap_or_else(|| Local::now().date_naive());

    // Compute schedule
    let sched = day_schedule(date, loc);
    let cur   = current_hour(loc);
    let mins  = minutes_remaining(loc) as i64;

    let hours: Vec<CliHour> = sched.hours.iter().map(|h| CliHour {
        index:      h.index,
        planet:     h.planet.name().to_string(),
        symbol:     h.planet.symbol().to_string(),
        emoji:      h.planet.emoji().to_string(),
        start_unix: h.start.timestamp(),
        end_unix:   h.end.timestamp(),
        is_day:     h.is_day,
        is_current: h.index == cur.index,
    }).collect();

    let out = CliOutput {
        date:              date.to_string(),
        current_planet:    cur.planet.name().to_string(),
        current_symbol:    cur.planet.symbol().to_string(),
        current_emoji:     cur.planet.emoji().to_string(),
        minutes_remaining: mins,
        hour_index:        cur.index,
        day_ruler:         sched.planetary_day.name().to_string(),
        day_ruler_symbol:  sched.planetary_day.symbol().to_string(),
        sunrise_unix:      sched.sunrise.timestamp(),
        sunset_unix:       sched.sunset.timestamp(),
        moon_phase:        sched.moon.phase.name().to_string(),
        moon_emoji:        sched.moon.phase.emoji().to_string(),
        moon_illumination: sched.moon.illumination,
        lunar_day:         sched.moon.lunar_day,
        hours,
        bar_icon:  status_bar_text(loc, DisplayMode::IconOnly),
        bar_short: status_bar_text(loc, DisplayMode::Short),
        bar_full:  status_bar_text(loc, DisplayMode::Full),
    };

    println!("{}", serde_json::to_string(&out).unwrap());
}
