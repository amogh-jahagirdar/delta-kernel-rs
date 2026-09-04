//! Integration tests for adaptive metadata table creation.
#![cfg(feature = "adaptive-metadata-in-dev")]

use delta_kernel::schema::schema_ref;
use delta_kernel::snapshot::Snapshot;
use delta_kernel::table_features::{ColumnMappingMode, TableFeature};
use delta_kernel::transaction::create_table::create_table;
use delta_kernel::DeltaResult;
use test_utils::{test_table_setup, TestCatalogCommitter};

const ADAPTIVE_METADATA_FEATURE: &str = "delta.feature.adaptiveMetadata-preview";

#[tokio::test]
async fn test_adaptive_metadata_create_reloads_v0() -> DeltaResult<()> {
    let (_temp_dir, table_path, engine) = test_table_setup()?;
    let schema = schema_ref! { nullable "id": INTEGER };

    create_table(&table_path, schema, "test")
        .with_table_properties([
            ("delta.checkpointPolicy", "v2"),
            ("delta.enableInCommitTimestamps", "true"),
            ("delta.universalFormat.enabledFormats", "iceberg"),
            ("delta.feature.vacuumProtocolCheck", "supported"),
            ("delta.feature.timestampNtz", "supported"),
            ("delta.enableTypeWidening", "true"),
            ("delta.enableIcebergWriterCompatV3", "true"),
            ("delta.enableIcebergCompatV3", "true"),
            ("delta.columnMapping.mode", "id"),
            ("delta.enableRowTracking", "true"),
            ("delta.enableDeletionVectors", "true"),
            ("delta.feature.catalogManaged", "supported"),
            ("format-version", "4"),
            (ADAPTIVE_METADATA_FEATURE, "supported"),
        ])
        .build(engine.as_ref(), Box::new(TestCatalogCommitter))?
        .commit(engine.as_ref())?
        .unwrap_committed();

    let v0 = Snapshot::builder_for(&table_path)
        .with_max_catalog_version(0)
        .build(engine.as_ref())?;
    assert_eq!(v0.version(), 0);
    assert_eq!(
        v0.table_properties().column_mapping_mode,
        Some(ColumnMappingMode::Id)
    );
    assert_eq!(v0.table_properties().enable_row_tracking, Some(true));
    assert_eq!(v0.table_properties().enable_deletion_vectors, Some(true));
    assert_eq!(
        v0.table_properties().enable_in_commit_timestamps,
        Some(true)
    );
    for feature in [
        TableFeature::AdaptiveMetadataPreview,
        TableFeature::IcebergWriterCompatV3,
        TableFeature::CatalogManaged,
        TableFeature::ColumnMapping,
        TableFeature::DeletionVectors,
        TableFeature::RowTracking,
        TableFeature::DomainMetadata,
        TableFeature::InCommitTimestamp,
    ] {
        assert!(v0.table_configuration().is_feature_enabled(&feature));
    }

    Ok(())
}
