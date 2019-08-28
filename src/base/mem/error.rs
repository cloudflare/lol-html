#[derive(Fail, Debug, PartialEq)]
#[fail(display = "{}B exceeded limits.", current_usage)]
pub struct ExceededLimitsError {
    pub current_usage: usize,
}

impl ExceededLimitsError {
    pub fn new(current_usage: usize) -> Self {
        ExceededLimitsError { current_usage }
    }
}
