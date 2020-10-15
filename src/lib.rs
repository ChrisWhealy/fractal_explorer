// *********************************************************************************************************************
// Author  : Chris Whealy
// Date    : Jan 2020
//
// Calculate the Mandelbrot set using the supplied values
// For a given point on the Mandelbrot set, calculate the corresponding Julia set
// *********************************************************************************************************************

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsValue};

use web_sys::{CanvasRenderingContext2d, ImageData};

const BAILOUT: f64 = 4.0;

enum FractalType {
  Mandelbrot,
  Julia,
}

struct Dimensions {
  width: u32,
  height: u32,
}

struct Range {
  min: f64,
  max: f64,
}

struct AxesRanges {
  x_range: Range,
  y_range: Range,
}

struct Point {
  x: f64,
  y: f64,
}
// *********************************************************************************************************************
// PRIVATE API
// *********************************************************************************************************************

/***********************************************************************************************************************
 * Draw either the Mandelbrot Set or a Julia Set
 */
fn draw_fractal(
  ctx: &CanvasRenderingContext2d,
  canvas: Dimensions,      // Canvas dimensions
  axes_ranges: AxesRanges, // Extent of X and Y axes
  mouse_loc: Point,        // Mouse pointer location on Mandelbrot set
  max_iters: u32,          // Stop after this many iterations
  c_map: JsValue,          // Selected colour map
  is_little_endian: bool,  // Is the processor little endian?
  f_type: FractalType,
) -> Result<(), JsValue> {
  // Deserialize the colour map
  let colour_map: Vec<Vec<u32>> = JsValue::into_serde(&c_map).unwrap();
  let mut image_data = Vec::new();

  // Build partial functions to scale (x,y) canvas locations to the fractal's coordinate space
  let scale = move |scale_factor: f64, length: f64| move |pos: f64| scale_factor * (pos / length);
  let scale_x = scale(
    axes_ranges.x_range.max - axes_ranges.x_range.min,
    (canvas.width - 1) as f64,
  );
  let scale_y = scale(
    axes_ranges.y_range.max - axes_ranges.y_range.min,
    (canvas.height - 1) as f64,
  );

  // Here's where the heavy lifting happens...
  for iy in 0..canvas.height {
    for ix in 0..canvas.width {
      let this_coord = Point {
        x: axes_ranges.x_range.min + scale_x(ix as f64),
        y: axes_ranges.y_range.min + scale_y(iy as f64),
      };

      // Determine the colour of the current pixel
      let this_colour = &colour_map[match f_type {
        FractalType::Mandelbrot => mandel_iter(&this_coord, &max_iters),
        FractalType::Julia => escape_time_mj(&mouse_loc, this_coord, &max_iters),
      }];

      // Insert RGBA byte data into the image_data vector according to the processor's endianness
      if is_little_endian {
        image_data.push(this_colour[0] as u8); // Red
        image_data.push(this_colour[1] as u8); // Green
        image_data.push(this_colour[2] as u8); // Blue
        image_data.push(0xff); // Hard-coded alpha value
      } else {
        image_data.push(0xff); // Hard-coded alpha value
        image_data.push(this_colour[2] as u8); // Blue
        image_data.push(this_colour[1] as u8); // Green
        image_data.push(this_colour[0] as u8); // Red
      }
    }
  }

  let image_data = ImageData::new_with_u8_clamped_array_and_sh(
    Clamped(&mut image_data),
    canvas.width,
    canvas.height,
  )?;
  ctx.put_image_data(&image_data, 0.0, 0.0)
}

/***********************************************************************************************************************
 * Return the iteration value of a particular pixel in the Mandelbrot set
 * This calculation bails out early if the current point is located within the main cardioid or the period-2 bulb
 */
fn mandel_iter(loc: &Point, max_iters: &u32) -> usize {
  if mandel_early_bailout(&loc) {
    *max_iters as usize
  } else {
    escape_time_mj(&loc, Point { x: 0.0, y: 0.0 }, max_iters)
  }
}

/***********************************************************************************************************************
 * Calculate whether the current point lies within the Mandelbrot Set's main cardioid or the period-2 bulb
 * If it does, then we can bail out early
 */
