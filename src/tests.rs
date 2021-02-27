#[cfg(test)]
use crate::data::LargeLongMap;

#[test]
fn test_large_long_map() {
    let mut map: LargeLongMap<i32> = LargeLongMap::new(10);
    map.insert(2, 2);
    map.insert(20, 20);
    map.insert(200, 200);
    map.insert(155, 155);
    map.insert(39482, 28);

    assert_eq!(2, *map.get(2).unwrap());
    assert_eq!(20, *map.get(20).unwrap());
    assert_eq!(200, *map.get(200).unwrap());
    assert_eq!(155, *map.get(155).unwrap());
    assert_eq!(28, *map.get(39482).unwrap());
}
