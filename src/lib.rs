#![no_std]


//! Crate to create simple C-style bitfields. Can be used in `#![no_std]` environments.
//! 
//! Properties of such bitfields:
//!  * they have the exact same memory layout and size as the underlying primitive type;
//!  * their size is checked at compile-time, so it's not possible to add a field that won't fit into the underlying type;
//!  * their fields can be accessed by name which aids readability;
//!  * each field has the same set of functions (`get`, `set`, `offset` and more);
//!  * each field has its own distinct type;
//!  * it's possible to skip (and not name) any number of bits
//!
//! The [bitfield] macro was inspired by [https://guiand.xyz/blog-posts/bitfields.html](https://guiand.xyz/blog-posts/bitfields.html).
//! 
//! Full example:
//! ```
//! #[macro_use] extern crate simple_bitfield;
//! use simple_bitfield::{bitfield, Bitfield, BitfieldField};
//! 
//! bitfield! {
//!     // Bitfield with underlying type `u32`
//!     struct MyBitfield<u32> {
//!         field1: 3, // First field (least significant) of size 3 bits
//!         field2: 9,
//!         _: 6,      // Fields named `_` are skipped (offsets are preserved)
//!         field3: 1  // Last bit (closest to the highest bit of `u32`)
//!     }
//! 
//!     // Multiple bitfields can be defined
//!     // within one macro invocation
//!    struct AnotherBitfield<u8> {
//!         _: 7,
//!        highest_bit: 1
//!    }
//! }
//!
//! fn main() {
//!    // Create bitfield object
//!    let mut a_bitfield = MyBitfield::new(12345);
//!
//!    // Get the field's value (of underlying type)
//!    let field3: u32 = a_bitfield.field3.get();
//!
//!    println!(
//!        "{:#b} => {:#b}, {:#b}, {:#b}",
//!        u32::from(a_bitfield), // Convert bitfield to underlying type
//!        field3,
//!        a_bitfield.field2.get(),
//!        a_bitfield.field1.get()
//!    );
//!
//!    // Update just that field
//!    a_bitfield.field1.set(0);
//!
//!    println!("{:#b}", u32::from(a_bitfield));
//!
//!    // The type can be inferred, of course
//!    let another_one: AnotherBitfield::AnotherBitfield = AnotherBitfield::new(184);
//!    
//!    // Fields cannot be moved!
//!    // let another_one_highest = another_one.highest_bit;
//!
//!    // Each field has its own type
//!    let another_one_highest: &AnotherBitfield::highest_bit = &another_one.highest_bit;
//!
//!    // The underlying type can be retrieved via
//!    // `<AnotherBitfield::AnotherBitfield as Bitfield>::BaseType`
//!    println!(
//!        "{:#b} => {:#b}",
//!        <AnotherBitfield::AnotherBitfield as Bitfield>::BaseType::from(another_one),
//!        another_one_highest.get()
//!    )
//! }
//! ```
//!
//! The [TestBitfield] module is only present in the documentation and shows how a bitfield is structured internally.

pub use static_assertions::const_assert;

/// Extracts bits from `$lo` (inclusive) to `$hi` (exclusive) from integer.
///
/// Example:
/// ```
/// use simple_bitfield::bits;
///
/// assert_eq!(bits!(0b110_1001, 0, 5), 0b1001);
/// ```
#[macro_export]
macro_rules! bits {
    ($var:expr, $lo:expr, $hi:expr) => {
        ($var >> $lo) & ((1 << ($hi - $lo)) - 1)
    }
}

pub trait Bitfield: Copy + Clone
{
    type BaseType: Into<Self>;
    const MAX_BITS: u8 = 8 * core::mem::size_of::<Self::BaseType>() as u8;
}

