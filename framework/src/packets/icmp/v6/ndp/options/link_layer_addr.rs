use crate::common::Result;
use crate::native::mbuf::MBuf;
use crate::packets::{buffer, Fixed, MacAddr, ParseError};
use packets::icmp::v6::ndp::options::{NdpOption, SOURCE_LINK_LAYER_ADDR, TARGET_LINK_LAYER_ADDR};
use std::fmt;

/*  From https://tools.ietf.org/html/rfc4861#section-4.6.1
    Source/Target Link-layer Address

     0                   1                   2                   3
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    |     Type      |    Length     |    Link-Layer Address ...
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    Type            1 for Source Link-layer Address
                    2 for Target Link-layer Address

    Length          The length of the option (including the type and
                    length fields) in units of 8 octets.  For example,
                    the length for IEEE 802 addresses is 1.

    Link-Layer Address
                    The variable length link-layer address.

                    The content and format of this field (including
                    byte and bit ordering) is expected to be specified
                    in specific documents that describe how IPv6
                    operates over different link layers.
*/

#[derive(Debug)]
#[repr(C, packed)]
struct LinkLayerAddressFields {
    option_type: u8,
    length: u8,
    addr: MacAddr,
}

impl Default for LinkLayerAddressFields {
    fn default() -> LinkLayerAddressFields {
        LinkLayerAddressFields {
            option_type: 1,
            length: 1,
            addr: MacAddr::UNSPECIFIED,
        }
    }
}

impl NdpOption for LinkLayerAddress {
    #![allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    fn do_push(mbuf: *mut MBuf) -> Result<Self>
    where
        Self: Sized,
    {
        let offset = unsafe { (*mbuf).data_len() };

        buffer::alloc(mbuf, offset, LinkLayerAddressFields::size())?;

        let fields =
            buffer::write_item::<LinkLayerAddressFields>(mbuf, offset, &Default::default())?;
        Ok(LinkLayerAddress { fields, offset })
    }
}

/// Link-layer address option
pub struct LinkLayerAddress {
    fields: *mut LinkLayerAddressFields,
    offset: usize,
}

impl LinkLayerAddress {
    /// Parses the link-layer address option from the message buffer at offset
    #[inline]
    pub fn parse(mbuf: *mut MBuf, offset: usize) -> Result<LinkLayerAddress> {
        let fields = buffer::read_item::<LinkLayerAddressFields>(mbuf, offset)?;
        if unsafe { (*fields).length } != (LinkLayerAddressFields::size() as u8 / 8) {
            Err(ParseError::new("Invalid link-layer address option length").into())
        } else {
            Ok(LinkLayerAddress { fields, offset })
        }
    }

    /// Returns the message buffer offset for this option
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn fields(&self) -> &LinkLayerAddressFields {
        unsafe { &(*self.fields) }
    }

    #[inline]
    fn fields_mut(&mut self) -> &mut LinkLayerAddressFields {
        unsafe { &mut (*self.fields) }
    }

    #[inline]
    pub fn option_type(&self) -> u8 {
        self.fields().option_type
    }

    #[inline]
    pub fn length(&self) -> u8 {
        self.fields().length
    }

    #[inline]
    pub fn addr(&self) -> MacAddr {
        self.fields().addr
    }

    #[inline]
    pub fn set_addr(&mut self, addr: MacAddr) {
        self.fields_mut().addr = addr;
    }

    #[inline]
    pub fn set_option_type(&mut self, option_type: u8) {
        if option_type == SOURCE_LINK_LAYER_ADDR || option_type == TARGET_LINK_LAYER_ADDR {
            self.fields_mut().option_type = option_type
        } else {
            //TODO: determine what to do when option_type is set incorrectly
        }
    }
}

impl fmt::Display for LinkLayerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {}, length: {}, addr: {}",
            self.option_type(),
            self.length(),
            self.addr()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_link_layer_address() {
        assert_eq!(8, LinkLayerAddressFields::size());
    }
}
