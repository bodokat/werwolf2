use super::*;

pub struct Dorfbewohner;

#[async_trait]
impl Role for Dorfbewohner {
    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Dorfbewohner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dorfbewohner")
    }
}
