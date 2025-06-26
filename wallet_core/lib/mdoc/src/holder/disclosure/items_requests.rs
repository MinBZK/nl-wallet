use std::collections::HashMap;
use std::collections::HashSet;

use attestation_types::attribute_paths::AttestationAttributePaths;
use attestation_types::attribute_paths::AttestationAttributePathsError;
use utils::vec_at_least::VecNonEmpty;

use crate::verifier::ItemsRequests;

impl ItemsRequests {
    pub fn try_into_attribute_paths(self) -> Result<AttestationAttributePaths, AttestationAttributePathsError> {
        let Self(requests) = self;

        let paths = requests
            .into_iter()
            .fold(HashMap::<_, HashSet<_>>::new(), |mut paths, request| {
                // For an mdoc items request, simply make a path of length 2,
                // consisting of the name space and element identifier.
                let attributes = request
                    .name_spaces
                    .into_iter()
                    .flat_map(|(namespace, elements)| {
                        let element_count = elements.len();

                        elements
                            .into_keys()
                            .zip(itertools::repeat_n(namespace, element_count))
                            .map(|(element_id, namespace)| VecNonEmpty::try_from(vec![namespace, element_id]).unwrap())
                    })
                    .collect::<HashSet<_>>();

                // In case a doc type occurs multiple times, merge the paths.
                paths.entry(request.doc_type).or_default().extend(attributes);

                paths
            });

        AttestationAttributePaths::try_new(paths)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Implement test for ItemsRequests::try_into_attribute_paths().
}
