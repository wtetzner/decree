
# Decree

Decree is a library for bitwise encoding and decoding. It was
originally designed for instruction encoding/decoding, but is really
more generic than that.

# Types

Decree has two primary traits: `BitSource` and `BitSink`. The design
of the API allows for reading a range of bits from a `BitSource`, and
writing them to a given bit position in a `BitSink`.

## BitSource

To implement a `BitSource`, specify the size (in bits, not bytes), and implement a `write` method that can write the contents to a `BitSink`.

```rust
use decree::{BitSource, BitSink, Error};

pub struct CompoundType {
    value1: u32,
    value2: u8,
}

impl BitSource for CompoundType {
    fn write(
        &self,
        sink: &mut (impl BitSink + ?Sized),
        start: usize,
        len: usize,
        pos: usize
    ) -> Result<(), Error> {
        self.value1.write(sink, 0, self.value1.size(), 0)?;
        self.value2.write(sink, 0, self.value2.size(), self.value1.size())?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.value1.size() + self.value2.size()
    }
}
```

### Implementations

All integral types implement both `BitSource` and `BitSink`.

Copyright © 2023 Walter Tetzner
