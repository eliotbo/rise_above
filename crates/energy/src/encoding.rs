pub trait EncodableToU32 {
    fn encode(&self, input: &mut u32, place: u8);
    fn encode_to_u32_with_precision(&self, input: &mut u32, place: u8, precision: u8);
}

impl EncodableToU32 for u8 {
    fn encode(&self, input: &mut u32, place: u8) {
        let precision = 8u8;
        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }

    fn encode_to_u32_with_precision(&self, input: &mut u32, place: u8, precision: u8) {
        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }
}

impl EncodableToU32 for u16 {
    fn encode(&self, input: &mut u32, place: u8) {
        let precision = 16u8;
        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }

    fn encode_to_u32_with_precision(&self, input: &mut u32, place: u8, precision: u8) {
        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }
}

impl EncodableToU32 for u32 {
    fn encode(&self, input: &mut u32, place: u8) {
        let precision = 32u8;

        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }

    fn encode_to_u32_with_precision(&self, input: &mut u32, place: u8, precision: u8) {
        let value_u32 = (*self as u32) << (place - precision);

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }
}

impl EncodableToU32 for f32 {
    /// Assumes that the precision is 32 and that the value is between 0 and 1
    fn encode(&self, input: &mut u32, place: u8) {
        let precision = 32u8;
        // let value_u32 = (*self as u32) << (place - precision);

        let value_u32 = (self * (1u32 << (precision - 1u8)) as f32) as u32;

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        *input = (*input & mask) | value_u32;
    }

    fn encode_to_u32_with_precision(&self, input: &mut u32, place: u8, precision: u8) {
        let value_f32_normalized = self * (1u32 << (precision - 1u8)) as f32;

        // println!("mask: {}", (1u32 << (precision - 1u8)));

        let delta_bits = (place - precision) as u32;
        //
        let value_u32 = (value_f32_normalized as u32) << delta_bits;

        let mut mask = 0;
        if precision < 32 {
            mask = u32::MAX - (((1 << precision) - 1) << (place - precision));
        }

        // println!("mask: {:#0b}", value_u32);

        *input = (*input & mask) | value_u32;
    }
}

pub trait DecodableToF32 {
    fn decode(&self, place: u8, precision: u8) -> f32;
}

impl DecodableToF32 for u32 {
    /// Assumes that the precision is 32
    fn decode(&self, place: u8, precision: u8) -> f32 {
        // let precision = 32u8;
        // println!("mask: {:#0b}", self);
        let value_u32 = self >> (place - precision);

        let mut mask = u32::MAX;
        if precision < 32 {
            mask = (1 << (precision)) - 1;
        }

        // println!("mask: {:#0b}", value_u32);
        let masked_value_u32 = value_u32 & mask;
        let value_f32 = masked_value_u32 as f32 / ((1u32 << (precision - 1u8)) as f32);

        value_f32
    }
}

/// Trait for converting an unsigned integer to a float that is between 0 and 1.
pub trait ConvertableToUnitf32 {
    fn to_f32(&self) -> f32;
}

impl ConvertableToUnitf32 for u8 {
    fn to_f32(&self) -> f32 {
        *self as f32 / 255.0
    }
}

impl ConvertableToUnitf32 for u16 {
    fn to_f32(&self) -> f32 {
        *self as f32 / 65535.0
    }
}

impl ConvertableToUnitf32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32 / 4294967295.0
    }
}

/// Trait for converting a float that is between 0 and 1 to an unsigned integer
pub trait ConvertableToU8 {
    fn to_u8(&self) -> u8;
}

impl ConvertableToU8 for f32 {
    fn to_u8(&self) -> u8 {
        (*self * 255.0) as u8
    }
}
