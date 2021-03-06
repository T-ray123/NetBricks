use crate::common::Result;
use crate::native::mbuf::MBuf;
use crate::native::zcsi;
use crate::packets::{buffer, Header, Packet};

/// Unit header
impl Header for () {}

/// The raw network packet
///
/// Simply a wrapper around the underlying buffer with packet semantic
#[derive(Debug)]
pub struct RawPacket {
    mbuf: *mut MBuf,
    owned: bool,
}

// Compare RawPackets. This probably isn't something you want to be doing a lot of at runtime.
impl PartialEq for RawPacket {
    fn eq(&self, other: &RawPacket) -> bool {
        unsafe {
            if (*self.mbuf).data_len() != (*other.mbuf).data_len() {
                false
            } else {
                let len = (*self.mbuf).data_len();
                let lhs_slice = &(*buffer::read_slice::<u8>(self.mbuf, 0, len).unwrap());
                let rhs_slice = &(*buffer::read_slice::<u8>(other.mbuf, 0, len).unwrap());
                lhs_slice == rhs_slice
            }
        }
    }
}

impl RawPacket {
    /// Creates a new packet by allocating a new buffer
    pub fn new() -> Result<Self> {
        unsafe {
            let mbuf = zcsi::mbuf_alloc();
            if mbuf.is_null() {
                Err(buffer::BufferError::FailAlloc.into())
            } else {
                Ok(RawPacket { mbuf, owned: true })
            }
        }
    }

    /// Creates a new packet and initialize the buffer with a byte array
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let packet = RawPacket::new()?;
        buffer::alloc(packet.mbuf, 0, data.len())?;
        buffer::write_slice(packet.mbuf, 0, data)?;
        Ok(packet)
    }

    /// Creates a new packet from a MBuf
    pub fn from_mbuf(mbuf: *mut MBuf) -> Self {
        RawPacket { mbuf, owned: false }
    }

    /// Returns the reference count of the underlying buffer
    #[inline]
    pub fn refcnt(&self) -> u16 {
        unsafe { (*self.mbuf).refcnt() }
    }

    /// Gives up the ownership of the underlying buffer
    ///
    /// This prevents freeing the `MBuf` when the variable goes out
    /// of scope.
    pub(crate) fn unown(&mut self) {
        self.owned = false;
    }
}

impl Packet for RawPacket {
    type Header = ();
    type Envelope = RawPacket;

    #[inline]
    fn envelope(&self) -> &Self::Envelope {
        self
    }

    #[inline]
    fn envelope_mut(&mut self) -> &mut Self::Envelope {
        self
    }

    #[doc(hidden)]
    #[inline]
    fn mbuf(&self) -> *mut MBuf {
        self.mbuf
    }

    #[inline]
    fn offset(&self) -> usize {
        0
    }

    #[doc(hidden)]
    #[inline]
    fn header(&self) -> &Self::Header {
        unreachable!("raw packet has no defined header!");
    }

    #[doc(hidden)]
    #[inline]
    fn header_mut(&mut self) -> &mut Self::Header {
        unreachable!("raw packet has no defined header!");
    }

    #[inline]
    fn header_len(&self) -> usize {
        0
    }

    #[doc(hidden)]
    #[inline]
    fn do_parse(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(envelope)
    }

    #[doc(hidden)]
    #[inline]
    fn do_push(envelope: Self::Envelope) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(envelope)
    }

    #[inline]
    fn remove(self) -> Result<Self::Envelope> {
        Ok(self)
    }

    #[inline]
    fn cascade(&mut self) {
        // noop
    }

    #[inline]
    fn deparse(self) -> Self::Envelope {
        self
    }

    #[inline]
    default fn reset(self) -> RawPacket {
        self
    }
}

// only free the underlying mbuf if it's created by the raw packet.
// otherwise if the mbuf is passed in externally on creation, then
// the external allocator is responsible for freeing the mbuf. for
// example, the receive operator bulk allocates mbufs, and then
// the send operator bulk frees them. raw packet will not attempt
// to free the mbuf on drop in that case.
impl Drop for RawPacket {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                zcsi::mbuf_free(self.mbuf);
            }
        }
    }
}

// because packet holds a raw pointer, by default, rust will deem
// the struct to be not sendable. explicitly implement the `Send`
// trait to ensure raw packets can go across thread boundaries.
unsafe impl Send for RawPacket {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packets::UDP_PACKET;
    use crate::testing::dpdk_test;

    #[dpdk_test]
    fn new_raw_packet() {
        assert!(RawPacket::new().is_ok());
    }

    #[dpdk_test]
    fn raw_packet_from_bytes() {
        assert!(RawPacket::from_bytes(&UDP_PACKET).is_ok());
    }
}
