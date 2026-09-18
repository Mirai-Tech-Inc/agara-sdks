"""Check public wire annotations against verbatim, pinned Rust serde declarations."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import get_args

import pytest
from typing_extensions import get_type_hints, is_typeddict

from agara_sdk import models

FIXTURES = Path(__file__).parent / "fixtures"
SOURCE = json.loads((FIXTURES / "rust-wire-dtos.json").read_text())
BINDINGS = SOURCE["bindings"]
DECLARATIONS = SOURCE["declarations"]


def declaration(name):
    raw = DECLARATIONS[name]["declaration"]
    match = re.search(r"\b(?:struct|enum) " + re.escape(name) + r"(?:<[^>]+>)? \{", raw)
    assert match is not None
    return raw[: match.start()], raw[match.end() :].rsplit("}", 1)[0]


def attribute(header, name):
    match = re.search(r"\b" + name + r'\s*=\s*"([^"]+)"', header)
    return match[1] if match else None


def renamed(name, rule, *, variant=False):
    if rule is None:
        return name
    if variant:
        words = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower().split("_")
    else:
        words = name.split("_")
    if rule == "camelCase":
        return words[0] + "".join(word[:1].upper() + word[1:] for word in words[1:])
    if rule == "SCREAMING_SNAKE_CASE":
        return "_".join(words).upper()
    if rule == "snake_case":
        return "_".join(words)
    if rule == "lowercase":
        return name.lower()
    raise AssertionError(f"unhandled serde renaming rule {rule}")


def fields(body, rule=None):
    result = {}
    annotations = []
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("#["):
            annotations.append(line)
            continue
        if not line or line.startswith("//"):
            continue
        match = re.fullmatch(r"(?:pub(?:\([^)]*\))? )?(\w+):\s*(.+?)(?:,)?", line)
        assert match, f"unhandled source DTO field: {line}"
        name, rust_type = match.groups()
        attrs = "\n".join(annotations)
        annotations = []
        if re.search(r"#\[serde\((?:skip|flatten)\)\]", attrs):
            continue
        wire_name = attribute(attrs, "rename") or renamed(name, rule)
        result[wire_name] = rust_type
    return result


def struct_fields(name):
    header, body = declaration(name)
    return fields(body, attribute(header, "rename_all"))


def variant_fields(name, variant):
    header, body = declaration(name)
    tuple_variant = re.search(r"\b" + variant + r"\((\w+)\)", body)
    if tuple_variant:
        result = struct_fields(tuple_variant[1])
    else:
        struct_variant = re.search(r"\b" + variant + r"\s*\{(.*?)\}", body, re.S)
        assert struct_variant, f"missing {name}::{variant}"
        result = fields(struct_variant[1])
    tag = attribute(header, "tag")
    assert tag is not None
    return (
        {tag: "String", **result},
        tag,
        renamed(variant, attribute(header, "rename_all"), variant=True),
    )


def expected_fields(binding):
    if "schema" in binding:
        return {
            key: "unknown"
            for variant in SOURCE["schemas"][binding["schema"]]["oneOf"]
            for key in variant["properties"]
        }
    name = binding["source_type"]
    if "variant" in binding:
        result, _, _ = variant_fields(name, binding["variant"])
    else:
        result = struct_fields(name)
    if "flattened_into" in binding:
        result.update(struct_fields(binding["flattened_into"]))
    return {key: value for key, value in result.items() if key not in binding.get("exclude", [])}


@pytest.mark.parametrize("name,binding", BINDINGS.items())
def test_every_rust_wire_model_matches_serde_field_names(name, binding):
    assert set(get_type_hints(getattr(models, name))) == set(expected_fields(binding))
    if "variant" in binding:
        _, tag, value = variant_fields(binding["source_type"], binding["variant"])
        assert get_args(get_type_hints(getattr(models, name))[tag]) == (value,)


def test_every_exported_trading_typeddict_has_independent_source_evidence():
    names = {name for name, value in vars(models).items() if is_typeddict(value)}
    assert names == set(BINDINGS)


def assert_source_object(name, value):
    expected = struct_fields(name)
    assert set(value) == set(expected)
    for field, rust_type in expected.items():
        nested = re.fullmatch(r"Vec<(\w+)>", rust_type)
        if nested and nested[1] in DECLARATIONS:
            for item in value[field]:
                assert_source_object(nested[1], item)
        elif rust_type in DECLARATIONS and DECLARATIONS[rust_type]["kind"] == "struct":
            assert_source_object(rust_type, value[field])


def test_realized_pnl_fixture_uses_actual_nested_camel_case_wire_shape():
    report = json.loads((FIXTURES / "responses.json").read_text())["responses"]["get_realized_pnl"]
    assert_source_object("RealizedPnlReportDto", report)
    assert report["amountScale"] == 12
    assert report["throughEvent"]["eventSeq"] == "9007199254740993"
    assert report["totals"]["last7Days"]["directMerges"] == "2000000000000"
    assert report["buckets"][0]["directMerges"] == "2000000000000"


def test_position_receipt_uses_actual_terminal_relayer_states():
    header, body = declaration("PositionOperationRelayerState")
    variants = re.findall(r"^\s*(\w+),", body, re.M)
    values = tuple(
        renamed(name, attribute(header, "rename_all"), variant=True) for name in variants
    )
    assert get_args(get_type_hints(models.PositionOperationReceipt)["relayer_state"]) == values
    receipt = json.loads((FIXTURES / "responses.json").read_text())["responses"]["merge_position"]
    assert receipt["relayer_state"] in values
