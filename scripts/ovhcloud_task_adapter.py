#!/usr/bin/env python3
"""Normalize source-locked OVHcloud asynchronous task evidence."""

from __future__ import annotations

import hashlib
import json
from typing import Any

from ovhcloud_probe_error import OvhcloudProbeError


TASK_CANDIDATE_PATHS = (
    "/notification/contactMean/{contactMeanId}/task",
    "/notification/contactMean/{contactMeanId}/task/{taskId}",
)
TASK_MODEL_NAMES = (
    "common.Task",
    "common.TaskError",
    "common.TaskProgress",
    "common.TaskStatusEnum",
)
TASK_FIELDS = (
    "createdAt",
    "errors",
    "finishedAt",
    "id",
    "link",
    "message",
    "progress",
    "startedAt",
    "status",
    "type",
    "updatedAt",
)
TASK_ERROR_FIELDS = ("message",)
TASK_PROGRESS_FIELDS = ("name", "status")
TASK_STATUSES = (
    "DONE",
    "ERROR",
    "PENDING",
    "RUNNING",
    "SCHEDULED",
    "WAITING_USER_INPUT",
)


def _digest(value: Any) -> str:
    encoded = json.dumps(
        value, ensure_ascii=True, sort_keys=True, separators=(",", ":")
    ).encode("ascii")
    return hashlib.sha256(encoded).hexdigest()


def _candidate_id(path: str) -> str:
    return (
        path.removeprefix("/")
        .lower()
        .replace("/{", "/by-")
        .replace("}", "")
    )


def _schema_models(schema: dict[str, Any]) -> dict[str, Any]:
    models = schema.get("models")
    if not isinstance(models, dict):
        raise OvhcloudProbeError("OVHcloud task models are invalid")
    selected = {}
    for name in TASK_MODEL_NAMES:
        model = models.get(name)
        if not isinstance(model, dict):
            raise OvhcloudProbeError("OVHcloud task model is unavailable")
        selected[name] = model
    return selected


def _property_contract(
    model: dict[str, Any], expected: tuple[str, ...]
) -> list[dict[str, Any]]:
    properties = model.get("properties")
    if not isinstance(properties, dict) or tuple(sorted(properties)) != expected:
        raise OvhcloudProbeError("OVHcloud task model fields changed")
    contract = []
    for name, value in sorted(properties.items()):
        if (
            not isinstance(value, dict)
            or not isinstance(value.get("type"), str)
            or type(value.get("canBeNull")) is not bool
            or type(value.get("required")) is not bool
        ):
            raise OvhcloudProbeError("OVHcloud task field is invalid")
        contract.append({
            "name": name,
            "nullable": value["canBeNull"],
            "required": value["required"],
            "type": value["type"],
        })
    return contract


def task_model_evidence(
    schema: dict[str, Any], schema_sha256: str
) -> dict[str, Any]:
    """Bind the exact common task, error, progress, and status models."""
    selected = _schema_models(schema)
    property_contracts = [
        {
            "id": "common-task",
            "model": "common.Task",
            "properties": _property_contract(selected["common.Task"], TASK_FIELDS),
        },
        {
            "id": "common-task-error",
            "model": "common.TaskError",
            "properties": _property_contract(
                selected["common.TaskError"], TASK_ERROR_FIELDS
            ),
        },
        {
            "id": "common-task-progress",
            "model": "common.TaskProgress",
            "properties": _property_contract(
                selected["common.TaskProgress"], TASK_PROGRESS_FIELDS
            ),
        },
    ]
    status = selected["common.TaskStatusEnum"]
    if status.get("enumType") != "string" or tuple(status.get("enum", ())) != TASK_STATUSES:
        raise OvhcloudProbeError("OVHcloud task statuses changed")
    rows = [{"model": selected[name], "name": name} for name in TASK_MODEL_NAMES]
    return {
        "count": len(rows),
        "error_fields": list(TASK_ERROR_FIELDS),
        "model_sha256": _digest(rows),
        "progress_fields": list(TASK_PROGRESS_FIELDS),
        "property_contracts": property_contracts,
        "source_sha256": schema_sha256,
        "statuses": list(TASK_STATUSES),
        "task_fields": list(TASK_FIELDS),
    }


def task_operations(schema: dict[str, Any]) -> list[dict[str, Any]]:
    """Select the two reviewed stable read-only task routes."""
    if (
        schema.get("basePath") != "https://api.eu.ovhcloud.com/v2"
        or schema.get("apiVersion") != "1.0"
    ):
        raise OvhcloudProbeError("OVHcloud task schema identity is invalid")
    apis = schema.get("apis")
    if not isinstance(apis, list):
        raise OvhcloudProbeError("OVHcloud task operations are invalid")
    by_path = {}
    for item in apis:
        if not isinstance(item, dict) or not isinstance(item.get("path"), str):
            raise OvhcloudProbeError("OVHcloud task path is invalid")
        operations = item.get("operations")
        if not isinstance(operations, list):
            raise OvhcloudProbeError("OVHcloud task operation is invalid")
        by_path[item["path"]] = operations

    selected = []
    for path in TASK_CANDIDATE_PATHS:
        matches = [
            operation
            for operation in by_path.get(path, [])
            if isinstance(operation, dict) and operation.get("httpMethod") == "GET"
        ]
        if len(matches) != 1:
            raise OvhcloudProbeError("OVHcloud task route is unavailable")
        operation = matches[0]
        status = operation.get("apiStatus")
        parameters = operation.get("parameters")
        if (
            not isinstance(status, dict)
            or status.get("value") != "PRODUCTION"
            or operation.get("noAuthentication") is not False
            or not isinstance(operation.get("responseType"), str)
            or not isinstance(parameters, list)
        ):
            raise OvhcloudProbeError("OVHcloud task route is not stable read-only")
        headers = []
        path_parameters = []
        for parameter in parameters:
            if not isinstance(parameter, dict) or not isinstance(
                parameter.get("paramType"), str
            ):
                raise OvhcloudProbeError("OVHcloud task parameter is invalid")
            if parameter["paramType"] in ("header", "path"):
                name = parameter.get("name")
                if not isinstance(name, str):
                    raise OvhcloudProbeError("OVHcloud task parameter name is invalid")
                (headers if parameter["paramType"] == "header" else path_parameters).append(name)
        actions = operation.get("iamActions")
        if not isinstance(actions, list) or len(actions) != 1:
            raise OvhcloudProbeError("OVHcloud task action is invalid")
        action = actions[0]
        if not isinstance(action, dict) or not isinstance(action.get("name"), str):
            raise OvhcloudProbeError("OVHcloud task action is invalid")
        selected.append(
            {
                "id": _candidate_id(path),
                "values": {
                    "actions": [action["name"]],
                    "authenticated": True,
                    "headers": sorted(headers),
                    "method": "GET",
                    "path": path,
                    "path_parameters": sorted(path_parameters),
                    "response_type": operation["responseType"],
                    "stability": "production",
                },
            }
        )
    return selected
