use core::convert::TryInto;

use alloc::vec::Vec;
use crc::{Crc, CRC_16_EN_13757};

use crate::{
    ffa::FrameFormatA, ffb::FrameFormatB, frameformat::FrameFormat, mbusaddress::MBusAddress,
};

const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_EN_13757);

pub struct WMBusPacket {
    pub link_layer: LinkLayer,
    pub ext_link_layer: Option<ExtendedLinkLayer>,
    pub application_layer: ApplicationLayer,
}

pub struct LinkLayer {
    pub length: Option<u8>,
    pub control: u8,
    pub address: MBusAddress,
}

#[derive(PartialEq)]
pub enum ExtendedLinkLayer {
    Short {
        cc: u8,
        acc: u8,
    },
    Long {
        cc: u8,
        acc: u8,
        sn: u32,
        payload_crc: Option<u16>,
    },
    ShortDest {
        cc: u8,
        acc: u8,
        dest: MBusAddress,
    },
    LongDest {
        cc: u8,
        acc: u8,
        dest: MBusAddress,
        sn: u32,
        payload_crc: Option<u16>,
    },
}

impl ExtendedLinkLayer {
    pub fn parse(rest: &[u8]) -> Result<Option<ExtendedLinkLayer>, ()> {
        let ell = match rest[0] {
            0x8C => Some(ExtendedLinkLayer::Short {
                cc: rest[1],
                acc: rest[2],
            }),
            0x8D => Some(ExtendedLinkLayer::Long {
                cc: rest[1],
                acc: rest[2],
                sn: u32::from_le_bytes(rest[3..7].try_into().unwrap()),
                payload_crc: Some(u16::from_le_bytes(rest[7..9].try_into().unwrap())),
            }),
            0x8E => Some(ExtendedLinkLayer::ShortDest {
                cc: rest[1],
                acc: rest[2],
                dest: MBusAddress::parse(rest[3..11].try_into().unwrap())?,
            }),
            0x8F => Some(ExtendedLinkLayer::LongDest {
                cc: rest[1],
                acc: rest[2],
                dest: MBusAddress::parse(rest[3..11].try_into().unwrap())?,
                sn: u32::from_le_bytes(rest[11..15].try_into().unwrap()),
                payload_crc: Some(u16::from_le_bytes(rest[15..17].try_into().unwrap())),
            }),
            _ => None,
        };
        Ok(ell)
    }
    pub fn size(&self) -> usize {
        match *self {
            ExtendedLinkLayer::Short { .. } => 1 + 2,
            ExtendedLinkLayer::Long { .. } => 1 + 8,
            ExtendedLinkLayer::ShortDest { .. } => 1 + 10,
            ExtendedLinkLayer::LongDest { .. } => 1 + 16,
        }
    }
}

pub struct ApplicationLayer {
    pub ci: u8,
    pub data: Vec<u8>,
}

impl WMBusPacket {
    pub fn parse_ffa(frame_bytes: &[u8]) -> Result<WMBusPacket, ()> {
        Self::parse(FrameFormatA, frame_bytes)
    }

    pub fn parse_ffb(frame_bytes: &[u8]) -> Result<WMBusPacket, ()> {
        Self::parse(FrameFormatB, frame_bytes)
    }

