/// A solution without allocation
pub fn fizz_buzz(i: u32) -> FBResult {
    if i % 3 == 0 {
        if i % 5 == 0 {
            FBResult::FizzBuzz
        } else {
            FBResult::Fizz
        }
    } else if i % 5 == 0 {
        FBResult::Buzz
    } else {
        FBResult::Num(i)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FBResult {
    Fizz,
    Buzz,
    FizzBuzz,
    Num(u32),
}

impl TryFrom<&str> for FBResult {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Fizz" => Ok(FBResult::Fizz),
            "Buzz" => Ok(FBResult::Buzz),
            "FizzBuzz" => Ok(FBResult::FizzBuzz),
            _ => Ok(FBResult::Num(value.parse()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{fizz_buzz, FBResult};

    #[test]
    fn test_fizz_buzz() {
        let content = include_str!("../fizzbuzz.out");

        for (i, line) in content.lines().enumerate() {
            let num = i + 1;

            assert_eq!(
                Ok(fizz_buzz(num as u32)),
                FBResult::try_from(line),
                "number {num}"
            );
        }
    }
}
