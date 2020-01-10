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

const BAILOUT : f64 = 4.0;

// *********************************************************************************************************************
// PRIVATE API
// *********************************************************************************************************************

/***********************************************************************************************************************
 * Draw either the Mandelbrot Set or a Julia Set
 */
fn draw_fractal(
  ctx              : &CanvasRenderingContext2d
, width            : u32     // Canvas width
, height           : u32     // Canvas height
, x_max            : f64     // Maximum extent of X axis
, x_min            : f64     // Minimum extent of X axis
, y_max            : f64     // Maximum extent of Y axis
, y_min            : f64     // Minimum extent of Y axis
, mandel_x         : f64     // X coord of mouse pointer on Mandelbrot set
, mandel_y         : f64     // Y coord of mouse pointer on Mandelbrot set
, max_iters        : u32     // Stop after this many iterations
, c_map            : JsValue // Selected colour map
, is_little_endian : bool    // Is the processor little endian?
, f_type           : FractalType
) -> Result<(), JsValue> {
  // Deserialize the colour map
  let colour_map : Vec<Vec<u32>> = JsValue::into_serde(&c_map).unwrap();

  let mut image_data = Vec::new();

  // Colour map data is stored in the byte order RGB
  // If we're running on a big-endian processor, the insertion order into the image_data vector must be reversed
  let insertion_order = if is_little_endian {
    vec!(0,1,2)
  }
  else {
    vec!(3,2,1)
  };

  // Here's where the heavy lifting happens...
  for iy in 0..height {
    for ix in 0..width {
      // Translate canvas (x,y) pixel location to the (x,y) location in fractal's coordinate space
      let x_coord = x_min + (x_max - x_min) * (ix as f64 / (width - 1) as f64);
      let y_coord = y_min + (y_max - y_min) * (iy as f64 / (height - 1) as f64);

      // Determine the colour of the current pixel
      let this_colour = &colour_map[
        match f_type {
          FractalType::Mandelbrot => mandel_iter(x_coord, y_coord, max_iters)
        , FractalType::Julia      => escape_time_mj(mandel_x, mandel_y, x_coord, y_coord, max_iters)
        }];

      // Insert RGBA byte data into the image_data vector according to the processor's endianness
      image_data.push(this_colour[insertion_order[0]] as u8);  // Red
      image_data.push(this_colour[insertion_order[1]] as u8);  // Green
      image_data.push(this_colour[insertion_order[2]] as u8);  // Blue
      image_data.push(0xff);                                   // Hard-coded alpha value
    }
  }

  let image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut image_data), width, height)?;
  ctx.put_image_data(&image_data, 0.0, 0.0)
}

/***********************************************************************************************************************
 * Return the iteration value of a particular pixel in the Mandelbrot set
 * This calculation bails out early if the current point is located within the main cardioid or the period-2 bulb
 */
fn mandel_iter(
  x_val     : f64
, y_val     : f64
, max_iters : u32
) -> usize {
  // Can we bail out early?
  if mandel_early_bailout(x_val, y_val) {
    max_iters as usize
  }
  else {
    // Nope, so we have to run the full calculation
    escape_time_mj(x_val, y_val, 0.0, 0.0, max_iters)
  }
}

/***********************************************************************************************************************
 * Calculate whether the current point lies within the Mandelbrot Set's main cardioid or the period-2 bulb
 * If it does, then we can bail out early
 */
fn mandel_early_bailout(x : f64, y : f64) -> bool {
  is_in_main_cardioid(x, y, sum_of_squares(x - 0.25, y)) ||
  is_in_period_2_bulb(x, y)
}

fn is_in_main_cardioid(x: f64, y: f64, temp: f64) -> bool { temp * (temp + x - 0.25) <= (y * y) / 4.0 }
fn is_in_period_2_bulb(x: f64, y: f64)            -> bool { sum_of_squares(x + 1.0, y) <= 0.0625 }

/***********************************************************************************************************************
 * Common escape time algorithm for calculating both the Mandelbrot and Julia Sets
 */
fn escape_time_mj(
  mandel_x  : f64
, mandel_y  : f64
, mut x     : f64
, mut y     : f64
, max_iters : u32
) -> usize {
  let mut iter_count : u32 = 0;

  // Count the number of iterations needed before the value at the current location either escapes to infinity or hits
  // the iteration limit
  while (sum_of_squares(x, y) <= BAILOUT) && iter_count < max_iters {
    let new_x = mandel_x + diff_of_squares(x, y);
    let new_y = mandel_y + (2.0 * x * y);
    x     = new_x;
    y     = new_y;
    iter_count += 1;
  }
      
  iter_count as usize
}

/***********************************************************************************************************************
 * Utility functions
 */
fn  sum_of_squares(val1: f64, val2 : f64) -> f64 { val1 * val1 + val2 * val2 }
fn diff_of_squares(val1: f64, val2 : f64) -> f64 { val1 * val1 - val2 * val2 }

enum FractalType {
  Mandelbrot
, Julia
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
  ctx              : &CanvasRenderingContext2d
, width            : u32     // Canvas width
, height           : u32     // Canvas height
, x_max            : f64     // Maximum extent of X axis
, x_min            : f64     // Minimum extent of X axis
, y_max            : f64     // Maximum extent of Y axis
, y_min            : f64     // Minimum extent of Y axis
, max_iters        : u32     // Stop after this many iterations
, c_map            : JsValue // Selected colour map
, is_little_endian : bool    // Is the processor little endian?
) -> Result<(), JsValue> {
  draw_fractal(
    ctx
  , width, height
  , x_max, x_min
  , y_max, y_min
  , 0.0, 0.0
  , max_iters
  , c_map
  , is_little_endian
  , FractalType::Mandelbrot)
}

/***********************************************************************************************************************
 * Draw a Julia Set
 */
#[wasm_bindgen]
pub fn draw_julia(
  ctx              : &CanvasRenderingContext2d
, width            : u32     // Canvas width
, height           : u32     // Canvas height
, x_max            : f64     // Maximum extent of X axis
, x_min            : f64     // Minimum extent of X axis
, y_max            : f64     // Maximum extent of Y axis
, y_min            : f64     // Minimum extent of Y axis
, mandel_x         : f64     // X coord of mouse point on Mandelbrot set
, mandel_y         : f64     // Y coord of mouse point on Mandelbrot set
, max_iters        : u32     // Stop after this many iterations
, c_map            : JsValue // Selected colour map
, is_little_endian : bool    // Is the processor little endian?
) -> Result<(), JsValue> {
  draw_fractal(
    ctx
  , width, height
  , x_max, x_min
  , y_max, y_min
  , mandel_x, mandel_y
  , max_iters
  , c_map
  , is_little_endian
  , FractalType::Julia)
}
