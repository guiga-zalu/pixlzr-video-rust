// use core::num::NonZeroU32;
// use std::path::PathBuf;

/* use image::{DynamicImage, RgbImage};
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

	Some((if is_negative { -1.0 } else { 1.0 }) * (if invert { 1.0 / factor } else { factor }))
}
 */
use std::{
	fmt::Debug,
	ops::{Index, IndexMut},
};

use palette::{IntoColor, LinSrgb, Oklab, Srgb};

type TypeInterior<T> = [T; 256];
type Type<T> = [Box<TypeInterior<Box<TypeInterior<T>>>>; 256];

/// Base size (ignoring the option)
/// (f32 x 3) x (256 ** 3) octets
/// = (4 x 3) x ((2 ** 8) ** 3) oct
/// = 3 x 2^26 oct
/// = 3 x 2^6 M oct
/// = 192 Megabytes
/// Pretty ok for me.
struct List<T>(pub Type<T>);
impl<T> List<T>
where
	T: Debug,
{
	fn new(f: fn() -> TypeInterior<T>) -> Self {
		println!(
			"Allocating {} Kilo octets",
			std::mem::size_of::<Type<T>>() >> 10
		);
		let range = 0..256;
		let list: Type<T> = range
			.clone()
			.map(|_| {
				Box::new(
					range
						.clone()
						.map(|_| Box::new(f()))
						.collect::<Vec<Box<TypeInterior<T>>>>()
						.try_into()
						.unwrap(),
				)
			})
			.collect::<Vec<Box<TypeInterior<Box<TypeInterior<T>>>>>>()
			.try_into()
			.unwrap();
		Self(list)
	}
}
impl<T> Index<(u8, u8)> for List<T> {
	type Output = Box<TypeInterior<T>>;
	fn index(&self, (r, g): (u8, u8)) -> &Self::Output {
		unsafe { self.0.get_unchecked(r as usize).get_unchecked(g as usize) }
	}
}
impl<T> IndexMut<(u8, u8)> for List<T> {
	fn index_mut(&mut self, (r, g): (u8, u8)) -> &mut Self::Output {
		unsafe {
			self.0
				.get_unchecked_mut(r as usize)
				.get_unchecked_mut(g as usize)
		}
	}
}
impl<T> Index<(u8, u8, u8)> for List<T> {
	type Output = T;
	fn index(&self, (r, g, b): (u8, u8, u8)) -> &Self::Output {
		unsafe { self[(r, g)].get_unchecked(b as usize) }
	}
}
impl<T> IndexMut<(u8, u8, u8)> for List<T> {
	fn index_mut(&mut self, (r, g, b): (u8, u8, u8)) -> &mut Self::Output {
		unsafe { self.index_mut((r, g)).get_unchecked_mut(b as usize) }
	}
}

