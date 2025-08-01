use anyhow::{Context as AnyhowContext, Result, bail};
use ffmpeg::{
    avcodec::{AVCodec, AVCodecContext},
    avformat::{AVFormatContextInput, AVFormatContextOutput},
    avutil::{AVAudioFifo, AVChannelLayout, AVFrame, AVSamples, ra},
    error::RsmpegError,
    ffi,
    swresample::SwrContext,
};
use std::{
    ffi::CStr,
    sync::atomic::{AtomicI64, Ordering},
};

/// The output bit rate in bit/s. Opus is highly efficient, so a modest bitrate is often sufficient.
const OUTPUT_BIT_RATE: i64 = 96000;
/// The number of output channels. Opus performs well with stereo audio.
const OUTPUT_CHANNELS: i32 = 2;
/// The sample rate for the Opus encoder. Opus's preferred sample rate is 48 kHz.
const OUTPUT_SAMPLE_RATE: i32 = 48000;

fn open_input_file(input_file: &CStr) -> Result<(AVFormatContextInput, AVCodecContext, usize)> {
    // Open the input file and find the best audio stream.
    let input_format_context =
        AVFormatContextInput::open(input_file).context("Could not open input file")?;
    let (audio_index, decoder) = input_format_context
        .find_best_stream(ffi::AVMEDIA_TYPE_AUDIO)?
        .context("Failed to find audio stream")?;

    let stream = &input_format_context.streams()[audio_index];
    let mut decode_context = AVCodecContext::new(&decoder);
    decode_context.apply_codecpar(&stream.codecpar())?;
    decode_context
        .open(None)
        .context("Could not open input codec")?;
    decode_context.set_pkt_timebase(stream.time_base);
    Ok((input_format_context, decode_context, audio_index))
}

fn open_output_file(
    output_file: &CStr,
    decode_context: &AVCodecContext,
) -> Result<(AVFormatContextOutput, AVCodecContext)> {
    // Create a new format context for the output container format.
    let mut output_format_context =
        AVFormatContextOutput::create(output_file).context("Failed to open output file.")?;

    // Find the Opus encoder to be used by its ID.
    // let encode_codec =
    //     AVCodec::find_encoder(ffi::AV_CODEC_ID_OPUS).context("Failed to find opus encoder")?;

    // Find the Opus encoder to be used by its short name.
    let encode_codec =
        AVCodec::find_encoder_by_name(c"libopus").context("Failed to find libopus encoder")?;

    let mut encode_context = AVCodecContext::new(&encode_codec);

    // Set the Opus encoder parameters.
    encode_context.set_ch_layout(AVChannelLayout::from_nb_channels(OUTPUT_CHANNELS).into_inner());
    // Use a fixed sample rate for the Opus output.
    encode_context.set_sample_rate(OUTPUT_SAMPLE_RATE);
    // Use the first supported sample format of the Opus encoder.
    encode_context.set_sample_fmt(encode_codec.sample_fmts().unwrap()[0]);
    encode_context.set_bit_rate(OUTPUT_BIT_RATE);

    // Open the encoder for the audio stream to use it later.
    encode_context.open(None)?;

    {
        // Create a new audio stream in the output file container.
        let mut stream = output_format_context.new_stream();
        stream.set_codecpar(encode_context.extract_codecpar());
        // Set the time base for the container based on the output sample rate.
        stream.set_time_base(ra(1, OUTPUT_SAMPLE_RATE));
    }

    Ok((output_format_context, encode_context))
}

fn init_resampler(
    decode_context: &mut AVCodecContext,
    encode_context: &mut AVCodecContext,
) -> Result<SwrContext> {
    // Set up the resampler to convert from the input format to the output format.
    let mut resample_context = SwrContext::new(
        &encode_context.ch_layout,
        encode_context.sample_fmt,
        encode_context.sample_rate,
        &decode_context.ch_layout,
        decode_context.sample_fmt,
        decode_context.sample_rate,
    )
    .context("Could not allocate resample context")?;
    resample_context
        .init()
        .context("Could not open resample context")?;
    Ok(resample_context)
}

fn add_samples_to_fifo(
    fifo: &mut AVAudioFifo,
    samples_buffer: &AVSamples,
    frame_size: i32,
) -> Result<()> {
    fifo.realloc(fifo.size() + frame_size);
    unsafe { fifo.write(samples_buffer.audio_data.as_ptr(), frame_size) }
        .context("Could not write data to FIFO")?;
    Ok(())
}

fn init_output_frame(
    nb_samples: i32,
    ch_layout: ffi::AVChannelLayout,
    sample_fmt: i32,
    sample_rate: i32,
) -> Result<AVFrame> {
    let mut frame = AVFrame::new();
    frame.set_nb_samples(nb_samples);
    frame.set_ch_layout(ch_layout);
    frame.set_format(sample_fmt);
    frame.set_sample_rate(sample_rate);

    frame
        .get_buffer(0)
        .context("Could not allocate output frame samples")?;

    Ok(frame)
}

