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
