// Decompresses zlib streams (RFC 1950) of DEFLATE data (RFC 1951), which is
// how Aseprite stores its cels' pixels. It decodes Huffman codes a bit at a
// time, after zlib's puff.c, which is plenty for files the size of a sprite.

const MAX_BITS: usize = 15;

// The base length of each length symbol from 257, and its extra bits.
const LENGTH_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LENGTH_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
// The base distance of each distance symbol, and its extra bits.
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
// The order that a dynamic block lists its code length code's lengths in.
const CODE_LENGTH_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

// The decompressed bytes of a zlib stream, once its Adler-32 checksum
// matches them. Fails on anything that isn't a whole, valid stream, like
// one cut short, or one that needs a preset dictionary.
pub fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let [method, flags, ..] = *data else {
        return Err("The zlib stream has no header.".to_string());
    };
    if method & 0x0F != 8 || method >> 4 > 7 {
        return Err(format!("The zlib stream's compression method is {method:#04X}, not DEFLATE."));
    }
    if (u16::from(method) << 8 | u16::from(flags)) % 31 != 0 {
        return Err("The zlib stream's header check fails.".to_string());
    }
    if flags & 0x20 != 0 {
        return Err("The zlib stream needs a preset dictionary.".to_string());
    }
    let mut reader = BitReader { data, at: 2, bit: 0 };
    let output = inflate(&mut reader)?;
    let at = reader.at + usize::from(reader.bit > 0);
    let Some(checksum) = data.get(at..at + 4) else {
        return Err("The zlib stream ends before its checksum.".to_string());
    };
    let expected = u32::from_be_bytes([checksum[0], checksum[1], checksum[2], checksum[3]]);
    if adler32(&output) != expected {
        return Err("The zlib stream's checksum doesn't match its data.".to_string());
    }
    Ok(output)
}

// The Adler-32 checksum that ends a zlib stream.
pub fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    b << 16 | a
}

// Reads bits from the lowest of each byte up, as DEFLATE packs them.
struct BitReader<'a> {
    data: &'a [u8],
    // The byte that the next bit is in, and the bit in it.
    at: usize,
    bit: u32,
}

impl BitReader<'_> {
    fn bit(&mut self) -> Result<u32, String> {
        let byte = *self.data.get(self.at).ok_or("The DEFLATE data ends early.")?;
        let value = u32::from(byte >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.at += 1;
        }
        Ok(value)
    }

    // `count` bits, the first one lowest.
    fn bits(&mut self, count: u32) -> Result<u32, String> {
        let mut value = 0;
        for index in 0..count {
            value |= self.bit()? << index;
        }
        Ok(value)
    }

    // Skips to the start of the next byte, as a stored block does.
    fn align(&mut self) {
        if self.bit > 0 {
            self.bit = 0;
            self.at += 1;
        }
    }
}

// A canonical Huffman code: how many codes each length has, and the symbols
// in the order of their codes.
struct Huffman {
    counts: [u16; MAX_BITS + 1],
    symbols: Vec<u16>,
}

impl Huffman {
    // The code with these lengths for symbols 0 on, where 0 means a symbol
    // has no code. Fails if the lengths ask for more codes than there are.
    fn new(lengths: &[u8]) -> Result<Huffman, String> {
        let mut counts = [0u16; MAX_BITS + 1];
        for &length in lengths {
            counts[usize::from(length)] += 1;
        }
        counts[0] = 0;
        let mut left = 1i32;
        for &count in &counts[1..] {
            left = (left << 1) - i32::from(count);
            if left < 0 {
                return Err("A Huffman code in the DEFLATE data has too many codes.".to_string());
            }
        }
        let mut offsets = [0u16; MAX_BITS + 2];
        for length in 1..=MAX_BITS {
            offsets[length + 1] = offsets[length] + counts[length];
        }
        let mut symbols = vec![0u16; usize::from(offsets[MAX_BITS + 1])];
        for (symbol, &length) in lengths.iter().enumerate() {
            if length != 0 {
                let offset = &mut offsets[usize::from(length)];
                symbols[usize::from(*offset)] = symbol as u16;
                *offset += 1;
            }
        }
        Ok(Huffman { counts, symbols })
    }

