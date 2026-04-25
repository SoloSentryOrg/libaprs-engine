/// Line-oriented packet source for file/stdin style transports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineTransport<'a> {
    input: &'a [u8],
}

impl<'a> LineTransport<'a> {
    /// Creates a transport over newline-separated packet bytes.
    #[must_use]
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    /// Iterates packet lines without trailing CR/LF bytes.
    #[must_use]
    pub fn packets(&self) -> Vec<&'a [u8]> {
        self.input
            .split(|byte| *byte == b'\n')
            .map(trim_trailing_carriage_return)
            .filter(|line| !line.is_empty())
            .collect()
    }
}

fn trim_trailing_carriage_return(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}
