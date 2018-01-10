extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};

pub struct BitGet<'a> {
    data: &'a [u8],
    current: u64,
    position: i32,
}

const BIT_MASKS: [u32; 33] = [
    0x00000000, 0x00000001, 0x00000003, 0x00000007, 0x0000000f, 0x0000001f, 0x0000003f, 0x0000007f,
    0x000000ff, 0x000001ff, 0x000003ff, 0x000007ff, 0x00000fff, 0x00001fff, 0x00003fff, 0x00007fff,
    0x0000ffff, 0x0001ffff, 0x0003ffff, 0x0007ffff, 0x000fffff, 0x001fffff, 0x003fffff, 0x007fffff,
    0x00ffffff, 0x01ffffff, 0x03ffffff, 0x07ffffff, 0x0fffffff, 0x1fffffff, 0x3fffffff, 0x7fffffff,
    0xffffffff,
];

macro_rules! gen_read_unchecked {
    ($name:ident, $t:ty, $bits:expr) => (
    pub fn $name(&mut self) -> $t {
        if self.position <= 64 - $bits {
            let res = (self.current >> self.position) as $t;
            self.position += $bits;
            res
        } else if self.position < 64 {
            let shifted = self.position;
            let little = (self.current >> shifted) as $t;
            self.read_unchecked();
            let big = self.current >> self.position << (64 - shifted);
            self.position += $bits - (64 - shifted);
            (big as $t) + little
        } else {
            self.read_unchecked();
            let res = (self.current >> self.position) as $t;
            self.position += $bits;
            res
        }
    });
}

macro_rules! gen_read {
    ($name:ident, $t:ty, $bits:expr) => (
    pub fn $name(&mut self) -> Option<$t> {
        if self.position <= 64 - $bits {
            let res = (self.current >> self.position) as $t;
            self.position += $bits;
            Some(res)
        } else if self.position < 64 {
            let shifted = self.position;
            let little = (self.current >> shifted) as $t;
            self.read().map(|_| {
                let big = self.current >> self.position << (64 - shifted);
                self.position += $bits - (64 - shifted);
                (big as $t) + little
            })
        } else {
            self.read().map(|_| {
                let res = (self.current >> self.position) as $t;
                self.position += $bits;
                res
            })
        }
    });
}

