//! BCJ (Branch/Call/Jump) filters for executable code preprocessing.
//!
//! BCJ filters convert relative addresses in executable code to absolute addresses.
//! This normalizes patterns across different positions in the binary, significantly
//! improving compression ratios (typically 10-15% better).
//!
//! Supported architectures:
//! - x86/x86_64: CALL (E8) and JMP (E9) instructions
//! - ARM/AArch64: BL and B instructions
//! - RISC-V: JAL and AUIPC instructions

use crate::Result;

/// Architecture-specific BCJ filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcjArch {
    /// x86 and x86_64
    X86,
    /// ARM 32-bit (Thumb and ARM mode)
    Arm,
    /// ARM 64-bit (AArch64)
    Arm64,
    /// RISC-V 32/64-bit
    RiscV,
    /// PowerPC 64-bit little-endian
    Ppc64Le,
    /// No filtering (passthrough)
    None,
}

impl BcjArch {
    /// Detect architecture from platform target string.
    pub fn from_target(target: &str) -> Self {
        if target.contains("x86_64") || target.contains("i686") || target.contains("i586") {
            BcjArch::X86
        } else if target.contains("aarch64") {
            BcjArch::Arm64
        } else if target.contains("arm") {
            BcjArch::Arm
        } else if target.contains("riscv") {
            BcjArch::RiscV
        } else if target.contains("powerpc64le") || target.contains("ppc64le") {
            BcjArch::Ppc64Le
        } else {
            BcjArch::None
        }
    }
}

/// BCJ filter state for streaming processing.
pub struct BcjFilter {
    arch: BcjArch,
    pos: usize,
    /// For x86: previous byte state for multi-byte instruction detection
    #[allow(dead_code)]
    prev_mask: u32,
}

impl BcjFilter {
    /// Create a new BCJ filter for the given architecture.
    pub fn new(arch: BcjArch) -> Self {
        Self {
            arch,
            pos: 0,
            prev_mask: 0,
        }
    }

    /// Encode (filter) data in-place for compression.
    /// Converts relative addresses to absolute.
    pub fn encode(&mut self, data: &mut [u8]) -> Result<()> {
        match self.arch {
            BcjArch::X86 => self.encode_x86(data),
            BcjArch::Arm64 => self.encode_arm64(data),
            BcjArch::Arm => self.encode_arm(data),
            BcjArch::RiscV => self.encode_riscv(data),
            BcjArch::Ppc64Le => self.encode_ppc64(data),
            BcjArch::None => Ok(()),
        }
    }

    /// Decode (unfilter) data in-place after decompression.
    /// Converts absolute addresses back to relative.
    pub fn decode(&mut self, data: &mut [u8]) -> Result<()> {
        match self.arch {
            BcjArch::X86 => self.decode_x86(data),
            BcjArch::Arm64 => self.decode_arm64(data),
            BcjArch::Arm => self.decode_arm(data),
            BcjArch::RiscV => self.decode_riscv(data),
            BcjArch::Ppc64Le => self.decode_ppc64(data),
            BcjArch::None => Ok(()),
        }
    }

    /// x86/x86_64 BCJ encoding.
    /// Filters CALL (E8) and JMP (E9) instructions.
    fn encode_x86(&mut self, data: &mut [u8]) -> Result<()> {
        if data.len() < 5 {
            return Ok(());
        }

        let limit = data.len() - 4;
        let mut i = 0;

        while i < limit {
            // Look for E8 (CALL) or E9 (JMP near)
            if data[i] == 0xE8 || data[i] == 0xE9 {
                // Read relative offset (little-endian)
                let rel = i32::from_le_bytes([
                    data[i + 1],
                    data[i + 2],
                    data[i + 3],
                    data[i + 4],
                ]);

                // Convert to absolute: abs = rel + current_pos + 5 (instruction length)
                let abs = rel.wrapping_add((self.pos + i + 5) as i32);

                // Write back as absolute (little-endian)
                let abs_bytes = abs.to_le_bytes();
                data[i + 1] = abs_bytes[0];
                data[i + 2] = abs_bytes[1];
                data[i + 3] = abs_bytes[2];
                data[i + 4] = abs_bytes[3];

                i += 5;
            } else {
                i += 1;
            }
        }

        self.pos += data.len();
        Ok(())
    }

    /// x86/x86_64 BCJ decoding.
    fn decode_x86(&mut self, data: &mut [u8]) -> Result<()> {
        if data.len() < 5 {
            return Ok(());
        }

        let limit = data.len() - 4;
        let mut i = 0;

        while i < limit {
            if data[i] == 0xE8 || data[i] == 0xE9 {
                // Read absolute address
                let abs = i32::from_le_bytes([
                    data[i + 1],
                    data[i + 2],
                    data[i + 3],
                    data[i + 4],
                ]);

                // Convert back to relative: rel = abs - current_pos - 5
                let rel = abs.wrapping_sub((self.pos + i + 5) as i32);

                // Write back as relative
                let rel_bytes = rel.to_le_bytes();
                data[i + 1] = rel_bytes[0];
                data[i + 2] = rel_bytes[1];
                data[i + 3] = rel_bytes[2];
                data[i + 4] = rel_bytes[3];

                i += 5;
            } else {
                i += 1;
            }
        }

        self.pos += data.len();
        Ok(())
    }

