//! Numeric catalogue layout hints retain their published wire representation.

use agara_sdk::catalogue::{CategorySectionMeta, GridColumnsHint};

#[rstest::rstest]
#[case(2, GridColumnsHint::Two)]
#[case(3, GridColumnsHint::Three)]
#[case(4, GridColumnsHint::Four)]
fn grid_columns_use_the_published_numeric_literals(
	#[case] count: u8,
	#[case] expected: GridColumnsHint,
) {
	// Arrange
	let value = serde_json::json!(count);

	// Act
	let decoded: GridColumnsHint = serde_json::from_value(value.clone()).unwrap();

	// Assert
	core::assert_eq!(decoded, expected);
	core::assert_eq!(decoded.columns(), count);
	core::assert_eq!(serde_json::to_value(decoded).unwrap(), value);
}

#[rstest::rstest]
#[case(serde_json::json!("2"))]
#[case(serde_json::json!(0))]
#[case(serde_json::json!(5))]
#[case(serde_json::json!(-1))]
#[case(serde_json::json!(2.0))]
#[case(serde_json::json!(true))]
fn unsupported_grid_column_representations_are_rejected(#[case] value: serde_json::Value) {
	// Arrange

	// Act
	let decoded = serde_json::from_value::<GridColumnsHint>(value);

	// Assert
	core::assert!(decoded.is_err());
}

#[test]
fn populated_category_section_metadata_decodes_without_losing_numeric_columns() {
	// Arrange
	let value = serde_json::json!({
		"selector_axis":"line", "default_line":1.5, "line_owner_side":"first", "grid_cols_hint":3,
	});

	// Act
	let metadata: CategorySectionMeta = serde_json::from_value(value.clone()).unwrap();

	// Assert
	core::assert_eq!(metadata.grid_cols_hint, Some(GridColumnsHint::Three));
	core::assert_eq!(metadata.selector_axis.as_deref(), Some("line"));
	core::assert_eq!(metadata.default_line, Some(1.5));
	core::assert_eq!(metadata.line_owner_side.as_deref(), Some("first"));
	core::assert_eq!(serde_json::to_value(metadata).unwrap(), value);
}
