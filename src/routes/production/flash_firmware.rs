use std::fs::File;

use probe_rs::flashing::{BinOptions, DownloadOptions, FileDownloadError, FlashLoader};
use probe_rs::{DebugProbeSelector, Probe};
use rocket::data::ToByteUnit;
use rocket::form::{Contextual, Form};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::task::{JoinError, spawn_blocking};

use crate::model::response::PlungerResponse;


#[derive(Debug, FromForm)]
pub(crate) struct CmFirmwareForm<'v> {
    #[field(validate = len(..32.mebibytes()))]
    blob: TempFile<'v>,
    #[field(validate = one_of(vec!["bin", "hex", "elf"]))]
    fw_type:  &'v str,
    #[field(validate = range(1..))]
    fw_ver: u32,
    probe_sn:  &'v str,
    #[field(validate = range(100..))]
    speed_khz: u32,
    probe_vid: u16,
    probe_pid: u16,
    target_name:  &'v str,
    skip_erase: bool,
}

#[post("/cm/flash", data = "<data>")]
pub(crate) async fn cm_firmware_flash<'r>(data: Form<Contextual<'r, CmFirmwareForm<'r>>>) -> (Status, Json<PlungerResponse>) {
    let form = match data.into_inner().value {
        Some(data) => data,
        None => return (Status::BadRequest, Json(PlungerResponse{ message: "No form posted".to_string(), details: None }))
    };

    let mut probe = match Probe::open(DebugProbeSelector{ product_id: form.probe_pid, vendor_id: form.probe_vid, serial_number: Some(form.probe_sn.to_string()) }) {
        Ok(p) => p,
        Err(err) => return (Status::InternalServerError, Json(PlungerResponse{ message: format!("Failed to open probe: {}", err), details: None }))
    };
    
    match probe.detach() {
        Ok(_) => (),
        Err(err) => return (Status::InternalServerError, Json(PlungerResponse{ message: format!("Failed to detach probe: {}", err), details: None })),
    };

    match probe.set_speed(form.speed_khz) {
        Ok(_) => (),
        Err(err) => return (Status::InternalServerError, Json(PlungerResponse{ message: format!("Failed to set speed: {}", err), details: None })),
    }

    let mut session = match probe.attach_under_reset(form.target_name.clone()) {
        Ok(s) => s,
        Err(err) => return (Status::InternalServerError, Json(PlungerResponse{ message: format!("Failed to open session: {}", err), details: None })),
    };

    let path = match form.blob.path() {
        Some(p) => p,
        None => return (Status::BadRequest, Json(PlungerResponse{ message: format!("Failed to open file: invalid path"), details: None })),
    };

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return (Status::BadRequest, Json(PlungerResponse{ message: format!("Failed to open file: {}", err), details: None })),
    };


    let memory_map = session.target().memory_map.clone();
    let mut loader = FlashLoader::new(memory_map, probe_rs::config::TargetDescriptionSource::BuiltIn);

    let download_result = match form.fw_type {
        "bin" => {
            loader.load_bin_data(&mut file, BinOptions { base_address: None, skip: 0 })
        },
        "hex" => {
            loader.load_hex_data(&mut file)
        },
        "elf" => {
            loader.load_elf_data(&mut file)
        },
        _ => {
            Err(FileDownloadError::Object("Not a valid Bin/Hex/Elf file"))
        }
    };

    // cb.boost_clock(&mut session)?;

    let mut option = DownloadOptions::new();
    option.verify = true;
    
    if form.skip_erase {
        option.keep_unwritten_bytes = true;
        option.skip_erase = true;
    }

    let ret = loader.commit(&mut session, option);

    match ret {
        Ok(_) => return (Status::Ok, Json(PlungerResponse{ message: format!("Firmware donwloaded"), details: None })),
        Err(err) => return (Status::Ok, Json(PlungerResponse{ message: format!("Failed to download firmware: {}", err), details: None })),
    }

}