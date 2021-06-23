use core::fmt::{Display, LowerHex};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BcdNumber<T: LowerHex>(T);

impl<T: LowerHex> Display for BcdNumber<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

macro_rules! impl_bcd {
    ($type:ty; $new:ident; $encode:ident; $bcd_max:expr, $mask:expr) => {
        impl BcdNumber<$type> {
            pub fn $new(bcd_value: $type) -> Result<Self, ()> {
                let mut bcd = bcd_value;
                for _ in 0..2 * core::mem::size_of::<$type>() {
                    let nibble = bcd & 0x0F;
                    if nibble < 0x0A {
                        bcd >>= 4;
                    } else {
                        return Err(());
                    }
                }
                Ok(Self(bcd_value))
            }

            pub fn $encode(mut number: $type) -> Result<Self, ()> {
                if number <= $bcd_max {
                    let mut result = 0;
                    let mut mask = $mask;
                    while mask > 0 {
                        result <<= 4;
                        while number >= mask {
                            number -= mask;
                            result += 1;
                        }
                        mask /= 10;
                    }

                    Ok(Self(result))
                } else {
                    Err(())
                }
            }

            pub fn decode(self: Self) -> $type {
                let mut bcd = self.0;
                let mut result = 0;
                for _ in 0..2 * core::mem::size_of::<$type>() {
                    let nibble = bcd >> (8 * core::mem::size_of::<$type>() - 4);
                    result = result * 10 + nibble;
                    bcd <<= 4;
                }
                result
            }
        }
    };
}

impl_bcd!(u16; new_u16; encode_u16; 9999, 1000);
impl_bcd!(u32; new_u32; encode_u32; 9999_9999, 1000_0000);
impl_bcd!(u64; new_u64; encode_u64; 9999_9999_9999_9999, 1000_0000_0000_0000);

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn can_encode_u32() {
        assert_eq!(0x11223344, BcdNumber::encode_u32(11223344).unwrap().0);
        assert_eq!(0x99999999, BcdNumber::encode_u32(99999999).unwrap().0);
        assert!(BcdNumber::encode_u32(100000000).is_err());
    }

    #[test]
    pub fn can_decode_u32() {
        assert_eq!(11223344, BcdNumber::new_u32(0x11223344).unwrap().decode());
        assert_eq!(99999999, BcdNumber::new_u32(0x99999999).unwrap().decode());
        assert!(BcdNumber::<u32>::new_u32(0x11F23344).is_err());
    }
}
