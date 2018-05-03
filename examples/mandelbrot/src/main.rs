#![feature(proc_macro)]

extern crate sprnd_macros;
extern crate sprnd;
use sprnd_macros::kernel;

#[kernel]
fn mandel(c_re: f32, c_im: f32, /*#[uniform]*/ count: u8) -> u8 {
    let mut z_re = c_re;
    let mut z_im = c_im;
    for i in 0..count {
        if z_re * z_re + z_im * z_im > 4.0 {
            return i;
        }
        let new_re = z_re * z_re - z_im * z_im;
        let new_im = 2.0 * z_re * z_im;
        z_re = c_re + new_re;
        z_im = c_im + new_im;
    }
    count-1
}

fn main() {
    let width = 800;
    let height = 600;
    let xmin = -2f32;
    let xmax = 1f32;
    let ymin = -1.5f32;
    let ymax = 1.5f32;
    let iterations = 256;
    let dx = (xmax - xmin) / width as f32;
    let dy = (ymax - ymin) / height as f32;

    let image = vec![0u8; width * height];

    for j in 0..height {
        let row = &image[j * width];
        // collect_into? par_iter()?
        sprnd::dispatch(row, |x| mandel(x, j as f32, iterations));
    }
}