fn main() {
	// process_video();

	let f = || [None::<[f32; 3]>; 256];
	println!("Pre-starting...");
	let mut as_oklab: List<Option<[f32; 3]>> = List::new(f);

	let range = u8::MIN..=u8::MAX;
	let count = (u8::MAX as f32).powi(3);
	// Get values
	range.clone().for_each(|r| {
		range.clone().for_each(|g| {
			range.clone().for_each(|b| {
				let rgb: LinSrgb<f32> = Srgb::new(r, g, b).into_linear();
				let oklab: Oklab<f32> = rgb.into_color();
				// println!("{:?} -> {:?}", [r, g, b], oklab);
				as_oklab[(r, g, b)] = Some([oklab.l, oklab.a, oklab.b]);
			})
		})
	});

	// Compare neighbours
	let mut diff_r_down: List<Option<[f32; 3]>> = List::new(f);
	let mut diff_r_up: List<Option<[f32; 3]>> = List::new(f);
	let mut diff_g_down: List<Option<[f32; 3]>> = List::new(f);
	let mut diff_g_up: List<Option<[f32; 3]>> = List::new(f);
	let mut diff_b_down: List<Option<[f32; 3]>> = List::new(f);
	let mut diff_b_up: List<Option<[f32; 3]>> = List::new(f);
	macro_rules! diff {
		(if $cond:expr; [$point_here:expr => $point_other:expr]; $oklab:ident => $list:ident) => {
			if $cond {
				let neighbour = as_oklab[$point_other].unwrap();
				$list[$point_here] = Some([
					$oklab[0] - neighbour[0],
					$oklab[1] - neighbour[1],
					$oklab[1] - neighbour[2],
				]);
			}
		};
	}
	range.clone().for_each(|r| {
		range.clone().for_each(|g| {
			range.clone().for_each(|b| {
				let oklab = as_oklab[(r, g, b)].unwrap();
				diff!(if r > 0; [(r, g, b) => (r - 1, g, b)]; oklab => diff_r_down);
				diff!(if r < 255; [(r, g, b) => (r + 1, g, b)]; oklab => diff_r_up);
				diff!(if g > 0; [(r, g, b) => (r, g - 1, b)]; oklab => diff_g_down);
				diff!(if g < 255; [(r, g, b) => (r, g + 1, b)]; oklab => diff_g_up);
				diff!(if b > 0; [(r, g, b) => (r, g, b - 1)]; oklab => diff_b_down);
				diff!(if b < 255; [(r, g, b) => (r, g, b + 1)]; oklab => diff_b_up);
			})
		})
	});

	println!("Unity: %%");
	let (min, max, average) = list_range(as_oklab, count);
	println!("\tmin:{:?} | max:{:?} | average:{:?}", min, max, average);

	let (min, max, average) = list_range(diff_r_down, count);
	println!(
		"(r-)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
	let (min, max, average) = list_range(diff_r_up, count);
	println!(
		"(r+)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
	let (min, max, average) = list_range(diff_g_down, count);
	println!(
		"(g-)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
	let (min, max, average) = list_range(diff_g_up, count);
	println!(
		"(g+)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
	let (min, max, average) = list_range(diff_b_down, count);
	println!(
		"(b-)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
	let (min, max, average) = list_range(diff_b_up, count);
	println!(
		"(b+)\tmin:{:?} | max:{:?} | average:{:?}",
		min, max, average
	);
}

fn display(f: f32) -> String {
	if f.is_sign_positive() { "+" } else { "-" }.to_string()
		+ &format!("{:.4}", f.abs())
}

fn list_range(
	l: List<Option<[f32; 3]>>,
	count: f32,
) -> ([String; 3], [String; 3], [String; 3]) {
	// List range
	let mut min = [f32::MAX; 3];
	let mut max = [f32::MIN; 3];
	let mut average = [0f32; 3];
	l.0.iter().for_each(|r| {
		r.iter().for_each(|g| {
			g.iter().for_each(|x| {
				if let Some(x) = x {
					x.iter()
						.zip(
							average
								.iter_mut()
								.zip(min.iter_mut().zip(max.iter_mut())),
						)
						.for_each(|(a, (sum, (min, max)))| {
							*min = a.min(*min);
							*max = a.max(*max);
							*sum += a;
						});
				}
			})
		})
	});
	average.iter_mut().for_each(|x| *x /= count);

	let min = [display(min[0]), display(min[1]), display(min[2])];
	let max = [display(max[0]), display(max[1]), display(max[2])];
	let average = [
		display(average[0]),
		display(average[1]),
		display(average[2]),
	];

	(min, max, average)
}

/* fn process_video() {
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

	video_rs::init().unwrap();

	let (mut decoder, mut encoder) = {
		let decoder = Decoder::new(&input.into()).expect("failed to create decoder");

		let (width, height) = decoder.size();
		let settings = EncoderSettings::for_h264_yuv420p(width as usize, height as usize, false);
		let encoder = Encoder::new(&output.into(), settings).expect("failed to create encoder");

		let framerate = 30.; //decoder.frame_rate();
		println!("Video frame rate: {}", framerate);

		(decoder, encoder)
	};
	let (width, height) = decoder.size_out();
	let array_dim = (height as usize, width as usize, 3);

	// let pb = ProgressBar::new(total_frames);

	for (_idx, frame) in decoder.decode_iter().take_while(Result::is_ok).enumerate() {
		// pb.inc(1);
		let (time, frame) = frame.unwrap();

		let image = RgbImage::from_raw(width, height, frame.into_raw_vec()).unwrap();
		let mut image = Pixlzr::from_image(&DynamicImage::ImageRgb8(image), block_size, block_size);
		image.shrink_by(FilterType::Lanczos3, factor);
		let image = image.to_image(FilterType::Nearest);

		let frame = unsafe { Array3::from_shape_vec_unchecked(array_dim, image.into_bytes()) };

		encoder
			.encode(&frame, &time)
			.expect("failed to encode frame");
	}
	// pb.finish_with_message("processado");

	encoder.finish().expect("failed to finish encoder");
}
 */
