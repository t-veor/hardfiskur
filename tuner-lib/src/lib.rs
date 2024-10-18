use core::slice;

use hardfiskur_engine::evaluation::trace::{EvalParameters, Parameter};
use zerocopy::{transmute_mut, transmute_ref};

#[no_mangle]
pub extern "C" fn parameter_len() -> usize {
    EvalParameters::LEN
}

#[no_mangle]
pub extern "C" fn initial_parameters(out_parameters: *mut [f32; 2], out_parameters_size: usize) {
    let out_parameters = unsafe { slice::from_raw_parts_mut(out_parameters, out_parameters_size) };
    fill_initial_parameters_internal(out_parameters);
}

fn fill_initial_parameters_internal(out_parameters: &mut [[f32; 2]]) {
    let out_parameters: &mut [Parameter; EvalParameters::LEN] = out_parameters
        .try_into()
        .expect("Wrong parameters length in initial_parameters");
    let out_parameters: &mut EvalParameters = transmute_mut!(out_parameters);

    *out_parameters = EvalParameters::default();
}

#[no_mangle]
pub extern "C" fn print_parameters(parameters: *const [f32; 2], parameters_size: usize) {
    let parameters = unsafe { slice::from_raw_parts(parameters, parameters_size) };
    print_parameters_internal(parameters);
}

fn print_parameters_internal(parameters: &[[f32; 2]]) {
    let parameters: &[Parameter; EvalParameters::LEN] = parameters
        .try_into()
        .expect("Wrong parameters length in print_parameters");
    let parameters: &EvalParameters = transmute_ref!(parameters);

    println!("{parameters}");
}