impl<'a> BitGet<'a> {
    pub fn new(data: &'a [u8]) -> BitGet<'a> {
        BitGet {
            data: data,
            current: 0,
            position: 64,
        }
    }

    gen_read!(read_i8, i8, 8);
    gen_read!(read_u8, u8, 8);
    gen_read!(read_i16, i16, 16);
    gen_read!(read_u16, u16, 16);
    gen_read!(read_i32, i32, 32);
    gen_read!(read_u32, u32, 32);
    gen_read!(read_i64, i64, 64);
    gen_read!(read_u64, u64, 64);

    gen_read_unchecked!(read_i8_unchecked, i8, 8);
    gen_read_unchecked!(read_u8_unchecked, u8, 8);
    gen_read_unchecked!(read_i16_unchecked, i16, 16);
    gen_read_unchecked!(read_u16_unchecked, u16, 16);
    gen_read_unchecked!(read_i32_unchecked, i32, 32);
    gen_read_unchecked!(read_u32_unchecked, u32, 32);
    gen_read_unchecked!(read_i64_unchecked, i64, 64);
    gen_read_unchecked!(read_u64_unchecked, u64, 64);

    pub fn read_i32_bits_unchecked(&mut self, bits: i32) -> i32 {
        self.read_u32_bits_unchecked(bits) as i32
    }

    /// Assumes that the number of bits are available in the bitstream and reads them into a u32
    pub fn read_u32_bits_unchecked(&mut self, bits: i32) -> u32 {
        if self.position <= 64 - bits {
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bits as usize];
            self.position += bits;
            res
        } else if self.position < 64 {
            let shifted = self.position;
            let little = (self.current >> shifted) as u32;
            self.read_unchecked();
            let had_read = 64 - shifted;
            let to_read = bits - had_read;
            let big = ((self.current >> self.position << had_read) as u32)
                & BIT_MASKS[bits as usize];
            self.position += to_read;
            big + little
        } else {
            self.read_unchecked();
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bits as usize];
            self.position += bits;
            res
        }
    }

    /// If the number of bits are available from the bitstream, read them into a u32
    pub fn read_u32_bits(&mut self, bits: i32) -> Option<u32> {
        if self.position <= 64 - bits {
            let res = ((self.current >> self.position) as u32) & BIT_MASKS[bits as usize];
            self.position += bits;
            Some(res)
        } else if self.position < 64 {
            let shifted = self.position;
            let little = (self.current >> shifted) as u32;
            self.read().map(|_| {
                let had_read = 64 - shifted;
                let to_read = bits - had_read;
                let big = ((self.current >> self.position << had_read) as u32)
                    & BIT_MASKS[bits as usize];
                self.position += to_read;
                big + little
            })
        } else {
            self.read().map(|_| {
                let res = ((self.current >> self.position) as u32) & BIT_MASKS[bits as usize];
                self.position += bits;
                res
            })
        }
    }

    /// Returns if the bitstream has no more bits left
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.position == 64
    }

    /// Approximately the number of bytes left (an underestimate)
    pub fn byte_size_lower_bound(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of bits left in the bitstream (exact)
    pub fn bit_size(&self) -> usize {
        (64 - self.position as usize) + self.data.len() * 8
    }

    fn ensure_current(&mut self) -> Option<()> {
        if self.position == 64 {
            self.read()
        } else {
            Some(())
        }
    }

    /// Advances bitstream to the next section if availablet. Don't assume that `position` is zero
    /// after this method, as the tail of the stream is packed into the highest bits.
    fn read(&mut self) -> Option<()> {
        if self.data.len() < 8 {
            if self.data.is_empty() {
                None
            } else {
                self.position = 64 - (self.data.len() * 8) as i32;
                self.current = 0;
                for i in 0..self.data.len() {
                    self.current += u64::from(self.data[i]) << (i * 8)
                }
                self.current <<= 8 * (8 - self.data.len() as i32);
                self.data = &self.data[self.data.len()..];
                Some(())
            }
        } else {
            self.current = LittleEndian::read_u64(self.data);
            self.position = 0;
            self.data = &self.data[8..];
            Some(())
        }
    }

    /// Advances bitstream to the next section. Will panic if no more data is present. Don't assume
    /// that `position` is zero after this method, as the tail of the stream is packed into the
    /// highest bits.
    fn read_unchecked(&mut self) {
        if self.data.len() < 8 {
            if self.data.is_empty() {
                panic!("Unchecked read when no data")
            } else {
                self.position = 64 - (self.data.len() * 8) as i32;
                self.current = 0;
                for i in 0..self.data.len() {
                    self.current += u64::from(self.data[i]) << (i * 8)
                }
                self.current <<= 8 * (8 - self.data.len() as i32);
                self.data = &self.data[self.data.len()..];
            }
        } else {
            self.current = LittleEndian::read_u64(self.data);
            self.position = 0;
            self.data = &self.data[8..];
        }
    }

    /// Reads a bit from the bitstream if available
    pub fn read_bit(&mut self) -> Option<bool> {
        self.ensure_current().map(|_| {
            let res = self.current & (1 << self.position);
            self.position += 1;
            res != 0
        })
    }

    /// Reads a bit from the bitstream
    pub fn read_bit_unchecked(&mut self) -> bool {
        if self.position == 64 {
            self.read_unchecked();
        }

        let res = self.current & (1 << self.position);
        self.position += 1;
        res != 0
    }

    /// Reads a `f32` from the bitstream if available
    pub fn read_f32(&mut self) -> Option<f32> {
        self.read_u32().map(|x| f32::from_bits(x))
    }

    /// Reads a `f32` from the bitstream
    pub fn read_f32_unchecked(&mut self) -> f32 {
        f32::from_bits(self.read_u32_unchecked())
    }

    /// Reads a value that takes up at most `bits` bits and doesn't exceed `max`. This function
    /// *assumes* that `max` has the same bitwidth as `bits`. It doesn't make sense to call this
    /// function `bits = 8` and `max = 30`, you'd change your argument to `bits = 5`. If `bits` are
    /// not available return `None`
    pub fn read_bits_max(&mut self, bits: i32, max: i32) -> Option<u32> {
        self.read_u32_bits(bits - 1).and_then(|data| {
            let max = max as u32;
            let up = data + (1 << (bits - 1));
            if up >= max {
                Some(data)
            } else {
                // Check the next bit
                self.read_bit().map(|x| if x { up } else { data })
            }
        })
    }

    /// Reads a value that takes up at most `bits` bits and doesn't exceed `max`. This function
    /// *assumes* that `max` has the same bitwidth as `bits`. It doesn't make sense to call this
    /// function `bits = 8` and `max = 30`, you'd change your argument to `bits = 5`
    pub fn read_bits_max_unchecked(&mut self, bits: i32, max: i32) -> u32 {
        let data = self.read_u32_bits_unchecked(bits - 1);
        let max = max as u32;

        // If the next bit is on, what would our value be
        let up = data + (1 << (bits - 1));

        // If we have the potential to equal or exceed max don't read the next bit, else read the
        // next bit
        if up >= max || !self.read_bit_unchecked() {
            data
        } else {
            up
        }
    }

    /// If the next bit is available and on, decode the next chunk of data (which can return None).
    ///   - None: Not enough data was available
    ///   - Some(None): Bit was off so data not decoded
    ///   - Some(x): Bit was on and data was decoded
    pub fn if_get<T, F>(&mut self, mut f: F) -> Option<Option<T>>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        self.read_bit()
            .and_then(|bit| if bit { f(self).map(Some) } else { Some(None) })
    }

    /// If the next bit is available and on, decode the next chunk of data (which can return None).
    ///   - Some(None): Bit was off so data not decoded
    ///   - Some(x): Bit was on and data was decoded
    pub fn if_get_unchecked<T, F>(&mut self, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Self) -> T,
    {
        if self.read_bit_unchecked() {
            Some(f(self))
        } else {
            None
        }
    }

    /// If the number of requested bytes are available return them to the client
    pub fn read_bytes(&mut self, bytes: i32) -> Option<Vec<u8>> {
        let off = if self.position % 8 == 0 { 0 } else { 1 };
        let bytes_in_position = 8 - self.position / 8;
        if (bytes_in_position - off) + (self.data.len() as i32) < bytes {
            None
        } else {
            let mut res = Vec::with_capacity(bytes as usize);
            for _ in 0..bytes {
                res.push(self.read_u8_unchecked());
            }
            Some(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BitGet;

    #[test]
    fn test_bit_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.byte_size_lower_bound(), 2);
        assert_eq!(bitter.bit_size(), 16);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.byte_size_lower_bound(), 0);
        assert_eq!(bitter.bit_size(), 15);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);

        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);
        assert_eq!(bitter.read_bit().unwrap(), true);
        assert_eq!(bitter.read_bit().unwrap(), false);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_unchecked_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);

        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bit_unchecked(), true);
        assert_eq!(bitter.read_bit_unchecked(), false);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_unchecked_bits_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);

        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 1);
        assert_eq!(bitter.read_u32_bits_unchecked(1), 0);

        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_bit_bits_reads() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));
        assert_eq!(bitter.read_u32_bits(1), Some(1));
        assert_eq!(bitter.read_u32_bits(1), Some(0));

        assert_eq!(bitter.read_u32_bits(1), None);
    }

    #[test]
    fn test_read_bytes() {
        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bytes(2), Some(vec![0b1010_1010, 0b0101_0101]));

        let mut bitter = BitGet::new(&[0b1010_1010, 0b0101_0101]);
        assert_eq!(bitter.read_bit_unchecked(), false);
        assert_eq!(bitter.read_bytes(2), None);
        assert_eq!(bitter.read_bytes(1), Some(vec![0b1101_0101]));
    }

    #[test]
    fn test_u8_reads() {
        let mut bitter = BitGet::new(&[0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2]);
        assert_eq!(bitter.read_u8(), Some(0xff));
        assert_eq!(bitter.read_u8(), Some(0xfe));
        assert_eq!(bitter.read_u8(), Some(0xfa));
        assert_eq!(bitter.read_u8(), Some(0xf7));
        assert_eq!(bitter.read_u8(), Some(0xf5));
        assert_eq!(bitter.read_u8(), Some(0xf0));
        assert_eq!(bitter.read_u8(), Some(0xb1));
        assert_eq!(bitter.read_u8(), Some(0xb2));
        assert_eq!(bitter.read_u8(), None);
    }

    #[test]
    fn test_u8_unchecked_reads() {
        let mut bitter = BitGet::new(&[0xff, 0xfe, 0xfa, 0xf7, 0xf5, 0xf0, 0xb1, 0xb2]);
        assert_eq!(bitter.read_u8_unchecked(), 0xff);
        assert_eq!(bitter.read_u8_unchecked(), 0xfe);
        assert_eq!(bitter.read_u8_unchecked(), 0xfa);
        assert_eq!(bitter.read_u8_unchecked(), 0xf7);
        assert_eq!(bitter.read_u8_unchecked(), 0xf5);
        assert_eq!(bitter.read_u8_unchecked(), 0xf0);
        assert_eq!(bitter.read_u8_unchecked(), 0xb1);
        assert_eq!(bitter.read_u8_unchecked(), 0xb2);
        assert_eq!(bitter.read_u8(), None);
    }

    #[test]
    fn test_u32_reads() {
        let mut bitter = BitGet::new(&[
            0xff,
            0x00,
            0xab,
            0xcd,
            0b1111_1110,
            0b0000_0001,
            0b0101_0110,
            0b1001_1011,
            0b0101_0101,
        ]);
        assert_eq!(bitter.read_u32(), Some(0xcdab00ff));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_u32(), Some(0xcdab00ff));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_u32(), None);
    }

    #[test]
    fn test_f32_reads() {
        let mut bitter = BitGet::new(&[
            0b0111_1011,
            0b0001_0100,
            0b1010_1110,
            0b0011_1101,
            0b1111_0110,
            0b0010_1000,
            0b0101_1100,
            0b0111_1011,
            0b0000_0010,
        ]);
        assert_eq!(bitter.read_f32(), Some(0.085));
        assert_eq!(bitter.read_bit(), Some(false));
        assert_eq!(bitter.read_f32(), Some(0.085));
    }

    #[test]
    fn test_u32_bits() {
        let mut bitter = BitGet::new(&[0xff, 0xdd, 0xee, 0xff, 0xdd, 0xee]);
        assert_eq!(bitter.read_u32_bits(10), Some(0x1ff));
        assert_eq!(bitter.read_u32_bits(10), Some(0x3b7));
        assert_eq!(bitter.read_u32_bits(10), Some(0x3fe));
        assert_eq!(bitter.read_u32_bits(10), Some(0x377));
        assert_eq!(bitter.read_u32_bits(10), None);
    }

    #[test]
    fn test_u32_unchecked() {
        let mut bitter = BitGet::new(&[
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
        ]);
        assert_eq!(bitter.read_u32_unchecked(), 0xffff_ffff);
        assert_eq!(bitter.read_u32_bits_unchecked(30), 0x3fff_ffff);
        assert_eq!(bitter.read_u32_unchecked(), 0xffff_ffff);
    }

    #[test]
    fn test_u32_bits_unchecked() {
        let mut bitter = BitGet::new(&[0xff, 0xdd, 0xee, 0xff, 0xdd, 0xee, 0xaa, 0xbb, 0xcc, 0xdd]);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x1ff);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x3b7);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x3fe);
        assert_eq!(bitter.read_u32_bits_unchecked(10), 0x377);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xee);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xaa);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xbb);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xcc);
        assert_eq!(bitter.read_u32_bits_unchecked(8), 0xdd);
        assert_eq!(bitter.read_bit(), None);
    }

    #[test]
    fn test_u32_bits_unchecked2() {
        let mut bitter = BitGet::new(&[0x9c, 0x73, 0xce, 0x39, 0xe7,
                                       0x9c, 0x73, 0xce, 0x39, 0xe7,
                                       0x9c, 0x73, 0xce, 0x39, 0xe7]);
        for _ in 0..10 {
            assert_eq!(bitter.read_u32_bits_unchecked(5), 28);
        }
    }

    #[test]
    fn test_u32_bits2() {
        let mut bitter = BitGet::new(&[0x9c, 0x73, 0xce, 0x39, 0xe7,
                                       0x9c, 0x73, 0xce, 0x39, 0xe7,
                                       0x9c, 0x73, 0xce, 0x39, 0xe7]);
        for _ in 0..10 {
            assert_eq!(bitter.read_u32_bits(5), Some(28));
        }
    }

    #[test]
    fn test_max_read() {
        let mut bitter = BitGet::new(&[0b1111_1000]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(8));

        let mut bitter = BitGet::new(&[0b1111_0000]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(16));

        let mut bitter = BitGet::new(&[0b1110_0010]);
        assert_eq!(bitter.read_bits_max(5, 20), Some(2));
    }

    #[test]
    fn test_max_read_unchecked() {
        let mut bitter = BitGet::new(&[0b1111_1000]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 8);

        let mut bitter = BitGet::new(&[0b1111_0000]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 16);

        let mut bitter = BitGet::new(&[0b1110_0010]);
        assert_eq!(bitter.read_bits_max_unchecked(5, 20), 2);
    }

    #[test]
    fn test_if_get() {
        let mut bitter = BitGet::new(&[0xff, 0x04]);
        assert_eq!(bitter.if_get(|s| s.read_u8()), Some(Some(0x7f)));
        assert_eq!(bitter.if_get(|s| s.read_u8()), Some(None));
        assert_eq!(bitter.if_get(|s| s.read_u8()), None);
    }
}