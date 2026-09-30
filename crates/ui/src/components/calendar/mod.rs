mod config;
mod handler;
mod ui;

pub use handler::{day_of_week, days_in_month, is_leap_year, next_month, previous_month};
pub use ui::{Calendar, DatePicker, SimpleDate};

#[cfg(test)]
mod tests {
    use super::SimpleDate;

    #[test]
    fn simple_date_clamps_month_before_day() {
        assert_eq!(
            SimpleDate::new(2024, 13, 31),
            SimpleDate {
                year: 2024,
                month: 12,
                day: 31
            }
        );
        assert_eq!(
            SimpleDate::new(2024, 0, 31),
            SimpleDate {
                year: 2024,
                month: 1,
                day: 31
            }
        );
    }

    #[test]
    fn simple_date_parse_requires_exact_documented_format() {
        assert_eq!(SimpleDate::parse("2024-02-29"), Some(SimpleDate::new(2024, 2, 29)));
        assert!(SimpleDate::parse("2024-02-30").is_none());
        assert!(SimpleDate::parse("2024-13-01").is_none());
        assert!(SimpleDate::parse(" 2024-02-29").is_none());
        assert_eq!(SimpleDate::new(2024, 2, 29).to_iso_string(), "2024-02-29");
    }

    #[test]
    fn simple_date_preserves_constructor_values() {
        assert_eq!(
            SimpleDate::new(2026, 9, 14),
            SimpleDate {
                year: 2026,
                month: 9,
                day: 14
            }
        );
    }
}
