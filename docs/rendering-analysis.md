# Rendering pipeline analysis

## Scope and source revision

This checkout is DB Pro, not an egui source fork. At source revision `0f1b34dbb147ce130a6ae5899c2523f816adb0b`, `Cargo.lock` resolves `egui`, `epaint`, `ecolor`, `eframe`, and `egui_glow` to `0.29.1`. `db-pro-native` activates eframe's `glow` feature and does not activate `wgpu`; therefore Glow is the shipped path. The crate source used below is the Cargo registry copy for `0.29.1`, not a modified local fork.

## Active path: Glow

| Stage | Representation and operation | Source |
|---|---|---|
| App color | `Color32` holds 8-bit sRGB-encoded RGB and linear alpha. `from_rgba_unmultiplied` decodes RGB, multiplies linear RGB by alpha, then encodes RGB to sRGB. `from_rgba_premultiplied` accepts the already premultiplied byte representation unchanged. | `ecolor-0.29.1/src/color32.rs`, `Color32` docs and constructors |
| Shape | Shapes carry `Color32`; tessellation expands paths, rectangles, strokes, and text into clipped meshes. | `epaint-0.29.1/src/tessellator.rs` |
| Vertex | Mesh vertices carry point-space position, normalized UV, and `Color32` described as premultiplied sRGBA. The Glow VBO uploads this color as four normalized bytes. | `epaint-0.29.1/src/mesh.rs`; `egui_glow-0.29.1/src/painter.rs`, `paint_mesh` |
| Texture | Font atlas data is converted to sRGBA bytes. Color images follow the same upload path. When the GL context supports sRGB textures, Glow allocates `SRGB8_ALPHA8`; otherwise it allocates `RGBA8`. The alpha channel is not sRGB encoded. | `epaint-0.29.1/src/image.rs`; `egui_glow-0.29.1/src/painter.rs`, `upload_texture_srgb` |
| Sampling | Texture minification and magnification use the `TextureOptions` selected by egui. The default is linear filtering. sRGB texture reads decode RGB to linear before hardware filtering; the fragment shader re-encodes the sampled value before multiplying it by the vertex color. | `epaint-0.29.1/src/textures.rs`; Glow `shader/fragment.glsl` |
| Shader | Vertex colors are passed as gamma-encoded byte values. Fragment output multiplies the vertex color and texture color in gamma space. For sRGB texture views, the shader explicitly encodes sampled linear RGB back to sRGB first. | Glow `shader/vertex.glsl`, `shader/fragment.glsl` |
| Blend | `FUNC_ADD`, premultiplied-alpha RGB factors `(ONE, ONE_MINUS_SRC_ALPHA)`. Alpha uses `(ONE_MINUS_DST_ALPHA, ONE)`. | `egui_glow-0.29.1/src/painter.rs`, `prepare_painting` |
| Framebuffer | If the GL context advertises `ARB_framebuffer_sRGB`, the painter disables `FRAMEBUFFER_SRGB` before drawing. This preserves the renderer's gamma-space shader/blend contract and avoids a framebuffer encode on top. eframe's GL surface config is platform-selected; egui_glow does not request a particular transfer encoding here. | Glow `painter.rs`, `prepare_painting`; eframe `native/glow_integration.rs` |
| Display | The resulting framebuffer bytes are presented by the platform window system. The app does not add a final shader conversion. | eframe native Glow integration |

### Premultiplication and color-space finding

The vertex data is not produced by simply multiplying sRGB byte values by alpha. `from_rgba_unmultiplied` performs the multiplication in linear light and then stores the sRGB encoding of that premultiplied result. The active Glow shader and blend state then combine those encoded values in gamma space. This makes the effective pipeline a deliberate hybrid: linear-premultiplied color storage, gamma-space texture/color multiplication, and gamma-space source-over blending. It is not the requested linear-light pipeline, and its behavior differs by foreground/background polarity for translucent edges. A linear renderer cannot be implemented by only changing the blend factors; it must decode the premultiplied RGB consistently, preserve texture coverage semantics, blend in linear space, and encode exactly once at output.

