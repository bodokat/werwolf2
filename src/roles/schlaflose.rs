use super::*;

pub struct Schlaflose;

#[async_trait]
impl Role for Schlaflose {
    async fn action<'a>(
        &self,
        player: &'a User,
        _player_roles: &HashMap<&'a User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        _ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) -> CommandResult<Vec<Action<'a>>> {
        Ok(vec![Action::SayRole(player)])
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Schlaflose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schlaflose")
    }
}
