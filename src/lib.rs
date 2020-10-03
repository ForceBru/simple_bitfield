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
//! use simple_bitfield::{bitfield, Field};
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
//!    println!("{:#b}", another_one_highest.get())
//! }
//! ```
//!
//! The [TestBitfield] module is only present in the documentation and shows how a bitfield is structured internally.

use core::ops::{Shl, Shr, BitAnd, BitOrAssign, BitXorAssign};

pub use static_assertions::const_assert;

pub trait Bitfield {
    //! The trait that's implemented for all bitfields.
    //! Used mainly to access the bitfield's underlying type, [Self::BaseType].

    /// The bitfield's underlying type.
    type BaseType: Copy;
    
    /// The maximum number of bits that the bitfield can hold.
    /// Used for compile-time checking that no newly added field requires a [Self::BaseType] wider than this.
    const MAX_BITS: u8 = 8 * core::mem::size_of::<Self::BaseType>() as u8;
}

pub trait Field<B: Bitfield>
    where B::BaseType:
        Shl<u8, Output=B::BaseType> +
        Shr<u8, Output=B::BaseType> +
        BitAnd<Output=B::BaseType> +
        BitOrAssign + BitXorAssign
{
    //! The trait that's implemented for all fields of all bitfields.
    //! Allows the nice `my_bitfield.some_field.get()` syntax.

    /// The field's size _in bits_. Specified by the user.
    const SIZE: u8;

    /// The field's offset from the underlying value's least significant bit,
    /// _in bits_. Computed automatically.
    const OFFSET: u8;

    /// The field's mask that can be used to extract the last [Self::SIZE] bits from any `B::BaseType`.
    /// Computed automatically.
    ///
    /// Example usage:
    /// ```
    /// use simple_bitfield::{ bitfield, Field };
    ///
    /// bitfield! {
    ///     struct TestBitfield<u32> {
    ///         field1: 4
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let the_bf = TestBitfield::new(123);
    ///
    ///     assert_eq!(TestBitfield::field1::MASK, 0b1111);
    ///     assert_eq!(0b1011_1010 & TestBitfield::field1::MASK, 0b1010);
    /// }
    /// ```
    const MASK: B::BaseType;
    
    /// Returns `true` if the field is not equal to zero.
    fn is_set(&self) -> bool;
    
    /// `true` if the field is within the bitfield's bounds. Used for compile-time checking.
    const VALID: bool = Self::SIZE + Self::OFFSET <= B::MAX_BITS;
    
    /// Returns the size of a field _at runtime_, while [Self::SIZE] is used on the _type_ of the field at compile-time.
    fn size(&self) -> u8 { Self::SIZE }

    /// Returns the offset of a field _at runtime_, while [Self::OFFSET] is used on the _type_ of the field at compile-time.
    fn offset(&self) -> u8 { Self::OFFSET }

    /// Returns the mask of a field _at runtime_, while [Self::MASK] is used on the _type_ of the field at compile-time.
    fn mask(&self) -> B::BaseType { Self::MASK }
    
    /// Returns the current value of the field.
    ///
    /// Example:
    /// ```
    /// use simple_bitfield::{ bitfield, Field };
    ///
    /// bitfield! {
    ///     struct TestBitfield<u32> {
    ///         field1: 4
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let my_bitfield = TestBitfield::new(0b10_1111);
    ///
    ///     assert_eq!(my_bitfield.field1.get(), 0b1111);
    /// }
    /// ```
    fn get(&self) -> B::BaseType {
        let data_ptr: *const B::BaseType = self as *const Self as *const B::BaseType;
        
        (unsafe { *data_ptr } >> Self::OFFSET) & Self::MASK
    }
    
    /// Sets the value of a field. If the value is wider than the field,
    /// the value's lowest [Self::SIZE] bits will be used.
    ///
    /// Example:
    /// ```
    /// use simple_bitfield::{ bitfield, Field };
    ///
    /// bitfield! {
    ///     struct TestBitfield<u32> {
    ///         field1: 4
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let mut my_bitfield = TestBitfield::new(0b10_1111);  // Must be mutable
    ///
    ///     my_bitfield.field1.set(0b1_1100);
    ///     assert_eq!(my_bitfield.field1.get(), 0b1100);
    /// }
    /// ```
    fn set(&mut self, new_value: B::BaseType) {
        let data_ptr: *mut B::BaseType = self as *const Self as *mut B::BaseType;
        
        let old_value: B::BaseType = self.get() << Self::OFFSET;
        
        unsafe {
            *data_ptr ^= old_value;
            *data_ptr |= (new_value & Self::MASK) << Self::OFFSET
        }
    }
}


