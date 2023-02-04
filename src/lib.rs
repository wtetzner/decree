use error::Error;
pub mod error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    BIG,
    LITTLE,
}

pub trait BitSource {
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error>;

    /// The number of bits contained in this source.
    fn size(&self) -> usize;
}

pub trait BitSink {
    fn write(&mut self, bytes: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error>;

    /// The number of bits contained in this sink. None is returned if
    /// the sink can grow arbitrarily large.
    fn size(&self) -> Option<usize>;
}

pub trait AsBitSource {
    type Source: BitSource;

    fn as_source_le(&self) -> Self::Source;
    fn as_source_with_bits_le(&self, bits: usize) -> Result<Self::Source, Error>;
}

impl<'a> AsBitSource for &'a [u8] {
    type Source = LittleEndian<&'a [u8]>;

    fn as_source_le(&self) -> Self::Source {
        LittleEndian::<&'a [u8]>::new(self)
    }

    fn as_source_with_bits_le(&self, bits: usize) -> Result<Self::Source, Error> {
        LittleEndian::<&'a [u8]>::with_bits(self, bits)
    }
}

#[derive(Debug, Clone)]
pub struct LittleEndian<T: AsRef<[u8]>> {
    bytes: T,
    bits: usize,
}

impl<T: AsRef<[u8]>> LittleEndian<T> {
    pub fn new<B: AsRef<[u8]>>(bytes: B) -> LittleEndian<B> {
        let bits = bytes.as_ref().len() * 8;
        LittleEndian {
            bytes,
            bits,
        }
    }

    pub fn with_bits<B: AsRef<[u8]>>(bytes: B, bits: usize)-> Result<LittleEndian<B>, Error> {
        let max_bits = bytes.as_ref().len() * 8;
        if bits > max_bits {
            return Err(Error::input_bits_out_of_range("LittleEndian", 0, bits, 0, max_bits));
        }
        Ok(LittleEndian { bytes, bits })
    }
}

impl<T: AsRef<[u8]>> BitSource for LittleEndian<T> {
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("LittleEndian", start, len, 0, BitSource::size(self))?;
        sink.write(self.bytes.as_ref(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.bits
    }
}

#[inline]
fn check_input_range(source: impl Into<String>, start: usize, len: usize, input_start: usize, input_end: usize) -> Result<(), Error> {
    let end = start + len - 1;
    if end > input_end || start < input_start {
        return Err(Error::input_bits_out_of_range(source, start, end, input_start, input_end));
    }
    Ok(())
}

#[inline]
fn check_output_range(sink: impl Into<String>, start: usize, len: usize, output_len: Option<usize>) -> Result<(), Error> {
    if let Some(output_len) = output_len {
        let space_to_write = output_len - start;
        if space_to_write < len {
            return Err(Error::output_bits_out_of_range(sink, len, start, output_len));
        }
        Ok(())
    } else {
        Ok(())
    }
}

impl BitSource for u8 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("u8", start, len, 0, 7)?;
        sink.write(&[*self], start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

impl BitSource for u16 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("u16", start, len, 0, 15)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        16
    }
}

impl BitSource for u32 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("u32", start, len, 0, 31)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        32
    }
}

impl BitSource for u64 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("u64", start, len, 0, 63)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        64
    }
}

impl BitSource for u128 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("u128", start, len, 0, 127)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        128
    }
}

impl BitSource for i8 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("i8", start, len, 0, 7)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

impl BitSource for i16 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("i16", start, len, 0, 15)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        16
    }
}

impl BitSource for i32 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("i32", start, len, 0, 31)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        32
    }
}

impl BitSource for i64 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("i64", start, len, 0, 63)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        64
    }
}

impl BitSource for i128 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<(), Error> {
        check_input_range("i128", start, len, 0, 127)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)?;
        Ok(())
    }

    #[inline]
    fn size(&self) -> usize {
        128
    }
}

impl BitSink for u8 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u8::from_le_bytes(bytes);
        Ok(())
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for i8 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i8::from_le_bytes(bytes);
        Ok(())
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for u16 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u16::from_le_bytes(bytes);
        Ok(())
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for i16 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i16::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for u32 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u32::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for i32 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i32::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for u64 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u64::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for i64 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i64::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for u128 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u128", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u128::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(128)
    }
}

impl BitSink for i128 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<(), Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i28", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i128::from_le_bytes(bytes);
        Ok(())
    }

    fn size(&self) -> Option<usize> {
        Some(128)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitChunk {
    byte: u8,
    bits: u8,
}

impl std::fmt::Display for BitChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.bits {
            0 => write!(f, "<empty>"),
            1 => write!(f, "{:01b}", self.byte),
            2 => write!(f, "{:02b}", self.byte),
            3 => write!(f, "{:03b}", self.byte),
            4 => write!(f, "{:04b}", self.byte),
            5 => write!(f, "{:05b}", self.byte),
            6 => write!(f, "{:06b}", self.byte),
            7 => write!(f, "{:07b}", self.byte),
            8 => write!(f, "{:08b}", self.byte),
            _ => panic!("Invalid number of bits"),
        }
    }
}

