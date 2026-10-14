//! java.time host shims.

use crate::vm::native::*;

mod chrono_field;
mod chrono_unit;
mod date_time_formatter;
mod date_time_formatter_builder;
mod instant;
mod local_date;
mod local_date_time;
mod zone_id;
mod zoned_date_time;

#[allow(unused_imports)]
pub(crate) use chrono_field::TABLE;
#[allow(unused_imports)]
pub(crate) use chrono_field::{
    static_am_pm_of_day as lazy_chrono_field_am_pm_of_day,
    static_clock_hour_of_am_pm as lazy_chrono_field_clock_hour_of_am_pm,
    static_day_of_month as lazy_chrono_field_day_of_month,
    static_day_of_week as lazy_chrono_field_day_of_week,
    static_day_of_week_in_month as lazy_chrono_field_day_of_week_in_month,
    static_day_of_year as lazy_chrono_field_day_of_year,
    static_epoch_day as lazy_chrono_field_epoch_day, static_era as lazy_chrono_field_era,
    static_hour_of_am_pm as lazy_chrono_field_hour_of_am_pm,
    static_hour_of_day as lazy_chrono_field_hour_of_day,
    static_instant_seconds as lazy_chrono_field_instant_seconds,
    static_minute_of_hour as lazy_chrono_field_minute_of_hour,
    static_month_of_year as lazy_chrono_field_month_of_year,
    static_nano_of_day as lazy_chrono_field_nano_of_day,
    static_nano_of_second as lazy_chrono_field_nano_of_second,
    static_offset_seconds as lazy_chrono_field_offset_seconds,
    static_proleptic_month as lazy_chrono_field_proleptic_month,
    static_second_of_minute as lazy_chrono_field_second_of_minute,
    static_year as lazy_chrono_field_year, static_year_of_era as lazy_chrono_field_year_of_era,
};
pub(crate) use chrono_unit::{
    days as lazy_chrono_unit_days, hours as lazy_chrono_unit_hours,
    millis as lazy_chrono_unit_millis, minutes as lazy_chrono_unit_minutes,
    months as lazy_chrono_unit_months, seconds as lazy_chrono_unit_seconds,
    weeks as lazy_chrono_unit_weeks, years as lazy_chrono_unit_years,
};
#[allow(unused_imports)]
pub(crate) use date_time_formatter::{
    lazy_dtf_basic_iso_date, lazy_dtf_iso_date, lazy_dtf_iso_date_time, lazy_dtf_iso_instant,
    lazy_dtf_iso_local_date, lazy_dtf_iso_local_date_time, lazy_dtf_iso_offset_date_time,
};

/// All java.time native tables, grouped for `register`.
pub(crate) const TIME_TABLE: &[&[NativeEntry]] = &[
    date_time_formatter::TABLE,
    date_time_formatter_builder::TABLE,
    zone_id::TABLE,
    local_date::TABLE,
    local_date_time::TABLE,
    zoned_date_time::TABLE,
    instant::TABLE,
    chrono_field::TABLE,
];
