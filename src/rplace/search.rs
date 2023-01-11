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

fn main() {
    let vector: Vec<u64> = [2, 3, 3, 5, 5, 5, 6, 6].to_vec();
    println!("Vector: {:?}", vector);

    for i in 0..10 {
        let value = i as u64;
        let result = search(&value, &vector);
        println!("Search: {} Result: {:?}", value, result);
    }
}

/*
// obtained from https://9to5answer.com/binary-search-for-the-closest-value-less-than-or-equal-to-the-search-value
fn search_rec<T>(value: T, vector: Vec<T>, start_idx: i64, end_idx: i64) -> i64 where T: PartialOrd {
    if start_idx == end_idx {
        if vector[start_idx] <= value {
            return start_idx;
        }
        return -1;
    }

    let mid_idx = start_idx + (end_idx - start_idx) / 2;

    if search_val < vector[mid_idx] {
        return search(value, vector, start_idx, mid_idx);
    }

    let ret = search(value, vector, mid_idx + 1, end_idx);
    if ret == -1 {
        return mid_idx
    }
    return ret;
}
*/
