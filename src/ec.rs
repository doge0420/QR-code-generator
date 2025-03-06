#[derive(Clone, Copy)]
pub enum EcLevel {
    H,
    Q,
    M,
    L,
}

impl EcLevel {
    pub fn ordinal(&self) -> u8 {
        match self {
            EcLevel::L => 0,
            EcLevel::M => 1,
            EcLevel::Q => 2,
            EcLevel::H => 3,
        }
    }
}

pub fn error_correction(data: &Vec<u8>, version: u8, ec_level: &EcLevel) -> Vec<u8> {
    let ec_level = ec_level.ordinal();
    let (block_1_size, block_1_count, block_2_size, block_2_count) =
        DATA_BYTES_PER_BLOCK[(version - 1) as usize][ec_level as usize];

    let group_1_size = block_1_count * block_1_size;

    let mut blocks = Vec::with_capacity(block_1_count + block_2_count);
    
    if group_1_size < data.len() {
        let (group_1, group_2) = data.split_at(group_1_size);

        group_1.chunks(block_1_size).for_each(|block| {
            blocks.push(block.to_vec());
        });
        if block_2_size > 0 {
            group_2.chunks(block_2_size).for_each(|block| {
                blocks.push(block.to_vec());
            });
        }
    } else {
        data.chunks(block_1_size).for_each(|block| {
            blocks.push(block.to_vec());
        });
    }
        
    let ec_blocks = blocks
        .iter()
        .map(|block| {
            let ec_size = EC_BYTES_PER_BLOCK[version as usize][ec_level as usize];
            let generator_polynomial = GENERATOR_POLYNOMIALS[ec_size];
            create_ec_for_block(Vec::from(block.clone()), ec_size, generator_polynomial)
        })
        .collect::<Vec<Vec<u8>>>();

    interleave(ec_blocks)
}

fn create_ec_for_block(block: Vec<u8>, ec_size: usize, generator_polynomial: &[u8]) -> Vec<u8> {
    let data_len = block.len();
    let mut codewords = block;
    codewords.resize(data_len + ec_size, 0);

    for i in 0..data_len {
        let lead_coeff = codewords[i];
        if lead_coeff == 0 {
            continue;
        }
        let log_lead_coeff = usize::from(LOG_TABLE[lead_coeff as usize]);

        codewords[i + 1..]
            .iter_mut()
            .zip(generator_polynomial.iter())
            .for_each(|(cw, &gen_coeff)| {
                *cw ^= EXP_TABLE[(usize::from(gen_coeff) + log_lead_coeff) % 255];
            });
    }

    codewords.split_off(data_len)
}

fn interleave(blocks: Vec<Vec<u8>>) -> Vec<u8> {
    let mut result = Vec::new();
    let max_len = blocks.iter().map(|block| block.len()).max().unwrap();
    for i in 0..max_len {
        for block in &blocks {
            if i < block.len() {
                result.push(block[i]);
            }
        }
    }
    result
}

#[cfg(test)]
mod interleave_tests {
    use super::*;

    #[test]
    fn interleave_works_with_equal_length_blocks() {
        let blocks = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let result = interleave(blocks);
        assert_eq!(result, vec![1, 4, 7, 2, 5, 8, 3, 6, 9]);
    }

    #[test]
    fn interleave_works_with_unequal_length_blocks() {
        let blocks = vec![vec![1, 2], vec![3, 4, 5], vec![6]];
        let result = interleave(blocks);
        assert_eq!(result, vec![1, 3, 6, 2, 4, 5]);
    }

    #[test]
    fn interleave_works_with_empty_blocks() {
        let blocks: Vec<Vec<u8>> = vec![vec![], vec![], vec![]];
        let result = interleave(blocks);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn interleave_works_with_single_block() {
        let blocks = vec![vec![1, 2, 3]];
        let result = interleave(blocks);
        assert_eq!(result, vec![1, 2, 3]);
    }
}

#[cfg(test)]
mod ec_tests {
    use super::*;

    #[test]
    fn create_ec_for_block_works_simple() {
        let block = vec![1, 2, 3];
        let ec_size = block.len();
        let generator_polynomial = GENERATOR_POLYNOMIALS[ec_size];

        let ec = create_ec_for_block(block, ec_size, generator_polynomial);
        assert!(ec.eq(&vec![92, 236, 176]));
    }

