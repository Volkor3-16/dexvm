use crate::vm::native::*;

pub(crate) mod civil;
#[cfg(feature = "tachiyomi")]
mod eu;
mod io;
mod lang;
mod math;
mod net;
mod nio;
#[cfg(feature = "android")]
mod os;
mod security;
mod text;
mod r#time;
mod util;

pub(crate) mod javax_crypto;

pub(crate) use self::io::*;
pub(crate) use self::nio::*;
#[cfg(feature = "tachiyomi")]
pub(crate) use eu::lazy_app_info_instance;
pub(crate) use lang::*;
#[cfg(feature = "android")]
pub(crate) use os::*;
pub(crate) use r#time::{
    lazy_chrono_field_am_pm_of_day, lazy_chrono_field_clock_hour_of_am_pm,
    lazy_chrono_field_day_of_month, lazy_chrono_field_day_of_week,
    lazy_chrono_field_day_of_week_in_month, lazy_chrono_field_day_of_year,
    lazy_chrono_field_epoch_day, lazy_chrono_field_era, lazy_chrono_field_hour_of_am_pm,
    lazy_chrono_field_hour_of_day, lazy_chrono_field_instant_seconds,
    lazy_chrono_field_minute_of_hour, lazy_chrono_field_month_of_year,
    lazy_chrono_field_nano_of_day, lazy_chrono_field_nano_of_second,
    lazy_chrono_field_offset_seconds, lazy_chrono_field_proleptic_month,
    lazy_chrono_field_second_of_minute, lazy_chrono_field_year, lazy_chrono_field_year_of_era,
    lazy_chrono_unit_days, lazy_chrono_unit_hours, lazy_chrono_unit_millis,
    lazy_chrono_unit_minutes, lazy_chrono_unit_months, lazy_chrono_unit_seconds,
    lazy_chrono_unit_weeks, lazy_chrono_unit_years, lazy_dtf_basic_iso_date, lazy_dtf_iso_date,
    lazy_dtf_iso_date_time, lazy_dtf_iso_instant, lazy_dtf_iso_local_date,
    lazy_dtf_iso_local_date_time, lazy_dtf_iso_offset_date_time,
};
pub(crate) use text::*;
pub(crate) use util::*;

/// Collect every java.* native table for `register`.
pub(crate) fn java_tables(out: &mut Vec<&'static [NativeEntry]>) {
    out.extend(lang::LANG_TABLE.iter().copied());
    out.extend(math::MATH_TABLE.iter().copied());
    out.extend(io::IO_TABLE.iter().copied());
    #[cfg(feature = "android")]
    out.extend(io::FILE_TABLE.iter().copied());
    out.extend(net::NET_TABLE.iter().copied());
    #[cfg(feature = "android")]
    out.extend(os::BUILD_TABLE.iter().copied());
    out.extend(nio::NIO_TABLE.iter().copied());
    out.extend(security::SECURITY_TABLE.iter().copied());
    out.push(javax_crypto::JAVAX_CRYPTO_TABLE);
    out.extend(text::TEXT_TABLE.iter().copied());
    out.extend(r#time::TIME_TABLE.iter().copied());
    out.extend(util::UTIL_TABLE.iter().copied());
    #[cfg(feature = "tachiyomi")]
    out.extend(eu::TABLE.iter().copied());
}
