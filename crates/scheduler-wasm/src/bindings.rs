use std::convert::TryFrom;

use chrono::NaiveDate;
use scheduler_core::SchedulerConfig;
use serde_wasm_bindgen::{from_value, to_value};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

use crate::config::{SchedulerConfigDto, SchedulerConfigPatch};
use crate::scheduler::SchedulerFacade;

/// Scheduler bindings exposed to JavaScript via wasm-bindgen.
#[wasm_bindgen]
pub struct WasmScheduler {
    facade: SchedulerFacade,
}

#[wasm_bindgen]
impl WasmScheduler {
    /// Constructs a scheduler using the supplied configuration override.
    #[wasm_bindgen(constructor)]
    pub fn new(config: Option<JsValue>) -> Result<WasmScheduler, JsValue> {
        let config = match config {
            Some(value) => {
                let patch: SchedulerConfigPatch =
                    from_value(value).map_err(|err| JsValue::from_str(&err.to_string()))?;
                Ok(patch.apply(SchedulerConfig::default()))
            }
            None => Ok(SchedulerConfig::default()),
        }?;
        Ok(Self {
            facade: SchedulerFacade::new(config),
        })
    }

    /// Returns the active scheduler configuration.
    #[wasm_bindgen(js_name = "currentConfig")]
    pub fn current_config(&self) -> Result<JsValue, JsValue> {
        to_value(&SchedulerConfigDto::from(self.facade.config()))
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    /// Builds the queue for the provided owner and reports the number of cards.
    #[wasm_bindgen(js_name = "queueLength")]
    pub fn queue_length(&mut self, owner_id: &str, iso_date: &str) -> Result<u32, JsValue> {
        let owner_id = parse_owner_id(owner_id)?;
        let today = parse_iso_date(iso_date)?;
        let length = self.facade.queue_length(owner_id, today);
        u32::try_from(length).map_err(|_| JsValue::from_str("queue length exceeds u32"))
    }
}

/// Provides the default scheduler configuration for bootstrapping the wasm module.
#[wasm_bindgen(js_name = "defaultConfig")]
pub fn default_config() -> Result<JsValue, JsValue> {
    to_value(&SchedulerConfigDto::from(&SchedulerConfig::default()))
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

/// Installs the console panic hook so Rust panics surface in the developer console.
#[wasm_bindgen(js_name = "initPanicHook")]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

fn parse_owner_id(value: &str) -> Result<Uuid, JsValue> {
    Uuid::parse_str(value).map_err(|err| JsValue::from_str(&format!("invalid owner id: {err}")))
}

fn parse_iso_date(value: &str) -> Result<NaiveDate, JsValue> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|err| JsValue::from_str(&format!("invalid ISO date: {err}")))
}
