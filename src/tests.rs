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