/// Creates bitfields.
///
/// Adapted from [https://guiand.xyz/blog-posts/bitfields.html](https://guiand.xyz/blog-posts/bitfields.html)
///
/// Example:
/// ```
/// use core::mem::{size_of, size_of_val};
/// use simple_bitfield::{
///     bitfield, Field // For field access
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
///     size_of::<u8>()
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
/// The memory representation of the bitfield is exactly the same as that of the underlying type.
#[macro_export]
macro_rules! bitfield {
    ($($visibility:vis struct $bitfield_name:ident < $big_type:ty > { $($field:tt : $size:literal),* })*) => {$(
        // Construct the whole module
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        $visibility mod $bitfield_name {
            //! This module represents a single bitfield.

            /// Struct with the actual data.
            #[repr(transparent)]
            #[derive(Copy, Clone)]
            pub struct $bitfield_name($big_type);
            impl $crate::Bitfield for $bitfield_name {
                type BaseType = $big_type;
            }

            impl From<$big_type> for $bitfield_name
            {
                fn from(val: $big_type) -> Self {
                    Self(val)
                }
            }

            impl From<$bitfield_name> for $big_type {
                fn from(val: $bitfield_name) -> Self {
                    val.0
                }
            }

            /// Creates a new bitfield
            pub const fn new(val: $big_type) -> $bitfield_name {
                // Can't use `val.into()` because `into` is not `const`.
                $bitfield_name(val)
            }

            /* Generate a zero-sized (!!) `struct` for each `$field`
            * and a zero-sized (!!) `struct Field` whose elements are objects of these structs.
            */
            $crate::bitfield!{
                impl
                $($field : $size),* end_marker // List of fields to process
    
                Fields, // Name of the struct that will hold the resulting fields
                $bitfield_name, // Name of the underlying bitfield struct that holds the actual data
                0, // Offset of the current bitfield
                processed // Empty (!) list of processed field names
            }

            $crate::const_assert!(Fields::VALID);

            /// Implement this so that accesses to fields of `$bitfield_name`
            /// actually access the zero-sized struct `Fields`
            impl core::ops::Deref for $bitfield_name {
                type Target = Fields;

                fn deref(&self) -> &Self::Target {
                    // We go through Deref here because Fields MUST NOT be moveable.
                    unsafe { &*(self as *const Self as *const Fields) } 
                }
            }

            impl core::ops::DerefMut for $bitfield_name {
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
        /// use simple_bitfield::bitfield;
        ///
        /// bitfield!{
        ///     struct BitfieldName<u8> {
        ///         first_two_bits: 2,
        ///         three_more_bits: 3
        ///     }
        /// }
        ///
        /// # fn main() {
        /// assert_eq!(core::mem::size_of::<BitfieldName::Fields>(), 0);
        /// # }
        /// ```
        #[repr(C)]
        pub struct $struct_name {
            $(pub $field_processed: $field_processed),*
        }

        impl $struct_name {
            /// `true` if ALL fields are valid, `false` otherwise
            const VALID: bool = $(<$field_processed as $crate::Field<$bitfield_type>>::VALID &)* true;
        }
    };

    (impl _ : $size:literal $(, $other_field:tt : $other_size:literal)* end_marker $struct_name:ident, $bitfield_type:ty, $curr_offset:expr, processed $(| $field_processed:ident)*) => {
        // Skip field that's equal to `_`
        $crate::bitfield!{
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
        /// It's actually a struct of size ZERO and implements `Field<UnderlyingBitfieldType>`, so that its value can be obtained with `get()` and changed with `set()`.
        ///
        /// This struct cannot be constructed explicitly:
        /// ```compile_fail
        /// use simple_bitfield::bitfield;
        ///
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
        impl $crate::Field<$bitfield_type> for $field {
            /// The field's size _in bits_.
            ///
            /// Used internally to calculate the fields' offsets and check if the bitfield's size is within the underlying type's size.
            const SIZE: u8 = $size;

            /// The field's offset from the beginning of the bitfield. Used internally to check bitfield's size.
            ///
            /// The first field's offset is 0, the second field's offset is `previous_field::SIZE` and so on.
            const OFFSET: u8 = $curr_offset;

            const MASK: <$bitfield_type as $crate::Bitfield>::BaseType = (1 << Self::SIZE) - 1;

            #[inline]
            fn is_set(&self) -> bool {
                self.get() != 0
            }
        }

        $crate::const_assert!(<$field as $crate::Field<$bitfield_type>>::VALID);

        // Process the next fields
        $crate::bitfield!{
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