pub trait BitfieldField<ParentBitfield>
    where ParentBitfield: Bitfield
{
    /// The field's size _in bits_.
    ///
    /// Used internally to calculate the fields' offsets and check if the bitfield's size is within the underlying type's size.
    const SIZE: u8;

    /// The field's offset from the beginning of the bitfield. Used internally to check bitfield's size.
    ///
    /// The first field's offset is 0, the second field's offset is `previous_field::SIZE` and so on.
    const OFFSET: u8;

    /// `true` if the size fits the parent type. Used fo compile-time size checking.
    const VALID: bool = Self::OFFSET + Self::SIZE <= ParentBitfield::MAX_BITS;

    /// Returns the field's size, [Self::SIZE].
    ///
    /// Example use:
    /// ```
    /// # #[macro_use] extern crate simple_bitfield;
    /// use simple_bitfield::{bitfield, BitfieldField};
    ///
    /// bitfield!{
    ///     struct TestBitfield<u8> {
    ///         some_field: 5
    ///     }
    /// }
    ///
    /// # fn main() {
    /// let the_bitfield = TestBitfield::new(5);
    ///
    /// assert_eq!(the_bitfield.some_field.size(), 5);
    /// # }
    /// ```
    fn size(&self) -> u8 { Self::SIZE }

    /// Returns the field's offset, [Self::OFFSET].
    fn offset(&self) -> u8 { Self::OFFSET }

    /// Returns the field's value.
    ///
    /// Example:
    /// ```
    /// # #[macro_use] extern crate simple_bitfield;
    /// use simple_bitfield::{bitfield, BitfieldField};
    ///
    /// bitfield!{
    ///     struct TestBitfield<u8> {
    ///         some_field: 5,
    ///         another_field: 2
    ///     }
    /// }
    ///
    /// # fn main() {
    /// let the_bitfield = TestBitfield::new(0b1_11_10011);
    ///
    /// assert_eq!(the_bitfield.some_field.get(), 0b10011);
    /// assert_eq!(the_bitfield.another_field.get(), 0b11);
    /// # }
    /// ```
    fn get(&self) -> ParentBitfield::BaseType;

    /// Return `true` if field is not zero, `false` otherwise
    fn is_set(&self) -> bool;

    /// Returns the mask that can be used to extract the first `Self::SIZE` bits of an integer.
    ///
    /// Example:
    /// ```
    /// # #[macro_use] extern crate simple_bitfield;
    /// use simple_bitfield::{bitfield, BitfieldField};
    ///
    /// bitfield! {
    ///     struct TestBitfield<u16> {
    ///         some_field: 2
    ///     }
    /// }
    ///
    /// # fn main() {
    /// let value = 0b01110;
    /// let the_bitfield = TestBitfield::new(value);
    ///
    /// assert_eq!(the_bitfield.some_field.mask(), 0b11);
    ///
    /// assert_eq!(
    ///     the_bitfield.some_field.mask(),
    ///     (1 << the_bitfield.some_field.size()) - 1
    /// );
    /// # }
    /// ```
    fn mask(&self) -> ParentBitfield::BaseType;

    /// Sets the field's value.
    ///
    /// If the passed value won't fit into the field, its lowest `field::SIZE` bits are used:
    /// ```
    /// # #[macro_use] extern crate simple_bitfield;
    /// use simple_bitfield::{BitfieldField};
    ///
    /// bitfield! {
    ///     struct TestBitfield<u16> {
    ///         some_field: 2
    ///     }
    /// }
    ///
    /// # fn main() {
    /// let value = 0b01110;
    /// let mut the_bitfield = TestBitfield::new(value);
    ///
    /// let new_value = 0b11000;
    /// the_bitfield.some_field.set(new_value);
    ///
    /// assert_eq!(
    ///     the_bitfield.some_field.get(),
    ///     new_value & the_bitfield.some_field.mask()
    /// );
    /// # }
    /// ```
    fn set(&mut self, value: ParentBitfield::BaseType);
}

