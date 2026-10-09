// cc-scan:allow LONG_FUNCTION — snapshot + row rendering are linear.
//! Widget geometry collection on top of egui 0.36's `WidgetRects`.
//!
//! [`GeometrySnapshot`] freezes one pass's worth of `WidgetRect` + `WidgetInfo`
//! data into owned, egui-free values so the inspector UI, a future rule engine,
//! and a future JSON exporter can all consume the same data without holding
//! egui locks. Rects are stored in *layer-local* coordinates — the same units
//! egui stores them in; `to_global_rect` applies the layer transform.
//!
//! Only what egui actually reports is captured. There is no synthesized parent
//! tree: `parent_id` is the real parent Ui id egui stores on every
//! [`egui::WidgetRect`], and `clipped` is derived by comparing `rect` against
//! `interact_rect` (egui pre-clips interact rects to the parent clip).

use egui::{Id, LayerId, Pos2, Rect};

/// Where a rect landed after the layer transform. `local` keeps the raw egui
/// value so consumers can re-transform; `global` is screen space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometryRect {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl GeometryRect {
    pub fn from_egui(rect: Rect) -> Self {
        Self {
            min_x: rect.min.x,
            min_y: rect.min.y,
            max_x: rect.max.x,
            max_y: rect.max.y,
        }
    }

    pub fn to_egui(self) -> Rect {
        Rect::from_min_max(egui::pos2(self.min_x, self.min_y), egui::pos2(self.max_x, self.max_y))
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

/// One widget's geometry + semantics, captured from a single pass.
#[derive(Clone, Debug)]
pub struct WidgetGeometry {
    /// egui `Id` — kept typed so the selection can be re-resolved next pass.
    /// Serialize via `id_debug` when exporting.
    pub id: Id,
    pub id_debug: String,
    /// Real parent-Ui id egui recorded for this widget.
    pub parent_id: Id,
    pub parent_debug: String,
    pub layer: LayerId,
    pub layer_debug: String,
    /// Full widget rect, layer-local.
    pub rect: GeometryRect,
    /// Interaction rect, layer-local, already clipped by the parent clip rect.
    pub interact_rect: GeometryRect,
    /// `interact_rect` is smaller than `rect` → the widget is clipped
    /// (e.g. scrolled out of view or inside a clipped frame).
    pub clipped: bool,
    pub senses_click: bool,
    pub senses_drag: bool,
    pub focusable: bool,
    pub enabled: bool,
    /// Semantic info (`Response::widget_info`), present when the widget
    /// reported it — always filled in debug builds.
    pub widget_type: Option<String>,
    pub label: Option<String>,
    pub hint_text: Option<String>,
    pub current_text_value: Option<String>,
    pub selected: Option<bool>,
    pub value: Option<f64>,
}

impl WidgetGeometry {
    fn from_egui(widget: &egui::WidgetRect, info: Option<&egui::WidgetInfo>) -> Self {
        Self {
            id: widget.id,
            id_debug: widget.id.short_debug_format(),
            parent_id: widget.parent_id,
            parent_debug: widget.parent_id.short_debug_format(),
            layer: widget.layer_id,
            layer_debug: widget.layer_id.short_debug_format(),
            rect: GeometryRect::from_egui(widget.rect),
            interact_rect: GeometryRect::from_egui(widget.interact_rect),
            clipped: widget.interact_rect != widget.rect,
            senses_click: widget.sense.senses_click(),
            senses_drag: widget.sense.senses_drag(),
            focusable: widget.sense.is_focusable(),
            enabled: widget.enabled,
            widget_type: info.map(|i| format!("{:?}", i.typ)),
            label: info.and_then(|i| i.label.clone()),
            hint_text: info.and_then(|i| i.hint_text.clone()),
            current_text_value: info.and_then(|i| i.current_text_value.clone()),
            selected: info.and_then(|i| i.selected),
            value: info.and_then(|i| i.value),
        }
    }

    /// This widget's rect in global/screen coordinates. Layers without an
    /// explicit transform use identity (untransformed layers aren't registered).
    pub fn to_global_rect(&self, ctx: &egui::Context) -> Rect {
        ctx.layer_transform_to_global(self.layer)
            .map(|t| t * self.rect.to_egui())
            .unwrap_or_else(|| self.rect.to_egui())
    }
}

/// All widgets from one pass, grouped by layer, inspector excluded.
#[derive(Clone, Debug, Default)]
pub struct GeometrySnapshot {
    /// Layer order → widgets, back-to-front within each layer.
    /// Sorted front-most order first at the layer level.
    pub layers: Vec<(LayerId, Vec<WidgetGeometry>)>,
}

impl GeometrySnapshot {
    /// Collect the previous pass's widgets. `exclude_area` drops every widget
    /// on the inspector window's layer so the tool never measures itself.
    pub fn collect(ctx: &egui::Context, exclude_area: Id) -> Self {
        ctx.viewport(|v| {
            let mut layers: Vec<(LayerId, Vec<WidgetGeometry>)> = v
                .prev_pass
                .widgets
                .layers()
                .filter(|(layer_id, _)| !is_area_layer(**layer_id, exclude_area))
                .map(|(layer_id, rects)| {
                    (
                        *layer_id,
                        rects
                            .iter()
                            .map(|w| WidgetGeometry::from_egui(w, v.prev_pass.widgets.info(w.id)))
                            .collect(),
                    )
                })
                .collect();
            layers.sort_by_key(|entry| std::cmp::Reverse(entry.0.order));
            Self { layers }
        })
    }

    /// Front-most widget whose interact rect contains `pos` (global space).
    /// Interactable layers only — `Tooltip`/`Debug` can't receive clicks.
    pub fn pick(&self, ctx: &egui::Context, pos: Pos2) -> Option<&WidgetGeometry> {
        for (layer_id, widgets) in &self.layers {
            if matches!(layer_id.order, egui::Order::Tooltip | egui::Order::Debug) {
                continue;
            }
            let to_local = ctx
                .layer_transform_from_global(*layer_id)
                .unwrap_or(egui::epaint::emath::TSTransform::IDENTITY);
            let local_pos = to_local * pos;
            for widget in widgets.iter().rev() {
                if widget.enabled && widget.interact_rect.to_egui().contains(local_pos) {
                    return Some(widget);
                }
            }
        }
        None
    }

    /// Look a widget back up by id — used to keep a selection live across
    /// frames (scroll, resize) without rebuilding the pick.
    pub fn find(&self, id: Id) -> Option<&WidgetGeometry> {
        self.layers
            .iter()
            .flat_map(|(_, widgets)| widgets.iter())
            .find(|w| w.id == id)
    }
}

/// A pinned widget selection that survives pointer movement.
#[derive(Clone, Debug)]
pub struct SelectedWidget {
    /// The geometry captured at pick time.
    pub picked: WidgetGeometry,
    /// Re-resolved geometry from the newest snapshot; `None` if the widget
    /// disappeared (e.g. popup closed, list scrolled it away).
    pub live: Option<WidgetGeometry>,
}

impl SelectedWidget {
    /// The freshest known geometry for display/highlight.
    pub fn current(&self) -> &WidgetGeometry {
        self.live.as_ref().unwrap_or(&self.picked)
    }
}

/// Does `layer` belong to area `area`? Used to keep the inspector out of its
/// own measurements.
pub fn is_area_layer(layer: LayerId, area: Id) -> bool {
    layer.id == area && layer.order == egui::Order::Middle
}

/// Map a pass's `TSTransform` table to a debug string for a layer.
pub fn transform_debug(ctx: &egui::Context, layer: LayerId) -> String {
    ctx.layer_transform_to_global(layer)
        .map(|t: egui::epaint::emath::TSTransform| {
            format!(
                "scale {:.2} offset {:.0},{:.0}",
                t.scaling, t.translation.x, t.translation.y
            )
        })
        .unwrap_or_else(|| "identity".to_owned())
}
