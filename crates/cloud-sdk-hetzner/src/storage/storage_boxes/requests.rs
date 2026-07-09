//! Storage Box request marker structs and query builders.

use crate::actions::{ActionId, ActionStatus};
use crate::cloud::shared::CloudQueryWriter;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

use super::types::{
    StorageBoxActionSortField, StorageBoxName, StorageBoxRequestError, StorageBoxSnapshotSortField,
    StorageBoxSortField, StorageBoxSubaccountSortField, StorageBoxSubaccountUsername,
};

/// Storage Box list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxListRequest<'a> {
    name: Option<StorageBoxName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(StorageBoxSortField, SortDirection)>,
}

impl<'a> StorageBoxListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per_page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(mut self, field: StorageBoxSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", u64::from(page.get()))?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", storage_box_sort(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for StorageBoxListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box type list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxTypeListRequest<'a> {
    name: Option<StorageBoxName<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
}

impl<'a> StorageBoxTypeListRequest<'a> {
    /// Creates an empty type-list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            page: None,
            per_page: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per_page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", u64::from(page.get()))?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        Ok(writer.len())
    }
}

impl Default for StorageBoxTypeListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box action list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxActionListRequest {
    action_id: Option<ActionId>,
    status: Option<ActionStatus>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(StorageBoxActionSortField, SortDirection)>,
}

impl StorageBoxActionListRequest {
    /// Creates an empty action-list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            action_id: None,
            status: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets action ID filtering for the global action list endpoint.
    #[must_use]
    pub const fn with_action_id(mut self, id: ActionId) -> Self {
        self.action_id = Some(id);
        self
    }

    /// Sets action status filtering.
    #[must_use]
    pub const fn with_status(mut self, status: ActionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per_page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(
        mut self,
        field: StorageBoxActionSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(id) = self.action_id {
            writer.u64_pair("id", id.get())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", u64::from(page.get()))?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", action_sort(field, direction))?;
        }
        if let Some(status) = self.status {
            writer.pair("status", status.as_api_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for StorageBoxActionListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box snapshot list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotListRequest<'a> {
    name: Option<StorageBoxName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    is_automatic: Option<bool>,
    sort: Option<(StorageBoxSnapshotSortField, SortDirection)>,
}

impl<'a> StorageBoxSnapshotListRequest<'a> {
    /// Creates an empty snapshot-list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            is_automatic: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets automatic-snapshot filtering.
    #[must_use]
    pub const fn with_is_automatic(mut self, value: bool) -> Self {
        self.is_automatic = Some(value);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(
        mut self,
        field: StorageBoxSnapshotSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(value) = self.is_automatic {
            writer.pair("is_automatic", bool_str(value))?;
        }
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", snapshot_sort(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for StorageBoxSnapshotListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box subaccount list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountListRequest<'a> {
    name: Option<StorageBoxName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    username: Option<StorageBoxSubaccountUsername<'a>>,
    sort: Option<(StorageBoxSubaccountSortField, SortDirection)>,
}

impl<'a> StorageBoxSubaccountListRequest<'a> {
    /// Creates an empty subaccount-list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            username: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets exact username filtering.
    #[must_use]
    pub const fn with_username(mut self, username: StorageBoxSubaccountUsername<'a>) -> Self {
        self.username = Some(username);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(
        mut self,
        field: StorageBoxSubaccountSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", subaccount_sort(field, direction))?;
        }
        if let Some(username) = self.username {
            writer.pair("username", username.as_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for StorageBoxSubaccountListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

const fn bool_str(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

const fn storage_box_sort(field: StorageBoxSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (StorageBoxSortField::Id, SortDirection::Asc) => "id:asc",
        (StorageBoxSortField::Id, SortDirection::Desc) => "id:desc",
        (StorageBoxSortField::Name, SortDirection::Asc) => "name:asc",
        (StorageBoxSortField::Name, SortDirection::Desc) => "name:desc",
        (StorageBoxSortField::Created, SortDirection::Asc) => "created:asc",
        (StorageBoxSortField::Created, SortDirection::Desc) => "created:desc",
        (StorageBoxSortField::StatsSize, SortDirection::Asc) => "stats.size:asc",
        (StorageBoxSortField::StatsSize, SortDirection::Desc) => "stats.size:desc",
        (StorageBoxSortField::StatsSizeFilesystem, SortDirection::Asc) => {
            "stats.size_filesystem:asc"
        }
        (StorageBoxSortField::StatsSizeFilesystem, SortDirection::Desc) => {
            "stats.size_filesystem:desc"
        }
    }
}

const fn action_sort(field: StorageBoxActionSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (StorageBoxActionSortField::Id, SortDirection::Asc) => "id:asc",
        (StorageBoxActionSortField::Id, SortDirection::Desc) => "id:desc",
        (StorageBoxActionSortField::Command, SortDirection::Asc) => "command:asc",
        (StorageBoxActionSortField::Command, SortDirection::Desc) => "command:desc",
        (StorageBoxActionSortField::Status, SortDirection::Asc) => "status:asc",
        (StorageBoxActionSortField::Status, SortDirection::Desc) => "status:desc",
        (StorageBoxActionSortField::Started, SortDirection::Asc) => "started:asc",
        (StorageBoxActionSortField::Started, SortDirection::Desc) => "started:desc",
        (StorageBoxActionSortField::Finished, SortDirection::Asc) => "finished:asc",
        (StorageBoxActionSortField::Finished, SortDirection::Desc) => "finished:desc",
    }
}

const fn snapshot_sort(
    field: StorageBoxSnapshotSortField,
    direction: SortDirection,
) -> &'static str {
    match (field, direction) {
        (StorageBoxSnapshotSortField::Id, SortDirection::Asc) => "id:asc",
        (StorageBoxSnapshotSortField::Id, SortDirection::Desc) => "id:desc",
        (StorageBoxSnapshotSortField::Name, SortDirection::Asc) => "name:asc",
        (StorageBoxSnapshotSortField::Name, SortDirection::Desc) => "name:desc",
        (StorageBoxSnapshotSortField::Created, SortDirection::Asc) => "created:asc",
        (StorageBoxSnapshotSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

const fn subaccount_sort(
    field: StorageBoxSubaccountSortField,
    direction: SortDirection,
) -> &'static str {
    match (field, direction) {
        (StorageBoxSubaccountSortField::Id, SortDirection::Asc) => "id:asc",
        (StorageBoxSubaccountSortField::Id, SortDirection::Desc) => "id:desc",
        (StorageBoxSubaccountSortField::Created, SortDirection::Asc) => "created:asc",
        (StorageBoxSubaccountSortField::Created, SortDirection::Desc) => "created:desc",
    }
}
