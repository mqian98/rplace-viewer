pub fn search<T>(value: &T, vector: &Vec<T>) -> Result<i64, i64> where T: PartialOrd + Copy {
    let element = | i: usize | -> T { 
        vector[i]
    };
    least_greater(value, 0, vector.len() as i64 - 1, element)
}

// obtained from https://www.geeksforgeeks.org/variants-of-binary-search/
pub fn least_greater<T, V>(value: &T, mut start_idx: i64, mut end_idx: i64, element: V) -> Result<i64, i64> where V: Fn(usize) -> T, T: PartialOrd + Copy {
    let mut ret_idx = end_idx + 1;
    let mut mid_idx;
    let mut found = false;

    while start_idx <= end_idx {
        mid_idx = start_idx + ((end_idx - start_idx + 1) / 2);
        match element(mid_idx as usize) {
            v if v < *value => {
                start_idx = mid_idx + 1;
            },
            v if v == *value => {
                start_idx = mid_idx + 1;
                found = true;
            },
            v if v > *value => {
                ret_idx = mid_idx;
                end_idx = mid_idx - 1;
            },
            _ => todo!()
        }
    }

    match found {
        true => Ok(ret_idx - 1),
        false => Err(ret_idx),
    }
}

fn test_search() {
    let vector: Vec<u64> = [2, 3, 3, 5, 5, 5, 6, 6].to_vec();
    println!("Vector: {:?}", vector);

    for i in 0..10u64 {
        let value = i as u64;
        let result = search(&value, &vector);
        let bin_search_result = vector.binary_search(&i);
        println!("Search: {} Result: {:?} {:?}", value, result, bin_search_result);
    }
}
