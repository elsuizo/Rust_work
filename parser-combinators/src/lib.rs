#[derive(Debug, Clone, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>
}
// type alias para que sea un poco mas legible
type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

// todos los parser que hagamos van a implementar este trait
pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
}

// NOTE(elsuizo:2020-12-14): para todas las funciones que tengan como Salida un ParseResult
// entonces van a impl este trait, piolaaa
impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

// map function
// TODO(elsuizo:2020-12-14): porque se ponia move???
fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| parser.parse(input)
                       .map(|(next_input, result)| (next_input, map_fn(result)))
}

fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2.parse(next_input).map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}

// NOTE(elsuizo:2020-12-14): o sea que la magia de este es que se queda con la parte izquierda de
// lo que parseamos
fn left<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>
{
    map(pair(parser1, parser2), |(left, _right)| left)
}

// NOTE(elsuizo:2020-12-14): y la magia de este es que se queda con la parte derecha de lo que
// parseamos
fn right<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>
{
    map(pair(parser1, parser2), |(_left, right)| right)
}

// NOTE(elsuizo:2020-12-15): este es uno de los parsers que va a impl solito Parser
// ya que es una funcion que toma un &str y devuelve un ParserResult

fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _                              => Err(input)
    }
}

fn identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _                                  => return Err(input)
    }

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

// NOTE(elsuizo:2020-12-15): uno o mas parser
fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();
        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input)
        }

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }
        Ok((input, result))
    }
}

// NOTE(elsuizo:2020-12-15): aca lo unico que hacemos es saltearnos el primer chequeo, por eso el
// nombre de zero o mas...

fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }
        Ok((input, result))
    }
}

// TODO(elsuizo:2020-12-15): esto tiene el problema de que usa dos veces el argumento parser
// y como sabemos es una violacion a las reglas de ownership
// fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
// where
//     P: Parser<'a, A>,
// {
//     map(pair(parser, zero_or_more(parser)), |(head, mut tail)| {
//         tail.insert(0, head);
//         tail
//     })
// }
//
fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _          => Err(input)
    }
}

fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A> where
    P: Parser<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((next_input, value));
            }
        }

        Err(input)
    }
}

fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}

fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

fn space0<'a>() -> impl Parser<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}

fn quoted_string<'a>() -> impl Parser<'a, String> {
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(pred(any_char, |c| *c != '"')),
                match_literal("\""),
            )
        ),
        |chars| chars.into_iter().collect()
    )
}

fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
    pair(identifier, right(match_literal("="), quoted_string()))
}

fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>> {
    zero_or_more(right(space1(), attribute_pair()))
}

fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)> {
    right(match_literal("<"), pair(identifier, attributes()))
}

fn single_element<'a>() -> impl Parser<'a, Element> {
    map(
        left(element_start(), match_literal("/>")),
        |(name, attributes)| Element {
            name,
            attributes,
            children: vec![]
        },
    )
}

// NOTE(elsuizo:2020-12-15): esta es la version Boxed para que sea un poco mas
// liviana al compilar...
struct BoxedParser<'a, Output> {
    parser: Box<dyn Parser<'a, Output> + 'a>,
}

impl<'a, Output> BoxedParser<'a, Output> {
    fn new<P>(parser: P) -> Self
    where
        P: Parser<'a, Output> + 'a,
    {
        BoxedParser {parser: Box::new(parser)}
    }
}

impl<'a, Output> Parser<'a, Output> for BoxedParser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self.parser.parse(input)
    }
}

//-------------------------------------------------------------------------
//                        tests
//-------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::Parser;
    #[test]
    fn right_combinators_test() {
        let tag_opener = super::right(super::match_literal("<"), super::identifier);
        assert_eq!(Ok(("/>", "my-first-element".to_string())), tag_opener.parse("<my-first-element/>"));
    }

    #[test]
    fn one_or_more_combinator_test() {
        let parser = super::one_or_more(super::match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), (), ()])), parser.parse("hahahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator_test() {
        let parser = super::zero_or_more(super::match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn predicate_combinator_test() {
        let parser = super::pred(super::any_char, |c| *c == 'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }

    #[test]
    fn quoted_string_parser_test() {
        assert_eq!(Ok(("", "Hello Joe!!!".to_string())), super::quoted_string().parse("\"Hello Joe!!!\""));
    }

    #[test]
    fn single_element_parse_test() {
        assert_eq!(Ok((
                    "",
                    super::Element {
                        name: "div".to_string(),
                        attributes: vec![("class".to_string(), "float".to_string())],
                        children: vec![]
                    }
        )), super::single_element().parse("<div class=\"float\"/>"));
    }
}

