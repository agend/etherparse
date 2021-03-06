use etherparse::*;

#[test]
fn vlan_print() {
    use VlanHeader::*;
    println!("{:?}", 
         Single(SingleVlanHeader{
            priority_code_point: 0,
            drop_eligible_indicator: false,
            vlan_identifier: 0x123,
            ether_type: 0x12
        }));
    println!("{:?}",
        Double(DoubleVlanHeader{
            outer: SingleVlanHeader{
                priority_code_point: 0,
                drop_eligible_indicator: false,
                vlan_identifier: 0x123,
                ether_type: 0x12
            },
            inner: SingleVlanHeader{
                priority_code_point: 0,
                drop_eligible_indicator: false,
                vlan_identifier: 0x123,
                ether_type: 0x12
            }
    }));
}

#[test]
fn vlan_header_read() {
    use std::io::Cursor;
    
    let input = SingleVlanHeader {
        ether_type: EtherType::Ipv4 as u16,
        priority_code_point: 2,
        drop_eligible_indicator: true,
        vlan_identifier: 1234,
    };

    //serialize
    let mut buffer: Vec<u8> = Vec::with_capacity(4);
    input.write(&mut buffer).unwrap();
    assert_eq!(4, buffer.len());

    //deserialize
    let mut cursor = Cursor::new(&buffer);
    let result = SingleVlanHeader::read(&mut cursor).unwrap();
    assert_eq!(4, cursor.position());

    //check equivalence
    assert_eq!(input, result);
}

#[test]
fn vlan_header_write() {
    use WriteError::ValueError;
    use ValueError::*;
    use ErrorField::*;
    fn base() -> SingleVlanHeader {
        SingleVlanHeader {
            ether_type: EtherType::Ipv4 as u16,
            priority_code_point: 2,
            drop_eligible_indicator: true,
            vlan_identifier: 1234,
        }
    };

    fn test_write(input: &SingleVlanHeader) -> Result<(), WriteError> {
        let mut buffer: Vec<u8> = Vec::new();
        let result = input.write(&mut buffer);
        assert_eq!(0, buffer.len());
        result
    };

    //priority_code_point
    assert_matches!(test_write(&{
                        let mut value = base();
                        value.priority_code_point = 8;
                        value
                    }),
                    Err(ValueError(U8TooLarge{value: 8, max: 7, field: VlanTagPriorityCodePoint})));

    //vlan_identifier
    assert_matches!(test_write(&{
                        let mut value = base();
                        value.vlan_identifier = 0x1000;
                        value
                    }),
                    Err(ValueError(U16TooLarge{value: 0x1000, max: 0xFFF, field: VlanTagVlanId})));
}

#[test]
fn double_vlan_header_read_write() {
    //normal package
    {
        const IN: DoubleVlanHeader = DoubleVlanHeader {
            outer: SingleVlanHeader {
                priority_code_point: 0,
                drop_eligible_indicator: false,
                vlan_identifier: 0x321,
                ether_type: EtherType::VlanTaggedFrame as u16
            },
            inner: SingleVlanHeader {
                priority_code_point: 1,
                drop_eligible_indicator: false,
                vlan_identifier: 0x456,
                ether_type: EtherType::Ipv4 as u16
            }
        };

        //write it
        let mut buffer = Vec::<u8>::new();
        IN.write(&mut buffer).unwrap();

        //read it
        use std::io::Cursor;
        let mut cursor = Cursor::new(&buffer);
        assert_eq!(DoubleVlanHeader::read(&mut cursor).unwrap(), IN);
    }
    //check that an error is thrown if the 
    {
        const IN: DoubleVlanHeader = DoubleVlanHeader {
            outer: SingleVlanHeader {
                priority_code_point: 0,
                drop_eligible_indicator: false,
                vlan_identifier: 0x321,
                ether_type: 1 //invalid
            },
            inner: SingleVlanHeader {
                priority_code_point: 1,
                drop_eligible_indicator: false,
                vlan_identifier: 0x456,
                ether_type: EtherType::Ipv4 as u16
            }
        };

        //write it
        let mut buffer = Vec::<u8>::new();
        IN.write(&mut buffer).unwrap();

        //read it
        use std::io::Cursor;
        let mut cursor = Cursor::new(&buffer);
        assert_matches!(DoubleVlanHeader::read(&mut cursor), 
                        Err(ReadError::VlanDoubleTaggingUnexpectedOuterTpid(1)));
    }
}

#[test]
fn single_from_slice() {
    let input = SingleVlanHeader {
        ether_type: EtherType::Ipv4 as u16,
        priority_code_point: 2,
        drop_eligible_indicator: true,
        vlan_identifier: 1234,
    };

    //write it
    let mut buffer = Vec::<u8>::new();
    input.write(&mut buffer).unwrap();

    //check that a too small slice results in an error
    assert_matches!(SingleVlanHeaderSlice::from_slice(&buffer[..3]), Err(ReadError::IoError(_)));

    //check that all fields are read correctly
    let slice = SingleVlanHeaderSlice::from_slice(&buffer).unwrap();
    assert_eq!(slice.priority_code_point(), input.priority_code_point);
    assert_eq!(slice.drop_eligible_indicator(), input.drop_eligible_indicator);
    assert_eq!(slice.vlan_identifier(), input.vlan_identifier);
    assert_eq!(slice.ether_type(), input.ether_type);

    //check that the to_header results in the same as the input
    assert_eq!(slice.to_header(), input);
}

#[test]
fn double_from_slice() {
    let input = DoubleVlanHeader {
        outer: SingleVlanHeader {
            ether_type: EtherType::ProviderBridging as u16,
            priority_code_point: 2,
            drop_eligible_indicator: true,
            vlan_identifier: 1234,
        },
        inner: SingleVlanHeader {
            ether_type: EtherType::Ipv6 as u16,
            priority_code_point: 7,
            drop_eligible_indicator: false,
            vlan_identifier: 4095,
        }
    };

    //write it
    let mut buffer = Vec::<u8>::new();
    input.write(&mut buffer).unwrap();

    //check that a too small slice results in an error
    assert_matches!(DoubleVlanHeaderSlice::from_slice(&buffer[..7]), Err(ReadError::IoError(_)));

    let slice = DoubleVlanHeaderSlice::from_slice(&buffer).unwrap();
    assert_eq!(slice.outer().priority_code_point(), input.outer.priority_code_point);
    assert_eq!(slice.outer().drop_eligible_indicator(), input.outer.drop_eligible_indicator);
    assert_eq!(slice.outer().vlan_identifier(), input.outer.vlan_identifier);
    assert_eq!(slice.outer().ether_type(), input.outer.ether_type);

    assert_eq!(slice.inner().priority_code_point(), input.inner.priority_code_point);
    assert_eq!(slice.inner().drop_eligible_indicator(), input.inner.drop_eligible_indicator);
    assert_eq!(slice.inner().vlan_identifier(), input.inner.vlan_identifier);
    assert_eq!(slice.inner().ether_type(), input.inner.ether_type);

    //check that the to_header results in the same as the input
    assert_eq!(slice.to_header(), input);
}
