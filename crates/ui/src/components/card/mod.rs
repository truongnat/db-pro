mod config;

pub use config::CARD_FRAME_STROKE_WIDTH;
mod handler;
mod ui;

pub use handler::{MetricTrendDirection, MetricTrendTone};
pub use ui::{card_content, card_footer, card_header, Card, MetricCard, MetricTrend};
