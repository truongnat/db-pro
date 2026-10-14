//! Capability resolution for the active connection / driver label.
use db_pro_core::domain::capabilities::DatabaseCapabilities;
use db_pro_core::domain::connection::DriverType;

/// The answer to "which capabilities apply to this connection?".
///
/// Deliberately not an `Option`: a lookup that cannot answer has to say *why*, so a
/// driver the UI has no provider entry for is a named state instead of `None`. `None`
/// conflated two different situations — "no connection is active" and "this driver has
/// no capability entry" — and a consumer could not tell them apart from a genuine
/// "the provider supports nothing".
#[derive(Debug, Clone)]
pub(crate) enum CapabilityLookup {
    /// The driver is a provider this build ships; the set is authoritative.
    Supported(DatabaseCapabilities),
    /// Nothing is connected, so there is nothing to resolve.
    NoActiveConnection,
    /// The connection names a driver with no provider entry in this build.
    UnsupportedDriver { driver: String },
}

impl CapabilityLookup {
    /// Resolves a connection's driver label to the provider entry the UI dispatches on.
    ///
    /// The label is what the runtime stored on the connection summary (`PostgreSQL`,
    /// `SQLite`, `MySQL`). A label with no entry resolves to
    /// [`CapabilityLookup::UnsupportedDriver`] rather than to a default driver's set.
    pub(crate) fn for_driver_label(label: &str) -> Self {
        let driver = if label.eq_ignore_ascii_case("sqlite") {
            Some(DriverType::SQLite)
        } else if label.eq_ignore_ascii_case("postgresql") || label.eq_ignore_ascii_case("postgres") {
            Some(DriverType::Postgres)
        } else if label.eq_ignore_ascii_case("mysql") {
            Some(DriverType::Mysql)
        } else if label.eq_ignore_ascii_case("sql server") || label.eq_ignore_ascii_case("sqlserver") {
            Some(DriverType::SqlServer)
        } else {
            None
        };
        match driver {
            Some(driver) => Self::Supported(DatabaseCapabilities::for_driver(driver)),
            None => Self::UnsupportedDriver {
                driver: label.to_owned(),
            },
        }
    }

    /// The resolved capability set, when one exists.
    pub(crate) fn resolved(&self) -> Option<&DatabaseCapabilities> {
        match self {
            Self::Supported(capabilities) => Some(capabilities),
            Self::NoActiveConnection | Self::UnsupportedDriver { .. } => None,
        }
    }

    /// Whether the resolved set satisfies `predicate`.
    ///
    /// A lookup that could not answer never satisfies a capability predicate, so an
    /// unresolved driver never enables a capability-gated action.
    pub(crate) fn allows(&self, predicate: impl FnOnce(&DatabaseCapabilities) -> bool) -> bool {
        self.resolved().is_some_and(predicate)
    }

    /// A user-facing reason this lookup has no capability set, if it has none.
    pub(crate) fn unavailable_reason(&self) -> Option<String> {
        match self {
            Self::Supported(_) => None,
            Self::NoActiveConnection => Some("no database connection is active".to_owned()),
            Self::UnsupportedDriver { driver } => Some(format!("{driver} has no provider entry in this build")),
        }
    }

    /// Capability-level reason a feature is unavailable for the resolved driver.
    ///
    /// Lookup-level failures (no connection / unknown driver) take precedence so
    /// consumers can use one call site for gated actions.
    pub(crate) fn feature_limitation(
        &self,
        feature: db_pro_core::domain::capabilities::CapabilityFeature,
    ) -> Option<String> {
        if let Some(reason) = self.unavailable_reason() {
            return Some(reason);
        }
        self.resolved()
            .and_then(|caps| caps.limitation(feature))
            .map(str::to_owned)
    }
}