/// Return boolean: if data is written.
fn encode_audio_frame(
    mut frame: Option<AVFrame>,
    output_format_context: &mut AVFormatContextOutput,
    encode_context: &mut AVCodecContext,
) -> Result<()> {
    static PTS: AtomicI64 = AtomicI64::new(0);

    if let Some(frame) = frame.as_mut() {
        frame.set_pts(PTS.fetch_add(frame.nb_samples as i64, Ordering::Relaxed));
    }

    encode_context.send_frame(frame.as_ref())?;
    loop {
        let mut packet = match encode_context.receive_packet() {
            Ok(packet) => packet,
            Err(RsmpegError::EncoderDrainError) | Err(RsmpegError::EncoderFlushedError) => {
                break;
            }
            Err(e) => Err(e).context("Could not encode frame")?,
        };

        output_format_context
            .write_frame(&mut packet)
            .context("Could not write frame")?;
    }
    Ok(())
}

fn load_encode_and_write(
    fifo: &mut AVAudioFifo,
    output_format_context: &mut AVFormatContextOutput,
    encode_context: &mut AVCodecContext,
) -> Result<()> {
    let frame_size = fifo.size().min(encode_context.frame_size);
    let mut frame = init_output_frame(
        frame_size,
        encode_context.ch_layout().clone().into_inner(),
        encode_context.sample_fmt,
        encode_context.sample_rate,
    )?;
    if unsafe { fifo.read(frame.data_mut().as_mut_ptr(), frame_size)? } < frame_size {
        bail!("Could not read data from FIFO");
    }
    encode_audio_frame(Some(frame), output_format_context, encode_context)?;
    Ok(())
}

fn transcode_aac_to_opus(input_file: &CStr, output_file: &CStr) -> Result<()> {
    // Open the input file for reading.
    let (mut input_format_context, mut decode_context, audio_stream_index) =
        open_input_file(input_file)?;

    // Open the output file for writing.
    let (mut output_format_context, mut encode_context) =
        open_output_file(output_file, &decode_context)?;

    // Initialize the resampler to be able to convert audio sample formats,
    // as Opus prefers 48 kHz.
    let mut resample_context = init_resampler(&mut decode_context, &mut encode_context)?;

    // Initialize the FIFO buffer to store audio samples to be encoded.
    let mut fifo = AVAudioFifo::new(
        encode_context.sample_fmt,
        encode_context.ch_layout.nb_channels,
        1,
    );

    // Write the header of the output file container.
    output_format_context
        .write_header(&mut None)
        .context("Could not write output file header")?;

    // Loop as long as we have input samples to read or output samples to write;
    // abort as soon as we have neither.
    loop {
        let output_frame_size = encode_context.frame_size;

        loop {
            // We get enough audio samples.
            if fifo.size() >= output_frame_size {
                break;
            }

            // Break when no more input packets.
            let packet = match input_format_context
                .read_packet()
                .context("Could not read frame")?
            {
                Some(x) => x,
                None => break,
            };

            // Ignore non audio stream packets.
            if packet.stream_index as usize != audio_stream_index {
                continue;
            }

            decode_context
                .send_packet(Some(&packet))
                .context("Could not send packet for decoding")?;

            loop {
                let frame = match decode_context.receive_frame() {
                    Ok(frame) => frame,
                    Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => {
                        break;
                    }
                    Err(e) => Err(e).context("Could not decode frame")?,
                };

                // Resample the decoded frame.
                let mut output_samples = AVSamples::new(
                    encode_context.ch_layout.nb_channels,
                    frame.nb_samples,
                    encode_context.sample_fmt,
                    0,
                )
                .context("Create samples buffer failed.")?;

                unsafe {
                    let converted_samples = resample_context
                        .convert(
                            output_samples.audio_data.as_mut_ptr(),
                            output_samples.nb_samples,
                            frame.extended_data as *const _,
                            frame.nb_samples,
                        )
                        .context("Could not convert input samples")?;

                    // Add the converted samples to the FIFO buffer.
                    add_samples_to_fifo(&mut fifo, &output_samples, converted_samples)?;
                }
            }
        }

        // If we still cannot get enough samples, break.
        if fifo.size() < output_frame_size {
            break;
        }

        // Write frame as much as possible.
        while fifo.size() >= output_frame_size {
            load_encode_and_write(&mut fifo, &mut output_format_context, &mut encode_context)?;
        }
    }

    // Flush encode context.
    encode_audio_frame(None, &mut output_format_context, &mut encode_context)?;

    output_format_context.write_trailer()?;

    Ok(())
}

#[test]
fn transcode_aac_to_opus_test() {
    std::fs::create_dir_all("tests/output/transcode_opus/").unwrap();
    // Assuming 'sample1_short.aac' is a valid input file.
    transcode_aac_to_opus(
        c"tests/assets/audios/sample1_short.aac",
        c"tests/output/transcode_opus/output.opus",
    )
    .unwrap();
}

/* #[test]
fn transcode_mp4_to_opus_test() {
    std::fs::create_dir_all("tests/output/transcode_opus/").unwrap();
    // Assuming 'big_buck_bunny.mp4' has an AAC audio stream.
    transcode_aac_to_opus(
        c"tests/assets/vids/big_buck_bunny.mp4",
        c"tests/output/transcode_opus/big_buck_bunny.opus",
    )
    .unwrap();
} */
