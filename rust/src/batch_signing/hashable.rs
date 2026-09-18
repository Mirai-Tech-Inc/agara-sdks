alloy::sol! {
	struct Call {
		address target;
		uint256 value;
		bytes data;
	}

	struct Batch {
		address wallet;
		uint256 seq;
		uint256 deadline;
		Call[] calls;
	}
}
