/// Ratios at or below this value would produce unstable or unusable heights.
pub const MIN_RATIO: f32 = 0.001;

/// Safe square fallback used when callers pass a non-finite, zero, negative, or tiny ratio.
pub const FALLBACK_RATIO: f32 = 1.0;
