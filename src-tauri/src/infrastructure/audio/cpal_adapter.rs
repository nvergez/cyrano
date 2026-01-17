//! cpal audio constants, adapter, and error conversions.
//!
//! Provides a concrete AudioCapture implementation backed by cpal.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::domain::CyranoError;
use crate::infrastructure::audio::resampler::LinearResampler;
use crate::traits::audio_capture::AudioCapture;

/// Target sample rate for Whisper compatibility (16kHz)
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// cpal-backed audio capture adapter.
pub struct CpalAdapter {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    is_capturing: bool,
}

impl CpalAdapter {
    /// Create a new adapter with an empty buffer.
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            is_capturing: false,
        }
    }

    fn build_stream(
        device: &cpal::Device,
        config: cpal::SupportedStreamConfig,
        buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream, CyranoError> {
        let device_sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        let resampler = LinearResampler::new(device_sample_rate, TARGET_SAMPLE_RATE);

        let err_callback = |err| log::error!("Audio stream error: {err}");

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let mut resampler = resampler;
                let buffer_clone = buffer.clone();
                let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        for frame in data.chunks(channels) {
                            let sample = frame.iter().sum::<f32>() / frame.len() as f32;
                            resampler.push_sample(sample, &mut buf);
                        }
                    }
                };
                device
                    .build_input_stream(&config.into(), data_callback, err_callback, None)
                    .map_err(CyranoError::from)?
            }
            cpal::SampleFormat::I16 => {
                let mut resampler = resampler;
                let buffer_clone = buffer.clone();
                let data_callback = move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buffer_clone.lock() {
                        for frame in data.chunks(channels) {
                            let sample = frame.iter().map(|&s| s as f32).sum::<f32>()
                                / frame.len() as f32
                                / 32768.0;
                            resampler.push_sample(sample, &mut buf);
                        }
                    }
                };
                device
                    .build_input_stream(&config.into(), data_callback, err_callback, None)
                    .map_err(CyranoError::from)?
            }
            _ => {
                return Err(CyranoError::RecordingFailed {
                    reason: format!("Unsupported sample format: {:?}", sample_format),
                });
            }
        };

        Ok(stream)
    }
}

impl AudioCapture for CpalAdapter {
    fn start_capture(&mut self) -> Result<(), CyranoError> {
        if self.is_capturing {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(CyranoError::MicAccessDenied)?;

        let config = get_input_config(&device)?;

        let stream = Self::build_stream(&device, config, self.buffer.clone())?;
        stream.play().map_err(CyranoError::from)?;

        self.stream = Some(stream);
        self.is_capturing = true;
        Ok(())
    }

    fn stop_capture(&mut self) -> Result<Vec<f32>, CyranoError> {
        self.stream = None;
        self.is_capturing = false;

        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| CyranoError::RecordingFailed {
                reason: format!("Failed to lock audio buffer: {e}"),
            })?;

        Ok(std::mem::take(&mut *buffer))
    }

    fn is_capturing(&self) -> bool {
        self.is_capturing
    }
}

fn get_input_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig, CyranoError> {
    let supported_configs: Vec<_> = device
        .supported_input_configs()
        .map_err(|e| match e {
            cpal::SupportedStreamConfigsError::DeviceNotAvailable => CyranoError::MicAccessDenied,
            _ => CyranoError::RecordingFailed {
                reason: format!("Failed to get supported configs: {e}"),
            },
        })?
        .collect();

    if supported_configs.is_empty() {
        return Err(CyranoError::RecordingFailed {
            reason: "No supported audio configurations found".to_string(),
        });
    }

    // Prefer F32 format; otherwise use the first available format.
    for config in &supported_configs {
        if config.sample_format() == cpal::SampleFormat::F32 {
            return Ok((*config).with_max_sample_rate());
        }
    }

    Ok(supported_configs[0].with_max_sample_rate())
}

// Error conversions from cpal errors to CyranoError

impl From<cpal::BuildStreamError> for CyranoError {
    fn from(e: cpal::BuildStreamError) -> Self {
        match e {
            cpal::BuildStreamError::DeviceNotAvailable => CyranoError::MicAccessDenied,
            cpal::BuildStreamError::StreamConfigNotSupported => CyranoError::RecordingFailed {
                reason: "Audio format not supported".to_string(),
            },
            _ => CyranoError::RecordingFailed {
                reason: e.to_string(),
            },
        }
    }
}

impl From<cpal::PlayStreamError> for CyranoError {
    fn from(e: cpal::PlayStreamError) -> Self {
        CyranoError::RecordingFailed {
            reason: e.to_string(),
        }
    }
}

impl From<cpal::DevicesError> for CyranoError {
    fn from(e: cpal::DevicesError) -> Self {
        CyranoError::RecordingFailed {
            reason: e.to_string(),
        }
    }
}

impl From<cpal::SupportedStreamConfigsError> for CyranoError {
    fn from(e: cpal::SupportedStreamConfigsError) -> Self {
        match e {
            cpal::SupportedStreamConfigsError::DeviceNotAvailable => CyranoError::MicAccessDenied,
            _ => CyranoError::RecordingFailed {
                reason: e.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_stream_error_conversion() {
        let err = cpal::BuildStreamError::DeviceNotAvailable;
        let cyrano_err: CyranoError = err.into();
        assert!(matches!(cyrano_err, CyranoError::MicAccessDenied));
    }

    #[test]
    fn test_supported_configs_error_conversion() {
        let err = cpal::SupportedStreamConfigsError::DeviceNotAvailable;
        let cyrano_err: CyranoError = err.into();
        assert!(matches!(cyrano_err, CyranoError::MicAccessDenied));
    }

    #[test]
    fn test_target_sample_rate() {
        assert_eq!(TARGET_SAMPLE_RATE, 16_000);
    }
}
