//! Realized-PnL values retain exact decimal USDC denominations and precision.

use core::str::FromStr;

use agara_sdk::pnl::RealizedPnlReportDto;
use rust_decimal::Decimal;

#[test]
fn realized_pnl_retains_decimal_usdc_precision_and_negative_merges() {
	// Arrange
	let body = core::include_str!("fixtures/http/get_realized_pnl.json");
	let raw: serde_json::Value = serde_json::from_str(body).unwrap();

	// Act
	let report: RealizedPnlReportDto = serde_json::from_str(body).unwrap();
	let sales = Decimal::from_str(&report.totals.all_time.sales).unwrap();
	let merges = Decimal::from_str(&report.totals.all_time.direct_merges).unwrap();
	let total = Decimal::from_str(&report.totals.all_time.total).unwrap();

	// Assert
	core::assert_eq!(report.currency, "USDC");
	core::assert_eq!(report.amount_scale, 12);
	core::assert_eq!(report.totals.all_time.sales, "4.250000000000");
	core::assert_eq!(report.totals.all_time.direct_merges, "-0.250000000000");
	core::assert_eq!(sales.checked_add(merges), Some(total));
	core::assert_eq!(total, Decimal::from(4));
	core::assert_eq!(
		report.buckets.last().unwrap().direct_merges,
		"-0.250000000000"
	);
	core::assert_eq!(serde_json::to_value(report).unwrap(), raw);
}
