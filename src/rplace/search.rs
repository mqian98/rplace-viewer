pub fn search<T>(value: &T, vector: &Vec<T>) -> Option<T> where T: PartialOrd + Copy {
    let ret_idx = least_greater(value, vector, &mut 0, &mut (vector.len() as i64 - 1));
    match ret_idx - 1 {
        x if x < 0 => None,
        x => Some(vector[x as usize]),
    }
}

// obtained from https://www.geeksforgeeks.org/variants-of-binary-search/
fn least_greater<T>(value: &T, vector: &Vec<T>, start_idx: &mut i64, end_idx: &mut i64) -> i64 where T: PartialOrd + Copy {
    let mut ret_idx = *end_idx + 1;
    let mut mid_idx;

    while *start_idx <= *end_idx {
        mid_idx = *start_idx + ((*end_idx - *start_idx + 1) / 2);
        match &vector[mid_idx as usize] {
            &v if v < *value => {
                *start_idx = mid_idx + 1;
            },
            &v if v > *value => {
                ret_idx = mid_idx;
                *end_idx = mid_idx - 1;
            },
            &v if v == *value => {
                *start_idx = mid_idx + 1;
            },
            &_ => todo!()
        }
    }

    return ret_idx;
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