    fn decode(&self, reader: &mut BitReader) -> Result<u16, String> {
        // The code so far, the first code of its length, and the index of
        // that code's symbol.
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for length in 1..=MAX_BITS {
            code |= reader.bit()? as i32;
            let count = i32::from(self.counts[length]);
            if code - first < count {
                return Ok(self.symbols[(index + code - first) as usize]);
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        Err("The DEFLATE data has a code that its Huffman code doesn't.".to_string())
    }
}

fn inflate(reader: &mut BitReader) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    loop {
        let last = reader.bit()? == 1;
        match reader.bits(2)? {
            0 => stored(reader, &mut output)?,
            1 => {
                let (lengths, distances) = fixed_codes()?;
                codes(reader, &mut output, &lengths, &distances)?;
            }
            2 => {
                let (lengths, distances) = dynamic_codes(reader)?;
                codes(reader, &mut output, &lengths, &distances)?;
            }
            _ => return Err("The DEFLATE data has a block of type 3, which doesn't exist.".to_string()),
        }
        if last {
            return Ok(output);
        }
    }
}

fn stored(reader: &mut BitReader, output: &mut Vec<u8>) -> Result<(), String> {
    reader.align();
    let header = reader.data.get(reader.at..reader.at + 4).ok_or("The DEFLATE data ends in a stored block's header.")?;
    let length = u16::from_le_bytes([header[0], header[1]]);
    let complement = u16::from_le_bytes([header[2], header[3]]);
    if length != !complement {
        return Err("A stored block's length doesn't match its complement.".to_string());
    }
    let start = reader.at + 4;
    let bytes = reader.data.get(start..start + usize::from(length)).ok_or("The DEFLATE data ends in a stored block.")?;
    output.extend_from_slice(bytes);
    reader.at = start + usize::from(length);
    Ok(())
}

fn fixed_codes() -> Result<(Huffman, Huffman), String> {
    let mut lengths = [0u8; 288];
    lengths[..144].fill(8);
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths[280..].fill(8);
    Ok((Huffman::new(&lengths)?, Huffman::new(&[5; 30])?))
}

fn dynamic_codes(reader: &mut BitReader) -> Result<(Huffman, Huffman), String> {
    let length_count = reader.bits(5)? as usize + 257;
    let distance_count = reader.bits(5)? as usize + 1;
    let code_length_count = reader.bits(4)? as usize + 4;
    if length_count > 286 || distance_count > 30 {
        return Err("A dynamic block has more codes than DEFLATE has symbols.".to_string());
    }
    let mut code_lengths = [0u8; 19];
    for &symbol in &CODE_LENGTH_ORDER[..code_length_count] {
        code_lengths[symbol] = reader.bits(3)? as u8;
    }
    let code_length_code = Huffman::new(&code_lengths)?;

    let mut lengths = vec![0u8; length_count + distance_count];
    let mut index = 0;
    while index < lengths.len() {
        let symbol = code_length_code.decode(reader)?;
        if symbol < 16 {
            lengths[index] = symbol as u8;
            index += 1;
            continue;
        }
        let (value, repeat) = match symbol {
            16 => {
                let previous = *lengths[..index].last().ok_or("A dynamic block repeats a length before the first one.")?;
                (previous, 3 + reader.bits(2)? as usize)
            }
            17 => (0, 3 + reader.bits(3)? as usize),
            _ => (0, 11 + reader.bits(7)? as usize),
        };
        if index + repeat > lengths.len() {
            return Err("A dynamic block repeats lengths past its last code.".to_string());
        }
        lengths[index..index + repeat].fill(value);
        index += repeat;
    }
    if lengths[256] == 0 {
        return Err("A dynamic block has no code for the end of the block.".to_string());
    }
    Ok((Huffman::new(&lengths[..length_count])?, Huffman::new(&lengths[length_count..])?))
}

// A block's literals and copies, up to its end.
fn codes(reader: &mut BitReader, output: &mut Vec<u8>, lengths: &Huffman, distances: &Huffman) -> Result<(), String> {
    loop {
        let symbol = usize::from(lengths.decode(reader)?);
        if symbol < 256 {
            output.push(symbol as u8);
            continue;
        }
        if symbol == 256 {
            return Ok(());
        }
        let symbol = symbol - 257;
        if symbol >= LENGTH_BASE.len() {
            return Err(format!("The DEFLATE data has the length symbol {}, which doesn't exist.", symbol + 257));
        }
        let length = usize::from(LENGTH_BASE[symbol]) + reader.bits(u32::from(LENGTH_EXTRA[symbol]))? as usize;
        let symbol = usize::from(distances.decode(reader)?);
        if symbol >= DISTANCE_BASE.len() {
            return Err(format!("The DEFLATE data has the distance symbol {symbol}, which doesn't exist."));
        }
        let distance = usize::from(DISTANCE_BASE[symbol]) + reader.bits(u32::from(DISTANCE_EXTRA[symbol]))? as usize;
        if distance > output.len() {
            return Err("The DEFLATE data copies from before its start.".to_string());
        }
        // One byte at a time, as a copy may overlap what it writes.
        let start = output.len() - distance;
        for offset in 0..length {
            output.push(output[start + offset]);
        }
    }
}
