"""Runtime key metadata and qualifier introspection work across supported Python versions."""

from typing import Literal, get_origin

import pytest
from typing_extensions import NotRequired, Required, get_type_hints, is_typeddict

from agara_sdk import catalogue, models, streaming
from tests.test_contract import sample

PUBLIC_MODELS = [
    (f"{module.__name__}.{name}", value)
    for module in (models, catalogue, streaming)
    for name, value in vars(module).items()
    if not name.startswith("_") and is_typeddict(value)
]


@pytest.mark.parametrize("name,model", PUBLIC_MODELS, ids=[name for name, _ in PUBLIC_MODELS])
def test_runtime_required_keys_agree_with_static_qualifiers(name, model):
    hints = get_type_hints(model, include_extras=True)
    optional = {key for key, annotation in hints.items() if get_origin(annotation) is NotRequired}
    assert model.__optional_keys__ == optional, name
    assert model.__required_keys__ == set(hints) - optional, name
    assert all(
        get_origin(annotation) not in (Required, NotRequired)
        for annotation in get_type_hints(model).values()
    )


@pytest.mark.parametrize(
    "annotation,expected",
    [
        (NotRequired[str], "wire-value"),
        (NotRequired[list[str]], []),
        (NotRequired[Literal["left", "right"]], "left"),
        (Required[str], "wire-value"),
    ],
)
def test_contract_fixture_builder_understands_pep655_qualifiers(annotation, expected):
    assert sample(annotation) == expected


def test_contract_samples_keep_optional_fields_in_route_coverage():
    value = sample(models.LimitOrderRequest)
    assert set(value) == set(get_type_hints(models.LimitOrderRequest))
    assert {"post_only", "expiration_unix_seconds"} <= set(value)


def test_inherited_and_forward_referenced_optional_fields():
    assert models.SignedOrderRequest.__optional_keys__ == {"post_only", "expiration_unix_seconds"}
    assert "condition_id" in catalogue.MarketDetail.__optional_keys__
    assert "market" in streaming.CurrentMarketChangedData.__required_keys__
    assert streaming.MarketResolutionCompletedData.__optional_keys__ == {"expected_cycle"}
