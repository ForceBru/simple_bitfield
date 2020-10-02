# `simple_bitfield` - yet another bitfield implementation for Rust

Easily create C-style bitfields that have _the same size as the underlying type_ and are `Copy + Clone` (requires the underlying type to be `Copy + Clone` as well):

```rust
use simple_bitfield::{bitfield, Bitfield, BitfieldField};

bitfield! {
    // Bitfield with underlying type `u32`
    struct MyBitfield<u32> {
        field1: 3, // First field (lest significant)
        field2: 9,
        _: 6,      // Fields named `_` are skipped (offsets are preserved)
        field3: 1  // Last bit (closest to the highest bit of `u32`)
    }

    // Multiple bitfields can be defined
    // within one macro invocation
    struct AnotherBitfield<u8> {
        _: 7,
        highest_bit: 1
    }
}

fn main() {
    // Create bitfield object
    let mut a_bitfield = MyBitfield::new(12345);

    // Get the field's value (of underlying type)
    let field3: u32 = a_bitfield.field3.get();

    println!(
        "{:#b} => {:#b}, {:#b}, {:#b}",
        u32::from(a_bitfield), // Convert bitfield to underlying type
        field3,
        a_bitfield.field2.get(),
        a_bitfield.field1.get()
    );

    // Update just that field
    a_bitfield.field1.set(0);

    println!("{:#b}", u32::from(a_bitfield));

    let another_one = AnotherBitfield::new(184);

    // The underlying type can be retrieved via
    // `<AnotherBitfield::__Bitfield as Bitfield>::BaseType`
    println!(
        "{:#b} => {:#b}",
        <AnotherBitfield::__Bitfield as Bitfield>::BaseType::from(another_one),
        another_one.highest_bit.get()
    )
}
```

# Syntax

Like in C:
```
(pub) struct BitfieldName<BaseType> {
    field_name: field_size,
    _: size_to_skip
}
```

# Credits

Initial idea was adapted from [https://guiand.xyz/blog-posts/bitfields.html](https://guiand.xyz/blog-posts/bitfields.html).