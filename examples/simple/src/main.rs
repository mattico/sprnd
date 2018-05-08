#![feature(proc_macro, stdsimd, proc_macro_non_items)]

extern crate sprnd_macros;
use sprnd_macros::*;
extern crate sprnd;

#[kernel]
fn simple(input: f32) -> f32 {
    if input < 3.0 {
        input * input
    } else {
        input.sqrt()
    }
}

fn main() {
    let input = [0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let mut output = [0f32; 8];

    dispatch!(&input, &mut output, |i| simple(i));
}

// The above gets translated into something like this, but with worse variable names:

/*
fn simple(input: &[f32], output: &mut [f32]) {
    assert!(input.len() <= output.len());

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { simple_avx2(input.as_ptr(), output.as_mut_ptr(), input.len()) };
        }

        unsafe fn simple_avx2(input: *const f32, output: *mut f32, count: usize) {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::*;

            let mut i = 0;
            while i < count {
                let inp = input.offset(i as isize);
                let outp = output.offset(i as isize);
                let inv = _mm256_loadu_ps(inp);
                let constv = _mm256_load_ps([3.0f32; 8].as_ptr());
                let gev = _mm256_castps_si256(_mm256_cmp_ps(inv, constv, _CMP_GE_OS));
                let ltv = _mm256_andnot_si256(gev, _mm256_set1_epi64x(-1));
                let sqrtv = _mm256_sqrt_ps(inv);
                _mm256_maskstore_ps(outp, gev, sqrtv);
                let sqrv = _mm256_mul_ps(inv, inv);
                _mm256_maskstore_ps(outp, ltv, sqrv);
                i += 8;
            }
        }
    }

    // scalar fallback
    for i in 0..input.len() {
        let in1 = input[i];
        let mut out1 = &mut output[i];
        *out1 = if in1 < 3.0 {
            in1 * in1
        } else {
            in1.sqrt()
        }
    }
}

fn main() {
    let input = [0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let mut output = [0f32; 8];

    simple(&input, &mut output);
}
*/