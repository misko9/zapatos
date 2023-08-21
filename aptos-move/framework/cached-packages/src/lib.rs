// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_framework::ReleaseBundle;
// use once_cell::sync::Lazy;

pub mod aptos_framework_sdk_builder;
pub mod aptos_stdlib;
pub mod aptos_token_objects_sdk_builder;
pub mod aptos_token_sdk_builder;

use std::path::PathBuf;
// #[cfg(unix)]
// const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/head.mrb"));
// #[cfg(windows)]
// const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\head.mrb"));

// static HEAD_RELEASE_BUNDLE: Lazy<ReleaseBundle> = Lazy::new(|| {
//     bcs::from_bytes::<ReleaseBundle>(HEAD_RELEASE_BUNDLE_BYTES).expect("bcs succeeds")
// });

// /// Returns the release bundle for the current code.
// pub fn head_release_bundle() -> &'static ReleaseBundle {
//     &HEAD_RELEASE_BUNDLE
// }

// static HEAD_RELEASE_BUNDLE: Lazy<ReleaseBundle> = Lazy::new(|| {
//     bcs::from_bytes::<ReleaseBundle>(HEAD_RELEASE_BUNDLE_BYTES).expect("bcs succeeds")
// });
pub fn head_release_bundle() -> ReleaseBundle {
    let mrb_path = match std::env::var("MRB_PATH"){
      Ok(s) => s.parse::<PathBuf>().expect("could not parse path"),
      _ => {
        std::env::var("OUT_DIR")
          .expect("no env OUT_DIR found")
          .parse::<PathBuf>()
          .expect("could not parse path of env OUT_DIR")
          .join("head.mrb")
      }
    };

    let mrb_bytes = std::fs::read(mrb_path).expect("failed to read head.mrb from MRB_PATH, or OUT_PATH/head.mrb");
    let rls_bundle = bcs::from_bytes::<ReleaseBundle>(&mrb_bytes).expect("bcs succeeds");
    rls_bundle
}
