alloy::sol! {
	struct Order {
		uint256 salt;
		address maker;
		uint256 tokenId;
		uint256 makerAmount;
		uint256 takerAmount;
		uint8 side;
		uint256 timestamp;
		bytes32 metadata;
		bytes32 builder;
	}
}