The current renderer is not using straight-alpha input with premultiplied blend factors: normal `Color32` construction premultiplies before tessellation. Raw use of `from_rgba_premultiplied` is an API contract and can still be incorrect if callers pass straight-alpha bytes.

### sRGB conversion risks

- **Texture decode and shader encode:** on sRGB-capable GL, sampling decodes RGB to linear and the shader encodes it back to sRGB. That is an intentional round trip in 0.29.1. Removing either side alone changes the shader's gamma-space arithmetic.
- **Framebuffer conversion:** egui_glow disables `FRAMEBUFFER_SRGB` when supported. If an application callback enables it after egui's state setup and fails to restore it, output may be encoded again. That is an integration-state risk, not demonstrated for DB Pro's callbacks.
- **Color premultiplication:** unmultiplied `Color32` inputs are premultiplied in linear space before storage. Treating stored channels as straight sRGB and premultiplying again would be a second, incorrect multiplication.
- **Fallback texture format:** `RGBA8` sampling does not perform hardware sRGB decode; the shader correspondingly skips the encode step because those bytes are already gamma encoded. Do not force the sRGB shader branch on that fallback.

### WGPU path (inactive in DB Pro)

The locked `egui-wgpu 0.29.1` renderer uses a `Uint32` vertex color and unpacks its sRGBA bytes in WGSL. Its preferred output formats are `Rgba8Unorm`/`Bgra8Unorm`, which preserve gamma-space bytes. It uses the same premultiplied RGB blend factors. If given an sRGB output target, it selects a separate shader entry point that converts fragment RGB to linear before target blending. egui-wgpu uploads egui-managed color textures as `Rgba8UnormSrgb`. These details do not describe the running DB Pro window because the WGPU feature is not active.

## Geometry, text, and filtering

- egui/epaint layout and mesh positions are logical points. eframe supplies native pixels-per-point, and `Context::pixels_per_point()` is the product of native display scale and user zoom. Glow's screen transform converts point positions to clip space from the physical framebuffer dimensions divided by that same scale.
- The text rasterizer rounds glyph positions/cursors to the physical pixel grid using the current pixels-per-point. Text atlas entries are coverage images and use linear texture filtering by default. Text therefore has grayscale coverage antialiasing; it is not rendered with MSAA.
- Strokes and rectangle edges are tessellated as geometry; fractional physical-pixel placement can spread coverage over adjacent pixels. A 1-point stroke is not necessarily one physical pixel at non-1× scale. Pixel alignment must use the live pixels-per-point and target only geometry, not text layout.
- `NativeOptions::multisampling` defaults to `0`, which means off. MSAA is therefore not enabled in this app's default native configuration. egui's own path antialiasing uses tessellated feathering.
- Linear texture sampling can soften scaled images and can change edge coverage. Nearest sampling is a useful diagnostic for texture sampling, but this Glow integration does not expose a global runtime sampler switch for the font atlas. The WGPU-only `predictable_texture_filtering` option cannot change the active backend.

## Diagnostic added

The Component Gallery now has a **Rendering diagnostics** page. It shows alpha samples over five backgrounds, a computed linear-light reference, 0.5/1/1.5/2 physical-pixel geometry, live DPI values, text sizes/families/alpha, and panel/hover/selection/tooltip/modal examples. The computed row is an expected-color reference rendered as opaque swatches; it does not switch the production renderer. The screen explicitly reports that Glow sampler and MSAA cannot be toggled there.

## Remaining evidence gaps

- No egui fork checkout exists in the searched workspace locations; the actual dependency is crates.io `0.29.1`.
- The current native host does not expose the selected GL surface transfer format in app state, and this environment has not produced a desktop capture from the diagnostics page yet.
- Nearest-vs-linear texture sampling, runtime MSAA on/off, and before/after linear renderer comparison require renderer-level hooks or a locally maintained egui/eframe fork.
- Windows HiDPI and macOS Retina runtime evidence are not available from the Linux source/build check.
