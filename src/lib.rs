mod error;
pub use decree_derive::BitSource;
pub use error::Error;

pub type Result<T> = core::result::Result<T, error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    BIG,
    LITTLE,
}

pub trait BitSource: Sized {
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize>;

    #[inline]
    fn write_to(&self, sink: &mut (impl BitSink + ?Sized), pos: usize) -> Result<usize> {
        self.write(sink, 0, BitSource::size(self), pos)
    }

    #[inline]
    fn bits_into<T: BitSink + Default>(&self) -> Result<T> {
        let mut sink = T::default();
        self.write_to(&mut sink, 0)?;
        Ok(sink)
    }

    #[inline]
    fn display_bits<'a>(&'a self) -> DisplayBits<'a, Self> {
        DisplayBits { source: self }
    }

    /// The number of bits contained in this source.
    fn size(&self) -> usize;
}

pub trait BitSink {
    fn write(
        &mut self,
        bytes: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize>;

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
pub struct BigEndian<T> {
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
        let mut bytes = vec![0u8; num_bytes];
        {
            let mut sink = LittleEndian::<Vec<u8>>::with_bits(&mut bytes, num_bits).unwrap();
            self.source.write_to(&mut sink, 0).unwrap();
        }
        for index in 0..num_bytes {
            let index = num_bytes - 1 - index;
            if index == num_bytes - 1 {
                let bits = num_bits % 8;
                let bits = if bits == 0 { 8 } else { bits };
                let chunk = BitChunk {
                    byte: bytes[index],
                    bits: bits as u8,
                };
                write!(f, "{}", chunk)?;
            } else {
                let chunk = BitChunk {
                    byte: bytes[index],
                    bits: 8,
                };
                write!(f, "{}", chunk)?;
            }
        }
        write!(f, "")
    }
}

impl<T: AsRef<[u8]>> LittleEndian<T> {
    pub fn new<B: AsRef<[u8]>>(bytes: B) -> LittleEndian<B> {
        let bits = bytes.as_ref().len() * 8;
        LittleEndian { bytes, bits }
    }

    pub fn with_bits<B: AsRef<[u8]>>(bytes: B, bits: usize) -> Result<LittleEndian<B>> {
        let max_bits = bytes.as_ref().len() * 8;
        if bits > max_bits {
            return Err(Error::input_bits_out_of_range(
                "LittleEndian",
                0,
                bits,
                0,
                max_bits,
            ));
        }
        Ok(LittleEndian { bytes, bits })
    }
}

impl<T: AsRef<[u8]>> BitSource for LittleEndian<T> {
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_input_range(
            "LittleEndian",
            start,
            len,
            0,
            BitSource::size(self)
        )?;
        sink.write(self.bytes.as_ref(), start, len, pos, Endianness::LITTLE)
    }

    fn size(&self) -> usize {
        self.bits
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> BitSink for LittleEndian<T> {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        let size = BitSink::size(self);
        check_output_range(
            "LittleEndian",
            pos,
            len,
            size
        )?;
        let mut bytes = self.bytes.as_mut();
        match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper),
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper),
        }
    }

    fn size(&self) -> Option<usize> {
        Some(self.bits)
    }
}

impl<T: AsRef<[u8]>> BigEndian<T> {
    pub fn new<B: AsRef<[u8]>>(bytes: B) -> BigEndian<B> {
        let bits = bytes.as_ref().len() * 8;
        BigEndian { bytes, bits }
    }

    pub fn with_bits<B: AsRef<[u8]>>(bytes: B, bits: usize) -> Result<BigEndian<B>> {
        let max_bits = bytes.as_ref().len() * 8;
        if bits > max_bits {
            return Err(Error::input_bits_out_of_range(
                "BigEndian",
                0,
                bits,
                0,
                max_bits,
            ));
        }
        Ok(BigEndian { bytes, bits })
    }
}

impl<T: AsRef<[u8]>> BitSource for BigEndian<T> {
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_input_range(
            "BigEndian",
            start,
            len,
            0,
            BitSource::size(self)
        )?;
        sink.write(self.bytes.as_ref(), start, len, pos, Endianness::BIG)
    }

    fn size(&self) -> usize {
        self.bits
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> BitSink for BigEndian<T> {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        let size = BitSink::size(self);
        check_output_range(
            "BigEndian",
            pos,
            len,
            size
        )?;
        let mut bytes = self.bytes.as_mut();
        match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_be_helper),
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_be_helper),
        }
    }

    fn size(&self) -> Option<usize> {
        Some(self.bits)
    }
}