/// Creates bitfields.
///
/// Adapted from [https://guiand.xyz/blog-posts/bitfields.html](https://guiand.xyz/blog-posts/bitfields.html)
///
/// Example:
/// ```
/// #[macro_use] extern crate simple_bitfield;
/// use core::mem::{size_of, size_of_val};
/// use simple_bitfield::{
///     Bitfield, // For access to the underlying type
///     BitfieldField // For field access
/// };
///
/// bitfield!{
///     struct BitfieldName<u8> {
///         first_two_bits: 2,
///         three_more_bits: 3
///     }
///
///     struct AnotherBitfield<u32> {
///         _: 31, // Skip first 31 bits
///         last_bit: 1
///     }
/// }
///
/// # pub fn main() {
/// let value: u8 = 0b111_011_10;
/// let my_bitfield: BitfieldName::BitfieldName = BitfieldName::new(value);
/// let another_bitfield = AnotherBitfield::new(value.into());
///
/// assert_eq!(
///     size_of_val(&my_bitfield),
///     size_of::<u8>()
/// );
/// assert_eq!(
///     size_of_val(&my_bitfield),
///     size_of::<<BitfieldName::BitfieldName as Bitfield>::BaseType>()
/// );
///
/// assert_eq!(my_bitfield.first_two_bits.size(), 2);
/// assert_eq!(my_bitfield.three_more_bits.size(), 3);
///
/// assert_eq!(my_bitfield.first_two_bits.get(), value & 0b11);
/// assert_eq!(
///     my_bitfield.three_more_bits.get(),
///     (value >> my_bitfield.first_two_bits.size()) & 0b111
/// );
///
/// assert_eq!(another_bitfield.last_bit.get(), 0);
/// # }
/// ```
///
/// The bitfield `BitfieldName` is actually a module. The type that holds the data is `BitfieldName::BitfieldName`,
/// which is unique for each bitfield. Each field is a zero-size struct that cannot be instantiated separately from the bitfield.
/// The memory representation of the bitfield is exactly the same as that of the underlying type
#[macro_export]
macro_rules! bitfield {
    ($($visibility:vis struct $struct_name:ident < $big_type:ty > { $($field:tt : $size:literal),* })*) => {$(
        // Construct the whole module
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        $visibility mod $struct_name {
            //! This module represents a single bitfield.

            /// Struct with the actual data.
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct $struct_name($big_type);
            impl $crate::Bitfield for $struct_name {
                type BaseType = $big_type;
            }

            impl From<<$struct_name as $crate::Bitfield>::BaseType> for $struct_name
            {
                fn from(val: <$struct_name as $crate::Bitfield>::BaseType) -> Self {
                    Self(val)
                }
            }

            impl From<$struct_name> for <$struct_name as $crate::Bitfield>::BaseType {
                fn from(val: $struct_name) -> Self {
                    val.0
                }
            }

            /// Creates a new bitfield
            pub const fn new(val: <$struct_name as $crate::Bitfield>::BaseType) -> $struct_name {
                // Can't use `val.into()` because `into` is not `const`.
                $struct_name(val)
            }

            /* Generate a zero-sized (!!) `struct` for each `$field`
            * and a zero-sized (!!) `struct Field` whose elements are objects of these structs.
            */
            bitfield!{
                impl
                $($field : $size),* end_marker // List of fields to process
    
                Fields, // Name of the struct that will hold the resulting fields
                $struct_name, // Name of the underlying bitfield struct that holds the actual data
                0, // Offset of the current bitfield
                processed // Empty (!) list of processed field names
            }

            $crate::const_assert!(Fields::VALID);

            /// Implement this so that accesses to fields of `$struct_name`
            /// actually access the zero-sized struct `Fields`
            impl core::ops::Deref for $struct_name {
                type Target = Fields;

                fn deref(&self) -> &Self::Target {
                    // We go through Deref here because Fields MUST NOT be moveable.
                    unsafe { &*(self as *const Self as *const Fields) } 
                }
            }

            impl core::ops::DerefMut for $struct_name {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    // We go through Deref here because Fields MUST NOT be moveable.
                    unsafe { &mut *(self as *mut Self as *mut Fields) } 
                }
            }
        }
    )*};

    (impl end_marker $struct_name:ident, $bitfield_type:ty, $curr_offset:expr, processed $(| $field_processed:ident)*) => {
        /// Struct whose fields' names' are those of the bitfield's fields.
        ///
        /// When accessing a field of a bitfield like `some_bitfield.a_field`, a reference to `some_bitfield` is created
        /// and `unsafe`ly treated as a reference to _this struct_.
        /// However, this should actually be OK because this struct can't be constructed since none of its fields can be constructed.
        ///
        /// This struct's size is zero:
        /// ```
        /// # #[macro_use] extern crate simple_bitfield;
        /// use core::mem::{size_of, size_of_val};
        /// use simple_bitfield::{
        ///     Bitfield, // For access to the underlying type
        ///     BitfieldField // For field access
        /// };
        ///
        /// bitfield!{
        ///     struct BitfieldName<u8> {
        ///         first_two_bits: 2,
        ///         three_more_bits: 3
        ///     }
        /// }
        ///
        /// # fn main() {
        /// let the_bitfield = BitfieldName::new(176);
        ///
        /// assert_eq!(size_of_val(&the_bitfield.first_two_bits), 0);
        /// assert_eq!(size_of_val(&the_bitfield.three_more_bits), 0);
        /// # }
        /// ```
        #[repr(C)]
        pub struct $struct_name {
            $(pub $field_processed: $field_processed),*
        }

        impl $struct_name {
            /// `true` if ALL fields are valid, `false` otherwise
            const VALID: bool = $(<$field_processed as $crate::BitfieldField<$bitfield_type>>::VALID &)* true;
        }
    };

    (impl _ : $size:literal $(, $other_field:tt : $other_size:literal)* end_marker $struct_name:ident, $bitfield_type:ty, $curr_offset:expr, processed $(| $field_processed:ident)*) => {
        // Skip field that's equal to `_`
        bitfield!{
            impl
            $($other_field : $other_size),* end_marker
            $struct_name, $bitfield_type,
            $curr_offset + $size,
            processed $(| $field_processed)*
        }
    };

    (impl $field:ident : $size:literal $(, $other_field:tt : $other_size:literal)* end_marker $struct_name:ident, $bitfield_type:ty, $curr_offset:expr, processed $(| $field_processed:ident)*) => {
        // Create one field

        /// The bitfield's field. Can't be constructed outside of a bitfield.
        ///
        /// It's actually a struct of size ZERO and implements `BitfieldField<UnderlyingBitfieldType>`, so that its value can be obtained with `get()` and changed with `set()`.
        ///
        /// This struct cannot be constructed explicitly:
        /// ```compile_fail
        /// # #[macro_use] extern crate simple_bitfield;
        /// bitfield!{
        ///     struct BitfieldName<u8> {
        ///         first_two_bits: 2,
        ///         three_more_bits: 3
        ///     }
        /// }
        ///
        /// # fn main() {
        /// let tried_to_construct_field = BitfieldName::first_two_bits(());
        /// # }
        /// ```
        #[allow(non_camel_case_types)]
        pub struct $field(());
        /*
         * `struct thing(())` is a "unit-valued tuple struct",
         * basically the same as `struct thing(<any type>)`,
         * which can be constucted like `thing(<value of type>)`,
         * but the constructor is invisible outside the module.
         *
         * https://stackoverflow.com/questions/50162597/what-are-the-differences-between-the-multiple-ways-to-create-zero-sized-structs
        */

        #[allow(dead_code)]
        impl $crate::BitfieldField<$bitfield_type> for $field {
            const OFFSET: u8 = $curr_offset;
            const SIZE: u8 = $size;

            #[inline]
            fn get(&self) -> <$bitfield_type as $crate::Bitfield>::BaseType {
                // &self points to value of type `$bitfield_type`
                const FIELD_LO: u8 = $field::OFFSET;
                const FIELD_HI: u8 = FIELD_LO + $field::SIZE;
                
                let bfptr = self as *const Self as *const $bitfield_type;
                let val = unsafe { (*bfptr).0 };

                bits!(val, FIELD_LO, FIELD_HI)
            }

            /// Set field value to the last `FIELD_SIZE` bits of `val`
            #[inline]
            fn set(&mut self, new: <$bitfield_type as $crate::Bitfield>::BaseType) {
                const FIELD_LO: u8 = $field::OFFSET;
                const FIELD_SIZE: u8 = $field::SIZE;
                const FIELD_HI: u8 = FIELD_LO + FIELD_SIZE;

                let bfptr = self as *mut Self as *mut $bitfield_type;
                let val = unsafe { &mut (*bfptr).0 };

                *val ^= bits!(*val, FIELD_LO, FIELD_HI) << FIELD_LO; // clear old value
                *val |= bits!(new, 0, FIELD_SIZE) << FIELD_LO // set new value
            }

            #[inline]
            fn is_set(&self) -> bool {
                self.get() != 0
            }

            #[inline]
            fn mask(&self) -> <$bitfield_type as $crate::Bitfield>::BaseType {
                (1 << Self::SIZE) - 1
            }
        }

        $crate::const_assert!(<$field as $crate::BitfieldField<$bitfield_type>>::VALID);

        // Process the next fields
        bitfield!{
            impl
            $($other_field : $other_size),* end_marker // Schedule the next fields
            $struct_name, $bitfield_type, // Pass along
            $curr_offset + $size, // INCREMENT the current offset!!
            processed $(| $field_processed)* | $field // Add the field name to processed fields
            /* The trick with field names being separated by pipes (`|`) like `| $field`
             * is needed because `$(| $field_processed)*` may be empty, but we apparently need SOME separator,
             * so the separator must be in front of the field name
             */
        }
    }
}


#[cfg(doc)]
bitfield! {
    pub struct TestBitfield<u32> {
        field_1: 2,
        _: 3,
        field_2: 5
    }
}


// Should be AFTER the macro definition
#[cfg(test)]
mod tests;