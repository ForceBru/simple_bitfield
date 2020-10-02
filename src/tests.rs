use super::{bitfield, Bitfield, BitfieldField};
use core::mem::{size_of, size_of_val};

bitfield! {
    pub struct TestBitfield<u32> {
        test: 5,
        fuck: 7,
        _: 8,
        shit: 2
    }
}

#[test]
fn validity() {
    assert!(TestBitfield::test::VALID);
    assert!(TestBitfield::fuck::VALID);
    assert!(TestBitfield::shit::VALID);
}

#[test]
fn size() {
    let the_bf: TestBitfield::__Bitfield = 7.into();

    assert_eq!(size_of_val(&the_bf), size_of::<<TestBitfield::__Bitfield as Bitfield>::BaseType>());

    assert_eq!(the_bf.test.size(), 5);
    assert_eq!(the_bf.fuck.size(), 7);
    assert_eq!(the_bf.shit.size(), 2);
}

#[test]
fn offset() {
    let the_bf = TestBitfield::new(7);

    assert_eq!(the_bf.test.offset(), 0);
    assert_eq!(the_bf.fuck.offset(), 5);
    assert_eq!(the_bf.shit.offset(), 5 + 7 + 8);
}

#[test]
fn data_get() {
    let val = 0b_1011111111_0110011_11010;
    let the_bf = TestBitfield::new(val);

    {
        let elem: &TestBitfield::test = &the_bf.test;
        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), (val >> elem.offset()) & mask)
    }

    {
        let elem: &TestBitfield::fuck = &the_bf.fuck;
        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), (val >> elem.offset()) & mask)
    }

    {
        let elem: &TestBitfield::shit = &the_bf.shit;
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
        let elem: &mut TestBitfield::test = &mut the_bf.test;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }

    {
        let new_val = 1234;
        let elem: &mut TestBitfield::fuck = &mut the_bf.fuck;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }

    {
        let new_val = 0b11101_11;
        let elem: &mut TestBitfield::shit = &mut the_bf.shit;
        elem.set(new_val);

        let mask = (1 << elem.size()) - 1;
        assert_eq!(elem.get(), new_val & mask)
    }
}