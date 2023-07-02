fn index_to_byte_and_bit(index: u32) -> (usize, usize, usize) {
  let byte = (index as f64 / 8_f64).floor() as usize;
  let bit = index as usize - (byte * 8); // instead of using module index % 8
  let mask = 1 << bit;
  
  (byte, bit, mask)
}

pub fn is_set(index: u32, bitmap: &Vec<u8>) -> bool {
  let (byte, bit, _) = index_to_byte_and_bit(index);

  (bitmap[byte] >> bit) % 2 == 1
} 
