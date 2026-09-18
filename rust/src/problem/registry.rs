mod first;
mod second;

use super::{KnownProblemCode, RecoveryStrategy, ResourceKind, WebSocketAction};

pub(super) struct Metadata {
	pub(super) title: &'static str,
	pub(super) detail: Option<&'static str>,
	pub(super) status: Option<u16>,
	pub(super) recovery: RecoveryStrategy,
	pub(super) resource: Option<ResourceKind>,
	pub(super) origin: bool,
	pub(super) edge: bool,
	pub(super) websocket: Option<WebSocketAction>,
}

pub(super) fn metadata(code: KnownProblemCode) -> Metadata {
	first::metadata(code).unwrap_or_else(|| second::metadata(code))
}
