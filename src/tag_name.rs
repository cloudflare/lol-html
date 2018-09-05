// NOTE: unfortunately `static fn` is still unavaliable in stable,
// so we need to use manually precalculated values in this enum.
// Consistency between hashing algorithm and these values is guaranteed
// by the dedicated test.
#[repr(u64)]
#[derive(Debug, Copy, Clone)]
pub enum TagName {
    Svg = 25452u64,
    Math = 596781u64,
    Textarea = 870730390854u64,
    Title = 26699306u64,
    Plaintext = 23680792701881u64,
    Script = 814463673u64,
    Style = 26016298u64,
    Iframe = 482056778u64,
    Xmp = 30293u64,
    Noembed = 21083266377u64,
    Noframes = 674703296856u64,
    Noscript = 675124329145u64,
    B = 7u64,
    Big = 7628u64,
    Blockquote = 265678647808810u64,
    Body = 250174u64,
    Br = 247u64,
    Center = 279569751u64,
    Code = 282922u64,
    Dd = 297u64,
    Div = 9691u64,
    Dl = 305u64,
    Dt = 313u64,
    Em = 338u64,
    Embed = 11083081u64,
    H1 = 416u64,
    H2 = 417u64,
    H3 = 418u64,
    H4 = 419u64,
    H5 = 420u64,
    H6 = 421u64,
    Head = 436425u64,
    Hr = 439u64,
    I = 14u64,
    Img = 14924u64,
    Li = 558u64,
    Listing = 18749373036u64,
    Menu = 600698u64,
    Meta = 600870u64,
    Nobr = 643319u64,
    Ol = 657u64,
    P = 21u64,
    Pre = 22250u64,
    Ruby = 780542u64,
    S = 24u64,
    Small = 25762353u64,
    Span = 808147u64,
    Strong = 832295532u64,
    Strike = 832289290u64,
    Sub = 25415u64,
    Sup = 25429u64,
    Table = 26418730u64,
    Tt = 825u64,
    U = 26u64,
    Ul = 849u64,
    Var = 27863u64,
    Mi = 590u64,
    Mo = 596u64,
    Mn = 595u64,
    Ms = 600u64,
    Mtext = 19704761u64,
    Desc = 305928u64,
    ForeignObject = 13428975859192539417u64,
    Font = 381561u64,
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
    // NOTE: All standard tag names contain only ASCII alpha characters
    // and digits from 1 to 6 (in numbered header tags, i.e. <h1> - <h6>).
    // Considering that tag names are case insensitive we have only
    // 26 + 6 = 32 characters. Thus, single character can be encoded in
    // 5 bits and we can fit up to 64 / 5 ≈ 12 characters in a 64-bit
    // integer. This is enough to encode all standard tag names, so
    // we can just compare integers instead of expensive string
    // comparison for tag names.
    //
    // The original idea of this tag hash-like thing belongs to Ingvar
    // Stepanyan and was implemented in lazyhtml. So, kudos to him for
    // comming up with this cool optimisation. This implementation differs
    // from the original one as it adds ability to encode digits from 1
    // to 6 which allows us to encode numbered header tags.
    //
    // In this implementation we reserve numbers from 0 to 5 for digits
    // from 1 to 6 and numbers from 6 to 31 for ASCII alphas. Otherwise,
    // if we use numbers from 0 to 25 for ASCII alphas we'll have an
    // ambiguity for repetitative `a` characters: both `a`,
    // `aaa` and even `aaaaa` will give us 0 as a hash. It's still a case
    // for digits, but considering that tag name can't start from digit
    // we are safe here, since we'll just get first character shifted left
    // by zeroes as repetitave 1 digits get added to the hash.
    #[inline]
    pub fn update_hash(hash: Option<u64>, ch: u8) -> Option<u64> {
        let mut hash = hash;

        if let Some(h) = hash {
            // NOTE: check if we still have space for yet another
            // character and if not then invalidate the hash.
            // Note, that we can't have `1` (which is encoded as 0b00000) as
            // a first character of a tag name, so it's safe to perform
            // check this way.
            hash = if h >> (64 - 5) == 0 {
                match ch {
                    // NOTE: apply 0x1F mask on ASCII alpha to convert it to the
                    // number from 1 to 26 (character case is controlled by one of
                    // upper bits which we eliminate with the mask). Then add
                    // 5, since numbers from 0 to 5 are reserved for digits.
                    // Aftwerards put result as 5 lower bits of the hash.
                    b'a'...b'z' | b'A'...b'Z' => Some((h << 5) | (ch as u64 & 0x1F) + 5),

                    // NOTE: apply 0x0F on ASCII digit to convert it to number
                    // from 1 to 6. Then substract 1 to make it zero-based.
                    // Afterwards, put result as lower bits of the hash.
                    b'1'...b'6' => Some((h << 5) | (ch as u64 & 0x0F) - 1),

                    // NOTE: for any other characters hash function is not
                    // applicable, so we completely invalidate the hash.
                    _ => None,
                }
            } else {
                None
            };
        }

        hash
    }

    pub fn get_hash(name: &str) -> Option<u64> {
        let mut hash = Some(0);

        for ch in name.bytes() {
            hash = TagName::update_hash(hash, ch);
        }

        hash
    }
}
