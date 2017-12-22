use untrusted;
use untrusted::{Reader, Input};
use untrustended::ReaderExt;

use errors::{ErrorKind, Result};

#[derive(Debug)]
pub struct SizeInfos {
    file_word_count: u16,
    header_size: u16,
    first_char: u16,
    last_char: u16,
    width_table_size: u16,
    height_table_size: u16,
    depth_table_size: u16,
    italics_correction_table_size: u16,
    lig_kern_table_size: u16,
    kern_table_size: u16,
    extensible_character_table_size: u16,
    font_parameters: u16,
}

impl SizeInfos {
    pub fn parse<'a>(input: &mut Reader<'a>) -> Result<Self> {
        Ok(SizeInfos {
            file_word_count: input.read_u16be()?,
            header_size: input.read_u16be()?,
            first_char: input.read_u16be()?,
            last_char: input.read_u16be()?,
            width_table_size: input.read_u16be()?,
            height_table_size: input.read_u16be()?,
            depth_table_size: input.read_u16be()?,
            italics_correction_table_size: input.read_u16be()?,
            lig_kern_table_size: input.read_u16be()?,
            kern_table_size: input.read_u16be()?,
            extensible_character_table_size: input.read_u16be()?,
            font_parameters: input.read_u16be()?,
        })
    }

    pub fn char_count(&self) -> usize { (self.last_char - self.first_char + 1) as _ }

    pub fn expected_file_size(&self) -> u16 {
        ( 6
        + self.header_size
        + self.char_count() as u16
        + self.width_table_size
        + self.height_table_size
        + self.depth_table_size
        + self.italics_correction_table_size
        + self.lig_kern_table_size
        + self.kern_table_size
        + self.extensible_character_table_size
        + self.font_parameters
        ) * 4
    }
}

#[derive(Debug, PartialEq)]
pub struct CharIndices {
    width: usize,
    height: usize,
    depth: usize,
    // TODO
}

#[derive(Debug)]
pub struct TFM {
    size_infos: SizeInfos,
    char_info: Vec<CharIndices>,
    width: Vec<f64>,
    height: Vec<f64>,
    depth: Vec<f64>,
    italics_correction: Vec<f64>,
    lig_kern: Vec<u32>,
    kern: Vec<f64>,
    extensible_character: Vec<u32>,
    font_parameters: Vec<f64>,
}

impl TFM {
    pub fn width(&self, ch: char) -> f64 {
        assert!(ch as u16 >= self.size_infos.first_char && ch as u16 <= self.size_infos.last_char);
        let offset = ch as u16 - self.size_infos.first_char;
        let index = self.char_info[offset as usize].width;
        self.width[index]
    }
}


pub fn read_fix_word<'a>(input: &mut Reader<'a>) -> Result<f64> {
    let data = input.read_u32be()?;
    let sign = data >> 31; // first bit
    let pre_dot_unsigned = (data >> 20) & 0x7ff; // next 11 bits
    let post_dot = data & 0xfffff; // remaining 20 bits

    println!("data: {:b}", data);
    println!("pre_dot: {:b} post_dot: {:b}", pre_dot_unsigned, post_dot);

    let pre_dot = if sign == 0 { pre_dot_unsigned as f64 } else { -(pre_dot_unsigned as f64)};
    let number = pre_dot + post_dot as f64 / 20f64.exp2();
    Ok(number)
}


pub fn char_info<'a>(input: &mut Reader<'a>) -> Result<CharIndices> {
    let data = input.read_u32be()?;
    let width = (data >> 24) as _;
    let height = ((data >> 20) & 0xf) as _;
    let depth = ((data >> 16) & 0xf) as _;
    Ok(CharIndices {
        width, height, depth
    })
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use untrusted;
    use untrustended;

    fn with_i<F, R, E: fmt::Debug>(value: &[u8], f: F) -> R
        where F: FnOnce(&mut untrusted::Reader) -> ::std::result::Result<R, E> {
        let inp = untrusted::Input::from(value);
        let mut reader = untrusted::Reader::new(inp);
        f(&mut reader).unwrap()
    }


    #[test]
    fn test_fix_word_parser() {
        //                  0b000000000010_10100000000000000000
        //                    twelve bits : twenty bits
        let positive = [0,          0b0010_1010, 0, 0];
        let negative = [0b10000000, 0b0011_0110, 0, 0];
        assert_eq!(with_i(&positive, read_fix_word),  2.625);
        assert_eq!(with_i(&negative, read_fix_word), -2.625);
    }
}