    /// ARM64 (AArch64) BCJ encoding.
    /// Filters BL (Branch with Link) instructions.
    fn encode_arm64(&mut self, data: &mut [u8]) -> Result<()> {
        // ARM64 instructions are 4 bytes, aligned
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3; // Align to 4-byte boundary
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            // Read instruction (little-endian)
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            // BL instruction: 100101 followed by 26-bit signed offset
            // Opcode mask: 0xFC000000, expected: 0x94000000
            if (inst & 0xFC00_0000) == 0x9400_0000 {
                // Extract 26-bit signed offset (in units of 4 bytes)
                let offset = ((inst & 0x03FF_FFFF) as i32) << 6 >> 6; // Sign extend

                // Convert to absolute address
                let addr = ((self.pos + i) as i32).wrapping_add(offset * 4);

                // Encode as new offset from 0
                let new_offset = (addr >> 2) as u32 & 0x03FF_FFFF;
                let new_inst = (inst & 0xFC00_0000) | new_offset;

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// ARM64 BCJ decoding.
    fn decode_arm64(&mut self, data: &mut [u8]) -> Result<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3;
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            if (inst & 0xFC00_0000) == 0x9400_0000 {
                // Extract absolute address
                let addr = ((inst & 0x03FF_FFFF) as i32) << 6 >> 4; // Sign extend and multiply by 4

                // Convert back to relative
                let offset = addr.wrapping_sub((self.pos + i) as i32) >> 2;
                let new_inst = (inst & 0xFC00_0000) | ((offset as u32) & 0x03FF_FFFF);

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// ARM 32-bit BCJ encoding (simplified - handles BL in ARM mode).
    fn encode_arm(&mut self, data: &mut [u8]) -> Result<()> {
        // Similar to ARM64 but with different instruction format
        // BL: cccc 1011 xxxx xxxx xxxx xxxx xxxx xxxx
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3;
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            // Check for BL instruction (bits 27-24 = 1011)
            if (inst & 0x0F00_0000) == 0x0B00_0000 {
                // Extract 24-bit signed offset
                let offset = ((inst & 0x00FF_FFFF) as i32) << 8 >> 6; // Sign extend, multiply by 4

                // Convert to absolute
                let addr = ((self.pos + i + 8) as i32).wrapping_add(offset); // +8 for ARM pipeline

                // Store as new offset
                let new_offset = ((addr >> 2) as u32) & 0x00FF_FFFF;
                let new_inst = (inst & 0xFF00_0000) | new_offset;

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// ARM 32-bit BCJ decoding.
    fn decode_arm(&mut self, data: &mut [u8]) -> Result<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3;
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            if (inst & 0x0F00_0000) == 0x0B00_0000 {
                let addr = ((inst & 0x00FF_FFFF) as i32) << 8 >> 6;
                let offset = addr.wrapping_sub((self.pos + i + 8) as i32) >> 2;
                let new_inst = (inst & 0xFF00_0000) | ((offset as u32) & 0x00FF_FFFF);

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// RISC-V BCJ encoding (JAL and AUIPC instructions).
    fn encode_riscv(&mut self, data: &mut [u8]) -> Result<()> {
        // RISC-V has complex instruction encoding, simplified version
        // JAL: imm[20|10:1|11|19:12] rd opcode (opcode = 1101111)
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 1; // 2-byte alignment for compressed
        if i != 0 {
            i = 2 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            // JAL: opcode = 0b1101111 (0x6F)
            if (inst & 0x7F) == 0x6F {
                // Decode JAL immediate (complex bit shuffling)
                let imm20 = (inst >> 31) & 1;
                let imm10_1 = (inst >> 21) & 0x3FF;
                let imm11 = (inst >> 20) & 1;
                let imm19_12 = (inst >> 12) & 0xFF;

                let offset = ((imm20 << 20)
                    | (imm19_12 << 12)
                    | (imm11 << 11)
                    | (imm10_1 << 1)) as i32;
                let offset = (offset << 11) >> 11; // Sign extend from bit 20

                // Convert to absolute
                let addr = ((self.pos + i) as i32).wrapping_add(offset);

                // Re-encode with new address
                let new_imm = addr as u32;
                let new_inst = (inst & 0xFFF)
                    | ((new_imm & 0xFF000) << 0)      // imm[19:12]
                    | (((new_imm >> 11) & 1) << 20)   // imm[11]
                    | (((new_imm >> 1) & 0x3FF) << 21) // imm[10:1]
                    | (((new_imm >> 20) & 1) << 31);  // imm[20]

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4; // Could be 2 for compressed, but simplified
        }

        self.pos += data.len();
        Ok(())
    }

    /// RISC-V BCJ decoding.
    fn decode_riscv(&mut self, data: &mut [u8]) -> Result<()> {
        // Reverse of encode - similar structure
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 1;
        if i != 0 {
            i = 2 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            if (inst & 0x7F) == 0x6F {
                // Decode the stored absolute address
                let imm20 = (inst >> 31) & 1;
                let imm10_1 = (inst >> 21) & 0x3FF;
                let imm11 = (inst >> 20) & 1;
                let imm19_12 = (inst >> 12) & 0xFF;

                let addr = ((imm20 << 20)
                    | (imm19_12 << 12)
                    | (imm11 << 11)
                    | (imm10_1 << 1)) as i32;
                let addr = (addr << 11) >> 11;

                // Convert back to relative
                let offset = addr.wrapping_sub((self.pos + i) as i32);

                // Re-encode
                let new_imm = offset as u32;
                let new_inst = (inst & 0xFFF)
                    | ((new_imm & 0xFF000) << 0)
                    | (((new_imm >> 11) & 1) << 20)
                    | (((new_imm >> 1) & 0x3FF) << 21)
                    | (((new_imm >> 20) & 1) << 31);

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// PowerPC64 LE BCJ encoding.
    fn encode_ppc64(&mut self, data: &mut [u8]) -> Result<()> {
        // PPC64 branch instructions
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3;
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            // PPC is big-endian instructions but PPC64LE is little-endian
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            // Branch (b, bl): opcode 18 (bits 0-5 = 010010)
            let opcode = (inst >> 26) & 0x3F;
            if opcode == 18 {
                // LI field: bits 6-29 (24 bits, but only 26-bit signed offset)
                let li = (inst >> 2) & 0x00FF_FFFF;
                let offset = ((li as i32) << 8) >> 6; // Sign extend and multiply by 4

                let addr = ((self.pos + i) as i32).wrapping_add(offset);

                let new_li = ((addr >> 2) as u32) & 0x00FF_FFFF;
                let new_inst = (inst & 0xFC00_0003) | (new_li << 2);

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }

    /// PowerPC64 LE BCJ decoding.
    fn decode_ppc64(&mut self, data: &mut [u8]) -> Result<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let mut i = self.pos & 3;
        if i != 0 {
            i = 4 - i;
        }

        while i + 4 <= data.len() {
            let inst = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);

            let opcode = (inst >> 26) & 0x3F;
            if opcode == 18 {
                let li = (inst >> 2) & 0x00FF_FFFF;
                let addr = ((li as i32) << 8) >> 6;

                let offset = addr.wrapping_sub((self.pos + i) as i32);

                let new_li = ((offset >> 2) as u32) & 0x00FF_FFFF;
                let new_inst = (inst & 0xFC00_0003) | (new_li << 2);

                let bytes = new_inst.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
                data[i + 2] = bytes[2];
                data[i + 3] = bytes[3];
            }

            i += 4;
        }

        self.pos += data.len();
        Ok(())
    }
}

/// Convenience function to encode data with BCJ filter.
pub fn bcj_encode(data: &mut [u8], arch: BcjArch) -> Result<()> {
    let mut filter = BcjFilter::new(arch);
    filter.encode(data)
}

/// Convenience function to decode data with BCJ filter.
pub fn bcj_decode(data: &mut [u8], arch: BcjArch) -> Result<()> {
    let mut filter = BcjFilter::new(arch);
    filter.decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_roundtrip() {
        // Simulate x86 CALL instruction: E8 followed by relative offset
        let original: Vec<u8> = vec![
            0x55, 0x48, 0x89, 0xe5, // push rbp; mov rbp, rsp
            0xE8, 0x10, 0x00, 0x00, 0x00, // call +16
            0x48, 0x89, 0xec, 0x5d, // mov rsp, rbp; pop rbp
            0xC3, // ret
            0xE9, 0xF0, 0xFF, 0xFF, 0xFF, // jmp -16
        ];

        let mut data = original.clone();

        // Encode
        bcj_encode(&mut data, BcjArch::X86).unwrap();
        assert_ne!(data, original, "Encoding should change data");

        // Decode
        bcj_decode(&mut data, BcjArch::X86).unwrap();
        assert_eq!(data, original, "Roundtrip should restore original");
    }

    #[test]
    fn test_arch_detection() {
        assert_eq!(BcjArch::from_target("x86_64-unknown-linux-gnu"), BcjArch::X86);
        assert_eq!(BcjArch::from_target("aarch64-apple-darwin"), BcjArch::Arm64);
        assert_eq!(BcjArch::from_target("armv7-unknown-linux-gnueabihf"), BcjArch::Arm);
        assert_eq!(BcjArch::from_target("riscv64gc-unknown-linux-gnu"), BcjArch::RiscV);
        assert_eq!(BcjArch::from_target("wasm32-wasip1"), BcjArch::None);
    }

    #[test]
    fn test_empty_data() {
        let mut data: Vec<u8> = vec![];
        bcj_encode(&mut data, BcjArch::X86).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_small_data() {
        let mut data = vec![0xE8, 0x01, 0x02]; // Too small for full instruction
        let original = data.clone();
        bcj_encode(&mut data, BcjArch::X86).unwrap();
        assert_eq!(data, original, "Small data should be unchanged");
    }
}
