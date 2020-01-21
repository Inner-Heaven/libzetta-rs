/// Very pricey way of parsing strings. Used because some ratios have `x` character, and some don't.
#[inline(always)]
pub fn parse_float(input: &mut String) -> Result<f64,std::num::ParseFloatError> {
        let last_char = {
            let chars = input.chars();
            chars.last()
        };
        if last_char == Some('x') {
            input.pop();
        }
        input.parse()
}