    fn parse<FF: FrameFormat>(_frame_format: FF, frame_bytes: &[u8]) -> Result<WMBusPacket, ()> {
        // Verify CRC
        let blocks = FF::frame_block_iter(frame_bytes);
        let mut payload = Vec::with_capacity(frame_bytes.len());
        let mut digest = CRC.digest();
        for (index, block) in blocks.enumerate() {
            if FF::block_has_crc(index) {
                let block_payload = &block[..block.len() - 2];
                payload.extend_from_slice(block_payload);
                digest.update(block_payload);
                let actual_checksum = digest.finalize();

                // Verify checksum
                let expected = u16::from_be_bytes(block[block.len() - 2..].try_into().unwrap());
                if actual_checksum != expected {
                    return Err(());
                }

                digest = CRC.digest();
            } else {
                // Append the entire block to the digest
                payload.extend_from_slice(block);
                digest.update(block);
            }
        }

        let ll = LinkLayer {
            length: Some(payload[0]),
            control: payload[1],
            address: MBusAddress::parse(payload[2..10].try_into().unwrap())?,
        };

        let rest = &payload[10..];
        let ell = ExtendedLinkLayer::parse(rest)?;
        let ell_size = if let Some(ell) = &ell {
            ell.size()
        }
        else {
            0
        };

        let rest = &rest[ell_size..];
        let apl = ApplicationLayer {
            ci: rest[0],
            data: rest[1..].to_vec(),
        };

        Ok(WMBusPacket {
            link_layer: ll,
            ext_link_layer: ell,
            application_layer: apl,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use crate::mbusaddress::{DeviceType, ManufacturerCode};

    use super::*;

    #[test]
    pub fn can_parse_ffa() {
        let packet = WMBusPacket::parse(
            FrameFormatA,
            &[
                0x4E, 0x44, 0x2D, 0x2C, 0x98, 0x27, 0x04, 0x67, 0x30, 0x04, 0x91, 0x53, 0x7A, 0xA6,
                0x10, 0x40, 0x25, 0x6D, 0x3C, 0xA0, 0xF7, 0x2F, 0xF1, 0xEF, 0x06, 0x80, 0x6C, 0x50,
                0xA1, 0x04, 0x21, 0xCB, 0xD1, 0x32, 0xE3, 0xB1, 0xD0, 0x11, 0x6A, 0x05, 0x57, 0x69,
                0x6E, 0x0E, 0x37, 0xC2, 0xE9, 0xF0, 0x86, 0x36, 0xFE, 0x31, 0xF6, 0x8E, 0x6B, 0x4D,
                0xEE, 0x5E, 0x38, 0x53, 0x16, 0xC2, 0x16, 0xA9, 0x6E, 0x27, 0x7D, 0x48, 0xB1, 0x45,
                0x92, 0x72, 0x38, 0x61, 0x46, 0xF7, 0x8C, 0x77, 0x66, 0xD5, 0x19, 0xFC, 0x44, 0x49,
                0x99, 0x3A, 0xDA, 0x5A, 0xAD, 0x95, 0xA5,
            ],
        )
        .unwrap();
        assert_eq!(0x4E, packet.link_layer.length.unwrap());
        assert_eq!(
            ManufacturerCode::KAM,
            packet.link_layer.address.manufacturer_code().unwrap()
        );
        assert_eq!(67042798, packet.link_layer.address.serial_number.decode());
        assert_eq!(0x30, packet.link_layer.address.version);
        assert_eq!(
            DeviceType::Heat,
            packet.link_layer.address.device_type().unwrap()
        );
        assert!(packet.ext_link_layer.is_none());
        assert_eq!(0x7A, packet.application_layer.ci);
        assert_eq!(0xA6, packet.application_layer.data[0]);
        assert_eq!(0xAD, *packet.application_layer.data.last().unwrap());
    }

    #[test]
    pub fn can_parse_ffb() {
        let packet = WMBusPacket::parse(
            FrameFormatB,
            &[
                0x13, 0x44, 0x2D, 0x2C, 0x78, 0x56, 0x34, 0x12, 0x01, 0x32, 0xA0, 0x00, 0x01, 0x02,
                0x03, 0x04, 0x05, 0x06, 0xC3, 0xC0,
            ],
        )
        .unwrap();
        assert_eq!(0x13, packet.link_layer.length.unwrap());
        assert_eq!(
            ManufacturerCode::KAM,
            packet.link_layer.address.manufacturer_code().unwrap()
        );
        assert_eq!(12345678, packet.link_layer.address.serial_number.decode());
        assert_eq!(0x01, packet.link_layer.address.version);
        assert_eq!(
            DeviceType::Repeater,
            packet.link_layer.address.device_type().unwrap()
        );
        assert!(packet.ext_link_layer.is_none());
        assert_eq!(0xA0, packet.application_layer.ci);
        assert_eq!(0x00, packet.application_layer.data[0]);
        assert_eq!(0x06, *packet.application_layer.data.last().unwrap());
    }
}