#[inline]
pub fn check_input_range(
    source: impl core::fmt::Display,
    start: usize,
    len: usize,
    input_start: usize,
    input_end: usize,
) -> Result<()> {
    let end = if len == 0 { start } else { start + len - 1 };
    if end > input_end || start < input_start {
        return Err(Error::input_bits_out_of_range(
            source,
            start,
            end,
            input_start,
            input_end,
        ));
    }
    Ok(())
}

#[inline]
fn check_output_range(
    sink: impl core::fmt::Display,
    start: usize,
    len: usize,
    output_len: Option<usize>,
) -> Result<()> {
    if let Some(output_len) = output_len {
        let space_to_write = output_len - start;
        if space_to_write < len {
            return Err(Error::output_bits_out_of_range(
                sink, len, start, output_len,
            ));
        }
        Ok(())
    } else {
        Ok(())
    }
}

impl BitSource for u8 {
    #[inline]
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
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
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_input_range("i128", start, len, 0, 127)?;
        sink.write(&(*self).to_le_bytes(), start, len, pos, Endianness::LITTLE)
    }

    #[inline]
    fn size(&self) -> usize {
        128
    }
}

impl BitSink for u8 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("u8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = u8::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for i8 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("i8", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = i8::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(8)
    }
}

impl BitSink for u16 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("u16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = u16::from_le_bytes(bytes);
        Ok(written)
    }

    #[inline]
    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for i16 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("i16", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = i16::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(16)
    }
}

impl BitSink for u32 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("u32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = u32::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for i32 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("i32", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = i32::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(32)
    }
}

impl BitSink for u64 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("u64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = u64::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for i64 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("i64", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = i64::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(64)
    }
}

impl BitSink for u128 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("u128", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
        *self = u128::from_le_bytes(bytes);
        Ok(written)
    }

    fn size(&self) -> Option<usize> {
        Some(128)
    }
}

