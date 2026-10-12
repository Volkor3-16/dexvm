//! java.time.temporal.ChronoField host shims: constants carrying the
//! enum constant name so `DateTimeFormatterBuilder.parseCaseInsensitive`
//! can read them back.

use crate::vm::native::*;

macro_rules! chrono_field_const {
    ($name:ident, $tag:expr) => {
        pub fn $name(_vm: &mut Vm, _args: &[JValue]) -> Result<JValue, NatErr> {
            let Ok(class) = _vm.ensure_class_by_desc("Ljava/time/temporal/ChronoField;") else {
                return Ok(JValue::Null);
            };
            Ok(JValue::Obj(_vm.arena.alloc(
                class,
                Vec::new(),
                Some(Native::Str($tag.into())),
            )))
        }
    };
}

chrono_field_const!(era, "ERA");
chrono_field_const!(year, "YEAR");
chrono_field_const!(year_of_era, "YEAR_OF_ERA");
chrono_field_const!(month_of_year, "MONTH_OF_YEAR");
chrono_field_const!(day_of_month, "DAY_OF_MONTH");
chrono_field_const!(day_of_year, "DAY_OF_YEAR");
chrono_field_const!(day_of_week, "DAY_OF_WEEK");
chrono_field_const!(day_of_week_in_month, "DAY_OF_WEEK_IN_MONTH");
chrono_field_const!(am_pm_of_day, "AM_PM_OF_DAY");
chrono_field_const!(clock_hour_of_am_pm, "CLOCK_HOUR_OF_AM_PM");
chrono_field_const!(hour_of_am_pm, "HOUR_OF_AM_PM");
chrono_field_const!(hour_of_day, "HOUR_OF_DAY");
chrono_field_const!(minute_of_hour, "MINUTE_OF_HOUR");
chrono_field_const!(second_of_minute, "SECOND_OF_MINUTE");
chrono_field_const!(nano_of_second, "NANO_OF_SECOND");
chrono_field_const!(nano_of_day, "NANO_OF_DAY");
chrono_field_const!(epoch_day, "EPOCH_DAY");
chrono_field_const!(proleptic_month, "PROLEPTIC_MONTH");
chrono_field_const!(instant_seconds, "INSTANT_SECONDS");
chrono_field_const!(offset_seconds, "OFFSET_SECONDS");

/// Native methods for Ljava/time/temporal/ChronoField;
pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "ERA",
        "()Ljava/time/temporal/ChronoField;",
        true,
        era
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "YEAR",
        "()Ljava/time/temporal/ChronoField;",
        true,
        year
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "YEAR_OF_ERA",
        "()Ljava/time/temporal/ChronoField;",
        true,
        year_of_era
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "MONTH_OF_YEAR",
        "()Ljava/time/temporal/ChronoField;",
        true,
        month_of_year
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "DAY_OF_MONTH",
        "()Ljava/time/temporal/ChronoField;",
        true,
        day_of_month
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "DAY_OF_YEAR",
        "()Ljava/time/temporal/ChronoField;",
        true,
        day_of_year
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "DAY_OF_WEEK",
        "()Ljava/time/temporal/ChronoField;",
        true,
        day_of_week
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "DAY_OF_WEEK_IN_MONTH",
        "()Ljava/time/temporal/ChronoField;",
        true,
        day_of_week_in_month
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "AM_PM_OF_DAY",
        "()Ljava/time/temporal/ChronoField;",
        true,
        am_pm_of_day
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "CLOCK_HOUR_OF_AM_PM",
        "()Ljava/time/temporal/ChronoField;",
        true,
        clock_hour_of_am_pm
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "HOUR_OF_AM_PM",
        "()Ljava/time/temporal/ChronoField;",
        true,
        hour_of_am_pm
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "HOUR_OF_DAY",
        "()Ljava/time/temporal/ChronoField;",
        true,
        hour_of_day
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "MINUTE_OF_HOUR",
        "()Ljava/time/temporal/ChronoField;",
        true,
        minute_of_hour
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "SECOND_OF_MINUTE",
        "()Ljava/time/temporal/ChronoField;",
        true,
        second_of_minute
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "NANO_OF_SECOND",
        "()Ljava/time/temporal/ChronoField;",
        true,
        nano_of_second
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "NANO_OF_DAY",
        "()Ljava/time/temporal/ChronoField;",
        true,
        nano_of_day
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "EPOCH_DAY",
        "()Ljava/time/temporal/ChronoField;",
        true,
        epoch_day
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "PROLEPTIC_MONTH",
        "()Ljava/time/temporal/ChronoField;",
        true,
        proleptic_month
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "INSTANT_SECONDS",
        "()Ljava/time/temporal/ChronoField;",
        true,
        instant_seconds
    ),
    ne!(
        "Ljava/time/temporal/ChronoField;",
        "OFFSET_SECONDS",
        "()Ljava/time/temporal/ChronoField;",
        true,
        offset_seconds
    ),
];
