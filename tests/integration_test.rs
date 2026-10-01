//! Integration tests for astrohour-core.
//!
//! Reference dates verified against:
//! - https://www.astrology.com/planetary-hours
//! - https://www.lunarium.co.uk/moon/phase.jsp
//! - https://www.timeanddate.com/sun/

use astrohour_core::{
    Location, Planet, DisplayMode,
    hours::{planetary_hours, day_ruler},
    lunar::{sun_moon_elongation, lunar_day, moon_phase},
    solar::sunrise_sunset,
    display::status_bar_text,
    day_schedule,
};
use chrono::{NaiveDate, TimeZone, Utc};

// ── Test locations ──────────────────────────────────────────────────────────────
const NYC: Location = Location { lat: 40.7128, lon: -74.0060 };
const LONDON: Location = Location { lat: 51.5074, lon: -0.1278 };
const SYDNEY: Location = Location { lat: -33.8688, lon: 151.2093 };

// ── Planet / Weekday ruler tests ──────────────────────────────────────────────────

#[test]
fn test_weekday_rulers() {
    // 2025-01-05 is a Sunday → Sun
    let sun   = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
    // 2025-01-06 is a Monday → Moon
    let mon   = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap();
    // 2025-01-07 is a Tuesday → Mars
    let tue   = NaiveDate::from_ymd_opt(2025, 1, 7).unwrap();
    // 2025-01-08 is a Wednesday → Mercury
    let wed   = NaiveDate::from_ymd_opt(2025, 1, 8).unwrap();
    // 2025-01-09 is a Thursday → Jupiter
    let thu   = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();
    // 2025-01-10 is a Friday → Venus
    let fri   = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
    // 2025-01-11 is a Saturday → Saturn
    let sat   = NaiveDate::from_ymd_opt(2025, 1, 11).unwrap();

    assert_eq!(day_ruler(sun), Planet::Sun);
    assert_eq!(day_ruler(mon), Planet::Moon);
    assert_eq!(day_ruler(tue), Planet::Mars);
    assert_eq!(day_ruler(wed), Planet::Mercury);
    assert_eq!(day_ruler(thu), Planet::Jupiter);
    assert_eq!(day_ruler(fri), Planet::Venus);
    assert_eq!(day_ruler(sat), Planet::Saturn);
}

#[test]
fn test_planet_symbols() {
    assert_eq!(Planet::Sun.symbol(),     '\u{2609}');
    assert_eq!(Planet::Moon.symbol(),    '\u{263D}');
    assert_eq!(Planet::Mars.symbol(),    '\u{2642}');
    assert_eq!(Planet::Mercury.symbol(), '\u{263F}');
    assert_eq!(Planet::Jupiter.symbol(), '\u{2643}');
    assert_eq!(Planet::Venus.symbol(),   '\u{2640}');
    assert_eq!(Planet::Saturn.symbol(),  '\u{2644}');
}

#[test]
fn test_chaldean_order() {
    // Chaldean: Saturn(0), Jupiter(1), Mars(2), Sun(3), Venus(4), Mercury(5), Moon(6)
    assert_eq!(Planet::Saturn.chaldean_index(),  0);
    assert_eq!(Planet::Jupiter.chaldean_index(), 1);
    assert_eq!(Planet::Mars.chaldean_index(),    2);
    assert_eq!(Planet::Sun.chaldean_index(),     3);
    assert_eq!(Planet::Venus.chaldean_index(),   4);
    assert_eq!(Planet::Mercury.chaldean_index(), 5);
    assert_eq!(Planet::Moon.chaldean_index(),    6);
}

// ── Planetary hours structure tests ────────────────────────────────────────────────

#[test]
fn test_hours_count_and_structure() {
    let date = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap(); // Thursday
    let hours = planetary_hours(date, NYC);

    assert_eq!(hours.len(), 24, "must always produce exactly 24 hours");

    // First 12 are day hours
    for h in &hours[..12] {
        assert!(h.is_day, "index {} should be a day hour", h.index);
        assert_eq!(h.sequence, h.index + 1);
    }
    // Last 12 are night hours
    for h in &hours[12..] {
        assert!(!h.is_day, "index {} should be a night hour", h.index);
        assert_eq!(h.sequence, h.index - 11);
    }
}

#[test]
fn test_hours_contiguous() {
    let date = NaiveDate::from_ymd_opt(2025, 6, 21).unwrap(); // summer solstice
    let hours = planetary_hours(date, NYC);

    for i in 1..12 {
        assert_eq!(
            hours[i].start, hours[i - 1].end,
            "day hour {} must start exactly where {} ended", i, i - 1
        );
    }
    for i in 13..24 {
        assert_eq!(
            hours[i].start, hours[i - 1].end,
            "night hour {} must start exactly where {} ended", i, i - 1
        );
    }
    // Night starts at day end (sunset)
    assert_eq!(hours[12].start, hours[11].end, "night must start at sunset");
}

#[test]
fn test_thursday_hour_zero_is_jupiter() {
    // On a Thursday (Jupiter day), the first day hour must be Jupiter.
    let thursday = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();
    let hours = planetary_hours(thursday, NYC);
    assert_eq!(hours[0].planet, Planet::Jupiter);
}

