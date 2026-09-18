import json
from pathlib import Path

import pytest

from agara_sdk import ProblemDetails, PublicFailure, streaming
from agara_sdk._problem_registry import METADATA

FIXTURES = json.loads((Path(__file__).parent / "fixtures/problem-contracts.json").read_text())[
    "fixtures"
]


@pytest.mark.parametrize("fixture", FIXTURES, ids=lambda f: f["name"])
def test_canonical_fixture_recovery_is_conservative(fixture):
    body = fixture["value"]
    schema = fixture["schema"]
    if schema == "websocket-failure-frame.schema.json":
        frame = streaming.decode_frame(body)
        if isinstance(frame, streaming.StreamError):
            failure = frame.failure
            assert failure is not None
            if not fixture["contract_valid"] or body["failure"]["code"].startswith("future"):
                if fixture["name"] == "websocket action mismatch":
                    assert (
                        frame.action,
                        frame.failure.recovery.strategy,
                    ) != streaming._ACTIVE_ACTION_BY_CODE.get(frame.failure.code)
                else:
                    assert not failure.recovery.known or isinstance(frame, streaming.UnknownFrame)
        else:
            assert not fixture["schema_valid"]
        return
    if schema in {"origin-problem-details.schema.json", "edge-problem.schema.json"}:
        failure = ProblemDetails.from_wire(body)
        if failure is None:
            assert not fixture["schema_valid"]
            return
        # An edge-shaped fixture with request_id is also a valid origin representation.
        if fixture["name"] == "edge cannot claim origin request id":
            return
    else:
        failure = PublicFailure.from_wire(body)
    if not fixture["contract_valid"] or body["code"].startswith("future"):
        assert not failure.recovery.known
    else:
        assert failure.recovery.known


@pytest.mark.parametrize(
    "code",
    [
        code
        for code, m in METADATA.items()
        if m["recovery"]["strategy"] in {"retry", "retry_after"} and m["http_status"]
    ],
)
def test_all_registered_retry_codes_require_exact_contract(code):
    metadata = METADATA[code]
    recovery = {"strategy": metadata["recovery"]["strategy"]}
    if recovery["strategy"] == "retry_after":
        recovery["after_seconds"] = 2
    body = {
        "type": metadata["urn"],
        "title": metadata["title"],
        "status": metadata["http_status"],
        "code": code,
        "request_id": "10000000-0000-4000-8000-000000000001",
        "recovery": recovery,
    }
    assert ProblemDetails.from_wire(body).recovery.is_retryable
    for patch in [
        {"type": "urn:agara:problem:wrong"},
        {"status": 400},
        {"request_id": "invalid"},
        {"recovery": {"strategy": "retry_after" if recovery["strategy"] == "retry" else "retry"}},
    ]:
        assert not ProblemDetails.from_wire({**body, **patch}).recovery.is_retryable
