// NOTE: 0L: These natives follow the pattern of the `debug` native. It is implemented differently than the other natives, in that the gas context is not used.

use crate::natives::helpers::make_module_natives;
use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeFunction;
#[allow(unused_imports)]
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{Reference, Struct, Value},
};
use smallvec::smallvec;
use vdf::{VDFParams, VDF};
use std::{collections::VecDeque, sync::Arc};

use move_core_types::{account_address::AccountAddress, vm_status::StatusCode};


#[inline]
fn native_verify(ty_args: Vec<Type>, mut args: VecDeque<Value>) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 5);

    let wesolowski = pop_arg!(args, bool); // will do pietrezak if `false`.
    let security = pop_arg!(args, u64);
    let difficulty = pop_arg!(args, u64);
    let solution = pop_arg!(args, Reference).read_ref()?.value_as::<Vec<u8>>()?;
    let challenge = pop_arg!(args, Reference).read_ref()?.value_as::<Vec<u8>>()?;

    // refuse to try anything with a security parameter above 2048 or a difficulty above 3_000_000_001 (which is the target on Wesolowski)
    if (security > 2048) || (difficulty > 3_000_000_001) {
      return Ok(NativeResult::err(0.into(), StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into()));
    }

    let result = if wesolowski {
      if difficulty > 3_000_000_001 {
        return Ok(NativeResult::err(0.into(), StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into()));
      }

      let v = vdf::WesolowskiVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    } else {

      if difficulty > 900_000_000 {
        return Ok(NativeResult::err(0.into(), StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into()));
      }

      let v = vdf::PietrzakVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    };

    let return_values = smallvec![Value::bool(result.is_ok())];


    Ok(NativeResult::ok(0.into(), return_values))
}

pub fn make_native_verify() -> NativeFunction {
    Arc::new(
        move |_context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_verify(ty_args, args)
        },
    )
}


#[inline]
fn native_extract_address_from_challenge(ty_args: Vec<Type>, mut args: VecDeque<Value>) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    let challenge_vec = pop_arg!(args, Reference).read_ref()?.value_as::<Vec<u8>>()?;

    // We want to use Diem AuthenticationKey::derived_address() here but this creates
    // libra (and as a result cyclic) dependency which we definitely do not want
    const AUTHENTICATION_KEY_LENGTH: usize = 32;
    let auth_key_vec = &challenge_vec[..AUTHENTICATION_KEY_LENGTH];
    // Address derived from the last `AccountAddress::LENGTH` bytes of authentication key
    let mut array = [0u8; AccountAddress::LENGTH];
    array.copy_from_slice(
        &auth_key_vec[AUTHENTICATION_KEY_LENGTH - AccountAddress::LENGTH..]
    );
    let address = AccountAddress::new(array);

    let return_values = smallvec![
        Value::address(address), Value::vector_u8(auth_key_vec[..16].to_owned())
    ];

    Ok(NativeResult::ok(0.into(), return_values))
}

pub fn make_native_extract() -> NativeFunction {
    Arc::new(
        move |_context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_extract_address_from_challenge(ty_args, args)
        },
    )
}

pub fn make_all() -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("verify", make_native_verify()),
        ("extract_address_from_challenge", make_native_extract()),
    ];

    make_module_natives(natives)
}


#[test]
fn sanity_test_vdf() {
  let security = 512u16;
  let difficulty = 100;
  let challenge = hex::decode("aa").unwrap();
  let solution = hex::decode("0051dfa4c3341c18197b72f5e5eecc693eb56d408206c206d90f5ec7a75f833b2affb0ea7280d4513ab8351f39362d362203ff3e41882309e7900f470f0a27eeeb7b").unwrap();

  let v = vdf::PietrzakVDFParams(security).new();
  v.verify(&challenge, difficulty, &solution).unwrap();
}

#[test]
fn round_trip() {
    let pietrzak_vdf = vdf::PietrzakVDFParams(512).new();
    let solution = pietrzak_vdf.solve(b"\xaa", 100).unwrap();
    dbg!(&hex::encode(&solution));
    assert!(pietrzak_vdf.verify(b"\xaa", 100, &solution).is_ok());
}
