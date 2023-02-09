use decree::{BitSource, Result};

#[derive(Debug, BitSource)]
enum InstrGen<T> where T: core::fmt::Debug {
    #[bitpattern("111[x:1]111[x:3]11[x:4]11[y:4-9]1[z:12]", z = y)]
    Foo { x: i8, y: T },
    #[bitpattern(
        "111[x:1]111[x:3]11[x:4]11[y:4-9]1[z:12]111[z:17-30]",
        x = 0,
        y = 1,
        z = 2
    )]
    Bar(i8, u16, u32),

    #[bitpattern("11101110011001101001110")]
    Baz,
}

#[derive(Debug, BitSource)]
enum Instr {
    #[bitpattern("111[x:1]111[x:3]11[x:4]11[y:4-9]1[z:12]", z = y)]
    Foo { x: i8, y: u64 },
    #[bitpattern(
        "111[x:1]111[x:3]11[x:4]11[y:4-9]1[z:12]111[z:17-30]",
        x = 0,
        y = 1,
        z = 2
    )]
    Bar(i8, u16, u32),

    #[bitpattern("11101110011001101001110")]
    Baz,
}

#[test]
fn test_instr_foo() -> Result<()> {
    let instr = Instr::Foo { x: 0, y: 0 };

    let converted: u64 = instr.bits_into()?;

    assert_eq!(converted, 0b111011101101100000010u64);

    let mut output = 0b11111111111111110011111111u128;
    let written = instr.write(&mut output, 1, 10, 6)?;

    assert_eq!(written, 10);

    assert_eq!(output, 0b11111111110110000001111111u128);

    Ok(())
}

#[test]
fn test_instr_foo_ones() -> Result<()> {
    let instr = Instr::Foo { x: -1, y: 0xFFFFFFFFFFFFFFFF };

    let converted: u64 = instr.bits_into()?;

    assert_eq!(converted, 0b111111111111111111111u64);

    let mut output = 0b11111111111111111111111111u128;
    let written = instr.write(&mut output, 1, 10, 6)?;

    assert_eq!(written, 10);

    assert_eq!(output, 0b11111111111111111111111111u128);

    Ok(())
}

#[derive(Debug, BitSource)]
#[bitpattern("1001110011000")]
struct Foo;

#[derive(Debug, BitSource)]
#[bitpattern("10101101")]
struct Bar;

#[test]
fn test_empty_struct_13bit_u128() -> Result<()> {
    let foo = Foo;

    let val_u128: u128 = foo.bits_into()?;
    assert_eq!(val_u128, 0b1001110011000u128);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_u64() -> Result<()> {
    let foo = Foo;

    let val_u64: u64 = foo.bits_into()?;
    assert_eq!(val_u64, 0b1001110011000u64);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_u32() -> Result<()> {
    let foo = Foo;

    let val_u32: u32 = foo.bits_into()?;
    assert_eq!(val_u32, 0b1001110011000u32);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_u16() -> Result<()> {
    let foo = Foo;

    let val_u16: u16 = foo.bits_into()?;
    assert_eq!(val_u16, 0b1001110011000u16);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_u8() -> Result<()> {
    let foo = Foo;

    let val_u8: Result<u8> = foo.bits_into();
    match val_u8 {
        Ok(_) => { panic!("Expected out of range error") },
        Err(_) => {},
    }

    Ok(())
}

#[test]
fn test_empty_struct_13bit_i128() -> Result<()> {
    let foo = Foo;

    let val_i128: i128 = foo.bits_into()?;
    assert_eq!(val_i128, 0b1001110011000i128);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_i64() -> Result<()> {
    let foo = Foo;

    let val_i64: i64 = foo.bits_into()?;
    assert_eq!(val_i64, 0b1001110011000i64);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_i32() -> Result<()> {
    let foo = Foo;

    let val_i32: i32 = foo.bits_into()?;
    assert_eq!(val_i32, 0b1001110011000i32);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_i16() -> Result<()> {
    let foo = Foo;

    let val_i16: i16 = foo.bits_into()?;
    assert_eq!(val_i16, 0b1001110011000i16);

    Ok(())
}

#[test]
fn test_empty_struct_13bit_i8() -> Result<()> {
    let foo = Foo;

    let val_i8: Result<i8> = foo.bits_into();
    match val_i8 {
        Ok(_) => { panic!("Expected out of range error"); },
        Err(_) => {},
    }

    Ok(())
}

#[test]
fn test_empty_struct_8bit_u128() -> Result<()> {
    let bar = Bar;

    let val_u128: u128 = bar.bits_into()?;
    assert_eq!(val_u128, 0b10101101u128);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_u64() -> Result<()> {
    let bar = Bar;

    let val_u64: u64 = bar.bits_into()?;
    assert_eq!(val_u64, 0b10101101u64);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_u32() -> Result<()> {
    let bar = Bar;

    let val_u32: u32 = bar.bits_into()?;
    assert_eq!(val_u32, 0b10101101u32);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_u16() -> Result<()> {
    let bar = Bar;

    let val_u16: u16 = bar.bits_into()?;
    assert_eq!(val_u16, 0b10101101u16);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_u8() -> Result<()> {
    let bar = Bar;

    let val_u8: u8 = bar.bits_into()?;
    assert_eq!(val_u8, 0b10101101u8);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_i128() -> Result<()> {
    let bar = Bar;

    let val_i128: i128 = bar.bits_into()?;
    assert_eq!(val_i128, 0b010101101i128);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_i64() -> Result<()> {
    let bar = Bar;

    let val_i64: i64 = bar.bits_into()?;
    assert_eq!(val_i64, 0b010101101i64);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_i32() -> Result<()> {
    let bar = Bar;

    let val_i32: i32 = bar.bits_into()?;
    assert_eq!(val_i32, 0b010101101i32);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_i16() -> Result<()> {
    let bar = Bar;

    let val_i16: i16 = bar.bits_into()?;
    assert_eq!(val_i16, 0b010101101i16);

    Ok(())
}

#[test]
fn test_empty_struct_8bit_i8() -> Result<()> {
    let bar = Bar;

    let val_i8: i8 = bar.bits_into()?;
    assert_eq!(val_i8, -83i8);

    Ok(())
}
