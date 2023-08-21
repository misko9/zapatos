// Copyright © Aptos Foundation

use crate::natives::create_signer::CreateSignerGasParameters;
use crate::{
    natives::helpers::{make_safe_native, SafeNativeContext, SafeNativeError, SafeNativeResult},
    safely_pop_arg,
};
use aptos_types::on_chain_config::{Features, TimedFeatures};
use move_core_types::{account_address::AccountAddress, vm_status::StatusCode};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use std::{collections::VecDeque, sync::Arc};

//////// 0L ////////
use vdf::{VDFParams, VDF};
use move_vm_types::values::VectorRef;

/***************************************************************************************************
 * native fun verify
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/

/// check if a vdf proof is valid
pub(crate) fn native_vdf_verify(
    gas_params: &CreateSignerGasParameters,
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 5);

    context.charge(gas_params.base)?;

    let wesolowski = safely_pop_arg!(arguments, bool); // will do pietrezak if `false`.
    let security = safely_pop_arg!(arguments, u64);
    let difficulty = safely_pop_arg!(arguments, u64);
    let ref_s = safely_pop_arg!(arguments, VectorRef);
    let solution = ref_s.as_bytes_ref();
    let ref_c = safely_pop_arg!(arguments, VectorRef);
    let challenge = ref_c.as_bytes_ref();

    // refuse to try anything with a security parameter above 2048 or a difficulty above 3_000_000_001 (which is the target on Wesolowski)
    if (security > 2048) || (difficulty > 3_000_000_001) {
      return Err(SafeNativeError::Abort {
            abort_code: StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into(),
        })
    }

    let result = if wesolowski {
      let v = vdf::WesolowskiVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    } else {

      if difficulty > 900_000_000 {
        return Err(SafeNativeError::Abort {
            abort_code: StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE.into(),
        })
      }

      let v = vdf::PietrzakVDFParams(security as u16).new();
      v.verify(&challenge, difficulty, &solution)
    };

    Ok(smallvec![Value::bool(result.is_ok())])
}

/// convenience to get an address from the vdf challenge.
pub(crate) fn native_extract_address_from_challenge(
    gas_params: &CreateSignerGasParameters,
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
  debug_assert!(ty_args.is_empty());

  context.charge(gas_params.base)?;

  let ref_v = safely_pop_arg!(arguments, VectorRef);
  let challenge_vec = ref_v.as_bytes_ref();
  const AUTHENTICATION_KEY_LENGTH: usize = 32;
  let auth_key_vec = &challenge_vec[..AUTHENTICATION_KEY_LENGTH];
  // Address derived from the last `AccountAddress::LENGTH` bytes of authentication key
  let mut array = [0u8; AUTHENTICATION_KEY_LENGTH];
  array.copy_from_slice(
    &auth_key_vec[..AUTHENTICATION_KEY_LENGTH]
  );
  let address = AccountAddress::new(array);

  let return_values = smallvec![
    Value::address(address), Value::vector_u8(array)
  ];

  Ok(return_values)
}

pub fn make_all(
    gas_param: CreateSignerGasParameters,
    timed_features: TimedFeatures,
    features: Arc<Features>,
) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "verify",
        make_safe_native(gas_param.clone(), timed_features.clone(), features.clone(), native_vdf_verify),
    ),
    (
        "extract_address_from_challenge",
        make_safe_native(gas_param, timed_features, features, native_extract_address_from_challenge),
    )];

    crate::natives::helpers::make_module_natives(natives)
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
