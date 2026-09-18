use core::time::Duration;

use reqwest::{
	Method, RequestBuilder, Response,
	header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
	error::{AgaraError, ResponseError, Result},
	input,
	problem::ProblemDetails,
};

use super::AgaraClient;

const MAX_AUTO_RETRY_WAIT: Duration = Duration::from_secs(60);
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const READ_POST_PATHS: &[&str] = &[
	"/trade/v1/orders/list",
	"/trade/v1/portfolio/positions/list",
	"/trade/v1/portfolio/open-orders/list",
];

async fn read_body(mut response: Response) -> Result<Vec<u8>> {
	let mut body = Vec::new();
	while let Some(chunk) = response.chunk().await? {
		if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(body.len()) {
			return Err(ResponseError::BodyTooLarge { limit: MAX_RESPONSE_BYTES }.into());
		}

		body.extend_from_slice(&chunk);
	}

	Ok(body)
}

impl<A> AgaraClient<A> {
	pub(crate) async fn get<T: DeserializeOwned>(
		&self,
		path: &str,
		query: &[(&str, String)],
	) -> Result<T> {
		let request = self.http.request(Method::GET, self.url(path)).query(query);

		self.execute(request, true).await
	}

	pub(crate) async fn get_query<Q: Serialize + input::Validate, T: DeserializeOwned>(
		&self,
		path: &str,
		query: &Q,
	) -> Result<T> {
		input::Validate::validate_input(query)?;
		let request = self.http.request(Method::GET, self.url(path)).query(query);

		self.execute(request, true).await
	}

	pub(super) async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
		let request = self.http.request(Method::DELETE, self.url(path));

		self.execute(request, false).await
	}

	pub(crate) async fn post<B: Serialize + input::Validate, T: DeserializeOwned>(
		&self,
		path: &str,
		body: &B,
	) -> Result<T> {
		input::Validate::validate_input(body)?;
		let request = self.http.request(Method::POST, self.url(path)).json(body);

		self.execute(request, READ_POST_PATHS.contains(&path)).await
	}

	pub(super) async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
		self.execute(self.http.request(Method::POST, self.url(path)), false).await
	}

	#[cfg(feature = "streaming")]
	pub(crate) fn stream_transport(&self) -> reqwest::Client {
		self.http.clone()
	}

	pub(crate) fn url(&self, path: &str) -> String {
		std::format!("{}{path}", self.base_url.as_str().trim_end_matches('/'))
	}

	async fn execute<T: DeserializeOwned>(
		&self,
		request: RequestBuilder,
		read_only: bool,
	) -> Result<T> {
		let mut request = request.timeout(self.timeout);
		if let Some(auth) = &self.authorization {
			request = request.header(AUTHORIZATION, auth.clone());
		}

		if self.retry.max_retries() == 0 || !read_only {
			return self.try_once(request).await;
		}

		let mut attempt = 0;
		loop {
			let cloned = request.try_clone().ok_or(ResponseError::RequestNotCloneable)?;
			match self.try_once(cloned).await {
				Ok(value) => return Ok(value),
				Err(error) => {
					if attempt >= self.retry.max_retries()
						|| (!error.is_retryable()
							&& !core::matches!(error, AgaraError::Transport(_)))
					{
						return Err(error);
					}

					attempt += 1;
					let delay = error.retry_after().unwrap_or_else(|| self.retry.backoff(attempt));
					if delay > MAX_AUTO_RETRY_WAIT {
						return Err(error);
					}

					tokio::time::sleep(delay).await;
				},
			}
		}
	}

	async fn try_once<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
		let response = request.send().await?;
		let status = response.status();
		let retry_after = super::parse_retry_after(&response);
		let content_type = response
			.headers()
			.get(CONTENT_TYPE)
			.and_then(|value| value.to_str().ok())
			.map(str::to_owned);
		let bytes = read_body(response).await?;
		if status.is_success() {
			if bytes.is_empty() {
				return Err(ResponseError::EmptyBody.into());
			}

			if !content_type.as_deref().is_some_and(|value| {
				let mime = value.split(';').next().unwrap_or_default().trim();
				mime == "application/json" || mime.ends_with("+json")
			}) {
				return Err(ResponseError::UnexpectedContentType {
					content_type: content_type.map(Into::into),
				}
				.into());
			}

			return serde_json::from_slice(&bytes)
				.map_err(|source| ResponseError::Json { source }.into());
		}

		let body: Value = serde_json::from_slice(&bytes)
			.unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
		let claims_problem = content_type
			.as_deref()
			.is_some_and(|value| value.starts_with("application/problem+json"))
			|| body.as_object().is_some_and(|value| {
				value.contains_key("type")
					|| value.contains_key("code")
					|| value.contains_key("recovery")
			});
		let problem = if claims_problem {
			let problem = ProblemDetails::parse_json(&bytes)
				.and_then(|problem| {
					problem.validate_http_status(status.as_u16())?;
					Ok(problem)
				})
				.map_err(|source| ResponseError::Problem {
					status: status.as_u16(),
					body: body.clone(),
					source,
				})?;
			Some(problem)
		} else {
			None
		};

		Err(AgaraError::from_response(
			status.as_u16(),
			body,
			retry_after,
			problem,
		))
	}
}
