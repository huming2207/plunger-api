use probe_rs::Probe;
use rocket::{http::Status, serde::json::Json, tokio::task::spawn_blocking};
use serde::{Deserialize, Serialize};

use crate::model::response::PlungerResponse;

#[derive(Serialize, Debug, Deserialize)]
pub enum ProbeType {
    CmsisDap,
    StLink,
    Ftdi,
    JLink
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfo {
    vid: u16,
    pid: u16,
    serial_num: Option<String>,
    probe_type: ProbeType
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfoObject {
    probes: Vec<ProbeInfo>
}

#[get("/probes")]
pub(crate) async fn get_all_probes() -> (Status, Json<PlungerResponse<ProbeInfoObject>>) {
    let probes = Probe::list_all();
    let ret = spawn_blocking(|| {
        let mut new_probes: Vec<ProbeInfo> = Vec::new();

        for probe in probes {
            let probe_type = match probe.probe_type {
                probe_rs::DebugProbeType::CmsisDap => ProbeType::CmsisDap,
                probe_rs::DebugProbeType::Ftdi => ProbeType::Ftdi,
                probe_rs::DebugProbeType::StLink => ProbeType::StLink,
                probe_rs::DebugProbeType::JLink => ProbeType::JLink,
            };
    
            let converted_probe = ProbeInfo { vid: probe.vendor_id, pid: probe.product_id, serial_num: probe.serial_number, probe_type };
    
            new_probes.push(converted_probe);
        }

        return new_probes
    }).await;

    match ret {
        Ok(ret) => return (Status::Ok, Json(PlungerResponse { message: "Failed to get probes".to_string(), details: Some(ProbeInfoObject{ probes: ret }) })),
        Err(err) => return (Status::InternalServerError, Json(PlungerResponse { message: "Failed to get probes".to_string(), details: None })),
    }
}