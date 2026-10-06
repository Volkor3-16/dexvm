use crate::vm::native::*;

pub(crate) mod civil;
mod io;
mod lang;
mod math;
mod net;
mod nio;
pub(crate) mod os;
mod security;
mod text;
mod r#time;
mod util;

pub(crate) mod eu;

pub(crate) mod javax_crypto;

pub(crate) use self::io::*;
pub(crate) use self::nio::*;
pub(crate) use lang::*;
pub(crate) use r#time::{
    lazy_chrono_unit_days, lazy_chrono_unit_hours, lazy_chrono_unit_millis,
    lazy_chrono_unit_minutes, lazy_chrono_unit_months, lazy_chrono_unit_seconds,
    lazy_chrono_unit_weeks, lazy_chrono_unit_years,
};
pub(crate) use text::*;
pub(crate) use util::*;

/// Collect every java.* native table for `register`.
pub(crate) fn java_tables(out: &mut Vec<&'static [NativeEntry]>) {
    out.extend(lang::LANG_TABLE);
    out.extend(math::MATH_TABLE);
    out.extend(io::IO_TABLE);
    #[cfg(feature = "android")]
    out.extend(io::FILE_TABLE);
    out.extend(net::NET_TABLE);
    out.extend(nio::NIO_TABLE);
    out.extend(os::BUILD_TABLE);
    out.extend(security::SECURITY_TABLE);
    out.push(javax_crypto::JAVAX_CRYPTO_TABLE);
    out.extend(text::TEXT_TABLE);
    out.extend(r#time::TIME_TABLE);
    out.extend(util::UTIL_TABLE);
    out.extend(eu::TABLE);
}
