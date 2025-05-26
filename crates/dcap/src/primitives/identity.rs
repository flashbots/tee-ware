use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnclaveIdentityV2 {
    #[serde(rename = "enclaveIdentity")]
    pub enclave_identity: EnclaveIdentity,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnclaveIdentity {
    pub id: String,
    pub version: u32,
    #[serde(rename = "issueDate")]
    pub issue_date: DateTime<Utc>,
    #[serde(rename = "nextUpdate")]
    pub next_update: DateTime<Utc>,
    #[serde(rename = "tcbEvaluationDataNumber")]
    pub tcb_evaluation_data_number: u32,
    pub miscselect: String,
    #[serde(rename = "miscselectMask")]
    pub miscselect_mask: String,
    pub attributes: String,
    #[serde(rename = "attributesMask")]
    pub attributes_mask: String,
    pub mrsigner: String,
    pub isvprodid: u16,
    #[serde(rename = "tcbLevels")]
    pub tcb_levels: Vec<TcbLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TcbLevel {
    pub tcb: Tcb,
    #[serde(rename = "tcbDate")]
    pub tcb_date: DateTime<Utc>,
    #[serde(rename = "tcbStatus")]
    pub tcb_status: TcbStatus,
    #[serde(rename = "advisoryIDs")]
    pub advisory_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tcb {
    pub isvsvn: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TcbStatus {
    UpToDate,
    OutOfDate,
    Revoked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_identity_v2_serde() -> eyre::Result<()> {
        let example = include_str!("./data/enclave_identity_v2.json");
        let _: EnclaveIdentityV2 = serde_json::from_str(example).unwrap();
        Ok(())
    }
}
