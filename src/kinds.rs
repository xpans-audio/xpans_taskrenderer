use std::{
    fs::File,
    io,
    path::Path,
    sync::{Arc, mpsc::Sender},
};
const BUFFER_SIZE: usize = 16_000_000;

use num::{Float, cast::AsPrimitive};
use xpans_renderconfig::{RenderConfig, headphones::Headphones, mono::Mono, stereo::Stereo};

use xpans_violet::{
    RendererBuilder,
    audio_input::{
        AudioInput, BufferedAudioInput, FractionalAudioInput,
        audio_decoder::{AudioDecoder, AudioDecoderInfo, AudioDecoderTask},
        interpolation::linear::WithLinearInterpolation,
    },
    audio_output::audio_encoder::{AudioEncoder, AudioEncoderInfo, AudioEncoderTask, Progress},
    config::{BuildInterpreter, BuildProcessor, GetDelaySamples, GetOutputChannels},
    spatial_input::spatial_decoder::SpatialDecoderInfo,
};
use xpans_xsr::SpatialSampleMap;

use crate::control::AtomicStatus;

/// Renders the task using the render config, audio and spatial data, and more:
pub fn render_config(
    config: RenderConfig,
    atomic_status: AtomicStatus,
    progress_sender: Sender<Progress>,
    spatial_scene: Arc<SpatialSampleMap<usize, u16, f32>>,
    audio_in: &Path,
    audio_out: &Path,
) -> io::Result<()> {
    match config {
        RenderConfig::Mono(mono) => start_render(
            mono,
            spatial_scene,
            audio_in,
            audio_out,
            progress_sender,
            atomic_status,
        ),
        RenderConfig::Stereo(stereo) => start_render(
            stereo,
            spatial_scene,
            audio_in,
            audio_out,
            progress_sender,
            atomic_status,
        ),
        RenderConfig::Headphones(headphones) => start_render(
            headphones,
            spatial_scene,
            audio_in,
            audio_out,
            progress_sender,
            atomic_status,
        ),
    }
}

fn build_audio_input_from_decoder<C, P>(
    config: &C,
    audio_in: P,
) -> io::Result<(C::Result, AudioDecoderTask<f32>)>
where
    C: BuildAudioInput<AudioDecoder<f32>> + GetDelaySamples,
    P: AsRef<Path>,
{
    let file = File::open(audio_in)?;
    let decoder_info = AudioDecoderInfo::new(file);
    let sample_rate = decoder_info.sample_rate();
    let channels = decoder_info.channels();
    let read_len = config.get_delay_samples(sample_rate);
    let read_len = read_len + values_in_cache_line::<f32>(128);
    let write_capacity = length_from_bytes::<f32>(BUFFER_SIZE) / channels;
    let (audio_decoder, audio_decoder_task) =
        decoder_info.into_pair(read_len as usize, write_capacity);
    let audio_in = config.build_audio_input(audio_decoder);
    Ok((audio_in, audio_decoder_task))
}

fn build_audio_encoder<C, P>(
    config: &C,
    audio_out: P,
    sample_rate: u32,
    duration: usize,
) -> io::Result<(AudioEncoder<f32>, AudioEncoderTask)>
where
    C: GetOutputChannels,
    P: AsRef<Path>,
{
    let file = File::create(audio_out)?;
    let encoder_info = AudioEncoderInfo::new(
        file,
        sample_rate,
        config.get_output_channels() as u16,
        duration,
    );
    let pair = encoder_info.into_pair(length_from_bytes::<f32>(BUFFER_SIZE));
    Ok(pair)
}
const fn values_in_cache_line<T>(cache_line_bytes: usize) -> usize {
    cache_line_bytes / core::mem::size_of::<T>()
}

fn length_from_bytes<T>(bytes: usize) -> usize {
    bytes / core::mem::size_of::<T>()
}

