use nom::{branch::alt, bytes::complete::tag, error::ErrorKind, Err, IResult};

#[derive(Debug)]
pub enum TDCommand {
    Toggle,
    Reset,
}

pub fn parse_command(input: &str) -> IResult<&str, TDCommand> {
    let (input, parsed) = alt((tag("toggle"), tag("reset")))(input)?;
    let out = match parsed {
        "toggle" => TDCommand::Toggle,
        "reset" => TDCommand::Reset,
        _ => {
            return Err(Err::Error(nom::error::Error {
                input,
                code: ErrorKind::Alt,
            }));
        }
    };
    Ok((input, out))
}