    #[test]
    fn create_ec_for_block_works_complex() {
        let block = vec![32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236];
        let ec_size = block.len();
        let generator_polynomial = GENERATOR_POLYNOMIALS[ec_size];

        let ec = create_ec_for_block(block, ec_size, generator_polynomial);
        assert!(ec.eq(&vec![
            168, 72, 22, 82, 217, 54, 156, 0, 46, 15, 180, 122, 16
        ]));
    }
}

/// https://github.com/kennytm/qrcode-rust/blob/master/src/ec.rs @ line 380
///
/// The number of error correction bytes per block per ec level.
static EC_BYTES_PER_BLOCK: [[usize; 4]; 40] = [
    // Normal versions.
    [7, 10, 13, 17],  // 1
    [10, 16, 22, 28], // 2
    [15, 26, 18, 22], // 3
    [20, 18, 26, 16], // 4
    [26, 24, 18, 22], // 5
    [18, 16, 24, 28], // 6
    [20, 18, 18, 26], // 7
    [24, 22, 22, 26], // 8
    [30, 22, 20, 24], // 9
    [18, 26, 24, 28], // 10
    [20, 30, 28, 24], // 11
    [24, 22, 26, 28], // 12
    [26, 22, 24, 22], // 13
    [30, 24, 20, 24], // 14
    [22, 24, 30, 24], // 15
    [24, 28, 24, 30], // 16
    [28, 28, 28, 28], // 17
    [30, 26, 28, 28], // 18
    [28, 26, 26, 26], // 19
    [28, 26, 30, 28], // 20
    [28, 26, 28, 30], // 21
    [28, 28, 30, 24], // 22
    [30, 28, 30, 30], // 23
    [30, 28, 30, 30], // 24
    [26, 28, 30, 30], // 25
    [28, 28, 28, 30], // 26
    [30, 28, 30, 30], // 27
    [30, 28, 30, 30], // 28
    [30, 28, 30, 30], // 29
    [30, 28, 30, 30], // 30
    [30, 28, 30, 30], // 31
    [30, 28, 30, 30], // 32
    [30, 28, 30, 30], // 33
    [30, 28, 30, 30], // 34
    [30, 28, 30, 30], // 35
    [30, 28, 30, 30], // 36
    [30, 28, 30, 30], // 37
    [30, 28, 30, 30], // 38
    [30, 28, 30, 30], // 39
    [30, 28, 30, 30], // 40
];

/// https://github.com/kennytm/qrcode-rust/blob/master/src/ec.rs @ line 250
///
/// `EXP_TABLE` encodes the value of 2<sup>n</sup> in the Galois Field GF(256).
static EXP_TABLE: &[u8] = b"\
\x01\x02\x04\x08\x10\x20\x40\x80\x1d\x3a\x74\xe8\xcd\x87\x13\x26\
\x4c\x98\x2d\x5a\xb4\x75\xea\xc9\x8f\x03\x06\x0c\x18\x30\x60\xc0\
\x9d\x27\x4e\x9c\x25\x4a\x94\x35\x6a\xd4\xb5\x77\xee\xc1\x9f\x23\
\x46\x8c\x05\x0a\x14\x28\x50\xa0\x5d\xba\x69\xd2\xb9\x6f\xde\xa1\
\x5f\xbe\x61\xc2\x99\x2f\x5e\xbc\x65\xca\x89\x0f\x1e\x3c\x78\xf0\
\xfd\xe7\xd3\xbb\x6b\xd6\xb1\x7f\xfe\xe1\xdf\xa3\x5b\xb6\x71\xe2\
\xd9\xaf\x43\x86\x11\x22\x44\x88\x0d\x1a\x34\x68\xd0\xbd\x67\xce\
\x81\x1f\x3e\x7c\xf8\xed\xc7\x93\x3b\x76\xec\xc5\x97\x33\x66\xcc\
\x85\x17\x2e\x5c\xb8\x6d\xda\xa9\x4f\x9e\x21\x42\x84\x15\x2a\x54\
\xa8\x4d\x9a\x29\x52\xa4\x55\xaa\x49\x92\x39\x72\xe4\xd5\xb7\x73\
\xe6\xd1\xbf\x63\xc6\x91\x3f\x7e\xfc\xe5\xd7\xb3\x7b\xf6\xf1\xff\
\xe3\xdb\xab\x4b\x96\x31\x62\xc4\x95\x37\x6e\xdc\xa5\x57\xae\x41\
\x82\x19\x32\x64\xc8\x8d\x07\x0e\x1c\x38\x70\xe0\xdd\xa7\x53\xa6\
\x51\xa2\x59\xb2\x79\xf2\xf9\xef\xc3\x9b\x2b\x56\xac\x45\x8a\x09\
\x12\x24\x48\x90\x3d\x7a\xf4\xf5\xf7\xf3\xfb\xeb\xcb\x8b\x0b\x16\
\x2c\x58\xb0\x7d\xfa\xe9\xcf\x83\x1b\x36\x6c\xd8\xad\x47\x8e\x01";

/// https://github.com/kennytm/qrcode-rust/blob/master/src/ec.rs @ line 269
///
/// `LOG_TABLE` is the inverse function of `EXP_TABLE`.
static LOG_TABLE: &[u8] = b"\
\xff\x00\x01\x19\x02\x32\x1a\xc6\x03\xdf\x33\xee\x1b\x68\xc7\x4b\
\x04\x64\xe0\x0e\x34\x8d\xef\x81\x1c\xc1\x69\xf8\xc8\x08\x4c\x71\
\x05\x8a\x65\x2f\xe1\x24\x0f\x21\x35\x93\x8e\xda\xf0\x12\x82\x45\
\x1d\xb5\xc2\x7d\x6a\x27\xf9\xb9\xc9\x9a\x09\x78\x4d\xe4\x72\xa6\
\x06\xbf\x8b\x62\x66\xdd\x30\xfd\xe2\x98\x25\xb3\x10\x91\x22\x88\
\x36\xd0\x94\xce\x8f\x96\xdb\xbd\xf1\xd2\x13\x5c\x83\x38\x46\x40\
\x1e\x42\xb6\xa3\xc3\x48\x7e\x6e\x6b\x3a\x28\x54\xfa\x85\xba\x3d\
\xca\x5e\x9b\x9f\x0a\x15\x79\x2b\x4e\xd4\xe5\xac\x73\xf3\xa7\x57\
\x07\x70\xc0\xf7\x8c\x80\x63\x0d\x67\x4a\xde\xed\x31\xc5\xfe\x18\
\xe3\xa5\x99\x77\x26\xb8\xb4\x7c\x11\x44\x92\xd9\x23\x20\x89\x2e\
\x37\x3f\xd1\x5b\x95\xbc\xcf\xcd\x90\x87\x97\xb2\xdc\xfc\xbe\x61\
\xf2\x56\xd3\xab\x14\x2a\x5d\x9e\x84\x3c\x39\x53\x47\x6d\x41\xa2\
\x1f\x2d\x43\xd8\xb7\x7b\xa4\x76\xc4\x17\x49\xec\x7f\x0c\x6f\xf6\
\x6c\xa1\x3b\x52\x29\x9d\x55\xaa\xfb\x60\x86\xb1\xbb\xcc\x3e\x5a\
\xcb\x59\x5f\xb0\x9c\xa9\xa0\x51\x0b\xf5\x16\xeb\x7a\x75\x2c\xd7\
\x4f\xae\xd5\xe9\xe6\xe7\xad\xe8\x74\xd6\xf4\xea\xa8\x50\x58\xaf";

/// https://github.com/kennytm/qrcode-rust/blob/master/src/ec.rs @ line 298
///
/// The index encodes the number of error correction codewords needed.
static GENERATOR_POLYNOMIALS: [&[u8]; 70] = [
    b"",
    b"\x00",
    b"\x19\x01",
    b"\xc6\xc7\x03",
    b"\x4b\xf9\x4e\x06",
    b"\x71\xa4\xa6\x77\x0a",
    b"\xa6\x00\x86\x05\xb0\x0f",
    b"\x57\xe5\x92\x95\xee\x66\x15",
    b"\xaf\xee\xd0\xf9\xd7\xfc\xc4\x1c",
    b"\x5f\xf6\x89\xe7\xeb\x95\x0b\x7b\x24",
    b"\xfb\x43\x2e\x3d\x76\x46\x40\x5e\x20\x2d",
    b"\xdc\xc0\x5b\xc2\xac\xb1\xd1\x74\xe3\x0a\x37",
    b"\x66\x2b\x62\x79\xbb\x71\xc6\x8f\x83\x57\x9d\x42",
    b"\x4a\x98\xb0\x64\x56\x64\x6a\x68\x82\xda\xce\x8c\x4e",
    b"\xc7\xf9\x9b\x30\xbe\x7c\xda\x89\xd8\x57\xcf\x3b\x16\x5b",
    b"\x08\xb7\x3d\x5b\xca\x25\x33\x3a\x3a\xed\x8c\x7c\x05\x63\x69",
    b"\x78\x68\x6b\x6d\x66\xa1\x4c\x03\x5b\xbf\x93\xa9\xb6\xc2\xe1\x78",
    b"\x2b\x8b\xce\x4e\x2b\xef\x7b\xce\xd6\x93\x18\x63\x96\x27\xf3\xa3\x88",
    b"\xd7\xea\x9e\x5e\xb8\x61\x76\xaa\x4f\xbb\x98\x94\xfc\xb3\x05\x62\x60\x99",
    b"\x43\x03\x69\x99\x34\x5a\x53\x11\x96\x9f\x2c\x80\x99\x85\xfc\xde\x8a\xdc\xab",
    b"\x11\x3c\x4f\x32\x3d\xa3\x1a\xbb\xca\xb4\xdd\xe1\x53\xef\x9c\xa4\xd4\xd4\xbc\xbe",
    b"\xf0\xe9\x68\xf7\xb5\x8c\x43\x62\x55\xc8\xd2\x73\x94\x89\xe6\x24\x7a\xfe\x94\xaf\xd2",
    b"\xd2\xab\xf7\xf2\x5d\xe6\x0e\x6d\xdd\x35\xc8\x4a\x08\xac\x62\x50\xdb\x86\xa0\x69\xa5\xe7",
    b"\xab\x66\x92\x5b\x31\x67\x41\x11\xc1\x96\x0e\x19\xb7\xf8\x5e\xa4\xe0\xc0\x01\x4e\x38\x93\xfd",
    b"\xe5\x79\x87\x30\xd3\x75\xfb\x7e\x9f\xb4\xa9\x98\xc0\xe2\xe4\xda\x6f\x00\x75\xe8\x57\x60\xe3\x15",
    b"\xe7\xb5\x9c\x27\xaa\x1a\x0c\x3b\x0f\x94\xc9\x36\x42\xed\xd0\x63\xa7\x90\xb6\x5f\xf3\x81\xb2\xfc\x2d",
    b"\xad\x7d\x9e\x02\x67\xb6\x76\x11\x91\xc9\x6f\x1c\xa5\x35\xa1\x15\xf5\x8e\x0d\x66\x30\xe3\x99\x91\xda\x46",
    b"\x4f\xe4\x08\xa5\xe3\x15\xb4\x1d\x09\xed\x46\x63\x2d\x3a\x8a\x87\x49\x7e\xac\x5e\xd8\xc1\x9d\x1a\x11\x95\x60",
    b"\xa8\xdf\xc8\x68\xe0\xea\x6c\xb4\x6e\xbe\xc3\x93\xcd\x1b\xe8\xc9\x15\x2b\xf5\x57\x2a\xc3\xd4\x77\xf2\x25\x09\x7b",
    b"\x9c\x2d\xb7\x1d\x97\xdb\x36\x60\xf9\x18\x88\x05\xf1\xaf\xbd\x1c\x4b\xea\x96\x94\x17\x09\xca\xa2\x44\xfa\x8c\x18\x97",
    b"\x29\xad\x91\x98\xd8\x1f\xb3\xb6\x32\x30\x6e\x56\xef\x60\xde\x7d\x2a\xad\xe2\xc1\xe0\x82\x9c\x25\xfb\xd8\xee\x28\xc0\xb4",
    b"\x14\x25\xfc\x5d\x3f\x4b\xe1\x1f\x73\x53\x71\x27\x2c\x49\x7a\x89\x76\x77\x90\xf8\xf8\x37\x01\xe1\x69\x7b\xb7\x75\xbb\xc8\xd2",
    b"\x0a\x06\x6a\xbe\xf9\xa7\x04\x43\xd1\x8a\x8a\x20\xf2\x7b\x59\x1b\x78\xb9\x50\x9c\x26\x45\xab\x3c\x1c\xde\x50\x34\xfe\xb9\xdc\xf1",
    b"\xf5\xe7\x37\x18\x47\x4e\x4c\x51\xe1\xd4\xad\x25\xd7\x2e\x77\xe5\xf5\xa7\x7e\x48\xb5\x5e\xa5\xd2\x62\x7d\x9f\xb8\xa9\xe8\xb9\xe7\x12",
    b"\x6f\x4d\x92\x5e\x1a\x15\x6c\x13\x69\x5e\x71\xc1\x56\x8c\xa3\x7d\x3a\x9e\xe5\xef\xda\x67\x38\x46\x72\x3d\xb7\x81\xa7\x0d\x62\x3e\x81\x33",
    b"\x07\x5e\x8f\x51\xf7\x7f\xca\xca\xc2\x7d\x92\x1d\x8a\xa2\x99\x41\x69\x7a\x74\xee\x1a\x24\xd8\x70\x7d\xe4\x0f\x31\x08\xa2\x1e\x7e\x6f\x3a\x55",
    b"\xc8\xb7\x62\x10\xac\x1f\xf6\xea\x3c\x98\x73\x00\xa7\x98\x71\xf8\xee\x6b\x12\x3f\xda\x25\x57\xd2\x69\xb1\x78\x4a\x79\xc4\x75\xfb\x71\xe9\x1e\x78",
    b"\x9a\x4b\x8d\xb4\x3d\xa5\x68\xe8\x2e\xe3\x60\xb2\x5c\x87\x39\xa2\x78\xc2\xd4\xae\xfc\xb7\x2a\x23\x9d\x6f\x17\x85\x64\x08\x69\x25\xc0\xbd\x9f\x13\x9c",
    b"\x9f\x22\x26\xe4\xe6\x3b\xf3\x5f\x31\xda\xb0\xa4\x14\x41\x2d\x6f\x27\x51\x31\x76\x71\xde\xc1\xfa\xf2\xa8\xd9\x29\xa4\xf7\xb1\x1e\xee\x12\x78\x99\x3c\xc1",
    b"\x51\xd8\xae\x2f\xc8\x96\x3b\x9c\x59\x8f\x59\xa6\xb7\xaa\x98\x15\xa5\xb1\x71\x84\xea\x05\x9a\x44\x7c\xaf\xc4\x9d\xf9\xe9\x53\x18\x99\xf1\x7e\x24\x74\x13\xe7",
    b"\x3b\x74\x4f\xa1\xfc\x62\x80\xcd\x80\xa1\xf7\x39\xa3\x38\xeb\x6a\x35\x1a\xbb\xae\xe2\x68\xaa\x07\xaf\x23\xb5\x72\x58\x29\x2f\xa3\x7d\x86\x48\x14\xe8\x35\x23\x0f",
    b"\x84\xa7\x34\x8b\xb8\xdf\x95\x5c\xfa\x12\x53\x21\x7f\x6d\xc2\x07\xd3\xf2\x6d\x42\x56\xa9\x57\x60\xbb\x9f\x72\xac\x76\xd0\xb7\xc8\x52\xb3\x26\x27\x22\xf2\x8e\x93\x37",
    b"\xfa\x67\xdd\xe6\x19\x12\x89\xe7\x00\x03\x3a\xf2\xdd\xbf\x6e\x54\xe6\x08\xbc\x6a\x60\x93\x0f\x83\x8b\x22\x65\xdf\x27\x65\xd5\xc7\xed\xfe\xc9\x7b\xab\xa2\xc2\x75\x32\x60",
    b"\x60\x43\x03\xf5\xd9\xd7\x21\x41\xf0\x6d\x90\x3f\x15\x83\x26\x65\x99\x80\x37\x1f\xed\x03\x5e\xa0\x14\x57\x4d\x38\xbf\x7b\xcf\x4b\x52\x00\x7a\x84\x65\x91\xd7\x0f\x79\xc0\x8a",
    b"\xbe\x07\x3d\x79\x47\xf6\x45\x37\xa8\xbc\x59\xf3\xbf\x19\x48\x7b\x09\x91\x0e\xf7\x01\xee\x2c\x4e\x8f\x3e\xe0\x7e\x76\x72\x44\xa3\x34\xc2\xd9\x93\xcc\xa9\x25\x82\x71\x66\x49\xb5",
    b"\x06\xac\x48\xfa\x12\xab\xab\xa2\xe5\xbb\xef\x04\xbb\x0b\x25\xe4\x66\x48\x66\x16\x21\x49\x5f\x63\x84\x01\x0f\x59\x04\x70\x82\x5f\xd3\xeb\xe3\x3a\x23\x58\x84\x17\x2c\xa5\x36\xbb\xe1",
    b"\x70\x5e\x58\x70\xfd\xe0\xca\x73\xbb\x63\x59\x05\x36\x71\x81\x2c\x3a\x10\x87\xd8\xa9\xd3\x24\x01\x04\x60\x3c\xf1\x49\x68\xea\x08\xf9\xf5\x77\xae\x34\x19\x9d\xe0\x2b\xca\xdf\x13\x52\x0f",
    b"\x4c\xa4\xe5\x5c\x4f\xa8\xdb\x6e\x68\x15\xdc\x4a\x13\xc7\xc3\x64\x5d\xbf\x2b\xd5\x48\x38\x8a\xa1\x7d\xbb\x77\xfa\xbd\x89\xbe\x4c\x7e\xf7\x5d\x1e\x84\x06\x3a\xd5\xd0\xa5\xe0\x98\x85\x5b\x3d",
    b"\xe4\x19\xc4\x82\xd3\x92\x3c\x18\xfb\x5a\x27\x66\xf0\x3d\xb2\x3f\x2e\x7b\x73\x12\xdd\x6f\x87\xa0\xb6\xcd\x6b\xce\x5f\x96\x78\xb8\x5b\x15\xf7\x9c\x8c\xee\xbf\x0b\x5e\xe3\x54\x32\xa3\x27\x22\x6c",
    b"\xac\x79\x01\x29\xc1\xde\xed\x40\x6d\xb5\x34\x78\xd4\xe2\xef\xf5\xd0\x14\xf6\x22\xe1\xcc\x86\x65\x7d\xce\x45\x8a\xfa\x00\x4d\x3a\x8f\xb9\xdc\xfe\xd2\xbe\x70\x58\x5b\x39\x5a\x6d\x05\x0d\xb5\x19\x9c",
    b"\xe8\x7d\x9d\xa1\xa4\x09\x76\x2e\xd1\x63\xcb\xc1\x23\x03\xd1\x6f\xc3\xf2\xcb\xe1\x2e\x0d\x20\xa0\x7e\xd1\x82\xa0\xf2\xd7\xf2\x4b\x4d\x2a\xbd\x20\x71\x41\x7c\x45\xe4\x72\xeb\xaf\x7c\xaa\xd7\xe8\x85\xcd",
    b"\xd5\xa6\x8e\x2b\x0a\xd8\x8d\xa3\xac\xb4\x66\x46\x59\x3e\xde\x3e\x2a\xd2\x97\xa3\xda\x46\x4d\x27\xa6\xbf\x72\xca\xf5\xbc\xb7\xdd\x4b\xd4\x1b\xed\x7f\xcc\xeb\x3e\xbe\xe8\x12\x2e\xab\x0f\x62\xf7\x42\xa3\x00",
    b"\x74\x32\x56\xba\x32\xdc\xfb\x59\xc0\x2e\x56\x7f\x7c\x13\xb8\xe9\x97\xd7\x16\x0e\x3b\x91\x25\xf2\xcb\x86\xfe\x59\xbe\x5e\x3b\x41\x7c\x71\x64\xe9\xeb\x79\x16\x4c\x56\x61\x27\xf2\xc8\xdc\x65\x21\xef\xfe\x74\x33",
    b"\x7a\xd6\xe7\x88\xc7\x0b\x06\xcd\x7c\x48\xd5\x75\xbb\x3c\x93\xc9\x49\x4b\x21\x92\xab\xf7\x76\xd0\x9d\xb1\xcb\xeb\x53\x2d\xe2\xca\xe5\xa8\x07\x39\xed\xeb\xc8\x7c\x6a\xfe\xa5\x0e\x93\x00\x39\x2a\x1f\xb2\xd5\xad\x67",
    b"\xb7\x1a\xc9\x57\xd2\xdd\x71\x15\x2e\x41\x2d\x32\xee\xb8\xf9\xe1\x66\x3a\xd1\xda\x6d\xa5\x1a\x5f\xb8\xc0\x34\xf5\x23\xfe\xee\xaf\xac\x4f\x7b\x19\x7a\x2b\x78\x6c\xd7\x50\x80\xc9\xeb\x08\x99\x3b\x65\x1f\xc6\x4c\x1f\x9c",
    b"\x26\xc5\x7b\xa7\x10\x57\xb2\xee\xe3\x61\x94\xf7\x1a\x5a\xe4\xb6\xec\xc5\x2f\xf9\x24\xd5\x36\x71\xb5\x4a\xb1\xcc\x9b\x3d\x2f\x2a\x00\x84\x90\xfb\xc8\x26\x26\x8a\x36\x2c\x40\x13\x16\xce\x10\x0a\xe4\xd3\xa1\xab\x2c\xc2\xd2",
    b"\x6a\x78\x6b\x9d\xa4\xd8\x70\x74\x02\x5b\xf8\xa3\x24\xc9\xca\xe5\x06\x90\xfe\x9b\x87\xd0\xaa\xd1\x0c\x8b\x7f\x8e\xb6\xf9\xb1\xae\xbe\x1c\x0a\x55\xef\xb8\x65\x7c\x98\xce\x60\x17\xa3\x3d\x1b\xc4\xf7\x97\x9a\xca\xcf\x14\x3d\x0a",
    b"\x3a\x8c\xed\x5d\x6a\x3d\xc1\x02\x57\x49\xc2\xd7\x9f\xa3\x0a\x9b\x05\x79\x99\x3b\xf8\x04\x75\x16\x3c\xb1\x90\x2c\x48\xe4\x3e\x01\x13\xaa\x71\x9e\x19\xaf\xc7\x8b\x5a\x01\xd2\x07\x77\x9a\x59\x9f\x82\x7a\x2e\x93\xbe\x87\x5e\x44\x42",
    b"\x52\x74\x1a\xf7\x42\x1b\x3e\x6b\xfc\xb6\xc8\xb9\xeb\x37\xfb\xf2\xd2\x90\x9a\xed\xb0\x8d\xc0\xf8\x98\xf9\xce\x55\xfd\x8e\x41\xa5\x7d\x17\x18\x1e\x7a\xf0\xd6\x06\x81\xda\x1d\x91\x7f\x86\xce\xf5\x75\x1d\x29\x3f\x9f\x8e\xe9\x7d\x94\x7b",
    b"\x39\x73\xe8\x0b\xc3\xd9\x03\xce\x4d\x43\x1d\xa6\xb4\x6a\x76\xcb\x11\x45\x98\xd5\x4a\x2c\x31\x2b\x62\x3d\xfd\x7a\x0e\x2b\xd1\x8f\x09\x68\x6b\xab\xe0\x39\xfe\xfb\xe2\xe8\xdd\xc2\xf0\x75\xa1\x52\xb2\xf6\xb2\x21\x32\x56\xd7\xef\xb4\xb4\xb5",
    b"\x6b\x8c\x1a\x0c\x09\x8d\xf3\xc5\xe2\xc5\xdb\x2d\xd3\x65\xdb\x78\x1c\xb5\x7f\x06\x64\xf7\x02\xcd\xc6\x39\x73\xdb\x65\x6d\xa0\x52\x25\x26\xee\x31\xa0\xd1\x79\x56\x0b\x7c\x1e\xb5\x54\x19\xc2\x57\x41\x66\xbe\xdc\x46\x1b\xd1\x10\x59\x07\x21\xf0",
    b"\xa1\xf4\x69\x73\x40\x09\xdd\xec\x10\x91\x94\x22\x90\xba\x0d\x14\xfe\xf6\x26\x23\xca\x48\x04\xd4\x9f\xd3\xa5\x87\xfc\xfa\x19\x57\x1e\x78\xe2\xea\x5c\xc7\x48\x07\x9b\xda\xe7\x2c\x7d\xb2\x9c\xae\x7c\x2b\x64\x1f\x38\x65\xcc\x40\xaf\xe1\xa9\x92\x2d",
    b"\x41\xca\x71\x62\x47\xdf\xf8\x76\xd6\x5e\x00\x7a\x25\x17\x02\xe4\x3a\x79\x07\x69\x87\x4e\xf3\x76\x46\x4c\xdf\x59\x48\x32\x46\x6f\xc2\x11\xd4\x7e\xb5\x23\xdd\x75\xeb\x0b\xe5\x95\x93\x7b\xd5\x28\x73\x06\xc8\x64\x1a\xf6\xb6\xda\x7f\xd7\x24\xba\x6e\x6a",
    b"\x1e\x47\x24\x47\x13\xc3\xac\x6e\x3d\x02\xa9\xc2\x5a\x88\x3b\xb6\xe7\x91\x66\x27\xaa\xe7\xd6\x43\xc4\xcf\x35\x70\xf6\x5a\x5a\x79\xb7\x92\x4a\x4d\x26\x59\x16\xe7\x37\x38\xf2\x70\xd9\x6e\x7b\x3e\xc9\xd9\x80\xa5\x3c\xb5\x25\xa1\xf6\x84\xf6\x12\x73\x88\xa8",
    b"\x2d\x33\xaf\x09\x07\x9e\x9f\x31\x44\x77\x5c\x7b\xb1\xcc\xbb\xfe\xc8\x4e\x8d\x95\x77\x1a\x7f\x35\xa0\x5d\xc7\xd4\x1d\x18\x91\x9c\xd0\x96\xda\xd1\x04\xd8\x5b\x2f\xb8\x92\x2f\x8c\xc3\xc3\x7d\xf2\xee\x3f\x63\x6c\x8c\xe6\xf2\x1f\xcc\x0b\xb2\xf3\xd9\x9c\xd5\xe7",
    b"\x89\x9e\xf7\xf0\x25\xee\xd6\x80\x63\xda\x2e\x8a\xc6\x80\x5c\xdb\x6d\x8b\xa6\x19\x42\x43\x0e\x3a\xee\x95\xb1\xc3\xdd\x9a\xab\x30\x50\x0c\x3b\xbe\xe4\x13\x37\xd0\x5c\x70\xe5\x25\x3c\x0a\x2f\x51\x00\xc0\x25\xab\xaf\x93\x80\x49\xa6\x3d\x95\x0c\x18\x5f\x46\x71\x28",
    b"\x05\x76\xde\xb4\x88\x88\xa2\x33\x2e\x75\x0d\xd7\x51\x11\x8b\xf7\xc5\xab\x5f\xad\x41\x89\xb2\x44\x6f\x5f\x65\x29\x48\xd6\xa9\xc5\x5f\x07\x2c\x9a\x4d\x6f\xec\x28\x79\x8f\x3f\x57\x50\xfd\xf0\x7e\xd9\x4d\x22\xe8\x6a\x32\xa8\x52\x4c\x92\x43\x6a\xab\x19\x84\x5d\x2d\x69",
    b"\xbf\xac\x71\x56\x07\xa6\xf6\xb9\x9b\xfa\x62\x71\x59\x56\xd6\xe1\x9c\xbe\x3a\x21\x90\x43\xb3\xa3\x34\x9a\xe9\x97\x68\xfb\xa0\x7e\xaf\xd0\xe1\x46\xe3\x92\x04\x98\x8b\x67\x19\x6b\x3d\xcc\x9f\xfa\xc1\xe1\x69\xa0\x62\xa7\x02\x35\x10\xf2\x53\xd2\xc4\x67\xf8\x56\xd3\x29\xab",
    b"\xf7\x9f\xdf\x21\xe0\x5d\x4d\x46\x5a\xa0\x20\xfe\x2b\x96\x54\x65\xbe\xcd\x85\x34\x3c\xca\xa5\xdc\xcb\x97\x5d\x54\x0f\x54\xfd\xad\xa0\x59\xe3\x34\xc7\x61\x5f\xe7\x34\xb1\x29\x7d\x89\xf1\xa6\xe1\x76\x02\x36\x20\x52\xd7\xaf\xc6\x2b\xee\xeb\x1b\x65\xb8\x7f\x03\x05\x08\xa3\xee",
    b"\x69\x49\x44\x01\x1d\xa8\x75\x0e\x58\xd0\x37\x2e\x2a\xd9\x06\x54\xb3\x61\x06\xf0\xc0\xe7\x9e\x40\x76\xa0\xcb\x39\x3d\x6c\xc7\x7c\x41\xbb\xdd\xa7\x27\xb6\x9f\xb4\xf4\xcb\xe4\xfe\x0d\xaf\x3d\x5a\xce\x28\xc7\x5e\x43\x39\x51\xe5\x2e\x7b\x59\x25\x1f\xca\x42\xfa\x23\xaa\xf3\x58\x33",
];

/// https://github.com/kennytm/qrcode-rust/blob/master/src/ec.rs @ line 438
///
/// The number of blocks in a group and the number of data codewords per block.
/// For a given version: (blocks in group 1, codewords in group 1, blocks in group 2, codewords in group 2)
static DATA_BYTES_PER_BLOCK: [[(usize, usize, usize, usize); 4]; 40] = [
    // Normal versions.
    [(19, 1, 0, 0), (16, 1, 0, 0), (13, 1, 0, 0), (9, 1, 0, 0)], // 1
    [(34, 1, 0, 0), (28, 1, 0, 0), (22, 1, 0, 0), (16, 1, 0, 0)], // 2
    [(55, 1, 0, 0), (44, 1, 0, 0), (17, 2, 0, 0), (13, 2, 0, 0)], // 3
    [(80, 1, 0, 0), (32, 2, 0, 0), (24, 2, 0, 0), (9, 4, 0, 0)], // 4
    [
        (108, 1, 0, 0),
        (43, 2, 0, 0),
        (15, 2, 16, 2),
        (11, 2, 12, 2),
    ], // 5
    [(68, 2, 0, 0), (27, 4, 0, 0), (19, 4, 0, 0), (15, 4, 0, 0)], // 6
    [(78, 2, 0, 0), (31, 4, 0, 0), (14, 2, 15, 4), (13, 4, 14, 1)], // 7
    [
        (97, 2, 0, 0),
        (38, 2, 39, 2),
        (18, 4, 19, 2),
        (14, 4, 15, 2),
    ], // 8
    [
        (116, 2, 0, 0),
        (36, 3, 37, 2),
        (16, 4, 17, 4),
        (12, 4, 13, 4),
    ], // 9
    [
        (68, 2, 69, 2),
        (43, 4, 44, 1),
        (19, 6, 20, 2),
        (15, 6, 16, 2),
    ], // 10
    [
        (81, 4, 0, 0),
        (50, 1, 51, 4),
        (22, 4, 23, 4),
        (12, 3, 13, 8),
    ], // 11
    [
        (92, 2, 93, 2),
        (36, 6, 37, 2),
        (20, 4, 21, 6),
        (14, 7, 15, 4),
    ], // 12
    [
        (107, 4, 0, 0),
        (37, 8, 38, 1),
        (20, 8, 21, 4),
        (11, 12, 12, 4),
    ], // 13
    [
        (115, 3, 116, 1),
        (40, 4, 41, 5),
        (16, 11, 17, 5),
        (12, 11, 13, 5),
    ], // 14
    [
        (87, 5, 88, 1),
        (41, 5, 42, 5),
        (24, 5, 25, 7),
        (12, 11, 13, 7),
    ], // 15
    [
        (98, 5, 99, 1),
        (45, 7, 46, 3),
        (19, 15, 20, 2),
        (15, 3, 16, 13),
    ], // 16
    [
        (107, 1, 108, 5),
        (46, 10, 47, 1),
        (22, 1, 23, 15),
        (14, 2, 15, 17),
    ], // 17
    [
        (120, 5, 121, 1),
        (43, 9, 44, 4),
        (22, 17, 23, 1),
        (14, 2, 15, 19),
    ], // 18
    [
        (113, 3, 114, 4),
        (44, 3, 45, 11),
        (21, 17, 22, 4),
        (13, 9, 14, 16),
    ], // 19
    [
        (107, 3, 108, 5),
        (41, 3, 42, 13),
        (24, 15, 25, 5),
        (15, 15, 16, 10),
    ], // 20
    [
        (116, 4, 117, 4),
        (42, 17, 0, 0),
        (22, 17, 23, 6),
        (16, 19, 17, 6),
    ], // 21
    [
        (111, 2, 112, 7),
        (46, 17, 0, 0),
        (24, 7, 25, 16),
        (13, 34, 0, 0),
    ], // 22
    [
        (121, 4, 122, 5),
        (47, 4, 48, 14),
        (24, 11, 25, 14),
        (15, 16, 16, 14),
    ], // 23
    [
        (117, 6, 118, 4),
        (45, 6, 46, 14),
        (24, 11, 25, 16),
        (16, 30, 17, 2),
    ], // 24
    [
        (106, 8, 107, 4),
        (47, 8, 48, 13),
        (24, 7, 25, 22),
        (15, 22, 16, 13),
    ], // 25
    [
        (114, 10, 115, 2),
        (46, 19, 47, 4),
        (22, 28, 23, 6),
        (16, 33, 17, 4),
    ], // 26
    [
        (122, 8, 123, 4),
        (45, 22, 46, 3),
        (23, 8, 24, 26),
        (15, 12, 16, 28),
    ], // 27
    [
        (117, 3, 118, 10),
        (45, 3, 46, 23),
        (24, 4, 25, 31),
        (15, 11, 16, 31),
    ], // 28
    [
        (116, 7, 117, 7),
        (45, 21, 46, 7),
        (23, 1, 24, 37),
        (15, 19, 16, 26),
    ], // 29
    [
        (115, 5, 116, 10),
        (47, 19, 48, 10),
        (24, 15, 25, 25),
        (15, 23, 16, 25),
    ], // 30
    [
        (115, 13, 116, 3),
        (46, 2, 47, 29),
        (24, 42, 25, 1),
        (15, 23, 16, 28),
    ], // 31
    [
        (115, 17, 0, 0),
        (46, 10, 47, 23),
        (24, 10, 25, 35),
        (15, 19, 16, 35),
    ], // 32
    [
        (115, 17, 116, 1),
        (46, 14, 47, 21),
        (24, 29, 25, 19),
        (15, 11, 16, 46),
    ], // 33
    [
        (115, 13, 116, 6),
        (46, 14, 47, 23),
        (24, 44, 25, 7),
        (16, 59, 17, 1),
    ], // 34
    [
        (121, 12, 122, 7),
        (47, 12, 48, 26),
        (24, 39, 25, 14),
        (15, 22, 16, 41),
    ], // 35
    [
        (121, 6, 122, 14),
        (47, 6, 48, 34),
        (24, 46, 25, 10),
        (15, 2, 16, 64),
    ], // 36
    [
        (122, 17, 123, 4),
        (46, 29, 47, 14),
        (24, 49, 25, 10),
        (15, 24, 16, 46),
    ], // 37
    [
        (122, 4, 123, 18),
        (46, 13, 47, 32),
        (24, 48, 25, 14),
        (15, 42, 16, 32),
    ], // 38
    [
        (117, 20, 118, 4),
        (47, 40, 48, 7),
        (24, 43, 25, 22),
        (15, 10, 16, 67),
    ], // 39
    [
        (118, 19, 119, 6),
        (47, 18, 48, 31),
        (24, 34, 25, 34),
        (15, 20, 16, 61),
    ], // 40
];
