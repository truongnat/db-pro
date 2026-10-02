//! Offscreen rs-ui paint bridge for the Explorer's existing host hitboxes.
use super::explorer_tree::{CODEX_CHEVRON_SLOT, CODEX_ROW_INDENT, CODEX_ROW_PADDING};
use crate::{app::explorer_tree::CodexTreeRow, DbProTheme};
use eframe::egui;
use rs_ui_core::{Color, DisplayList, DisplayListBuilder, Point, Radius, Rect, ScaleFactor, Size, Srgb8, TextRunId};
use rs_ui_renderer::{RenderFrame, RendererOptions, UiRenderer, Viewport};
use rs_ui_text::{FontFamily, FontWeight, TextMetrics, TextStyle, TextSystem};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

type Shared = Arc<Mutex<ExplorerPainter>>;
type TextCacheKey = (String, &'static str, u32, [u8; 4], Option<u32>);
fn id() -> egui::Id {
    egui::Id::new("rs-ui-explorer-painter")
}

/// Initialize the canonical renderer once on the native host, before drawing.
pub fn install_explorer_renderer(ctx: &egui::Context) -> Result<(), Box<dyn Error + Send + Sync>> {
    let painter = ExplorerPainter::new()?;
    ctx.data_mut(|data| data.insert_temp(id(), Arc::new(Mutex::new(painter))));
    Ok(())
}

fn shared(ctx: &egui::Context) -> Option<Shared> {
    ctx.data(|data| data.get_temp(id()))
}

#[cfg(test)]
pub(crate) fn cached_run_count(ctx: &egui::Context) -> usize {
    shared(ctx)
        .expect("installed renderer")
        .lock()
        .expect("Explorer renderer mutex poisoned")
        .runs
        .len()
}

#[cfg(test)]
pub(crate) fn viewport_rect(ctx: &egui::Context) -> egui::Rect {
    shared(ctx)
        .expect("installed renderer")
        .lock()
        .expect("Explorer renderer mutex poisoned")
        .viewport
}

pub(crate) fn begin(ui: &egui::Ui, theme: DbProTheme) -> bool {
    let Some(shared) = shared(ui.ctx()) else {
        return false;
    };
    let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
    let scale = ui.ctx().pixels_per_point();
    let rect = ui.available_rect_before_wrap();
    painter.viewport = egui::Rect::from_min_max(
        egui::pos2(
            (rect.min.x * scale).round() / scale,
            (rect.min.y * scale).round() / scale,
        ),
        egui::pos2(
            (rect.max.x * scale).round() / scale,
            (rect.max.y * scale).round() / scale,
        ),
    );
    painter.list = DisplayListBuilder::new();
    painter.used.clear();
    let local = Rect::from_min_size(
        Point::ZERO,
        Size::new(painter.viewport.width(), painter.viewport.height()),
    );
    painter
        .list
        .fill_rect(local, color(theme.surface_panel))
        .push_clip(local);
    painter.active = true;
    true
}

pub(crate) fn row(ui: &egui::Ui, theme: &DbProTheme, rect: egui::Rect, row: &CodexTreeRow<'_>, hovered: bool) -> bool {
    let Some(shared) = shared(ui.ctx()) else {
        return false;
    };
    let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
    if !painter.active {
        return false;
    }
    if rect.intersects(painter.viewport) {
        painter.paint_row(theme, rect, row, hovered);
    }
    true
}

pub(crate) fn active(ctx: &egui::Context) -> bool {
    shared(ctx).is_some_and(|shared| shared.lock().expect("Explorer renderer mutex poisoned").active)
}

pub(crate) fn message(
    ui: &mut egui::Ui,
    text: &str,
    tint: egui::Color32,
    icon: lucide_icons::Icon,
    clickable: bool,
) -> egui::Response {
    let row = CodexTreeRow {
        depth: 0,
        is_expandable: false,
        is_expanded: false,
        icon,
        icon_color: tint,
        label: text,
        is_selected: false,
        is_dimmed: false,
        status_dot: None,
        badge_text: None,
        badge_accent: false,
        count_text: None,
        detail_text: None,
    };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 26.0),
        if clickable {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    if let Some(shared) = shared(ui.ctx()) {
        let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
        let local = painter.rect(rect);
        painter.list.push_clip(local);
        if response.hovered() && clickable {
            painter
                .list
                .fill_rounded_rect(local, Radius::all(4.0), color(tint.linear_multiply(0.1)));
        }
        let x = rect.left() + 4.0;
        painter.label(
            &char::from(row.icon).to_string(),
            "lucide",
            13.0,
            tint,
            egui::pos2(x + 7.0, rect.center().y),
            0.5,
        );
        painter.label(text, "Inter", 12.5, tint, egui::pos2(x + 20.0, rect.center().y), 0.0);
        painter.list.pop_clip();
    }
    response
}

pub(crate) fn scrollbar(ui: &egui::Ui, rect: egui::Rect, tint: egui::Color32) {
    if let Some(shared) = shared(ui.ctx()) {
        let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
        let local = painter.rect(rect);
        painter.list.fill_rounded_rect(local, Radius::all(3.0), color(tint));
    }
}

pub(crate) fn error_detail(ui: &mut egui::Ui, text: &str, theme: DbProTheme) {
    let shared = shared(ui.ctx()).expect("active Explorer renderer");
    let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
    let width = ui.available_width();
    let key = (
        text.to_owned(),
        "monospace",
        12.0_f32.to_bits(),
        theme.text_secondary.to_array(),
        Some(width.to_bits()),
    );
    let run = if let Some(run) = painter.runs.get(&key) {
        *run
    } else {
        let run = painter.text.shape(
            text,
            TextStyle {
                family: FontFamily::Monospace,
                size_px: 12.0,
                line_height_px: 15.0,
                color: color(theme.text_secondary),
                ..Default::default()
            },
            Some(width),
        );
        painter.runs.insert(key, run);
        run
    };
    painter.used.insert(run);
    let height = painter.text.run_metrics(run).expect("error text layout").size.height;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let local = painter.rect(rect);
    painter.list.push_clip(local).text(run, local.min).pop_clip();
    response.on_hover_text(text);
}

pub(crate) fn empty_state(ui: &mut egui::Ui, theme: DbProTheme) -> egui::Response {
    let shared = shared(ui.ctx()).expect("active Explorer renderer");
    let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
    ui.add_space(36.0);
    for (value, family, size, tint, height, gap) in [
        (
            char::from(lucide_icons::Icon::Database).to_string(),
            "lucide",
            28.0,
            theme.text_muted,
            32.0,
            8.0,
        ),
        (
            "No connections".to_owned(),
            "Inter Medium",
            14.0,
            theme.text_primary,
            20.0,
            3.0,
        ),
        (
            "Create a database connection to begin.".to_owned(),
            "Inter",
            11.0,
            theme.text_muted,
            20.0,
            12.0,
        ),
    ] {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), egui::Sense::hover());
        painter.label(&value, family, size, tint, rect.center(), 0.5);
        ui.add_space(gap);
    }
    let width = painter
        .text_run("New connection", "Inter", 12.0, theme.text_primary)
        .1
        .size
        .width
        + 36.0;
    let (row, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
    let rect = egui::Rect::from_center_size(row.center(), egui::vec2(width, 28.0));
    let response = ui.interact(rect, ui.id().with("new-connection"), egui::Sense::click());
    let local = painter.rect(rect);
    painter.list.fill_rounded_rect(
        local,
        Radius::all(4.0),
        color(if response.hovered() {
            theme.surface_hover
        } else {
            theme.surface_elevated
        }),
    );
    painter.label(
        &char::from(lucide_icons::Icon::Plus).to_string(),
        "lucide",
        12.0,
        theme.text_primary,
        egui::pos2(rect.left() + 12.0, rect.center().y),
        0.5,
    );
    painter.label(
        "New connection",
        "Inter",
        12.0,
        theme.text_primary,
        egui::pos2(rect.left() + 25.0, rect.center().y),
        0.0,
    );
    response
}

