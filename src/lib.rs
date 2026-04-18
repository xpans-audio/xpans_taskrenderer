/*!
A rendering library dedicated to offline, non-realtime rendering of spatial
audio scenes in the xpans Ecosystem
*/
mod control;
mod kinds;
mod settings;

pub use control::{AtomicStatus, Control, Status, manage_control};
pub use kinds::render_config;
use serde::{Deserialize, Serialize};
// pub use settings::*;
use xpans_renderconfig::RenderConfig;

pub use xpans_violet::audio_output::audio_encoder::Progress;

/**
An extension of a render configuration, adding a 'name'
field for increased identification of different tasks.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTask {
    pub name: String,
    #[serde(flatten)]
    pub config: RenderConfig,
}

/// A list of tasks.
pub type TaskList = Vec<RenderTask>;
