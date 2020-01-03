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
// *********************************************************************************************************************
//
//                                                 P R I V A T E   A P I
//
// *********************************************************************************************************************
// *********************************************************************************************************************



/***********************************************************************************************************************
 * Draw either the Mandelbrot Set or a Julia Set
 */
fn draw_fractal(
  ctx       : &CanvasRenderingContext2d
, width     : u32     // Canvas width
, height    : u32     // Canvas height
, x_max     : f64     // Maximum extent of X axis
, x_min     : f64     // Minimum extent of X axis
, y_max     : f64     // Maximum extent of Y axis
, y_min     : f64     // Minimum extent of Y axis
, mandel_x  : f64     // X coord of mouse point on Mandelbrot set
, mandel_y  : f64     // Y coord of mouse point on Mandelbrot set
, max_iters : u32     // Stop after this many iterations
, c_map     : JsValue // Selected colour map
, f_type    : FractalType
) -> Result<(), JsValue> {
  let colour_map : Vec<Vec<u32>> = JsValue::into_serde(&c_map).unwrap();
  let mut image_data = Vec::new();

  // Here's where the heavy lifting happens...
  for iy in 0..height {
    for ix in 0..width {
      // Translate canvas (x,y) pixel location to the (x,y) location in Mandelbrot Set's coordinate space
      let x_coord = x_min + (x_max - x_min) * (ix as f64 / (width - 1) as f64);
      let y_coord = y_min + (y_max - y_min) * (iy as f64 / (height - 1) as f64);

      // Determine the colour of the current pixel
      let this_colour = match f_type {
        FractalType::Mandelbrot => &colour_map[mandel_iter(x_coord, y_coord, max_iters)]
      , FractalType::Julia      => &colour_map[julia_iter(x_coord, y_coord, mandel_x, mandel_y, max_iters)]
      };

      // Might get into trouble here because this insertion order assumes we're running on a little-endian processor...
      image_data.push(this_colour[0] as u8);  // Red
      image_data.push(this_colour[1] as u8);  // Green
      image_data.push(this_colour[2] as u8);  // Blue
      image_data.push(this_colour[3] as u8);  // Alpha
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
  let temp : f64 = sum_of_squares(x_val - 0.25, y_val);
  
  // Does the current x_val lie within the main cardioid?
  if temp * (temp + x_val - 0.25) <= (y_val * y_val) / 4.0 ||
     // Or the period-2 bulb? 
     sum_of_squares(x_val + 1.0, y_val) <= 0.0625 {
    // Yup, so we can bail out early
    return max_iters as usize;
  }
  else {
    // Nope, so we have to run the full calculation
    let mut iter_count : u32 = 0;
    let mut x          : f64 = 0.0;
    let mut y          : f64 = 0.0;
    let mut x_sqr      : f64 = 0.0;
    let mut y_sqr      : f64 = 0.0;

    // Determine if the value at the current location escapes to infinity or not.
    while iter_count < max_iters && (x_sqr + y_sqr <= BAILOUT) {
      y     = y_val + (2.0 * x * y);
      x     = x_val + (x_sqr - y_sqr);
      x_sqr = x * x;
      y_sqr = y * y;

      iter_count += 1;
    }

    return iter_count as usize;
  }
}

/***********************************************************************************************************************
 * Return the iteration value of a particular pixel in the Julia set
 */
fn julia_iter(
  mut x_coord : f64
, mut y_coord : f64
, mandel_x    : f64
, mandel_y    : f64
, max_iters   : u32
) -> usize {
  let mut iter_count : u32 = 0;
  let mut new_x      : f64 = 0.0;
  let mut new_y      : f64 = 0.0;

  // Determine if the value at the current location escapes to infinity or not.
  while (sum_of_squares(new_x, new_y) <= BAILOUT) && iter_count < max_iters {
    new_x   = diff_of_squares(x_coord, y_coord) + mandel_x;
    new_y   = 2.0 * x_coord * y_coord + mandel_y;
    x_coord = new_x;
    y_coord = new_y;
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
// *********************************************************************************************************************
//
//                                                  P U B L I C    A P I
//
// *********************************************************************************************************************
// *********************************************************************************************************************



/***********************************************************************************************************************
 * Draw a Mandelbrot Set
 */
#[wasm_bindgen]
pub fn draw_mandel(
  ctx       : &CanvasRenderingContext2d
, width     : u32     // Canvas width
, height    : u32     // Canvas height
, x_max     : f64     // Maximum extent of X axis
, x_min     : f64     // Minimum extent of X axis
, y_max     : f64     // Maximum extent of Y axis
, y_min     : f64     // Minimum extent of Y axis
, max_iters : u32     // Stop after this many iterations
, c_map     : JsValue // Selected colour map
) -> Result<(), JsValue> {
  draw_fractal(ctx, width, height, x_max, x_min, y_max, y_min, 0.0, 0.0, max_iters, c_map, FractalType::Mandelbrot)
}

/***********************************************************************************************************************
 * Draw a Julia Set
 */
#[wasm_bindgen]
pub fn draw_julia(
  ctx       : &CanvasRenderingContext2d
, width     : u32     // Canvas width
, height    : u32     // Canvas height
, x_max     : f64     // Maximum extent of X axis
, x_min     : f64     // Minimum extent of X axis
, y_max     : f64     // Maximum extent of Y axis
, y_min     : f64     // Minimum extent of Y axis
, mandel_x  : f64     // X coord of mouse point on Mandelbrot set
, mandel_y  : f64     // Y coord of mouse point on Mandelbrot set
, max_iters : u32     // Stop after this many iterations
, c_map     : JsValue // Selected colour map
) -> Result<(), JsValue> {
  draw_fractal(ctx, width, height, x_max, x_min, y_max, y_min, mandel_x, mandel_y, max_iters, c_map, FractalType::Julia)
}
