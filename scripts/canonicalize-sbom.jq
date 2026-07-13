del(.creationInfo.created, .documentNamespace)
| .files |= sort_by(.SPDXID)
| .packages |= sort_by(.SPDXID)
| .relationships |= sort_by(
    .spdxElementId,
    .relationshipType,
    .relatedSpdxElement
)