impl BitChunk {
    #[inline]
    pub fn byte(&self) -> u8 {
        self.byte
    }

    #[inline]
    pub fn bits(&self) -> usize {
        self.bits.into()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits() == 0
    }
}
 
#[inline]
fn next_chunk_le(bytes: &[u8], start: usize, len: usize) -> BitChunk {
    let chunk = if len == 0 {
        BitChunk { byte: 0, bits: 0 }
    } else {
        let byte = start / 8;
        let bit = start - (byte * 8);
        let num_bits = usize::min(len, 8 - bit);
        let mask = mask(num_bits) << bit;
        let value = (bytes[byte] & mask) >> bit;
        BitChunk {
            byte: value,
            bits: num_bits.try_into().unwrap(),
        }
    };
    chunk
}

#[inline]
fn write_chunk_le_helper(chunk: BitChunk, bytes: &mut [u8], start: usize) -> BitChunk {
    if chunk.bits() == 0 {
        return chunk;
    }
    let byte = start / 8;
    let bit = start - (byte * 8);
    let num_bits = usize::min(chunk.bits(), 8 - bit);
    let mask = mask(num_bits);
    let value = (bytes[byte] & !(mask << bit)) | ((chunk.byte() & mask) << bit);
    bytes[byte] = value;
    let chunk = BitChunk {
        byte: chunk.byte() >> num_bits,
        bits: (chunk.bits() - num_bits).try_into().unwrap(),
    };
    chunk
}

#[inline]
fn write_chunk_le(chunk: BitChunk, bytes: &mut [u8], start: usize) {
    let mut chunk = chunk;
    let mut start = start;
    while !chunk.is_empty() {
        let bits = chunk.bits();
        chunk = write_chunk_le_helper(chunk, bytes, start);
        start += bits;
    }
}

#[inline]
fn mask(len: usize) -> u8 {
    match len {
        0 => 0,
        1 => 0b1,
        2 => 0b11,
        3 => 0b111,
        4 => 0b1111,
        5 => 0b11111,
        6 => 0b111111,
        7 => 0b1111111,
        _ => 0b11111111,
    }
}

fn write_bits_le(source: &[u8], start: usize, len: usize, pos: usize, sink: &mut [u8]) -> Result<(), Error> {
    let mut pos = pos;
    let mut start = start;
    let mut len = len;
    let mut chunk = next_chunk_le(source, start, len);
    while !chunk.is_empty() {
        write_chunk_le(chunk, sink, pos);
        start += chunk.bits();
        pos += chunk.bits();
        len -= chunk.bits();
        chunk = next_chunk_le(source, start, len);
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Debug, Binary};

    #[test]
    fn test_bit_writing() {
        test_bit_write(
            0b11110001u8,
            0b11111111u8,
            1, 3,
            2,
            Ok(0b11100011),
        );
        test_bit_write(
            0b11110001u8,
            0b11111111u8,
            0, 8,
            0,
            Ok(0b11110001),
        );

        test_bit_write(
            0b00000000_00011111_10000000_00000000u32,
            00000000u8,
            14, 5,
            0,
         Ok(0b11110u8),
        );

        test_bit_write(
            0b00000000_00011111_10000000_00000000u32,
            0b00000000_00000001_11111110_00000000u32,
            14, 5,
            15,
         Ok(0b00000000_00001111_01111110_00000000u32),
        );

        test_bit_write(
            0b11110001u64,
            0b11111111u8,
            0, 64,
            0,
            Err(Error::output_bits_out_of_range("u8", 64, 0, 8)),
        );
        test_bit_write(
            0b11110001u64,
            0b11111111u8,
            7, 50,
            3,
            Err(Error::output_bits_out_of_range("u8", 50, 3, 8)),
        );
    }

    fn test_bit_write<S: BitSink + Eq + Debug + Binary>(source: impl BitSource, sink: S, start: usize, len: usize, pos: usize, expected: Result<S, Error>) {
        let mut sink = sink;
        let result = match source.write(&mut sink, start, len, pos) {
            Ok(_) => Ok(sink),
            Err(err) => Err(err),
        };

        if result != expected {
            let result_str = match result {
                Ok(value) => format!("{:b}", value),
                Err(error) => format!("Err({})", error),
            };
            let expected_str = match expected {
                Ok(value) => format!("{:b}", value),
                Err(error) => format!("Err({})", error),
            };
            panic!("Expected {}, but found {}", expected_str, result_str);
        }
    }
}
