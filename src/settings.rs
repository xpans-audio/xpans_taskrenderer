//! Unused
#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::env;

const AUDIO_DECODER_CHUNK_LENGTH: &'static str = "AUDIO_DECODER_CHUNK_LENGTH";
const AUDIO_DECODER_BUFFER_SIZE: &'static str = "AUDIO_DECODER_BUFFER_SIZE";

/// This has no effect, as settings are currently unimplemented.
struct Settings {
    audio_decoder: AudioDecoderSettings,
    spatial_decoder: SpatialDecoderSettings,
    audio_encoder: AudioEncoderSettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct AudioDecoderSettings {
    chunk_size: usize,
    buffer_size: usize,
}

impl AudioDecoderSettings {
    pub fn apply_env(&mut self) {
        let buf_size = env::var(AUDIO_DECODER_BUFFER_SIZE);
        let chunk_len = env::var(AUDIO_DECODER_CHUNK_LENGTH);
        if let Ok(buf_size) = buf_size {
            self.buffer_size = usize::from_str_radix(&buf_size, 10).unwrap();
        }
        if let Ok(chunk_len) = chunk_len {
            self.buffer_size = usize::from_str_radix(&chunk_len, 10).unwrap();
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct SpatialDecoderSettings {
    buffer_size: usize,
}

impl SpatialDecoderSettings {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct AudioEncoderSettings {
    chunk_size: usize,
    buffer_size: usize,
}

impl AudioEncoderSettings {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Interpolation {
    Linear,
}
