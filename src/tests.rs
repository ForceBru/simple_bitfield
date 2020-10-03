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
    let the_bf = TestBitfield::new(7);

    assert_eq!(the_bf.field1.offset(), 0);
    assert_eq!(the_bf.field2.offset(), 5);
    assert_eq!(the_bf.field3.offset(), 5 + 7 + 8);
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