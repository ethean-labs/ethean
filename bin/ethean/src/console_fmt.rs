//! Ethean console line format: Ream-like columns, green-forward palette.

use nu_ansi_term::{Color, Style};
use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// Compact UTC line: `timestamp LEVEL target: message fields`
#[derive(Debug, Default, Clone)]
pub struct EtheanConsole;

impl<S, N> FormatEvent<S, N> for EtheanConsole
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let level = *meta.level();
        let ansi = writer.has_ansi_escapes();

        write_timestamp(&mut writer, ansi)?;
        write!(writer, " ")?;
        write_level(&mut writer, level, ansi)?;
        write!(writer, " ")?;
        write_target(&mut writer, meta.target(), level, ansi)?;
        write!(writer, ": ")?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

fn write_timestamp(writer: &mut Writer<'_>, ansi: bool) -> fmt::Result {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let micros = now.subsec_micros();
    let stamp = format_utc(secs, micros);
    if ansi {
        write!(writer, "{}", Style::new().dimmed().paint(stamp))
    } else {
        write!(writer, "{stamp}")
    }
}

fn write_level(writer: &mut Writer<'_>, level: Level, ansi: bool) -> fmt::Result {
    let label = match level {
        Level::ERROR => "ERROR",
        Level::WARN => "WARN",
        Level::INFO => "INFO",
        Level::DEBUG => "DEBUG",
        Level::TRACE => "TRACE",
    };
    if !ansi {
        return write!(writer, "{label}");
    }
    let painted = match level {
        Level::ERROR => Color::LightRed.bold().paint(label),
        Level::WARN => Color::LightYellow.bold().paint(label),
        // Good-path / finality lines: green (Ethean accent, not Ream yellow).
        Level::INFO => Color::LightGreen.bold().paint(label),
        Level::DEBUG => Color::LightCyan.paint(label),
        Level::TRACE => Color::Purple.paint(label),
    };
    write!(writer, "{painted}")
}

fn write_target(writer: &mut Writer<'_>, target: &str, level: Level, ansi: bool) -> fmt::Result {
    if !ansi {
        return write!(writer, "{target}");
    }
    // Headers soft-green on INFO; level-tinted otherwise (distinct from Ream yellow targets).
    let painted = match level {
        Level::ERROR => Color::LightRed.paint(target),
        Level::WARN => Color::Yellow.paint(target),
        Level::INFO => Color::Green.paint(target),
        Level::DEBUG => Color::Cyan.paint(target),
        Level::TRACE => Color::Purple.paint(target),
    };
    write!(writer, "{painted}")
}

fn format_utc(unix_secs: u64, micros: u32) -> String {
    let sod = (unix_secs % 86_400) as u32;
    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;
    let (year, month, day) = civil_from_unix_days((unix_secs / 86_400) as i64);
    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}.{micros:06}Z")
}

fn civil_from_unix_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (year as i32, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_epoch_zero() {
        assert_eq!(format_utc(0, 0), "1970-01-01T00:00:00.000000Z");
    }
}
