use core::slice;
use std::ffi::{c_char, CStr};

use hardfiskur_core::board::Board;
use hardfiskur_engine::evaluation::{
    trace::{EvalParameters, EvalTrace, Parameter},
    EvalContext,
};
use zerocopy::{transmute_mut, transmute_ref};

#[no_mangle]
pub extern "C" fn hf_parameter_len() -> usize {
    EvalParameters::LEN
}

#[no_mangle]
pub extern "C" fn hf_initial_parameters(out_parameters: *mut [f64; 2], out_parameters_size: usize) {
    let out_parameters = if out_parameters.is_null() {
        &mut []
    } else {
        unsafe { slice::from_raw_parts_mut(out_parameters, out_parameters_size) }
    };
    fill_initial_parameters_internal(out_parameters);
}

fn fill_initial_parameters_internal(out_parameters: &mut [[f64; 2]]) {
    let out_parameters: &mut [Parameter; EvalParameters::LEN] = out_parameters
        .try_into()
        .expect("Wrong parameters length in initial_parameters");
    let out_parameters: &mut EvalParameters = transmute_mut!(out_parameters);

    *out_parameters = EvalParameters::default();
}

#[no_mangle]
pub extern "C" fn hf_print_parameters(parameters: *const [f64; 2], parameters_size: usize) {
    let parameters = if parameters.is_null() {
        &[]
    } else {
        unsafe { slice::from_raw_parts(parameters, parameters_size) }
    };
    print_parameters_internal(parameters);
}

fn print_parameters_internal(parameters: &[[f64; 2]]) {
    let parameters: &[Parameter; EvalParameters::LEN] = parameters
        .try_into()
        .expect("Wrong parameters length in print_parameters");
    let parameters: &EvalParameters = transmute_ref!(parameters);

    println!("{parameters}");
}

#[no_mangle]
pub extern "C" fn hf_get_fen_eval_result(
    fen: *const c_char,
    out_coeffs: *mut i16,
    out_coeffs_size: usize,
) {
    let fen = if fen.is_null() {
        ""
    } else {
        unsafe { CStr::from_ptr(fen) }
            .to_str()
            .expect("Cuold not convert FEN to &str")
    };

    let out_coeffs = if out_coeffs.is_null() {
        &mut []
    } else {
        unsafe { slice::from_raw_parts_mut(out_coeffs, out_coeffs_size) }
    };
    let out_coeffs: &mut [i16; EvalTrace::LEN] = out_coeffs
        .try_into()
        .expect("Wrong coefficient length in get_fen_eval_result");

    get_fen_eval_result_internal(fen, out_coeffs);
}

fn get_fen_eval_result_internal(fen: &str, out_coeffs: &mut [i16; EvalTrace::LEN]) {
    let board = Board::try_parse_fen(fen).expect("Could not parse FEN");
    let trace: &mut EvalTrace = transmute_mut!(out_coeffs);
    // trace.zero();

    let mut new_trace = EvalTrace::default();
    let (_score, _phase) = EvalContext::new(&board).evaluate_ex(&mut new_trace);
    *trace = new_trace;
}
