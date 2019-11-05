use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read};
pub const LEN: usize = 13;

#[derive(PartialEq, Debug)]
pub struct Header {
    pub magic: u32,
    pub opcode: u8,
    pub length: u32,
    pub check_sum: [u8; 4],
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let mut wtr = vec![];
        wtr.write_u32::<LittleEndian>(header.magic).unwrap();
        wtr.write_u8(header.opcode).unwrap();
        wtr.write_u32::<LittleEndian>(header.length).unwrap();
        wtr.extend(&header.check_sum);
        wtr
    }
}

impl From<Vec<u8>> for Header {
    fn from(vec: Vec<u8>) -> Self {
        let mut rdr = Cursor::new(vec);
        let magic = rdr.read_u32::<LittleEndian>().unwrap();
        let opcode = rdr.read_u8().unwrap();
        let length = rdr.read_u32::<LittleEndian>().unwrap();
        let mut check_sum: [u8; 4] = Default::default();
        Read::read_exact(&mut rdr, &mut check_sum).unwrap();

        Header {
            length,
            opcode,
            magic,
            check_sum,
        }
    }
}
