//! Validated, opt-in retry policy for operations the client classifies as safe reads.

use core::time::Duration;

use rand::Rng;

use crate::{
	input,
	validation::{Field, RetryPolicyError, ValidationError},
};

/// Largest finite number of automatic retries accepted by a policy.
pub const MAX_RETRIES: u32 = 1000;

pub(crate) const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_millis(250);
pub(crate) const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(8);
pub(crate) const DEFAULT_JITTER: f64 = 0.2;
const DEFAULT_RETRIES: u32 = 3;
const MAX_BACKOFF_EXPONENT: u32 = 31;

/// A finite jitter fraction in the closed interval [0, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jitter(f64);

/// A bounded count of retries after the first attempt; zero disables automatic retries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetryLimit(u32);

/// Immutable exponential-backoff policy; invalid settings cannot be built or mutated into it.
#[derive(Clone, Debug, dissolve_derive::Dissolve)]
#[dissolve(visibility = "pub(crate)")]
pub struct RetryPolicy {
	max_retries: RetryLimit,
	initial_backoff: Duration,
	max_backoff: Duration,
	jitter: Jitter,
}

pub(crate) fn jittered_backoff(
	initial: Duration,
	max: Duration,
	jitter: Jitter,
	attempt: u32,
) -> Duration {
	let exponent = attempt.saturating_sub(1).min(MAX_BACKOFF_EXPONENT);
	let base = initial.saturating_mul(2u32.saturating_pow(exponent)).min(max);
	if jitter.raw() == 0.0 {
		return base;
	}

	let spread = base.as_secs_f64() * jitter.raw();
	let delta = rand::rng().random_range(-spread..=spread);

	Duration::try_from_secs_f64((base.as_secs_f64() + delta).max(0.0)).unwrap_or(max).min(max)
}

impl Jitter {
	/// Reject NaN, infinities and fractions outside [0, 1].
	pub fn new(value: f64) -> Result<Self, RetryPolicyError> {
		if !value.is_finite() || !(0.0..=1.0).contains(&value) {
			return Err(RetryPolicyError::InvalidJitter);
		}

		Ok(Self(value))
	}

	/// Return the validated fraction; no mutable access to the inner value is exposed.
	pub const fn raw(self) -> f64 {
		self.0
	}
}

impl RetryLimit {
	/// Construct a count in the inclusive range zero through [`MAX_RETRIES`].
	pub fn new(value: u32) -> Result<Self, RetryPolicyError> {
		if value > MAX_RETRIES {
			return Err(RetryPolicyError::ExcessiveAttempts);
		}

		Ok(Self(value))
	}

	/// Return the maximum number of additional attempts.
	pub const fn raw(self) -> u32 {
		self.0
	}
}

#[bon::bon]
impl RetryPolicy {
	/// Build bounded backoff settings; the maximum delay must cover the initial delay.
	/// This policy is never applied automatically to order, batch or position mutations.
	#[builder]
	pub fn new(
		#[builder(default = DEFAULT_RETRIES)] max_retries: u32,
		#[builder(default = DEFAULT_INITIAL_BACKOFF)] initial_backoff: Duration,
		#[builder(default = DEFAULT_MAX_BACKOFF)] max_backoff: Duration,
		#[builder(default = DEFAULT_JITTER)] jitter: f64,
	) -> Result<Self, ValidationError> {
		let max_retries = RetryLimit::new(max_retries)?;
		let jitter = Jitter::new(jitter)?;
		if initial_backoff.is_zero() {
			return Err(RetryPolicyError::ZeroInitialBackoff.into());
		}

		if max_backoff < initial_backoff {
			return Err(RetryPolicyError::MaxBelowInitial.into());
		}

		input::duration(initial_backoff, Field::InitialBackoff)?;
		input::duration(max_backoff, Field::MaxBackoff)?;

		Ok(Self { max_retries, initial_backoff, max_backoff, jitter })
	}

	/// Disable retries while retaining valid delay settings.
	pub fn none() -> Self {
		Self { max_retries: RetryLimit(0), ..Self::default() }
	}

	/// Maximum retries after the initial request; zero means every request is sent once.
	pub const fn max_retries(&self) -> u32 {
		self.max_retries.raw()
	}

	/// Initial delay before exponential growth and jitter.
	pub const fn initial_backoff(&self) -> Duration {
		self.initial_backoff
	}

	/// Upper bound on a computed delay, including jitter.
	pub const fn max_backoff(&self) -> Duration {
		self.max_backoff
	}

	/// Fraction of randomized delay around the exponential base.
	pub const fn jitter(&self) -> f64 {
		self.jitter.raw()
	}

	pub(crate) fn backoff(&self, attempt: u32) -> Duration {
		jittered_backoff(self.initial_backoff, self.max_backoff, self.jitter, attempt)
	}
}

impl Default for RetryPolicy {
	fn default() -> Self {
		Self {
			max_retries: RetryLimit(DEFAULT_RETRIES),
			initial_backoff: DEFAULT_INITIAL_BACKOFF,
			max_backoff: DEFAULT_MAX_BACKOFF,
			jitter: Jitter(DEFAULT_JITTER),
		}
	}
}

impl Default for Jitter {
	fn default() -> Self {
		Self(DEFAULT_JITTER)
	}
}
