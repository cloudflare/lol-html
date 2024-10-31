use crate::rewriter::AsciiCompatibleEncoding;
use encoding_rs::Encoding;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// A charset encoding that can be shared and modified.
///
/// This is, for instance, used to adapt the charset dynamically in a [`crate::HtmlRewriter`] if it
/// encounters a `meta` tag that specifies the charset (that behavior is dependent on
/// [`crate::Settings::adjust_charset_on_meta_tag`]).
#[derive(Clone)]
pub struct SharedEncoding {
    encoding: Arc<AtomicPtr<Encoding>>,
}

impl SharedEncoding {
    #[must_use]
    pub fn new(encoding: AsciiCompatibleEncoding) -> Self {
        let encoding: &'static Encoding = encoding.into();
        Self {
            // *mut T doesn't have aliasing requirements
            encoding: Arc::new(AtomicPtr::new(ptr::from_ref(encoding).cast_mut())),
        }
    }

    #[must_use]
    pub fn get(&self) -> &'static Encoding {
        let encoding = self.encoding.load(Ordering::Relaxed);
        unsafe { &*encoding }
    }

    pub fn set(&self, encoding: AsciiCompatibleEncoding) {
        let encoding: &'static Encoding = encoding.into();
        self.encoding
            .store(ptr::from_ref(encoding).cast_mut(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use crate::base::SharedEncoding;
    use crate::AsciiCompatibleEncoding;
    use encoding_rs::Encoding;

    /// This serves as a map from integer to [`Encoding`], which allows more efficient
    /// sets/gets of the [`SharedEncoding`].
    static ALL_ENCODINGS: [&Encoding; 228] = [
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::IBM866_INIT,
        &encoding_rs::MACINTOSH_INIT,
        &encoding_rs::KOI8_R_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::BIG5_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::KOI8_R_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::IBM866_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::WINDOWS_1250_INIT,
        &encoding_rs::WINDOWS_1251_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::WINDOWS_1253_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::WINDOWS_1255_INIT,
        &encoding_rs::BIG5_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::WINDOWS_1256_INIT,
        &encoding_rs::IBM866_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::WINDOWS_1257_INIT,
        &encoding_rs::WINDOWS_1258_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::EUC_JP_INIT,
        &encoding_rs::KOI8_R_INIT,
        &encoding_rs::KOI8_R_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::KOI8_U_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::GB18030_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::BIG5_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::ISO_8859_8_I_INIT,
        &encoding_rs::KOI8_R_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::KOI8_U_INIT,
        &encoding_rs::WINDOWS_1250_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::WINDOWS_1251_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::WINDOWS_1253_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::WINDOWS_1255_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::BIG5_INIT,
        &encoding_rs::WINDOWS_1256_INIT,
        &encoding_rs::IBM866_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::WINDOWS_1257_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::WINDOWS_1258_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::UTF_16BE_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::EUC_JP_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_13_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::ISO_8859_14_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::MACINTOSH_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_13_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::ISO_8859_14_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::BIG5_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::ISO_8859_13_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_14_INIT,
        &encoding_rs::WINDOWS_874_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_16_INIT,
        &encoding_rs::ISO_8859_10_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::ISO_8859_15_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::UTF_16BE_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::MACINTOSH_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_I_INIT,
        &encoding_rs::SHIFT_JIS_INIT,
        &encoding_rs::MACINTOSH_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::ISO_2022_JP_INIT,
        &encoding_rs::ISO_2022_JP_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::WINDOWS_1250_INIT,
        &encoding_rs::WINDOWS_1251_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::WINDOWS_1253_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::WINDOWS_1255_INIT,
        &encoding_rs::WINDOWS_1256_INIT,
        &encoding_rs::WINDOWS_1257_INIT,
        &encoding_rs::WINDOWS_1258_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_I_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::EUC_KR_INIT,
        &encoding_rs::X_MAC_CYRILLIC_INIT,
        &encoding_rs::X_USER_DEFINED_INIT,
        &encoding_rs::GBK_INIT,
        &encoding_rs::UTF_16LE_INIT,
        &encoding_rs::WINDOWS_1252_INIT,
        &encoding_rs::ISO_8859_2_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::ISO_8859_3_INIT,
        &encoding_rs::ISO_8859_4_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::WINDOWS_1254_INIT,
        &encoding_rs::ISO_8859_7_INIT,
        &encoding_rs::X_MAC_CYRILLIC_INIT,
        &encoding_rs::REPLACEMENT_INIT,
        &encoding_rs::ISO_8859_6_INIT,
        &encoding_rs::ISO_8859_8_INIT,
        &encoding_rs::UTF_8_INIT,
        &encoding_rs::ISO_8859_5_INIT,
        &encoding_rs::EUC_JP_INIT,
    ];

    #[test]
    fn test_encoding_round_trip() {
        let shared_encoding = SharedEncoding::new(AsciiCompatibleEncoding::utf_8());

        for encoding in ALL_ENCODINGS {
            if let Some(ascii_compat_encoding) = AsciiCompatibleEncoding::new(encoding) {
                shared_encoding.set(ascii_compat_encoding);
                assert_eq!(shared_encoding.get(), encoding);
            }
        }
    }
}
