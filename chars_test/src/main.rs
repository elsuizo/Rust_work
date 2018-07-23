// vemos si esta cierto char en un string
fn main() {
    let s = "piolAalsALKJAL:KJFLAKS";
    for c in s.chars() {
        match c {
            'A'...'Z' => println!("letra mayuscula: {}", c),
            'a'...'z' => println!("letra minuscula"),
            '0'...'9' => println!("numero"),
            _         => println!("otra cosa")
        }
    }
}
