import json
from pathlib import Path

import httpx
import pytest
import responses

from agara_sdk import AgaraClient, batch_is_terminal
from agara_sdk.aio import AsyncAgaraClient

BASE = "https://offline.invalid"


def batch(status, **fields):
    return {
        "batch_hash": "old",
        "status": status,
        "seq": 4,
        "deadline_unix_seconds": 1900000000,
        "origin": "PRESIGNED",
        "tx_hash": None,
        "executed_at": None,
        "failure": None,
        "superseded_by_batch_hash": None,
        "heals_batch_hash": None,
        "unwound_at": None,
        "created_at": "2026-09-17T00:00:00Z",
        **fields,
    }


@responses.activate
def test_batch_unwind_and_successor_following():
    responses.get(
        BASE + "/trade/v1/batches/old", json=batch("FAILED", superseded_by_batch_hash="new")
    )
    responses.get(BASE + "/trade/v1/batches/new", json=batch("PENDING", batch_hash="new"))
    responses.get(
        BASE + "/trade/v1/batches/new",
        json=batch(
            "SETTLED", batch_hash="new", tx_hash="0xabc", executed_at="2026-09-17T00:00:00Z"
        ),
    )
    client = AgaraClient("token", BASE)
    result = client.follow_position_operation(
        {"batch_hash": "old", "status": "PENDING", "as_of": "2026-09-17T00:00:00Z"},
        poll_interval=0.001,
    )
    assert result["status"] == "SETTLED"
    assert result["batch_hash"] == "new"
    assert not batch_is_terminal(batch("FAILED"))
    assert batch_is_terminal(batch("FAILED", unwound_at="2026-09-17T00:00:00Z"))
    receipt = {
        "operation": "SPLIT",
        "condition_id": "condition",
        "relayer_transaction_id": "receipt",
        "transaction_hash": None,
        "relayer_state": "CONFIRMED",
        "as_of": "2026-09-17T00:00:00Z",
    }
    assert client.follow_position_operation(receipt) == receipt


@pytest.mark.asyncio
async def test_async_batch_group_requires_completion_marker():
    group = {
        "group_id": "id",
        "op_count": 21,
        "chunk_count": 2,
        "completed_at": None,
        "chunks": [],
        "as_of": "2026-09-17T00:00:00Z",
    }
    states = iter([group, {**group, "completed_at": "2026-09-17T00:00:01Z"}])
    async with httpx.AsyncClient(
        transport=httpx.MockTransport(lambda r: httpx.Response(200, json=next(states)))
    ) as transport:
        client = AsyncAgaraClient("token", BASE, client=transport)
        assert (await client.wait_for_batch_group("id", poll_interval=0.001))["completed_at"]


@responses.activate
def test_signed_batch_mixed_outcomes_retain_failure_contract():
    from tests.test_contract import SIGNED

    fixture = json.loads((Path(__file__).parent / "fixtures/problem-contracts.json").read_text())[
        "fixtures"
    ][0]["value"]
    failure = {k: v for k, v in fixture.items() if k in {"code", "title", "detail", "recovery"}}
    response = {
        "results": [
            {
                "index": 0,
                "outcome": "accepted",
                "order_id": "id",
                "source": "AGARA",
                "status": "PENDING",
                "pending_operation": "SUBMIT",
                "as_of": "2026-09-17T00:00:00Z",
            },
            {"index": 1, "outcome": "rejected", "failure": failure},
        ],
        "as_of": "2026-09-17T00:00:00Z",
    }
    responses.post(BASE + "/trade/v1/orders/signed/batch", json=response)
    assert AgaraClient("token", BASE).submit_signed_orders([SIGNED, SIGNED]) == response
