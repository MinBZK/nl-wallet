use attestation_data::attributes::Attribute;
use attestation_data::attributes::Attributes;
use indexmap::IndexMap;
use itertools::Either;

use crate::pid::constants::PID_ATTESTATION_TYPE;

pub mod auth_code_flow;
pub mod brp;
pub mod constants;
pub mod digid;
pub mod digid_mock;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

/// Lay out the attributes of a PID for mdoc, which places every attribute in a single namespace named
/// after the attestation type, without nesting.
pub(crate) fn into_mdoc_attributes(attributes: Attributes) -> Attributes {
    let entries = attributes
        .into_inner()
        .into_iter()
        .flat_map(|(name, attribute)| match attribute {
            Attribute::Object(group) => Either::Right(group.into_iter()),
            value => Either::Left(std::iter::once((name, value))),
        })
        .collect::<IndexMap<_, _>>();

    Attributes::from(IndexMap::from([(
        String::from(PID_ATTESTATION_TYPE),
        Attribute::Object(entries),
    )]))
}