fn mandel_early_bailout(loc: &Point) -> bool {
  is_in_main_cardioid(&loc, sum_of_squares(loc.x - 0.25, loc.y)) || is_in_period_2_bulb(&loc)
}

fn is_in_main_cardioid(loc: &Point, temp: f64) -> bool {
  temp * (temp + loc.x - 0.25) <= (loc.y * loc.y) / 4.0
}

fn is_in_period_2_bulb(loc: &Point) -> bool {
  sum_of_squares(loc.x + 1.0, loc.y) <= 0.0625
}

/***********************************************************************************************************************
 * Common escape time algorithm for calculating both the Mandelbrot and Julia Sets
 */
fn escape_time_mj(mandel_point: &Point, mut start_val: Point, max_iters: &u32) -> usize {
  let mut iter_count: u32 = 0;

  // Count the number of iterations needed before the value at the current location either escapes to infinity or hits
  // the iteration limit
  while (sum_of_squares(start_val.x, start_val.y) <= BAILOUT) && iter_count < *max_iters {
    let new_x = mandel_point.x + diff_of_squares(start_val.x, start_val.y);
    let new_y = mandel_point.y + (2.0 * start_val.x * start_val.y);
    start_val.x = new_x;
    start_val.y = new_y;
    iter_count += 1;
  }

  iter_count as usize
}

/***********************************************************************************************************************
 * Utility functions
 */
fn sum_of_squares(val1: f64, val2: f64) -> f64 {
  val1 * val1 + val2 * val2
}

fn diff_of_squares(val1: f64, val2: f64) -> f64 {
  val1 * val1 - val2 * val2
}

// *********************************************************************************************************************
// PUBLIC API
// *********************************************************************************************************************

/***********************************************************************************************************************
 * Dummy entry point
 */
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
  Ok(())
}

/***********************************************************************************************************************
 * Draw a Mandelbrot Set
 */
#[wasm_bindgen]
pub fn draw_mandel(
  ctx: &CanvasRenderingContext2d,
  width: u32,             // Canvas width
  height: u32,            // Canvas height
  x_max: f64,             // Maximum extent of X axis
  x_min: f64,             // Minimum extent of X axis
  y_max: f64,             // Maximum extent of Y axis
  y_min: f64,             // Minimum extent of Y axis
  max_iters: u32,         // Stop after this many iterations
  c_map: JsValue,         // Selected colour map
  is_little_endian: bool, // Is the processor little endian?
) -> Result<(), JsValue> {
  draw_fractal(
    ctx,
    Dimensions { width, height },
    AxesRanges {
      x_range: Range {
        max: x_max,
        min: x_min,
      },
      y_range: Range {
        max: y_max,
        min: y_min,
      },
    },
    Point { x: 0.0, y: 0.0 },
    max_iters,
    c_map,
    is_little_endian,
    FractalType::Mandelbrot,
  )
}

/***********************************************************************************************************************
 * Draw a Julia Set
 */
#[wasm_bindgen]
pub fn draw_julia(
  ctx: &CanvasRenderingContext2d,
  width: u32,             // Canvas width
  height: u32,            // Canvas height
  x_max: f64,             // Maximum extent of X axis
  x_min: f64,             // Minimum extent of X axis
  y_max: f64,             // Maximum extent of Y axis
  y_min: f64,             // Minimum extent of Y axis
  mandel_x: f64,          // X coord of mouse point on Mandelbrot set
  mandel_y: f64,          // Y coord of mouse point on Mandelbrot set
  max_iters: u32,         // Stop after this many iterations
  c_map: JsValue,         // Selected colour map
  is_little_endian: bool, // Is the processor little endian?
) -> Result<(), JsValue> {
  draw_fractal(
    ctx,
    Dimensions { width, height },
    AxesRanges {
      x_range: Range {
        max: x_max,
        min: x_min,
      },
      y_range: Range {
        max: y_max,
        min: y_min,
      },
    },
    Point {
      x: mandel_x,
      y: mandel_y,
    },
    max_iters,
    c_map,
    is_little_endian,
    FractalType::Julia,
  )
}
