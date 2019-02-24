fn main() {
    let result = (0..=20).into_iter()
        .map(|value| {
            if value % 2 == 0 {
                Ok(value)
            } else {
                Err(())
            }
        })
        .flatten()
        .collect::<Vec<i32>>();

    println!("result: {:?}", result);
}
