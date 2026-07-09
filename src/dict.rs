//! Typed object-dictionary entries and typed SDO access.
//!
//! Entries carry the object's CiA 301 data type as a type parameter, so the
//! width, signedness and transfer mode of every access are fixed at compile
//! time. Dictionaries are usually generated from a device's EDS file by the
//! `canopen-async-codegen` crate rather than written by hand:
//!
//! ```ignore
//! pub mod state {
//!     pub const STATE_OF_CHARGE: SdoEntry<u8> = SdoEntry::new(0x4241, 1, "State of charge");
//!     pub const CURRENT: SdoEntry<i32> = SdoEntry::new(0x4241, 4, "Current");
//! }
//!
//! let soc = client.sdo.read(state::STATE_OF_CHARGE).await?;
//! ```

use core::marker::PhantomData;

/// A fixed-size scalar transported little-endian in an SDO transfer
/// (CiA 301 basic data types).
pub trait SdoScalar: Copy {
    /// Transfer size in bytes (1..=8).
    const SIZE: usize;
    /// Decode from exactly `SIZE` little-endian bytes.
    fn from_le_bytes(bytes: &[u8]) -> Self;
    /// Encode into the first `SIZE` bytes of `out`.
    fn write_le_bytes(self, out: &mut [u8]);
}

macro_rules! impl_sdo_scalar {
    ($($ty:ty),*) => {$(
        impl SdoScalar for $ty {
            const SIZE: usize = core::mem::size_of::<$ty>();

            fn from_le_bytes(bytes: &[u8]) -> Self {
                <$ty>::from_le_bytes(bytes.try_into().unwrap())
            }

            fn write_le_bytes(self, out: &mut [u8]) {
                out[..Self::SIZE].copy_from_slice(&self.to_le_bytes());
            }
        }
    )*};
}

impl_sdo_scalar!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

impl SdoScalar for bool {
    const SIZE: usize = 1;

    fn from_le_bytes(bytes: &[u8]) -> Self {
        bytes[0] != 0
    }

    fn write_le_bytes(self, out: &mut [u8]) {
        out[0] = self as u8;
    }
}

/// Byte-oriented object kinds accessed via
/// [`SdoClient::read_bytes`](crate::SdoClient::read_bytes) /
/// [`write_bytes`](crate::SdoClient::write_bytes).
pub trait SdoBytes {}
impl SdoBytes for VisibleString {}
impl SdoBytes for OctetString {}

/// Marker type for VISIBLE_STRING objects; accessed via
/// [`SdoClient::read_bytes`](crate::SdoClient::read_bytes) /
/// [`write_bytes`](crate::SdoClient::write_bytes).
#[derive(Debug, Clone, Copy)]
pub struct VisibleString;

/// Marker type for OCTET_STRING objects; accessed like [`VisibleString`].
#[derive(Debug, Clone, Copy)]
pub struct OctetString;

/// Marker type for DOMAIN objects (bulk data); accessed via
/// [`SdoClient::read_domain`](crate::SdoClient::read_domain) /
/// [`write_domain`](crate::SdoClient::write_domain).
#[derive(Debug, Clone, Copy)]
pub struct Domain;

/// A typed object-dictionary entry: index, sub-index and the human-readable
/// name from the device description (used for logging only).
pub struct SdoEntry<T> {
    pub index: u16,
    pub sub: u8,
    name: &'static str,
    _ty: PhantomData<T>,
}

// Manual impls: derive would needlessly require T: Copy/Clone.
impl<T> Clone for SdoEntry<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for SdoEntry<T> {}

impl<T> SdoEntry<T> {
    pub const fn new(index: u16, sub: u8, name: &'static str) -> Self {
        Self {
            index,
            sub,
            name,
            _ty: PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> core::fmt::Debug for SdoEntry<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} ({:#06X}:{:#04X})", self.name, self.index, self.sub)
    }
}

#[cfg(feature = "defmt")]
impl<T> defmt::Format for SdoEntry<T> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{} ({:#06X}:{:#04X})", self.name, self.index, self.sub);
    }
}

/// A homogeneous ARRAY object: sub-index 0 holds the element count,
/// sub-indices `1..=len` hold the elements.
pub struct SdoArray<T> {
    pub index: u16,
    /// Number of elements (highest sub-index).
    pub len: u8,
    name: &'static str,
    _ty: PhantomData<T>,
}

impl<T> Clone for SdoArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for SdoArray<T> {}

impl<T> SdoArray<T> {
    pub const fn new(index: u16, len: u8, name: &'static str) -> Self {
        Self {
            index,
            len,
            name,
            _ty: PhantomData,
        }
    }

    /// The element at 1-based sub-index `sub` (`1..=len`).
    pub const fn entry(&self, sub: u8) -> SdoEntry<T> {
        debug_assert!(sub >= 1 && sub <= self.len);
        SdoEntry::new(self.index, sub, self.name)
    }

    /// Sub-index 0: the number of elements the device reports.
    pub const fn sub_count(&self) -> SdoEntry<u8> {
        SdoEntry::new(self.index, 0, self.name)
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> core::fmt::Debug for SdoArray<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}[{}] ({:#06X})", self.name, self.len, self.index)
    }
}

#[cfg(feature = "defmt")]
impl<T> defmt::Format for SdoArray<T> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}[{}] ({:#06X})", self.name, self.len, self.index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_roundtrips() {
        let mut buf = [0u8; 8];

        SdoScalar::write_le_bytes(-2i32, &mut buf);
        assert_eq!(<i32 as SdoScalar>::from_le_bytes(&buf[..4]), -2);
        assert_eq!(<i32 as SdoScalar>::SIZE, 4);

        SdoScalar::write_le_bytes(0xAABB_CCDD_EE11_2233u64, &mut buf);
        assert_eq!(<u64 as SdoScalar>::from_le_bytes(&buf[..8]), 0xAABB_CCDD_EE11_2233);

        true.write_le_bytes(&mut buf);
        assert!(<bool as SdoScalar>::from_le_bytes(&buf[..1]));
    }

    #[test]
    fn array_entries() {
        const CELLS: SdoArray<u16> = SdoArray::new(0x4242, 13, "Cells voltage");
        assert_eq!(CELLS.entry(3).index, 0x4242);
        assert_eq!(CELLS.entry(3).sub, 3);
        assert_eq!(CELLS.sub_count().sub, 0);
    }
}
