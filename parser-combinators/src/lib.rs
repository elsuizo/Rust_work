/// este codigo proviene del genial blog-post:
/// https://bodil.lol/parser-combinators/
/// Que trata de como funcionan los parsers

#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

fn the_letter_a(input: &str) -> Result<(&str, ()), &str> {
    match input.chars().next() {
        Some('a') => Ok((&input['a'.len_utf8()..], ())),
        _         => Err(input)
    }
}

fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str> {

    move |input| match input.get(0..expected.len()) {
        Some(next) if next == expected => {
            Ok((&input[expected.len()..], ()))
        },
        _ => Err(input)
    }

}

// NOTE(elsuizo:2020-12-12): lo bueno que cuando llamamos a las funciones del
// estilo is_alphabetic o is_alphanumeric estas son agnosticas del lenguage
// osea que solo se fijan si son unicode correctos...

fn identifier(input: &str) -> Result<(&str, String), &str> {
    let mut matched = String::new();
    let mut chars = input.chars();

    // nos fijamos si el proximo es un caracter valido
    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _                                  => return Err(input)
    }
    // despues de aca podemos continuar...
    // NOTE(elsuizo:2020-12-12): porque sera que usa alphabetic y alphanumeric???
    while let Some(next) = chars.next() {
        if next.is_alphanumeric() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

#[cfg(test)]
mod tests {
    #[test]
    fn literal_parser() {
        // creamos la funcion
        let parse_joe = super::match_literal("Hello Joe!!!");
        assert_eq!(Ok(("", ())), parse_joe("Hello Joe!!!"));

        assert_eq!(Ok(("Hello Robert!!!", ())), parse_joe("Hello Joe!!!Hello Robert!!!"));

        assert_eq!(Err("Hello Mike!!!"), parse_joe("Hello Mike!!!"));
    }

}
