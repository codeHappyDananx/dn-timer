#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrRequest {
    pub image_path: Option<String>,
    pub character_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub success: bool,
    pub character_name: Option<String>,
    #[serde(rename = "results")]
    pub dungeon_cds: Vec<DungeonCdItem>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DungeonCdItem {
    pub name: String,
    pub count: String,
    pub confidence: f32,
}

pub struct OcrEngine {
    bridge: super::python_bridge::PythonBridge,
}

impl OcrEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            bridge: super::python_bridge::PythonBridge::new()?,
        })
    }

    pub fn recognize(
        &self,
        image_path: Option<&str>,
        character_name: Option<&str>,
    ) -> anyhow::Result<OcrResult> {
        let resp = self.bridge.recognize(image_path, character_name)?;

        let result: OcrResult = serde_json::from_value(resp)
            .map_err(|e| anyhow::anyhow!("Failed to parse OCR result: {}", e))?;

        Ok(result)
    }

    pub fn ping(&self) -> anyhow::Result<bool> {
        let req = serde_json::json!({"method": "ping"});
        let resp = self.bridge.request(req)?;
        Ok(resp
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }
}
