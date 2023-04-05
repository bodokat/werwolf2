use super::*;

#[derive(Clone, Default)]
pub struct Schlaflose;

impl Role for Schlaflose {
    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Schlaflose)
    }

    fn name(&self) -> String {
        "Schlaflose".into()
    }
}

#[async_trait]
impl RoleBehavior for Schlaflose {
    async fn after<'a>(&mut self, data: &GameData<'a>, index: usize) {
        data.players[index].say(format!("Du bist jetzt {}", data.roles[index]));
    }
}

impl Display for Schlaflose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schlaflose")
    }
}
