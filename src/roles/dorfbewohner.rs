use super::*;

#[derive(Clone, Default)]
pub struct Dorfbewohner;

impl Role for Dorfbewohner {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Dorfbewohner)
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn name(&self) -> String {
        "Dorfbewohner".into()
    }
}

#[async_trait]
impl RoleBehavior for Dorfbewohner {}

impl Display for Dorfbewohner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dorfbewohner")
    }
}