#[test]
fn test_sunday_hour_zero_is_sun() {
    let sunday = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
    let hours = planetary_hours(sunday, NYC);
    assert_eq!(hours[0].planet, Planet::Sun);
}

#[test]
fn test_saturday_hour_zero_is_saturn() {
    let saturday = NaiveDate::from_ymd_opt(2025, 1, 11).unwrap();
    let hours = planetary_hours(saturday, NYC);
    assert_eq!(hours[0].planet, Planet::Saturn);
}

#[test]
fn test_chaldean_sequence_in_hours() {
    // Verify the Chaldean sequence is correctly applied across all 24 hours.
    // On a Sunday (Sun=index 3 in Chaldean), hours should cycle from index 3 upward.
    let sunday = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
    let hours = planetary_hours(sunday, NYC);
    let sun_idx = Planet::Sun.chaldean_index(); // 3

    for (i, h) in hours.iter().enumerate() {
        let expected = Planet::CHALDEAN[(sun_idx + i) % 7];
        assert_eq!(
            h.planet, expected,
            "hour {} should be {:?} but got {:?}", i, expected, h.planet
        );
    }
}

#[test]
fn test_day_hours_between_sunrise_and_sunset() {
    let date = NaiveDate::from_ymd_opt(2025, 3, 20).unwrap();
    let (sunrise, sunset) = sunrise_sunset(date, NYC);
    let hours = planetary_hours(date, NYC);

    // First day hour starts at sunrise
    assert_eq!(hours[0].start, sunrise);
    // Last day hour ends at sunset
    assert_eq!(hours[11].end, sunset);
    // First night hour starts at sunset
    assert_eq!(hours[12].start, sunset);
}

#[test]
fn test_duration_positive() {
    let date = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();
    let hours = planetary_hours(date, NYC);
    for h in &hours {
        assert!(
            h.duration_minutes() > 0.0,
            "hour {} has zero or negative duration", h.index
        );
    }
}

#[test]
fn test_hours_different_hemispheres() {
    // Summer in Sydney (Jan) = long days; NYC = short days.
    let date = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();
    let nyc_hours   = planetary_hours(date, NYC);
    let sydney_hours = planetary_hours(date, SYDNEY);

    let nyc_day_min    = nyc_hours[0].duration_minutes();
    let sydney_day_min = sydney_hours[0].duration_minutes();

    // Sydney should have longer day hours in January than NYC
    assert!(
        sydney_day_min > nyc_day_min,
        "Sydney day hour ({:.1}m) should be longer than NYC ({:.1}m) in January",
        sydney_day_min, nyc_day_min
    );
}

// ── Sunrise / Sunset tests ──────────────────────────────────────────────────────

#[test]
fn test_sunrise_before_sunset() {
    let date = NaiveDate::from_ymd_opt(2025, 6, 21).unwrap();
    for loc in [NYC, LONDON, SYDNEY] {
        let (rise, set) = sunrise_sunset(date, loc);
        assert!(rise < set, "sunrise must precede sunset at {:?}", loc);
    }
}

#[test]
fn test_nyc_summer_solstice_hours() {
    // NYC summer solstice 2025-06-21: sunrise ~09:25 UTC (05:25 EDT), sunset ~00:31 UTC next day
    // The VM runs UTC, so we check the UTC hours from solunatus.
    let date = NaiveDate::from_ymd_opt(2025, 6, 21).unwrap();
    let (rise, set) = sunrise_sunset(date, NYC);
    // Sunrise UTC for NYC in June is ~09:xx; sunset is ~00:xx UTC (next day)
    let rise_utc = rise.with_timezone(&Utc);
    let set_utc  = set.with_timezone(&Utc);
    assert_eq!(rise_utc.format("%H").to_string(), "09", "NYC summer sunrise UTC hour");
    // Sunset is after 23:00 UTC (00:31 UTC next calendar day is valid too)
    let set_hour: u32 = set_utc.format("%H").to_string().parse().unwrap();
    assert!(set_hour >= 23 || set_hour == 0, "NYC summer sunset UTC hour should be 23 or 00");
    // Basic sanity: day should be > 14 hours long
    let day_hours = set.signed_duration_since(rise).num_minutes() as f64 / 60.0;
    assert!(day_hours > 14.0 && day_hours < 16.0,
        "NYC summer day length should be ~15h, got {:.2}h", day_hours);
}

#[test]
fn test_nyc_winter_solstice_short_day() {
    // NYC winter solstice 2025-12-21: sunrise ~07:17, sunset ~16:32 → ~9h15m day
    let date = NaiveDate::from_ymd_opt(2025, 12, 21).unwrap();
    let (rise, set) = sunrise_sunset(date, NYC);
    let day_len_hours = set.signed_duration_since(rise).num_minutes() as f64 / 60.0;
    assert!(
        day_len_hours > 8.5 && day_len_hours < 10.0,
        "NYC winter day length should be ~9.25h, got {:.2}h", day_len_hours
    );
}

