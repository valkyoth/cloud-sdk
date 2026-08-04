//! Source-locked operation-to-response bindings.

use cloud_sdk::ServiceId;

use crate::identity::{CLOUD_SERVICE_ID, STORAGE_SERVICE_ID};

const TABLE: &str = include_str!("response_operations.tsv");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResponseShape {
    Empty,
    Action,
    Actions,
    ActionsPage,
    Resource,
    ResourceList,
    ResourcePage,
    Composite,
    Metrics,
    ZoneFile,
    Pricing,
    Folders,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResponseBinding {
    pub(super) service_id: ServiceId,
    pub(super) status: u16,
    pub(super) shape: ResponseShape,
    pub(super) root: &'static str,
    pub(super) required: &'static str,
}

pub(super) fn find(operation_id: &str) -> Option<ResponseBinding> {
    TABLE.lines().skip(1).find_map(|line| {
        let mut fields = line.split('\t');
        let api = fields.next()?;
        let operation = fields.next()?;
        let status = fields.next()?.parse::<u16>().ok()?;
        let shape = parse_shape(fields.next()?)?;
        let root = fields.next()?;
        let required = fields.next()?;
        if fields.next().is_some() || operation != operation_id {
            return None;
        }
        let service_id = match api {
            "cloud" => CLOUD_SERVICE_ID,
            "hetzner" => STORAGE_SERVICE_ID,
            _ => return None,
        };
        Some(ResponseBinding {
            service_id,
            status,
            shape,
            root,
            required,
        })
    })
}

fn parse_shape(value: &str) -> Option<ResponseShape> {
    match value {
        "empty" => Some(ResponseShape::Empty),
        "action" => Some(ResponseShape::Action),
        "actions" => Some(ResponseShape::Actions),
        "actions-page" => Some(ResponseShape::ActionsPage),
        "resource" => Some(ResponseShape::Resource),
        "resource-list" => Some(ResponseShape::ResourceList),
        "resource-page" => Some(ResponseShape::ResourcePage),
        "composite" => Some(ResponseShape::Composite),
        "metrics" => Some(ResponseShape::Metrics),
        "zonefile" => Some(ResponseShape::ZoneFile),
        "pricing" => Some(ResponseShape::Pricing),
        "folders" => Some(ResponseShape::Folders),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ResponseShape, TABLE, find};
    use crate::identity::{CLOUD_SERVICE_ID, STORAGE_SERVICE_ID};

    #[test]
    fn table_has_one_parseable_binding_per_active_operation() {
        assert_eq!(TABLE.lines().skip(1).count(), 208);
        for line in TABLE.lines().skip(1) {
            let operation = line.split('\t').nth(1);
            assert!(
                operation.and_then(find).is_some(),
                "invalid binding: {line}"
            );
        }
    }

    #[test]
    fn representative_bindings_are_exact() {
        let server = find("get_server");
        assert!(server.is_some());
        let Some(server) = server else {
            unreachable!("security fixture construction failed")
        };
        assert_eq!(server.service_id, CLOUD_SERVICE_ID);
        assert_eq!(server.status, 200);
        assert_eq!(server.shape, ResponseShape::Resource);
        assert_eq!(server.root, "server");

        let storage = find("list_storage_boxes");
        assert!(storage.is_some());
        let Some(storage) = storage else {
            unreachable!("security fixture construction failed")
        };
        assert_eq!(storage.service_id, STORAGE_SERVICE_ID);
        assert_eq!(storage.shape, ResponseShape::ResourcePage);
    }
}