pub(crate) fn finish(ui: &egui::Ui) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(shared) = shared(ui.ctx()) else {
        return Ok(());
    };
    let mut painter = shared.lock().expect("Explorer renderer mutex poisoned");
    painter.active = false;
    let viewport = painter.viewport;
    painter.list.pop_clip();
    let list = std::mem::take(&mut painter.list).build();
    painter.render(ui.ctx(), &list)?;
    if let Some(texture) = &painter.texture {
        ui.painter().with_clip_rect(viewport).image(
            texture.id(),
            viewport,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    Ok(())
}

fn color(value: egui::Color32) -> Color {
    let [r, g, b, a] = value.to_srgba_unmultiplied();
    Color::from_srgba8(Srgb8 { r, g, b, a })
}

struct ExplorerPainter {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: UiRenderer,
    text: TextSystem,
    runs: HashMap<TextCacheKey, TextRunId>,
    used: HashSet<TextRunId>,
    list: DisplayListBuilder,
    viewport: egui::Rect,
    active: bool,
    previous: Option<(DisplayList, [u32; 2], f32)>,
    texture: Option<egui::TextureHandle>,
}

impl ExplorerPainter {
    fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        // EGL enumeration cannot run while the Glow host owns its current context.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))?;
        tracing::info!(adapter = ?adapter.get_info(), "rs-ui Explorer renderer");
        let renderer = UiRenderer::new(
            &device,
            &queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            RendererOptions::default(),
        )?;
        let text = TextSystem::with_fonts([
            crate::theme::INTER_REGULAR.to_vec(),
            crate::theme::INTER_REGULAR_EXT.to_vec(),
            crate::theme::INTER_MEDIUM.to_vec(),
            crate::theme::INTER_MEDIUM_EXT.to_vec(),
            lucide_icons::LUCIDE_FONT_BYTES.to_vec(),
        ]);
        Ok(Self {
            device,
            queue,
            renderer,
            text,
            runs: HashMap::new(),
            used: HashSet::new(),
            list: DisplayListBuilder::new(),
            viewport: egui::Rect::NOTHING,
            active: false,
            previous: None,
            texture: None,
        })
    }

    fn rect(&self, rect: egui::Rect) -> Rect {
        Rect::from_min_size(
            Point::new(rect.min.x - self.viewport.min.x, rect.min.y - self.viewport.min.y),
            Size::new(rect.width(), rect.height()),
        )
    }

    fn label(
        &mut self,
        value: &str,
        family: &'static str,
        size: f32,
        tint: egui::Color32,
        point: egui::Pos2,
        align: f32,
    ) -> f32 {
        let (run, metrics) = self.text_run(value, family, size, tint);
        self.list.text(
            run,
            Point::new(
                point.x - self.viewport.min.x - metrics.size.width * align,
                point.y - self.viewport.min.y - metrics.size.height * 0.5,
            ),
        );
        metrics.size.width
    }

    fn text_run(
        &mut self,
        value: &str,
        family: &'static str,
        size: f32,
        tint: egui::Color32,
    ) -> (TextRunId, TextMetrics) {
        let key = (value.to_owned(), family, size.to_bits(), tint.to_array(), None);
        let style = TextStyle {
            family: if family == "monospace" {
                FontFamily::Monospace
            } else {
                FontFamily::Named(family)
            },
            size_px: size,
            line_height_px: size * 1.3,
            color: color(tint),
            weight: if family == "Inter Medium" {
                FontWeight::Medium
            } else {
                FontWeight::Regular
            },
        };
        let run = *self
            .runs
            .entry(key)
            .or_insert_with(|| self.text.shape(value, style, None));
        self.used.insert(run);
        let metrics = self.text.run_metrics(run).expect("retained Explorer text run");
        (run, metrics)
    }

    fn truncate(&mut self, value: &str, family: &'static str, size: f32, tint: egui::Color32, width: f32) -> String {
        if width <= 12.0 || self.text_run(value, family, size, tint).1.size.width <= width {
            return value.to_owned();
        }
        let chars: Vec<char> = value.chars().collect();
        let (mut low, mut high) = (0, chars.len());
        while low < high {
            let mid = (low + high).div_ceil(2);
            let candidate = chars[..mid].iter().collect::<String>() + "…";
            if self.text_run(&candidate, family, size, tint).1.size.width <= width {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        chars[..low].iter().collect::<String>() + "…"
    }

    fn paint_row(&mut self, theme: &DbProTheme, rect: egui::Rect, row: &CodexTreeRow<'_>, hovered: bool) {
        let local = self.rect(rect);
        self.list.push_clip(local);
        if row.is_selected || hovered {
            self.list.fill_rounded_rect(
                local,
                Radius::all(4.0),
                color(if row.is_selected {
                    theme.surface_active
                } else {
                    theme.surface_hover
                }),
            );
        }
        if row.is_selected {
            let pill = self.rect(egui::Rect::from_min_max(
                rect.min + egui::vec2(1.0, 4.0),
                egui::pos2(rect.min.x + 3.5, rect.max.y - 4.0),
            ));
            self.list.fill_rounded_rect(pill, Radius::all(1.2), color(theme.accent));
        }
        let y = rect.center().y;
        let mut x = rect.min.x + CODEX_ROW_PADDING + row.depth as f32 * CODEX_ROW_INDENT;
        if row.is_expandable {
            let glyph = char::from(if row.is_expanded {
                lucide_icons::Icon::ChevronDown
            } else {
                lucide_icons::Icon::ChevronRight
            })
            .to_string();
            self.label(
                &glyph,
                "lucide",
                10.5,
                if hovered {
                    theme.text_secondary
                } else {
                    theme.text_muted
                },
                egui::pos2(x + CODEX_CHEVRON_SLOT * 0.5, y),
                0.5,
            );
        }
        x += CODEX_CHEVRON_SLOT;
        if let Some(dot) = row.status_dot {
            let dot_rect = self.rect(egui::Rect::from_center_size(
                egui::pos2(x + 3.0, y),
                egui::vec2(6.0, 6.0),
            ));
            self.list.fill_rounded_rect(dot_rect, Radius::all(3.0), color(dot));
            x += 10.0;
        }
        self.label(
            &char::from(row.icon).to_string(),
            "lucide",
            13.0,
            row.icon_color,
            egui::pos2(x + 7.0, y),
            0.5,
        );
        x += 17.0;
        let mut right = rect.max.x - CODEX_ROW_PADDING;
        if let Some(badge) = row.badge_text {
            let tint = if row.badge_accent {
                theme.accent
            } else {
                theme.text_muted
            };
            let width = self.text_run(badge, "Inter", 9.5, tint).1.size.width + 8.0;
            let bg = self.rect(egui::Rect::from_min_size(
                egui::pos2(right - width, y - 8.0),
                egui::vec2(width, 16.0),
            ));
            self.list.fill_rounded_rect(
                bg,
                Radius::all(3.0),
                color(if row.badge_accent {
                    theme.accent_soft
                } else {
                    theme.surface_hover
                }),
            );
            self.label(badge, "Inter", 9.5, tint, egui::pos2(right - width * 0.5, y), 0.5);
            right -= width + 4.0;
        }
        if let Some(count) = &row.count_text {
            right -= self.label(count, "Inter", 11.0, theme.text_muted, egui::pos2(right, y), 1.0) + 4.0;
        }
        let label_clip = self.rect(egui::Rect::from_min_max(
            egui::pos2(x, rect.min.y),
            egui::pos2((right - 4.0).max(x), rect.max.y),
        ));
        self.list.push_clip(label_clip);
        let label_color = if row.is_selected {
            theme.accent
        } else if row.is_dimmed {
            theme.text_muted
        } else {
            theme.text_primary
        };
        let label = self.truncate(row.label, "Inter", 12.5, label_color, (right - 4.0 - x).max(0.0));
        let width = self.label(
            &label,
            "Inter",
            12.5,
            if row.is_selected {
                theme.accent
            } else if row.is_dimmed {
                theme.text_muted
            } else {
                theme.text_primary
            },
            egui::pos2(x, y),
            0.0,
        );
        if let Some(detail) = row.detail_text {
            let available = right - 4.0 - x - width - 6.0;
            if available > 8.0 {
                let detail = self.truncate(detail, "monospace", 10.5, theme.text_muted, available);
                self.label(
                    &detail,
                    "monospace",
                    10.5,
                    theme.text_muted,
                    egui::pos2(x + width + 6.0, y),
                    0.0,
                );
            }
        }
        self.list.pop_clip().pop_clip();
    }

    fn render(&mut self, ctx: &egui::Context, list: &DisplayList) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scale = ctx.pixels_per_point();
        let size = [
            (self.viewport.width() * scale).round().max(1.0) as u32,
            (self.viewport.height() * scale).round().max(1.0) as u32,
        ];
        if self
            .previous
            .as_ref()
            .is_some_and(|(old, old_size, old_scale)| old == list && *old_size == size && *old_scale == scale)
        {
            return Ok(());
        }
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Explorer framebuffer"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let stride = (size[0] * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Explorer readback"),
            size: stride as u64 * size[1] as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.renderer.resize(&self.device, size);
        self.renderer.render(
            RenderFrame {
                device: &self.device,
                queue: &self.queue,
                encoder: &mut encoder,
                target: &view,
                viewport: Viewport::new(size, ScaleFactor::new(scale)),
            },
            list,
            Some(&mut self.text),
        );
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(size[1]),
                },
            },
            target.size(),
        );
        let submission = self.queue.submit([encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(5)),
        })?;
        receiver.recv_timeout(Duration::from_secs(5))??;
        let bytes = buffer.slice(..).get_mapped_range()?;
        let packed: Vec<u8> = bytes
            .chunks_exact(stride as usize)
            .flat_map(|row| row[..size[0] as usize * 4].iter().copied())
            .collect();
        let image = egui::ColorImage::from_rgba_unmultiplied([size[0] as usize, size[1] as usize], &packed);
        drop(bytes);
        buffer.unmap();
        match &mut self.texture {
            Some(texture) => texture.set(image, egui::TextureOptions::NEAREST),
            None => self.texture = Some(ctx.load_texture("rs-ui Explorer", image, egui::TextureOptions::NEAREST)),
        }
        self.previous = Some((list.clone(), size, scale));
        self.runs.retain(|_, run| {
            if self.used.contains(run) {
                true
            } else {
                self.text.remove_run(*run);
                false
            }
        });
        Ok(())
    }
}