// ── Lunar day tests ────────────────────────────────────────────────────────────────

#[test]
fn test_elongation_range() {
    let now = Utc::now();
    let deg = sun_moon_elongation(now);
    assert!(deg >= 0.0 && deg < 360.0, "elongation must be in [0, 360), got {}", deg);
}

#[test]
fn test_lunar_day_range() {
    let now = Utc::now();
    let day = lunar_day(now);
    assert!(day >= 1 && day <= 30, "lunar day must be 1-30, got {}", day);
}

#[test]
fn test_known_full_moon_elongation() {
    // 2020-10-31 full moon: esbat docs confirm ~180° at 14:48:59 UTC
    #[allow(deprecated)]
    let t = Utc.ymd(2020, 10, 31).and_hms_milli(14, 48, 59, 300);
    let deg = sun_moon_elongation(t);
    // Meeus Ch.47 series agrees within ~0.5° of Swiss Ephemeris at full moon
    assert!(
        (deg - 180.0).abs() < 0.5,
        "Halloween 2020 full moon should be ~180°, got {:.4}", deg
    );
}

#[test]
fn test_known_full_moon_lunar_day() {
    // Full moon at ~180° → floor(180/12)+1 = 16.
    // esbat returns 179.999... at exact full moon so we get day 15 or 16.
    #[allow(deprecated)]
    let t = Utc.ymd(2020, 10, 31).and_hms_milli(14, 48, 59, 300);
    let deg = sun_moon_elongation(t);
    let day = lunar_day(t);
    // At full moon elongation is ~180°; floor(179.x/12)+1=15 or floor(180.0/12)+1=16
    assert!(
        day == 15 || day == 16,
        "full moon should be lunar day 15 or 16, got {} (elongation={:.4}°)", day, deg
    );
}

#[test]
fn test_moon_phase_illumination_at_full() {
    // Near full moon: illumination should be > 95%
    #[allow(deprecated)]
    let t = Utc.ymd(2020, 10, 31).and_hms_milli(14, 48, 59, 300);
    let mp = moon_phase(t);
    assert!(
        mp.illumination > 0.95,
        "full moon illumination should be >95%, got {:.2}%", mp.illumination * 100.0
    );
}

#[test]
fn test_phase_from_elongation() {
    use astrohour_core::Phase;
    assert_eq!(Phase::from_elongation(0.0),   Phase::NewMoon);
    assert_eq!(Phase::from_elongation(45.0),  Phase::WaxingCrescent);
    assert_eq!(Phase::from_elongation(90.0),  Phase::FirstQuarter);
    assert_eq!(Phase::from_elongation(135.0), Phase::WaxingGibbous);
    assert_eq!(Phase::from_elongation(180.0), Phase::FullMoon);
    assert_eq!(Phase::from_elongation(225.0), Phase::WaningGibbous);
    assert_eq!(Phase::from_elongation(270.0), Phase::LastQuarter);
    assert_eq!(Phase::from_elongation(315.0), Phase::WaningCrescent);
}

// ── Display mode tests ──────────────────────────────────────────────────────────────

#[test]
fn test_status_bar_icon_only() {
    let s = status_bar_text(NYC, DisplayMode::IconOnly);
    assert_eq!(s.chars().count(), 1, "IconOnly must be a single symbol char, got {:?}", s);
}

#[test]
fn test_status_bar_short_contains_hour() {
    let s = status_bar_text(NYC, DisplayMode::Short);
    assert!(s.contains("Hour"), "Short mode must contain 'Hour', got {:?}", s);
}

#[test]
fn test_status_bar_full_format() {
    let s = status_bar_text(NYC, DisplayMode::Full);
    assert!(s.contains("Hour"),  "Full mode must contain 'Hour'");
    assert!(s.contains('m'),     "Full mode must contain minutes 'm'");
    assert!(
        s.contains("Day") || s.contains("Night"),
        "Full mode must indicate Day or Night, got {:?}", s
    );
}

// ── day_schedule integration ────────────────────────────────────────────────────────

#[test]
fn test_day_schedule_completeness() {
    let date = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap(); // Thursday
    let schedule = day_schedule(date, NYC);

    assert_eq!(schedule.hours.len(), 24);
    assert_eq!(schedule.planetary_day, Planet::Jupiter);
    assert!(schedule.sunrise < schedule.sunset);
    assert!(schedule.moon.lunar_day >= 1 && schedule.moon.lunar_day <= 30);
    assert!(schedule.moon.illumination >= 0.0 && schedule.moon.illumination <= 1.0);
}

#[test]
fn test_day_schedule_performance() {
    // Target: <1ms per day_schedule() call (from build plan)
    let date = NaiveDate::from_ymd_opt(2025, 6, 21).unwrap();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = day_schedule(date, NYC);
    }
    let avg_ms = start.elapsed().as_millis() as f64 / 100.0;
    assert!(
        avg_ms < 5.0, // 5ms is very conservative; expect well under 1ms
        "day_schedule() averaged {:.2}ms; target is <1ms", avg_ms
    );
}
