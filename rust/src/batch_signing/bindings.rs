alloy::sol! {
	interface IERC20 {
		function transfer(address to, uint256 amount) external returns (bool);
	}

	interface IConditionalTokens {
		function splitPosition(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] calldata partition, uint256 amount) external;
		function mergePositions(address collateralToken, bytes32 parentCollectionId, bytes32 conditionId, uint256[] calldata partition, uint256 amount) external;
	}
}
