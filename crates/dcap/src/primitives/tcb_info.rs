use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcbInfo {
    #[serde(rename = "tcbInfo")]
    pub tcb_info: TcbInfoData,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcbInfoData {
    pub version: u32,
    #[serde(rename = "issueDate")]
    pub issue_date: String,
    #[serde(rename = "nextUpdate")]
    pub next_update: String,
    pub fmspc: String,
    #[serde(rename = "pceId")]
    pub pce_id: String,
    #[serde(rename = "tcbType")]
    pub tcb_type: u32,
    #[serde(rename = "tcbEvaluationDataNumber")]
    pub tcb_evaluation_data_number: u32,
    #[serde(rename = "tcbLevels")]
    pub tcb_levels: Vec<TcbLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcbLevel {
    pub tcb: Tcb,
    #[serde(rename = "tcbDate")]
    pub tcb_date: String,
    #[serde(rename = "tcbStatus")]
    pub tcb_status: TcbStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TcbStatus {
    OutOfDate,
    OutOfDateConfigurationNeeded,
    SWHardeningNeeded,
    ConfigurationAndSWHardeningNeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tcb {
    pub pcesvn: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcb_info_v2() {
        let _: TcbInfo = serde_json::from_str(include_str!("data/tcb_info_v2.json")).unwrap();
    }
}
