// SPDX-License-Identifier: Apache-2.0

use handline_core::{decode_qr, sniff_frame};
use proptest::prelude::*;

proptest! {
    #[test]
    fn garbage_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..400)) {
        let _ = sniff_frame(&bytes);
        let lossy = String::from_utf8_lossy(&bytes);
        let _ = decode_qr(&lossy);
        let _ = decode_qr(&format!("handline:{lossy}"));
    }
}
