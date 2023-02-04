use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Input source ({src}) has bit range [{input_start}, {input_end}], but range [{start}, {end}] was asked for.")]
    InputBitsOutOfRange {
        src: String,
        start: usize,
        end: usize,
        input_start: usize,
        input_end: usize,
    },
    #[error("Bit range [{output_start}, {output_end}] is specified in sink ({sink}), but {len} bits are being written.")]
    OutputBitsOutOfRange {
        sink: String,
        len: usize,
        output_start: usize,
        output_end: usize,
    },
}

impl Error {
    pub fn input_bits_out_of_range(
        source: impl Into<String>,
        start: usize,
        end: usize,
        input_start: usize,
        input_end: usize,
    ) -> Error {
        Error::InputBitsOutOfRange {
            src: source.into(),
            start, end, input_start, input_end
        }
    }

    pub fn output_bits_out_of_range(
        sink: impl Into<String>,
        len: usize,
        output_start: usize,
        output_len: usize,
    ) -> Error {
        Error::OutputBitsOutOfRange {
            sink: sink.into(),
            len,
            output_start,
            output_end: output_len - 1,
        }
    }
}
