//! All standard tag names contain only ASCII alpha characters
//! and digits from 1 to 6 (in numbered header tags, i.e. <h1> - <h6>).
//! Considering that tag names are case insensitive we have only
//! 26 + 6 = 32 characters. Thus, single character can be encoded in
//! 5 bits and we can fit up to 64 / 5 ≈ 12 characters in a 64-bit
//! integer. This is enough to encode all standard tag names, so
//! we can just compare integers instead of expensive string
//! comparison for tag names.
//!
//! The original idea of this tag hash-like thing belongs to Ingvar
//! Stepanyan and was implemented in lazyhtml. So, kudos to him for
//! comming up with this cool optimisation. This implementation differs
//! from the original one as it adds ability to encode digits from 1
//! to 6 which allows us to encode numbered header tags.
//!
//! In this implementation we reserve numbers from 0 to 5 for digits
//! from 1 to 6 and numbers from 6 to 31 for ASCII alphas. Otherwise,
//! if we use numbers from 0 to 25 for ASCII alphas we'll have an
//! ambiguity for repetitative `a` characters: both `a`,
//! `aaa` and even `aaaaa` will give us 0 as a hash. It's still a case
//! for digits, but considering that tag name can't start with a digit
//! we are safe here, since we'll just get first character shifted left
//! by zeroes as repetitave 1 digits get added to the hash.

// NOTE: unfortunately `static fn` is still unavaliable in stable,
// so we need to use manually precalculated values in this enum.
// Consistency between hashing algorithm and these values is guaranteed
// by the dedicated test.
#[repr(u64)]
#[derive(Debug, Copy, Clone)]
pub enum TagName {
    B = 7u64,
    Big = 7_628u64,
    Blockquote = 265_678_647_808_810u64,
    Body = 250_174u64,
    Br = 247u64,
    Center = 279_569_751u64,
    Code = 282_922u64,
    Dd = 297u64,
    Desc = 305_928u64,
    Div = 9691u64,
    Dl = 305u64,
    Dt = 313u64,
    Em = 338u64,
    Embed = 11_083_081u64,
    Font = 381_561u64,
    ForeignObject = 13_428_975_859_192_539_417u64,
    Frameset = 402_873_737_561u64,
    H1 = 416u64,
    H2 = 417u64,
    H3 = 418u64,
    H4 = 419u64,
    H5 = 420u64,
    H6 = 421u64,
    Head = 436_425u64,
    Hr = 439u64,
    I = 14u64,
    Iframe = 482_056_778u64,
    Img = 14_924u64,
    Input = 15_325_017u64,
    Keygen = 548_352_339u64,
    Li = 558u64,
    Listing = 18_749_373_036u64,
    Math = 596_781u64,
    Menu = 600_698u64,
    Meta = 600_870u64,
    Mi = 590u64,
    Mn = 595u64,
    Mo = 596u64,
    Ms = 600u64,
    Mtext = 19_704_761u64,
    Nobr = 643_319u64,
    Noembed = 21_083_266_377u64,
    Noframes = 674_703_296_856u64,
    Noscript = 675_124_329_145u64,
    Ol = 657u64,
    P = 21u64,
    Plaintext = 23_680_792_701_881u64,
    Pre = 22_250u64,
    Ruby = 780_542u64,
    S = 24u64,
    Script = 814_463_673u64,
    Select = 816_359_705u64,
    Small = 25_762_353u64,
    Span = 808_147u64,
    Strike = 832_289_290u64,
    Strong = 832_295_532u64,
    Style = 26_016_298u64,
    Sub = 25_415u64,
    Sup = 25_429u64,
    Svg = 25_452u64,
    Table = 26_418_730u64,
    Template = 870_357_441_322u64,
    Textarea = 870_730_390_854u64,
    Title = 26_699_306u64,
    Tt = 825u64,
    U = 26u64,
    Ul = 849u64,
    Var = 27_863u64,
    Xmp = 30_293u64,
}

impl PartialEq<u64> for TagName {
    fn eq(&self, hash: &u64) -> bool {
        *self as u64 == *hash
    }
}

impl PartialEq<TagName> for u64 {
    fn eq(&self, hash: &TagName) -> bool {
        *self == *hash as u64
    }
}

impl TagName {
    #[inline]
    pub fn update_hash(hash: &mut Option<u64>, ch: u8) {
        if let Some(h) = *hash {
            // NOTE: check if we still have space for yet another
            // character and if not then invalidate the hash.
            // Note, that we can't have `1` (which is encoded as 0b00000) as
            // a first character of a tag name, so it's safe to perform
            // check this way.
            *hash = if h >> (64 - 5) == 0 {
                match ch {
                    // NOTE: apply 0x1F mask on ASCII alpha to convert it to the
                    // number from 1 to 26 (character case is controlled by one of
                    // upper bits which we eliminate with the mask). Then add
                    // 5, since numbers from 0 to 5 are reserved for digits.
                    // Aftwerards put result as 5 lower bits of the hash.
                    b'a'..=b'z' | b'A'..=b'Z' => Some((h << 5) | ((u64::from(ch) & 0x1F) + 5)),

                    // NOTE: apply 0x0F mask on ASCII digit to convert it to number
                    // from 1 to 6. Then substract 1 to make it zero-based.
                    // Afterwards, put result as lower bits of the hash.
                    b'1'..=b'6' => Some((h << 5) | ((u64::from(ch) & 0x0F) - 1)),

                    // NOTE: for any other characters hash function is not
                    // applicable, so we completely invalidate the hash.
                    _ => None,
                }
            } else {
                None
            };
        }
    }

    pub fn get_hash(name: &str) -> Option<u64> {
        let mut hash = Some(0);

        for ch in name.bytes() {
            TagName::update_hash(&mut hash, ch);
        }

        hash
    }
}

macro_rules! tag_is_one_of {
    ($tag_name_hash:expr, [$($tag:ident),+]) => {
        $($tag_name_hash == TagName::$tag)||+
    };
}
