use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlameBarMetrics {
    pub percentage: f32,
    pub color: Color32,
}

pub fn flame_bar_metrics(node_val: f32, total_val: f32, is_bottleneck: bool, theme: &DbProTheme) -> FlameBarMetrics {
    let percentage = if node_val.is_finite() && total_val.is_finite() && total_val > 0.0 {
        (node_val / total_val).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let color = if percentage > 0.45 || is_bottleneck {
        theme.danger
    } else if percentage > 0.20 {
        theme.warning
    } else {
        theme.success
    };

    FlameBarMetrics { percentage, color }
}

pub fn node_icon_and_color(
    is_bottleneck: bool,
    has_relation: bool,
    has_index: bool,
    theme: &DbProTheme,
) -> (Icon, Color32) {
    if is_bottleneck {
        (Icon::Flame, theme.danger)
    } else if has_index {
        (Icon::KeyRound, theme.accent)
    } else if has_relation {
        (Icon::Table, theme.text_secondary)
    } else {
        (Icon::GitCommitVertical, theme.text_secondary)
    }
}

pub fn calculate_row_skew(planned: Option<usize>, actual: usize) -> Option<String> {
    if let Some(planned_rows) = planned {
        if planned_rows > 0 && actual > 0 {
            let ratio = (actual as f64) / (planned_rows as f64);
            if !(0.1..=10.0).contains(&ratio) {
                return Some(if ratio > 1.0 {
                    format!("{:.0}x rows skew", ratio)
                } else {
                    format!("{:.1}x rows skew", ratio)
                });
            }
        }
    }
    None
}

pub fn node_stat_text(
    has_runtime_stats: bool,
    actual_time_ms: f32,
    cost_estimate: f32,
    pct: f32,
    rows: usize,
) -> String {
    if has_runtime_stats {
        format!("{:.2}ms ({:.0}%) · {} rows", actual_time_ms, pct * 100.0, rows)
    } else {
        format!("cost {:.1} ({:.0}%) · {} rows", cost_estimate, pct * 100.0, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flame_bar_metrics() {
        let theme = DbProTheme::light();
        let m_hotspot = flame_bar_metrics(50.0, 100.0, false, &theme);
        assert_eq!(m_hotspot.percentage, 0.5);
        assert_eq!(m_hotspot.color, theme.danger);

        let m_warn = flame_bar_metrics(30.0, 100.0, false, &theme);
        assert_eq!(m_warn.percentage, 0.3);
        assert_eq!(m_warn.color, theme.warning);

        let m_ok = flame_bar_metrics(10.0, 100.0, false, &theme);
        assert_eq!(m_ok.percentage, 0.1);
        assert_eq!(m_ok.color, theme.success);

        let m_forced_bottleneck = flame_bar_metrics(5.0, 100.0, true, &theme);
        assert_eq!(m_forced_bottleneck.color, theme.danger);
    }

    #[test]
    fn test_flame_bar_metrics_handles_non_finite_and_non_positive_values() {
        let theme = DbProTheme::light();

        for (node_val, total_val) in [
            (f32::NAN, 100.0),
            (f32::INFINITY, 100.0),
            (50.0, f32::NAN),
            (50.0, f32::INFINITY),
            (-50.0, 100.0),
            (50.0, 0.0),
            (50.0, -100.0),
        ] {
            let metrics = flame_bar_metrics(node_val, total_val, false, &theme);
            assert_eq!(metrics.percentage, 0.0, "inputs: {node_val:?}, {total_val:?}");
            assert!(metrics.percentage.is_finite(), "percentage must remain finite");
        }
    }

    #[test]
    fn test_calculate_row_skew() {
        assert_eq!(calculate_row_skew(Some(10), 200), Some("20x rows skew".to_string()));
        assert_eq!(calculate_row_skew(Some(100), 5), Some("0.1x rows skew".to_string()));
        assert_eq!(calculate_row_skew(Some(100), 100), None);
        assert_eq!(calculate_row_skew(None, 100), None);
    }

    #[test]
    fn test_node_icon_and_color() {
        let theme = DbProTheme::light();
        let (icon, color) = node_icon_and_color(true, false, false, &theme);
        assert_eq!(char::from(icon), char::from(Icon::Flame));
        assert_eq!(color, theme.danger);

        let (icon2, color2) = node_icon_and_color(false, false, true, &theme);
        assert_eq!(char::from(icon2), char::from(Icon::KeyRound));
        assert_eq!(color2, theme.accent);

        let (icon3, color3) = node_icon_and_color(false, true, false, &theme);
        assert_eq!(char::from(icon3), char::from(Icon::Table));
        assert_eq!(color3, theme.text_secondary);
    }
}
