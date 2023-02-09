use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Input source ({src}) has bit range [{input_start}, {input_end}], but range [{start}, {end}] was requested.{message}")]
    InputBitsOutOfRange {
        src: String,
        message: String,
        start: usize,
        end: usize,
        input_start: usize,
        input_end: usize,
    },
    #[error("Bit range [{output_start}, {output_end}] is specified in sink ({sink}), but {len} bits are being written.{message}")]
    OutputBitsOutOfRange {
        sink: String,
        message: String,
        len: usize,
        output_start: usize,
        output_end: usize,
    },
    #[error("{message}")]
    WriteFailed {
        message: String,

        #[source]
        cause: Box<Error>,
    },
}

impl Error {
    pub fn input_bits_out_of_range(
        source: impl core::fmt::Display,
        start: usize,
        end: usize,
        input_start: usize,
        input_end: usize,
    ) -> Error {
        Error::InputBitsOutOfRange {
            src: source.to_string(),
            message: String::new(),
            start,
            end,
            input_start,
            input_end,
        }
    }

    pub fn output_bits_out_of_range(
        sink: impl core::fmt::Display,
        len: usize,
        output_start: usize,
        output_len: usize,
    ) -> Error {
        Error::OutputBitsOutOfRange {
            sink: sink.to_string(),
            message: String::new(),
            len,
            output_start,
            output_end: output_start + output_len - 1,
        }
    }

    pub fn write_failed(
        self,
        message: impl core::fmt::Display,
    ) -> Error {
        Error::WriteFailed {
            message: message.to_string(),
            cause: Box::new(self),
        }
    }
}
