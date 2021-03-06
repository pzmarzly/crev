use crate::{id, proof, Result};
use chrono::{self, prelude::*};
use crev_common::{
    self,
    serde::{as_rfc3339_fixed, from_rfc3339_fixed},
};
use serde_yaml;
use std::fmt;

const BEGIN_BLOCK: &str = "-----BEGIN CREV TRUST -----";
const BEGIN_SIGNATURE: &str = "-----BEGIN CREV TRUST SIGNATURE-----";
const END_BLOCK: &str = "-----END CREV TRUST-----";

const CURRENT_TRUST_PROOF_SERIALIZATION_VERSION: i64 = -1;

fn cur_version() -> i64 {
    CURRENT_TRUST_PROOF_SERIALIZATION_VERSION
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    High,
    Medium,
    Low,
    None,
    Distrust,
}

impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Medium
    }
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::TrustLevel::*;
        f.write_str(match self {
            Distrust => "distrust",
            None => "none",
            Low => "low",
            Medium => "medium",
            High => "high",
        })
    }
}

impl TrustLevel {
    #[allow(unused)]
    fn from_str(s: &str) -> Result<TrustLevel> {
        Ok(match s {
            "distrust" => TrustLevel::Distrust,
            "none" => TrustLevel::None,
            "low" => TrustLevel::Low,
            "medium" => TrustLevel::Medium,
            "high" => TrustLevel::High,
            _ => bail!("Unknown level: {}", s),
        })
    }
}

/// Body of a Trust Proof
#[derive(Clone, Debug, Builder, Serialize, Deserialize)]
pub struct Trust {
    #[builder(default = "cur_version()")]
    version: i64,
    #[builder(default = "crev_common::now()")]
    #[serde(
        serialize_with = "as_rfc3339_fixed",
        deserialize_with = "from_rfc3339_fixed"
    )]
    pub date: chrono::DateTime<FixedOffset>,
    pub from: crate::PubId,
    pub ids: Vec<crate::PubId>,
    #[builder(default = "Default::default()")]
    pub trust: TrustLevel,
    #[serde(skip_serializing_if = "String::is_empty", default = "Default::default")]
    #[builder(default = "Default::default()")]
    comment: String,
}

/// Like `Trust` but serializes for interactive editing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustDraft {
    #[serde(
        skip_serializing,
        default = "cur_version"
    )]
    version: i64,
    #[serde(
        serialize_with = "as_rfc3339_fixed",
        deserialize_with = "from_rfc3339_fixed"
    )]
    pub date: chrono::DateTime<FixedOffset>,
    pub from: crate::PubId,
    pub ids: Vec<crate::PubId>,
    pub trust: TrustLevel,
    #[serde(default = "Default::default")]
    comment: String,
}

impl From<Trust> for TrustDraft {
    fn from(trust: Trust) -> Self {
        TrustDraft {
            version: trust.version,
            date: trust.date,
            from: trust.from,
            ids: trust.ids,
            trust: trust.trust,
            comment: trust.comment,
        }
    }
}

impl From<TrustDraft> for Trust {
    fn from(trust: TrustDraft) -> Self {
        Trust {
            version: trust.version,
            date: trust.date,
            from: trust.from,
            ids: trust.ids,
            trust: trust.trust,
            comment: trust.comment,
        }
    }
}
impl fmt::Display for Trust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crev_common::serde::write_as_headerless_yaml(self, f)
    }
}

impl fmt::Display for TrustDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crev_common::serde::write_as_headerless_yaml(self, f)
    }
}

impl Trust {
    pub(crate) const BEGIN_BLOCK: &'static str = BEGIN_BLOCK;
    pub(crate) const BEGIN_SIGNATURE: &'static str = BEGIN_SIGNATURE;
    pub(crate) const END_BLOCK: &'static str = END_BLOCK;
}

impl proof::ContentCommon for Trust {
    fn date(&self) -> &chrono::DateTime<FixedOffset> {
        &self.date
    }

    fn author(&self) -> &crate::PubId {
        &self.from
    }
}

impl Trust {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(&s)?)
    }

    pub fn sign_by(self, id: &id::OwnId) -> Result<proof::Proof> {
        super::Content::from(self).sign_by(id)
    }
}

impl TrustDraft {
    pub fn parse(s: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(&s)?)
    }
}