fn start_render<C>(
    config: C,
    spatial_scene: Arc<SpatialSampleMap<usize, u16, f32>>,
    audio_in: &Path,
    audio_out: &Path,
    progress_sender: Sender<Progress>,
    atomic_status: AtomicStatus,
) -> io::Result<()>
where
    C: BuildAudioInput<AudioDecoder<f32>>
        + BuildInterpreter<f32>
        + BuildProcessor<
            C::Result,
            AudioEncoder<f32>,
            Interpretation = <C as BuildInterpreter<f32>>::Interpretation,
        > + GetOutputChannels
        + GetDelaySamples,
    C::Result: AudioInput<Sample = f32>,
    <C as BuildInterpreter<f32>>::Interpretation: Default + Clone,
{
    let (audio_input, audio_decoder_task) = build_audio_input_from_decoder(&config, audio_in)?;

    let decoder_info = audio_decoder_task.info();
    let sample_rate = decoder_info.sample_rate();
    let duration = decoder_info.duration();

    let spatial_decoder_info =
        SpatialDecoderInfo::new(spatial_scene, decoder_info.channels(), duration as usize);

    let spatial_wc = length_from_bytes::<f32>(BUFFER_SIZE);
    let (spatial_decoder, spatial_decoder_task) = spatial_decoder_info.into_pair(spatial_wc);

    let (audio_encoder, audio_encoder_task) =
        build_audio_encoder(&config, audio_out, sample_rate, duration as usize)?;

    let status_to_decoder = atomic_status.clone();
    std::thread::spawn(|| {
        audio_decoder_task.run(
            512,
            cancelled_fn(status_to_decoder.clone()),
            paused_fn(status_to_decoder),
        );
    });

    let status_to_spatial = atomic_status.clone();
    std::thread::spawn(|| {
        spatial_decoder_task.run(
            cancelled_fn(status_to_spatial.clone()),
            paused_fn(status_to_spatial),
        );
    });

    let status_to_encoder = atomic_status.clone();
    std::thread::spawn(|| {
        audio_encoder_task.run(
            32,
            cancelled_fn(status_to_encoder.clone()),
            paused_fn(status_to_encoder),
            progress_sender,
        );
    });

    let builder = RendererBuilder::new()
        .set_audio_input(audio_input)
        .set_spatial_input(spatial_decoder)
        .set_audio_output(audio_encoder)
        .set_source_interpreter(config.build_interpreter())
        .set_sample_processor(config.build_processor());

    let mut renderer = builder.build().unwrap();
    let cancelled = cancelled_fn(atomic_status.clone());
    let paused = paused_fn(atomic_status);
    loop {
        if cancelled() {
            break;
        }
        if paused() {
            continue;
        }
        if let None = renderer.render_available_frames() {
            break;
        }
    }
    Ok(())
}

fn cancelled_fn(atomic_status: AtomicStatus) -> impl Fn() -> bool {
    move || return atomic_status.get().cancelled()
}
fn paused_fn(atomic_status: AtomicStatus) -> impl Fn() -> bool {
    move || return atomic_status.get().paused()
}

pub trait BuildAudioInput<BaseInput>
where
    BaseInput: AudioInput,
{
    type Result: AudioInput;
    fn build_audio_input(&self, base_input: BaseInput) -> Self::Result;
}

impl<BaseInput, S> BuildAudioInput<BaseInput> for Headphones
where
    BaseInput: BufferedAudioInput<Sample = S> + 'static,
    S: AsPrimitive<usize> + Float,
{
    type Result = Box<dyn FractionalAudioInput<Sample = S>>;
    fn build_audio_input(&self, base_input: BaseInput) -> Self::Result {
        let interpolated = base_input.with_linear_interpolation();
        Box::new(interpolated)
    }
}
pub trait UseGivenInput {}

impl<T, BaseInput> BuildAudioInput<BaseInput> for T
where
    T: UseGivenInput,
    BaseInput: AudioInput,
{
    type Result = BaseInput;

    fn build_audio_input(&self, base_input: BaseInput) -> Self::Result {
        base_input
    }
}

impl UseGivenInput for Stereo {}
impl UseGivenInput for Mono {}
