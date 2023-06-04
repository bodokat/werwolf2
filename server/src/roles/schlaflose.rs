use super::{async_trait, Data, Display, Group, Role, RoleBehavior, Team};

pub static Schlaflose: &'static dyn Role = &SchlafloseImpl;

#[derive(Clone, Default)]
struct SchlafloseImpl;

impl Role for SchlafloseImpl {
    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(SchlafloseImpl)
    }

    fn name(&self) -> String {
        "Schlaflose".into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl RoleBehavior for SchlafloseImpl {
    async fn after<'a>(&mut self, data: &Data<'a>, index: usize) {
        data.players[index].say(format!("Du bist jetzt {}", data.roles[index].name()));
    }
}

impl Display for SchlafloseImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schlaflose")
    }
}
