use core::num::NonZeroU32;
use std::path::PathBuf;

use image::{DynamicImage, RgbImage};
// use indicatif::ProgressBar;

use ndarray::Array3;
use pixlzr::{FilterType, Pixlzr};
use video_rs::{self, Decoder, Encoder, EncoderSettings};

const DEFAULT_BLOCK_SIZE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(32) };

const DEFAULT_SHRINKING_FACTOR: f32 = 1f32;

fn parse_shrinking_factor(shrinking_factor: &String) -> Option<f32> {
	let mut base_pos: usize = 0;
	let mut invert = false;
	let mut is_negative = false;
	if shrinking_factor[base_pos..].starts_with("+") {
		base_pos += 1;
	} else if shrinking_factor[base_pos..].starts_with("-") {
		is_negative = true;
		base_pos += 1;
	}
	if shrinking_factor[base_pos..].starts_with("1/") {
		invert = true;
		base_pos += 2;
	}

	let factor: f32 = shrinking_factor[base_pos..].parse().ok()?;

	Some(
		(if is_negative { -1.0 } else { 1.0 })
			* (if invert { 1.0 / factor } else { factor }),
	)
}

// enum Bool {
// 	True,
// 	False,
// }
// impl From<Option<()>> for Bool {
// 	fn from(value: Option<()>) -> Self {
// 		match value {
// 			None => Bool::False,
// 			Some(_) => Bool::True,
// 		}
// 	}
// }
// impl From<Bool> for bool {
// 	fn from(value: Bool) -> Self {
// 		match value {
// 			Bool::True => true,
// 			Bool::False => false,
// 		}
// 	}
// }

fn main() {
	process_video();
}

fn process_video() {
	// "../testes-finais/green-and-blue.mp4".to_string().into(),
	// "../07 - using pixlzr/coisas/video.mp4".to_string().into(),
	let args = std::env::args().collect::<Vec<_>>();
	let input: PathBuf = args.get(1).expect("need input file").into();
	let output: PathBuf = args.get(2).expect("need output file").into();
	let block_size: u32 = args
		.get(3)
		.and_then(|s| s.parse().ok())
		.unwrap_or(DEFAULT_BLOCK_SIZE)
		.into();
	let factor: f32 = args
		.get(4)
		.and_then(parse_shrinking_factor)
		.unwrap_or(DEFAULT_SHRINKING_FACTOR);

	println!("Input: {}", input.display());
	println!("Output: {}", output.display());
	println!("Block size: {}", block_size);
	println!("Factor: {}", factor);

	video_rs::init().unwrap();

	let (mut decoder, mut encoder) = {
		let decoder =
			Decoder::new(&input.into()).expect("failed to create decoder");

		let (width, height) = decoder.size();
		let settings = EncoderSettings::for_h264_yuv420p(
			width as usize,
			height as usize,
			false,
		);
		let encoder = Encoder::new(&output.into(), settings)
			.expect("failed to create encoder");

		let framerate = 30.; //decoder.frame_rate();
		println!("Video frame rate: {}", framerate);

		(decoder, encoder)
	};
	let (width, height) = decoder.size_out();
	let array_dim = (height as usize, width as usize, 3);

	// let pb = ProgressBar::new(total_frames);

	for (_idx, frame) in
		decoder.decode_iter().take_while(Result::is_ok).enumerate()
	{
		// pb.inc(1);
		let (time, frame) = frame.unwrap();

		let image =
			RgbImage::from_raw(width, height, frame.into_raw_vec()).unwrap();
		let mut image = Pixlzr::from_image(
			&DynamicImage::ImageRgb8(image),
			block_size,
			block_size,
		);
		image.shrink_by(FilterType::Lanczos3, factor);
		let image = image.to_image(FilterType::Nearest);

		let frame = unsafe {
			Array3::from_shape_vec_unchecked(array_dim, image.into_bytes())
		};

		encoder
			.encode(&frame, &time)
			.expect("failed to encode frame");
	}
	// pb.finish_with_message("processado");

	encoder.finish().expect("failed to finish encoder");
}
