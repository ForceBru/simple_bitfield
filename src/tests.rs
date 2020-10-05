// This is needed for tests: https://stackoverflow.com/questions/28185854/how-do-i-test-crates-with-no-std
extern crate std;

use super::{bitfield, Field, Bitfield};
use core::mem::{size_of, size_of_val};

bitfield! {
    pub struct TestBitfield<u32> {
        field1: 5,
        field2: 7,
        _: 8,
        field3: 2
    }

    // From docs
    struct MyBitfield<u32> {
        field1: 3, // First field (least significant) of size 3 bits
        field2: 9,
        _: 6,      // Fields named `_` are skipped (offsets are preserved)
        field3: 1  // Last bit (closest to the highest bit of `u32`)
    }

    pub struct AnotherOne<u8> {
        f1: 3, f2: 1
    }
}

#[test]
fn validity() {
    assert!(TestBitfield::field1::VALID);
    assert!(TestBitfield::field2::VALID);
    assert!(TestBitfield::field3::VALID);
}

#[test]
fn size() {
    let the_bf: TestBitfield::TestBitfield = 7.into();

    assert_eq!(
        size_of_val(&the_bf),
        size_of::<<TestBitfield::TestBitfield as Bitfield>::BaseType>()
    );

    assert_eq!(the_bf.field1.size(), 5);
    assert_eq!(the_bf.field2.size(), 7);
    assert_eq!(the_bf.field3.size(), 2);
}

#[test]
fn offset() {
    assert_eq!(TestBitfield::field1::OFFSET, 0);
    assert_eq!(TestBitfield::field2::OFFSET, 5);
    assert_eq!(TestBitfield::field3::OFFSET, 5 + 7 + 8);

    let the_bf = TestBitfield::new(7);

    assert_eq!(the_bf.field1.offset(), TestBitfield::field1::OFFSET);
    assert_eq!(the_bf.field2.offset(), TestBitfield::field2::OFFSET);
    assert_eq!(the_bf.field3.offset(), TestBitfield::field3::OFFSET);
}

#[test]
fn mask() {
    assert_eq!(TestBitfield::field1::MASK, 0b11111);
    assert_eq!(TestBitfield::field2::MASK, 0b1111111);
    assert_eq!(TestBitfield::field3::MASK, 0b11);

    let bf = TestBitfield::new(1);

    assert_eq!(bf.field1.mask(), TestBitfield::field1::MASK);
    assert_eq!(bf.field2.mask(), TestBitfield::field2::MASK);
    assert_eq!(bf.field3.mask(), TestBitfield::field3::MASK);
}

#[test]
fn data_get() {
    let val = 0b_1011111111_0110011_11010;
    let the_bf = TestBitfield::TestBitfield::from(val);

    {
        let elem: &TestBitfield::field1 = &the_bf.field1;
        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), (val >> elem.offset()) & mask)
    }

    {
        let elem: &TestBitfield::field2 = &the_bf.field2;
        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), (val >> elem.offset()) & mask)
    }

    {
        let elem: &TestBitfield::field3 = &the_bf.field3;
        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), (val >> elem.offset()) & mask)
    }
}

#[test]
fn data_set() {
    let val = 0b_1011111111_0110011_11010;
    let mut the_bf = TestBitfield::new(val);

    {
        let new_val = 4;
        let elem: &mut TestBitfield::field1 = &mut the_bf.field1;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }

    {
        let new_val = 1234;
        let elem: &mut TestBitfield::field2 = &mut the_bf.field2;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }

    {
        let new_val = 0b11101_11;
        let elem: &mut TestBitfield::field3 = &mut the_bf.field3;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }
}

#[test]
fn data_set_checked() {
    let val = 0b1_1010100_11111;
    let mut bitf = TestBitfield::new(val);

    let new_val = 0b10111111;
    assert!(new_val > bitf.field2.mask());
    assert_eq!(
        bitf.field2.set_checked(new_val),
        Err(new_val & bitf.field2.mask())
    );

    bitf.field2.set_checked(0).unwrap()
}

#[test]
fn use_in_struct() {
    #[allow(dead_code)]
    #[repr(packed)]
    struct SomeStruct {
        bitfield1: TestBitfield::TestBitfield,
        bitfield2: AnotherOne::AnotherOne
    }

    assert_eq!(
        size_of::<SomeStruct>(),
        size_of::<<TestBitfield::TestBitfield as Bitfield>::BaseType>() +
        size_of::<<AnotherOne::AnotherOne as Bitfield>::BaseType>()
    );
    
    let the_struct = SomeStruct {
        bitfield1: TestBitfield::new(0b1_01010),
        bitfield2: AnotherOne::new(0b11_000)
    };

    // warning: borrow of packed field is unsafe and requires unsafe function or block (error E0133)
    // This is because `the_struct.bitfield1` borrows `the_struct` via the `Deref` trait
    assert_eq!(the_struct.bitfield1.field2.get(), 1);
    assert_eq!(the_struct.bitfield2.f2.get(), 1)
}

#[test]
fn use_as_function_arguments() {
    fn example(bf: &TestBitfield::TestBitfield) -> &TestBitfield::field2 {
        &bf.field2
    }

    let the_bf = TestBitfield::new(0b110);

    assert_eq!(example(&the_bf).get(), 0)
}

#[test]
fn printing() {
    let mut a_bitfield = MyBitfield::new(12345);
    a_bitfield.field1.set(0);

    std::println!("{}", a_bitfield.field1.get());

    std::println!("{}\n{:?}", a_bitfield, a_bitfield)
}