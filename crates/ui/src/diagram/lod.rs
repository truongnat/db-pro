#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErLod {
    Compact,
    Standard,
    Detailed,
}

impl ErLod {
    pub fn from_zoom(zoom: f32) -> Self {
        // Compact only covers the far-zoom overview band. At mid zooms
        // (0.45-0.74) a column-less card leaves a visibly empty body, so
        // Standard — which renders up to 6 columns — takes over earlier.
        if zoom < 0.45 {
            Self::Compact
        } else if zoom < 1.15 {
            Self::Standard
        } else {
            Self::Detailed
        }
    }

    #[inline]
    pub fn shows_columns(&self) -> bool {
        match self {
            Self::Compact => false,
            Self::Standard | Self::Detailed => true,
        }
    }

    #[inline]
    pub fn shows_data_types(&self) -> bool {
        match self {
            Self::Compact => false,
            Self::Standard | Self::Detailed => true,
        }
    }

    #[inline]
    pub fn shows_edge_labels(&self) -> bool {
        match self {
            Self::Compact | Self::Standard => false,
            Self::Detailed => true,
        }
    }

    #[inline]
    pub fn max_columns(&self) -> usize {
        match self {
            Self::Compact => 0,
            Self::Standard => 6,
            Self::Detailed => 12,
        }
    }
}
