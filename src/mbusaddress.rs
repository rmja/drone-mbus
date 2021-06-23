use core::{convert::TryInto, fmt::Display};

use crate::bcd::{self, BcdNumber};

#[derive(PartialEq)]
pub struct MBusAddress {
    pub manufacturer_code: u16,
    pub serial_number: BcdNumber<u32>,
    pub version: u8,
    pub device_type: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum ManufacturerCode {
    APT = 0x8614, // Apator
    DME = 0x11A5, // Diehl
    GAV = 0x1C36, // Carlo Gavazzi
    HYD = 0x2324, // Hydrometer
    KAM = 0x2C2D, // Kamstrup
    LUG = 0x32A7, // Landis+Gyr GmbH
    SON = 0x4DEE, // Sontex
    TCH = 0x5068, // Techem
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum DeviceType {
    Other = 0x00,
    Electricity = 0x02,
    Heat = 0x04,
    WarmWater = 0x06,
    Water = 0x07,
    Cooling = 0x0A,
    CoolingInlet = 0x0B,
    HeatInlet = 0x0C,
    HeatCooling = 0x0D,
    Unknown = 0x0F,
    ColdWater = 0x16,
    Repeater = 0x32,
}

type Identifier = [u8; 8];

enum FieldLayout {
    Default, // The default layout according to EN13757, i.e. Manufacturer, serial number, version, type
    Diehl, // The layout used by Diehl on some of its meters, i.e. Manufacturer, version, type, serial number
}

impl Display for MBusAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}:{:?}/{:?}/{:?}", self.manufacturer_code, self.serial_number, self.version, self.device_type)
    }
}

impl MBusAddress {
    pub fn new(
        manufacturer_code: ManufacturerCode,
        serial_number: u32,
        version: u8,
        device_type: DeviceType,
    ) -> Self {
        Self {
            manufacturer_code: manufacturer_code as u16,
            serial_number: BcdNumber::encode_u32(serial_number).unwrap(),
            version,
            device_type: device_type as u8,
        }
    }

    pub fn manufacturer_code(&self) -> Option<ManufacturerCode> {
        num_traits::FromPrimitive::from_u16(self.manufacturer_code)
    }

    pub fn device_type(&self) -> Option<DeviceType> {
        num_traits::FromPrimitive::from_u8(self.device_type)
    }

    pub fn parse(identifier: Identifier) -> Result<MBusAddress, ()> {
        let layout = Self::get_layout(identifier);
        match layout {
            FieldLayout::Default => {
                let serial_number_bcd = u32::from_le_bytes(identifier[2..6].try_into().unwrap());
                let serial_number = bcd::BcdNumber::new_u32(serial_number_bcd);
                serial_number.and_then(|serial_number| {
                    Ok(Self {
                        manufacturer_code: u16::from_le_bytes(identifier[0..2].try_into().unwrap()),
                        serial_number,
                        version: identifier[6],
                        device_type: identifier[7],
                    })
                })
            },
            FieldLayout::Diehl => {
                let serial_number_bcd = u32::from_le_bytes(identifier[4..8].try_into().unwrap());
                let serial_number = bcd::BcdNumber::new_u32(serial_number_bcd);
                serial_number.and_then(|serial_number| {
                    Ok(Self {
                        manufacturer_code: u16::from_le_bytes(identifier[0..2].try_into().unwrap()),
                        serial_number,
                        version: identifier[2],
                        device_type: identifier[3],
                    })
                })
            }
        }
    }

    fn get_layout(identifier: Identifier) -> FieldLayout {
        let manufacturer_code = u16::from_le_bytes(identifier[0..2].try_into().unwrap());
        if manufacturer_code == ManufacturerCode::HYD as u16 {
            // These indexes are not correct according to the standard, but are used by Diehl
            let version = identifier[2];
            let device_type = identifier[3];

            if (device_type == 0x04 || device_type == 0x0C) && version == 0x20 {
                // Sharky 775
                let serial_number_bcd = u32::from_le_bytes(identifier[4..8].try_into().unwrap());
                if let Ok(serial_number) = BcdNumber::new_u32(serial_number_bcd) {
                    let serial_number = serial_number.decode();
                    if (serial_number >= 44000000 && serial_number < 48350000)
                        || (serial_number >= 51200000 && serial_number < 51273000)
                    {
                        return FieldLayout::Diehl;
                    }
                }
            } else if device_type == 0x04
                && (version == 0x2A || version == 0x2B || version == 0x2E || version == 0x2F)
            {
                return FieldLayout::Diehl;
            } else if device_type == 0x06 && (version == 0x8B) {
                return FieldLayout::Diehl;
            } else if device_type == 0x0C
                && (version == 0x2E || version == 0x2F || version == 0x53)
            {
                return FieldLayout::Diehl;
            } else if device_type == 0x16 && version == 0x25 {
                return FieldLayout::Diehl;
            }
        }

        FieldLayout::Default
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn parse_default() {
        let address = MBusAddress::parse([0x2D,0x2C,0x78,0x56,0x34,0x12,0x01,0x32]).unwrap();
        assert_eq!(ManufacturerCode::KAM, address.manufacturer_code().unwrap());
        assert_eq!(12345678, address.serial_number.decode());
        assert_eq!(0x01, address.version);
        assert_eq!(DeviceType::Repeater, address.device_type().unwrap());
    }

    #[test]
    pub fn parse_hydromenter_default() {
        let address = MBusAddress::parse([0x24, 0x23, 0x95, 0x27, 0x80, 0x49, 0x20, 0x0C]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(49802795, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::HeatInlet, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x59, 0x91, 0x95, 0x49, 0x20, 0x04]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(49959159, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x06, 0x34, 0x27, 0x51, 0x20, 0x04]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(51273406, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x02, 0x84, 0x84, 0x51, 0x20, 0x04]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(51848402, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x83, 0x70, 0x29, 0x53, 0x20, 0x04]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(53297083, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());
    }

    #[test]
    pub fn parse_hydromenter_reversed() {
        let address = MBusAddress::parse([0x24, 0x23, 0x20, 0x04, 0x69, 0x02, 0x71, 0x47]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(47710269, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x20, 0x0C, 0x18, 0x59, 0x78, 0x47]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(47785918, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::HeatInlet, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x53, 0x0C, 0x95, 0x26, 0x86, 0x47]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(47862695, address.serial_number.decode());
        assert_eq!(0x53, address.version);
        assert_eq!(DeviceType::HeatInlet, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x20, 0x0C, 0x61, 0x04, 0x34, 0x48]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(48340461, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::HeatInlet, address.device_type().unwrap());

        let address = MBusAddress::parse([0x24, 0x23, 0x20, 0x04, 0x02, 0x29, 0x27, 0x51]).unwrap();
        assert_eq!(ManufacturerCode::HYD, address.manufacturer_code().unwrap());
        assert_eq!(51272902, address.serial_number.decode());
        assert_eq!(0x20, address.version);
        assert_eq!(DeviceType::Heat, address.device_type().unwrap());
    }
}