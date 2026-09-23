use attestation_types::credential_format::Format;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use openid4vc::metadata::issuer_metadata::CredentialMetadata;
use sd_jwt_vc_metadata::VerifiedTypeMetadataDocuments;
use sea_orm::FromJsonQueryResult;
use sea_orm::entity::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::attestation_copy;

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "attestation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub attestation_type: String,
    #[sea_orm(column_name = "expiration_date_time")]
    pub expiration: Option<DateTime<Utc>>,
    #[sea_orm(column_name = "not_before_date_time")]
    pub not_before: Option<DateTime<Utc>>,
    pub attestation_format: AttestationFormat,
    pub extended_types: ExtendedTypesModel,
    pub metadata: AttestationMetadataModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AttestationFormat {
    #[sea_orm(string_value = "dc+sd-jwt")]
    SdJwt,
    #[sea_orm(string_value = "mso_mdoc")]
    Mdoc,
}

impl From<Format> for AttestationFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::SdJwt => Self::SdJwt,
            Format::MsoMdoc => Self::Mdoc,
        }
    }
}

impl From<AttestationFormat> for Format {
    fn from(format: AttestationFormat) -> Self {
        match format {
            AttestationFormat::SdJwt => Self::SdJwt,
            AttestationFormat::Mdoc => Self::MsoMdoc,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, Constructor)]
#[serde(transparent)]
pub struct ExtendedTypesModel {
    pub attestation_types: Vec<String>,
}

/// The metadata that describes a stored attestation, which is either the chain of SD-JWT VC Type Metadata documents
/// it was issued with or the Credential Metadata taken from the Credential Issuer metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(rename_all = "snake_case")]
pub enum AttestationMetadataModel {
    TypeMetadata(VerifiedTypeMetadataDocuments),
    CredentialMetadata(CredentialMetadata),
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AttestationCopy,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AttestationCopy => Entity::has_many(attestation_copy::Entity).into(),
        }
    }
}

impl Related<attestation_copy::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AttestationCopy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
