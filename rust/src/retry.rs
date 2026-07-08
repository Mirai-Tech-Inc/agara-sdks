//! Opt-in retry policy for transient failures (429 / 5xx / transport).
//! Off by default — construct one and pass it to the client builder.

use core::time::Duration;

use rand::Rng;

/// Default first-retry delay, shared by [`RetryPolicy`] and the stream
/// reconnect backoff.
pub(crate) const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_millis(250);

/// Default backoff ceiling.
pub(crate) const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(8);

/// Default fractional jitter (±20%).
pub(crate) const DEFAULT_JITTER: f64 = 0.2;

/// Exponential backoff with jitter. A 429's `Retry-After` hint takes
/// precedence over the computed backoff when `respect_retry_after` is
/// set.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
	/// Maximum retries after the first attempt. `0` disables retrying.
	pub max_retries: u32,
	/// Delay before the first retry.
	pub initial_backoff: Duration,
	/// Ceiling on any single backoff.
	pub max_backoff: Duration,
	/// Fractional jitter applied to each backoff (`0.2` = ±20%).
	pub jitter: f64,
	/// Honor the server's `Retry-After` on a 429 instead of the computed
	/// backoff.
	pub respect_retry_after: bool,
}

impl Default for RetryPolicy {
	fn default() -> Self {
		Self {
			max_retries: 3,
			initial_backoff: DEFAULT_INITIAL_BACKOFF,
			max_backoff: DEFAULT_MAX_BACKOFF,
			jitter: DEFAULT_JITTER,
			respect_retry_after: true,
		}
	}
}

impl RetryPolicy {
	/// A policy that never retries.
	pub fn none() -> Self {
		Self { max_retries: 0, ..Self::default() }
	}

	/// Backoff for the given retry `attempt` (1-based).
	pub(crate) fn backoff(&self, attempt: u32) -> Duration {
		jittered_backoff(self.initial_backoff, self.max_backoff, self.jitter, attempt)
	}
}

/// Exponential backoff (`initial * 2^(attempt-1)`, clamped to `max`) with
/// symmetric `±jitter`. `attempt` is 1-based. Shared by the HTTP retry
/// policy and the stream reconnect loop.
pub(crate) fn jittered_backoff(
	initial: Duration,
	max: Duration,
	jitter: f64,
	attempt: u32,
) -> Duration {
	let exp = attempt.saturating_sub(1).min(32);
	let base = initial.saturating_mul(2u32.saturating_pow(exp)).min(max);
	if jitter <= 0.0 {
		return base;
	}

	let spread = base.as_secs_f64() * jitter;
	let delta = rand::rng().random_range(-spread..=spread);
	Duration::from_secs_f64((base.as_secs_f64() + delta).max(0.0))
}
