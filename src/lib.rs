mod error;
pub use decree_derive::BitSource;
pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    BIG,
    LITTLE,
}

pub trait BitSource: Sized {
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error>;

    #[inline]
    fn write_to(&self, sink: &mut (impl BitSink + ?Sized), pos: usize) -> Result<usize, Error> {
        self.write(sink, 0, BitSource::size(self), pos)
    }

    #[inline]
    fn bits_into<T: BitSink + Default>(&self) -> Result<T, Error> {
        let mut sink = T::default();
        self.write_to(&mut sink, 0)?;
        Ok(sink)
    }

    #[inline]
    fn display_bits<'a>(&'a self) -> DisplayBits<'a, Self> {
        DisplayBits {
            source: self,
        }
    }

    /// The number of bits contained in this source.
    fn size(&self) -> usize;
}

pub trait BitSink {
    fn write(&mut self, bytes: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error>;

    /// The number of bits contained in this sink. None is returned if
    /// the sink can grow arbitrarily large.
    fn size(&self) -> Option<usize>;
}

#[derive(Debug, Clone)]
pub struct LittleEndian<T> {
    bytes: T,
    bits: usize,
}

#[derive(Debug, Clone)]
pub struct DisplayBits<'a, T: BitSource> {
    source: &'a T,
}

impl<'a, T: BitSource> core::fmt::Display for DisplayBits<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num_bits = self.source.size();
        let num_bytes = num_bits / 8;
        let num_bytes = if num_bits % 8 != 0 {
            num_bytes + 1
        } else {
            num_bytes
        };
        println!("<num-bytes>: {}, num-bits: {}", num_bytes, num_bits);
        let mut bytes = vec![0u8; num_bytes];
        println!("sink bytes: {:?}", bytes);
        {
            let mut sink = LittleEndian::<Vec<u8>>::with_bits(&mut bytes, num_bits).unwrap();
            println!("sink bits: {:?}", BitSink::size(&sink));
            println!("writing to...");
            self.source.write_to(&mut sink, 0).unwrap();
            println!("wrote");
        }
        for index in 0..num_bytes {
            let index = num_bytes - 1 - index;
            if index == num_bytes - 1 {
                let bits = num_bits % 8;
                println!("bits: {}", bits);
                let bits = if bits == 0 { 8 } else { bits };
                let chunk = BitChunk { byte: bytes[index], bits: bits as u8 };
                println!("chunk: {}", chunk);
                write!(f, "{}", chunk)?;
            } else {
                let chunk = BitChunk { byte: bytes[index], bits: 8 };
                write!(f, "{}", chunk)?;
            }
        }
        write!(f, "")
    }
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
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("LittleEndian", start, len, 0, BitSource::size(self))?;
        sink.write(self.bytes.as_ref(), start, len, pos, Endianness::LITTLE)
    }

    fn size(&self) -> usize {
        self.bits
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> BitSink for LittleEndian<T> {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        let size = BitSink::size(self);
        let mut bytes = self.bytes.as_mut();
        check_output_range("LittleEndian", pos, len, size)?;
        write_bits_le(source, start, len, pos, &mut bytes)
    }

    fn size(&self) -> Option<usize> {
        Some(self.bits)
    }
}

#[inline]
pub fn check_input_range(source: impl Into<String>, start: usize, len: usize, input_start: usize, input_end: usize) -> Result<(), Error> {
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
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("u8", start, len, 0, 7)?;
        sink.write(&[*self], start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

impl BitSource for u16 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("u16", start, len, 0, 15)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        16
    }
}

impl BitSource for u32 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("u32", start, len, 0, 31)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        32
    }
}

impl BitSource for u64 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("u64", start, len, 0, 63)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        64
    }
}

impl BitSource for u128 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("u128", start, len, 0, 127)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        128
    }
}

impl BitSource for i8 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("i8", start, len, 0, 7)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        8
    }
}

impl BitSource for i16 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("i16", start, len, 0, 15)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        16
    }
}

impl BitSource for i32 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("i32", start, len, 0, 31)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        32
    }
}

impl BitSource for i64 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("i64", start, len, 0, 63)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        64
    }
}

impl BitSource for i128 {
    #[inline]
    fn write(&self, sink: &mut (impl BitSink + ?Sized), start: usize, len: usize, pos: usize) -> Result<usize, Error> {
        check_input_range("i128", start, len, 0, 127)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        128
    }
}

impl BitSink for u8 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u8::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for i8 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i8::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for u16 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u16::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for i16 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i16::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for u32 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u32::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for i32 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i32::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for u64 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u64::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for i64 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i64::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for u128 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("u128", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = u128::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(128)
    }
}

impl BitSink for i128 {
    fn write(&mut self, source: &[u8], start: usize, len: usize, pos: usize, endianness: Endianness) -> Result<usize, Error> {
        if endianness == Endianness::BIG {
            todo!();
        }
        check_output_range("i28", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = write_bits_le(source, start, len, pos, &mut bytes)?;
        *self = i128::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(128)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BitChunk {
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
    println!("    writing {} bits to byte {} at bit {}", num_bits, byte, bit);
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
fn write_chunk_le(chunk: BitChunk, bytes: &mut [u8], start: usize) -> usize {
    let mut chunk = chunk;
    let mut written = 0;
    let mut start = start;
    while !chunk.is_empty() {
        let bits = chunk.bits();
        println!("  writing chunk: {}", chunk);
        chunk = write_chunk_le_helper(chunk, bytes, start);
        let bits_written = bits - chunk.bits();
        start += bits_written;
        written += bits_written;
    }
    written
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

fn write_bits_le(source: &[u8], start: usize, len: usize, pos: usize, sink: &mut [u8]) -> Result<usize, Error> {
    let mut pos = pos;
    let mut start = start;
    let mut len = len;
    let mut chunk = next_chunk_le(source, start, len);
    let mut written = 0;
    while !chunk.is_empty() {
        written += write_chunk_le(chunk, sink, pos);
        start += chunk.bits();
        pos += chunk.bits();
        len -= chunk.bits();
        chunk = next_chunk_le(source, start, len);
    }
    println!("  bytes: {:?}", print_bytes(sink));
    Ok(written)
}

fn print_bytes(bytes: &[u8]) -> String {
    let mut string = String::new();
    string.push('[');
    let mut first = true;
    for byte in bytes {
        if first {
            first = false;
        } else {
            string.push_str(", ");
        }
        string.push_str(&format!("{:08b}", byte));
    }
    string.push(']');
    string
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
