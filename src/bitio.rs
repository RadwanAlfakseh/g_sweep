use std::fs::File;
use std::io::{self, Read, Write, stdout};

pub struct BitFile {
    file: File,
    pub mask: u8,
    pub rack: i32,
    pub pacifier_counter: i32,
}

impl BitFile {
    /// Equivalent to OpenOutputBitFile and OpenInputBitFile
    /// Mode is determined by the standard library's File::create or File::open
    pub fn open(name: &str, read_mode: bool) -> io::Result<Self> {
        let file = if read_mode {
            File::open(name)?
        } else {
            File::create(name)?
        };

        Ok(BitFile {
            file,
            mask: 0x80,
            rack: 0,
            pacifier_counter: 0,
        })
    }

    /// Equivalent to OutputBit
    pub fn output_bit(&mut self, bit: i32) -> io::Result<()> {
        if bit != 0 {
            self.rack |= self.mask as i32;
        }
        self.mask >>= 1;

        if self.mask == 0 {
            let buffer = [self.rack as u8];
            if self.file.write(&buffer)? != 1 {
                return Err(io::Error::new(io::ErrorKind::Other, "Fatal error in OutputBit"));
            }

            if (self.pacifier_counter & 4095) == 0 {
                print!(".");
                stdout().flush()?;
            }
            self.pacifier_counter += 1;
            self.rack = 0;
            self.mask = 0x80;
        }
        Ok(())
    }

    /// Equivalent to OutputBits
    pub fn output_bits(&mut self, code: u64, count: u32) -> io::Result<()> {
        if count == 0 { return Ok(()); }

        let mut bit_mask = 1u64 << (count - 1);
        while bit_mask != 0 {
            if (code & bit_mask) != 0 {
                self.rack |= self.mask as i32;
            }
            self.mask >>= 1;

            if self.mask == 0 {
                let buffer = [self.rack as u8];
                if self.file.write(&buffer)? != 1 {
                    return Err(io::Error::new(io::ErrorKind::Other, "Fatal error in OutputBits"));
                }

                if (self.pacifier_counter & 2047) == 0 {
                    print!(".");
                    stdout().flush()?;
                }
                self.pacifier_counter += 1;
                self.rack = 0;
                self.mask = 0x80;
            }
            bit_mask >>= 1;
        }
        Ok(())
    }

    /// Equivalent to InputBit
    pub fn input_bit(&mut self) -> io::Result<i32> {
        if self.mask == 0x80 {
            let mut buffer = [0u8; 1];
            if self.file.read(&mut buffer)? == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Fatal error in InputBit"));
            }
            self.rack = buffer[0] as i32;

            if (self.pacifier_counter & 2047) == 0 {
                print!(".");
                stdout().flush()?;
            }
            self.pacifier_counter += 1;
        }

        let value = self.rack & (self.mask as i32);
        self.mask >>= 1;
        if self.mask == 0 {
            self.mask = 0x80;
        }

        Ok(if value != 0 { 1 } else { 0 })
    }

    /// Equivalent to InputBits
    pub fn input_bits(&mut self, bit_count: u32) -> io::Result<u64> {
        if bit_count == 0 { return Ok(0); }

        let mut bit_mask = 1u64 << (bit_count - 1);
        let mut return_value = 0u64;

        while bit_mask != 0 {
            if self.mask == 0x80 {
                let mut buffer = [0u8; 1];
                if self.file.read(&mut buffer)? == 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Fatal error in InputBits"));
                }
                self.rack = buffer[0] as i32;

                if (self.pacifier_counter & 2047) == 0 {
                    print!(".");
                    stdout().flush()?;
                }
                self.pacifier_counter += 1;
            }

            if (self.rack & (self.mask as i32)) != 0 {
                return_value |= bit_mask;
            }

            bit_mask >>= 1;
            self.mask >>= 1;
            if self.mask == 0 {
                self.mask = 0x80;
            }
        }
        Ok(return_value)
    }

    /// Equivalent to CloseOutputBitFile
    pub fn close_output(mut self) -> io::Result<()> {
        if self.mask != 0x80 {
            self.file.write_all(&[self.rack as u8])?;
        }
        // File is closed automatically when self is dropped
        Ok(())
    }

    /// Equivalent to CloseInputBitFile
    pub fn close_input(self) {
        // File is closed automatically
    }
}

/// Equivalent to FilePrintBinary
pub fn file_print_binary(file: &mut File, code: u32, bits: u32) -> io::Result<()> {
    if bits == 0 { return Ok(()); }
    let mut mask = 1u32 << (bits - 1);
    while mask != 0 {
        if (code & mask) != 0 {
            file.write_all(b"1")?;
        } else {
            file.write_all(b"0")?;
        }
        mask >>= 1;
    }
    Ok(())
}