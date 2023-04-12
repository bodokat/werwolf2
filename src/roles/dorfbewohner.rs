use super::{async_trait, Display, Group, Role, RoleBehavior, Team};

pub static Dorfbewohner: &'static dyn Role = &DorfbewohnerImpl;
#[derive(Clone, Default)]
struct DorfbewohnerImpl;

impl Role for DorfbewohnerImpl {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(DorfbewohnerImpl)
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
impl RoleBehavior for DorfbewohnerImpl {}

impl Display for DorfbewohnerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dorfbewohner")
    }
}
