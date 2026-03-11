//! Registry snapshot export tool.
//!
//! Parses a `RegistrySourceRecord` list, validates it, filters inactive
//! records, sorts by `manufacturer_id`, and emits a canonical
//! `Vec<ManufacturerRecord>` JSON snapshot ready for routing.

use std::collections::HashSet;

use postcad_registry::manufacturer::{
    AttestationStatus, ManufacturerCapability, ManufacturerCountry, ManufacturerMaterial,
    ManufacturerRecord,
};
use serde::Deserialize;

// ── Input format ─────────────────────────────────────────────────────────────

/// Source record format for `registry-export --input`.
///
/// A simplified, author-facing schema that maps onto [`ManufacturerRecord`].
/// Fields use the names defined in the export spec; optional fields default
/// to sensible values when absent.
///
/// | Source field          | Output field (`ManufacturerRecord`)  | Default when absent         |
/// |-----------------------|--------------------------------------|-----------------------------|
/// | `manufacturer_id`     | `manufacturer_id`                    | —                           |
/// | `country`             | `country`                            | —                           |
/// | `capabilities`        | `capabilities`                       | —                           |
/// | `materials`           | `materials_supported`                | —                           |
/// | `certifications`      | `attestation_statuses`               | —                           |
/// | `active`              | `is_active`                          | —                           |
/// | `display_name`        | `display_name`                       | `manufacturer_id`           |
/// | `jurisdictions_served`| `jurisdictions_served`               | `[country]`                 |
/// | `sla_days`            | `sla_days`                           | `0`                         |
#[derive(Debug, Deserialize)]
pub struct RegistrySourceRecord {
    pub manufacturer_id: String,
    pub country: ManufacturerCountry,
    pub capabilities: Vec<ManufacturerCapability>,
    pub materials: Vec<ManufacturerMaterial>,
    pub certifications: Vec<AttestationStatus>,
    pub active: bool,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub jurisdictions_served: Option<Vec<ManufacturerCountry>>,
    #[serde(default)]
    pub sla_days: Option<u32>,
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors returned by [`export_registry`].
#[derive(Debug)]
pub enum ExportError {
    ParseFailed(String),
    DuplicateManufacturerId(String),
    EmptyCapabilities(String),
    EmptyMaterials(String),
}

impl ExportError {
    pub fn code(&self) -> &'static str {
        match self {
            ExportError::ParseFailed(_) => "registry_parse_failed",
            ExportError::DuplicateManufacturerId(_) => "duplicate_manufacturer_id",
            ExportError::EmptyCapabilities(_) => "empty_capabilities",
            ExportError::EmptyMaterials(_) => "empty_materials",
        }
    }
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::ParseFailed(msg) => write!(f, "parse failed: {}", msg),
            ExportError::DuplicateManufacturerId(id) => {
                write!(f, "duplicate manufacturer_id: '{}'", id)
            }
            ExportError::EmptyCapabilities(id) => {
                write!(f, "manufacturer '{}' has no capabilities", id)
            }
            ExportError::EmptyMaterials(id) => {
                write!(f, "manufacturer '{}' has no materials", id)
            }
        }
    }
}

// ── Export function ───────────────────────────────────────────────────────────

/// Convert a registry source JSON string to a canonical snapshot JSON string.
///
/// Steps applied in order:
/// 1. Parse as `Vec<RegistrySourceRecord>`.
/// 2. Validate unique `manufacturer_id` across all records (including inactive).
/// 3. Validate required fields: `capabilities` and `materials` must be non-empty.
/// 4. Filter: discard records where `active == false`.
/// 5. Sort by `manufacturer_id` (lexicographic ascending — deterministic).
/// 6. Map to [`ManufacturerRecord`] and serialize as pretty-printed JSON.
pub fn export_registry(source_json: &str) -> Result<String, ExportError> {
    let records: Vec<RegistrySourceRecord> =
        serde_json::from_str(source_json).map_err(|e| ExportError::ParseFailed(e.to_string()))?;

    // Uniqueness check across all records (active and inactive).
    let mut seen = HashSet::new();
    for r in &records {
        if !seen.insert(r.manufacturer_id.clone()) {
            return Err(ExportError::DuplicateManufacturerId(
                r.manufacturer_id.clone(),
            ));
        }
    }

    // Required-field validation.
    for r in &records {
        if r.capabilities.is_empty() {
            return Err(ExportError::EmptyCapabilities(r.manufacturer_id.clone()));
        }
        if r.materials.is_empty() {
            return Err(ExportError::EmptyMaterials(r.manufacturer_id.clone()));
        }
    }

    // Filter inactive, then sort.
    let mut active: Vec<&RegistrySourceRecord> = records.iter().filter(|r| r.active).collect();
    active.sort_by(|a, b| a.manufacturer_id.cmp(&b.manufacturer_id));

    // Map to ManufacturerRecord.
    let snapshot: Vec<ManufacturerRecord> = active
        .iter()
        .map(|r| {
            let display_name = r
                .display_name
                .clone()
                .unwrap_or_else(|| r.manufacturer_id.clone());
            let jurisdictions_served = r
                .jurisdictions_served
                .clone()
                .unwrap_or_else(|| vec![r.country.clone()]);
            let sla_days = r.sla_days.unwrap_or(0);
            ManufacturerRecord {
                attestation_statuses: r.certifications.clone(),
                capabilities: r.capabilities.clone(),
                country: r.country.clone(),
                display_name,
                is_active: true,
                jurisdictions_served,
                manufacturer_id: r.manufacturer_id.clone(),
                materials_supported: r.materials.clone(),
                sla_days,
            }
        })
        .collect();

    serde_json::to_string_pretty(&snapshot).map_err(|e| ExportError::ParseFailed(e.to_string()))
}
