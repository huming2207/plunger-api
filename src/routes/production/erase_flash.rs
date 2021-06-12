use probe_rs::DebugProbeSelector;
use rocket::{form::{Contextual, Form}, http::Status, serde::json::Json};

use crate::{massprod::eraser::{base_eraser::BaseEraser, eraser_error::EraserError, stm32l0_eraser::STM32L0Eraser}, model::response::PlungerResponse};

#[derive(Debug, FromForm)]
pub(crate) struct CmFirmwareEraseForm {
    probe_sn: String,
    probe_vid: u16,
    probe_pid: u16,
    target_name: String,
}

#[delete("/cm/flash", data = "<data>")]
pub(crate) async fn erase_flash(data: Form<CmFirmwareEraseForm>) -> (Status, Json<PlungerResponse>) {
    let form = data;

    if form.target_name.starts_with("STM32L0") {
        let mut eraser = match STM32L0Eraser::new(form.target_name.clone(), DebugProbeSelector{ serial_number: Some(form.probe_sn.clone()), vendor_id: form.probe_vid, product_id: form.probe_pid }) {
            Ok(eraser) => eraser,
            Err(err) => return match err {
                EraserError::InvalidTarget => (Status::Ok, Json(PlungerResponse{ message: "Invalid target".to_string(), details: None })),
                EraserError::InvalidProtectionLevel => (Status::Ok, Json(PlungerResponse{ message: "Invalid RDP level".to_string(), details: None})),
                EraserError::SessionError(_) => (Status::Ok, Json(PlungerResponse{ message: "Invalid session".to_string(), details: None })),
                EraserError::DebugProbeError(_) => (Status::Ok, Json(PlungerResponse{ message: "Something wrong with debug probe".to_string(), details: None }))
            },
        };
        match eraser.mass_erase() {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    } else {
        (Status::Ok, Json(PlungerResponse{ message: "Something wrong with debug probe".to_string(), details: None }))
    }




}