impl BitSink for i128 {
    fn write(
        &mut self,
        source: &[u8],
        start: usize,
        len: usize,
        pos: usize,
        endianness: Endianness,
    ) -> Result<usize> {
        if len == 0 {
            return Ok(0);
        }
        check_output_range("i28", pos, len, BitSink::size(self))?;
        let mut bytes = self.to_le_bytes();
        let written = match endianness {
            Endianness::BIG => write_bits(source, start, len, pos, &mut bytes, next_chunk_be, write_chunk_le_helper)?,
            Endianness::LITTLE => write_bits(source, start, len, pos, &mut bytes, next_chunk_le, write_chunk_le_helper)?,
        };
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
fn next_chunk_be(bytes: &[u8], start: usize, len: usize) -> BitChunk {
    let chunk = if len == 0 {
        BitChunk { byte: 0, bits: 0 }
    } else {
        let byte = bytes.len() - 1 - (start / 8);
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
    let bit = start % 8;
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
fn write_chunk_be_helper(chunk: BitChunk, bytes: &mut [u8], start: usize) -> BitChunk {
    if chunk.bits() == 0 {
        return chunk;
    }
    let byte = bytes.len() - 1 - (start / 8);
    let bit = start % 8;
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
fn write_chunk(chunk: BitChunk, bytes: &mut [u8], start: usize, helper: fn(BitChunk, &mut [u8], usize) -> BitChunk) -> usize {
    let mut chunk = chunk;
    let mut written = 0;
    let mut start = start;
    while !chunk.is_empty() {
        let bits = chunk.bits();
        chunk = helper(chunk, bytes, start);
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

fn write_bits(
    source: &[u8],
    start: usize,
    len: usize,
    pos: usize,
    sink: &mut [u8],
    next_chunk: fn(&[u8], usize, usize) -> BitChunk,
    write_chunk_helper: fn(BitChunk, &mut [u8], usize) -> BitChunk,
) -> Result<usize> {
    let mut pos = pos;
    let mut start = start;
    let mut len = len;
    let mut chunk = next_chunk(source, start, len);
    let mut written = 0;
    while !chunk.is_empty() {
        written += write_chunk(chunk, sink, pos, write_chunk_helper);
        start += chunk.bits();
        pos += chunk.bits();
        len -= chunk.bits();
        chunk = next_chunk(source, start, len);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Binary, Debug};

    #[test]
    fn test_bit_writing() {
        test_bit_write(0b11110001u8, 0b11111111u8, 1, 3, 2, Ok(0b11100011));
        test_bit_write(0b11110001u8, 0b11111111u8, 0, 8, 0, Ok(0b11110001));

        test_bit_write(
            0b00000000_00011111_10000000_00000000u32,
            00000000u8,
            14,
            5,
            0,
            Ok(0b11110u8),
        );

        test_bit_write(
            0b00000000_00011111_10000000_00000000u32,
            0b00000000_00000001_11111110_00000000u32,
            14,
            5,
            15,
            Ok(0b00000000_00001111_01111110_00000000u32),
        );

        test_bit_write(
            0b11110001u64,
            0b11111111u8,
            0,
            64,
            0,
            Err(Error::output_bits_out_of_range("u8", 64, 0, 8)),
        );
        test_bit_write(
            0b11110001u64,
            0b11111111u8,
            7,
            50,
            3,
            Err(Error::output_bits_out_of_range("u8", 50, 3, 8)),
        );
    }

    #[test]
    fn test_big_endian() -> Result<()> {
        let mut output_bytes = [0u8; 3];
        let mut output = BigEndian::<&[u8]>::new(&mut output_bytes);
        let input = 0b011110001110101010101u128;
        input.write(&mut output, 0, 11, 1)?;
        let expected_output = [0b00000000u8, 0b00001010, 0b10101010];
        compare_arrays(&output_bytes, &expected_output);

        fn write_big_endian(bytes: &mut [u8], pos: usize, input: impl BitSource) -> Result<()> {
            let mut output = BigEndian::<&[u8]>::new(bytes);
            input.write_to(&mut output, pos)?;
            Ok(())
        }

        fn write_big_endian_len(bytes: &mut [u8], pos: usize, len: usize, input: impl BitSource) -> Result<()> {
            let mut output = BigEndian::<&[u8]>::new(bytes);
            input.write(&mut output, 0, len, pos)?;
            Ok(())
        }

        {
            let input = 0b111_11111111_11111111;
            {
                let pos = 0;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b00000111, 0b11111111, 0b11111111]);
            }
            {
                let pos = 1;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b00001111, 0b11111111, 0b11111110]);
            }
            {
                let pos = 2;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b00011111, 0b11111111, 0b11111100]);
            }
            {
                let pos = 3;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b00111111, 0b11111111, 0b11111000]);
            }
            {
                let pos = 4;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b01111111, 0b11111111, 0b11110000]);
            }
            {
                let pos = 5;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0b11111111, 0b11111111, 0b11100000]);
            }
            {
                let pos = 6;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00000001, 0b11111111, 0b11111111, 0b11000000]);
            }
            {
                let pos = 7;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00000011, 0b11111111, 0b11111111, 0b10000000]);
            }
            {
                let pos = 8;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00000111, 0b11111111, 0b11111111, 0]);
            }
            {
                let pos = 9;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00001111, 0b11111111, 0b11111110, 0]);
            }
            {
                let pos = 10;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00011111, 0b11111111, 0b11111100, 0]);
            }
            {
                let pos = 11;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b00111111, 0b11111111, 0b11111000, 0]);
            }
            {
                let pos = 12;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b01111111, 0b11111111, 0b11110000, 0]);
            }
            {
                let pos = 13;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0b11111111, 0b11111111, 0b11100000, 0]);
            }
            {
                let pos = 14;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00000001, 0b11111111, 0b11111111, 0b11000000, 0]);
            }
            {
                let pos = 15;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00000011, 0b11111111, 0b11111111, 0b10000000, 0]);
            }
            {
                let pos = 16;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00000111, 0b11111111, 0b11111111, 0, 0]);
            }
            {
                let pos = 17;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00001111, 0b11111111, 0b11111110, 0, 0]);
            }
            {
                let pos = 18;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00011111, 0b11111111, 0b11111100, 0, 0]);
            }
            {
                let pos = 19;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b00111111, 0b11111111, 0b11111000, 0, 0]);
            }
            {
                let pos = 20;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b01111111, 0b11111111, 0b11110000, 0, 0]);
            }
            {
                let pos = 21;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0, 0b11111111, 0b11111111, 0b11100000, 0, 0]);
            }
            {
                let pos = 22;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00000001, 0b11111111, 0b11111111, 0b11000000, 0, 0]);
            }
            {
                let pos = 23;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00000011, 0b11111111, 0b11111111, 0b10000000, 0, 0]);
            }
            {
                let pos = 24;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00000111, 0b11111111, 0b11111111, 0, 0, 0]);
            }
            {
                let pos = 25;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00001111, 0b11111111, 0b11111110, 0, 0, 0]);
            }
            {
                let pos = 26;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00011111, 0b11111111, 0b11111100, 0, 0, 0]);
            }
            {
                let pos = 27;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b00111111, 0b11111111, 0b11111000, 0, 0, 0]);
            }
            {
                let pos = 28;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b01111111, 0b11111111, 0b11110000, 0, 0, 0]);
            }
            {
                let pos = 29;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0, 0b11111111, 0b11111111, 0b11100000, 0, 0, 0]);
            }
            {
                let pos = 30;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0b00000001, 0b11111111, 0b11111111, 0b11000000, 0, 0, 0]);
            }
            {
                let pos = 31;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0b00000011, 0b11111111, 0b11111111, 0b10000000, 0, 0, 0]);
            }
            {
                let pos = 32;
                let mut output = [0u8; 8];
                write_big_endian(&mut output, pos, input)?;
                compare_arrays(&output, &[0, 0b00000111, 0b11111111, 0b11111111, 0, 0, 0, 0]);
            }
            {
                let pos = 33;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0, 0b00001111, 0b11111111, 0b11111110, 0, 0, 0, 0]);
            }
            {
                let pos = 34;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0, 0b00011111, 0b11111111, 0b11111100, 0, 0, 0, 0]);
            }
            {
                let pos = 35;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0, 0b00111111, 0b11111111, 0b11111000, 0, 0, 0, 0]);
            }
            {
                let pos = 36;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0, 0b01111111, 0b11111111, 0b11110000, 0, 0, 0, 0]);
            }
            {
                let pos = 37;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0, 0b11111111, 0b11111111, 0b11100000, 0, 0, 0, 0]);
            }
            {
                let pos = 38;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00000001, 0b11111111, 0b11111111, 0b11000000, 0, 0, 0, 0]);
            }
            {
                let pos = 39;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00000011, 0b11111111, 0b11111111, 0b10000000, 0, 0, 0, 0]);
            }
            {
                let pos = 40;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00000111, 0b11111111, 0b11111111, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 41;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00001111, 0b11111111, 0b11111110, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 42;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00011111, 0b11111111, 0b11111100, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 43;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b00111111, 0b11111111, 0b11111000, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 44;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b01111111, 0b11111111, 0b11110000, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 45;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 19, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111111, 0b11100000, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 46;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 18, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111111, 0b11000000, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 47;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 17, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111111, 0b10000000, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 48;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 16, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111111, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 49;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 15, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111110, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 50;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 14, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111100, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 51;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 13, input)?;
                compare_arrays(&output, &[0b11111111, 0b11111000, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 52;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 12, input)?;
                compare_arrays(&output, &[0b11111111, 0b11110000, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 53;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 11, input)?;
                compare_arrays(&output, &[0b11111111, 0b11100000, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 54;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 10, input)?;
                compare_arrays(&output, &[0b11111111, 0b11000000, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 55;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 9, input)?;
                compare_arrays(&output, &[0b11111111, 0b10000000, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 56;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 8, input)?;
                compare_arrays(&output, &[0b11111111, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 57;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 7, input)?;
                compare_arrays(&output, &[0b11111110, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 58;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 6, input)?;
                compare_arrays(&output, &[0b11111100, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 59;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 5, input)?;
                compare_arrays(&output, &[0b11111000, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 60;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 4, input)?;
                compare_arrays(&output, &[0b11110000, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 61;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 3, input)?;
                compare_arrays(&output, &[0b11100000, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 62;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 2, input)?;
                compare_arrays(&output, &[0b11000000, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 63;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 1, input)?;
                compare_arrays(&output, &[0b10000000, 0, 0, 0, 0, 0, 0, 0]);
            }
            {
                let pos = 64;
                let mut output = [0u8; 8];
                write_big_endian_len(&mut output, pos, 0, input)?;
                compare_arrays(&output, &[0, 0, 0, 0, 0, 0, 0, 0]);
            }
        }

        Ok(())
    }

    fn compare_arrays(found: &[u8], expected: &[u8]) {
        if found != expected {
            panic!("Expected {}, found {}", print_array(&expected), print_array(&found));
        }
    }

    fn print_array(bytes: &[u8]) -> String {
        let mut result = String::new();
        result.push('[');
        let mut first = true;
        for byte in bytes {
            if first {
                first = false;
            } else {
                result.push_str(", ");
            }
            result.push_str(&format!("{:08b}", byte));
        }
        result.push(']');
        result
    }

    fn test_bit_write<S: BitSink + Eq + Debug + Binary>(
        source: impl BitSource,
        sink: S,
        start: usize,
        len: usize,
        pos: usize,
        expected: Result<S>,
    ) {
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
