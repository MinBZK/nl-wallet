use crate::DeviceRequest;

impl DeviceRequest {
    /// Returns `true` if this request has any attributes at all.
    pub fn has_attributes(&self) -> bool {
        self.doc_requests
            .iter()
            .flat_map(|doc_request| doc_request.items_request.0.name_spaces.values())
            .any(|name_space| !name_space.is_empty())
    }
}
