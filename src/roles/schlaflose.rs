use super::*;

#[derive(Clone, Default)]
pub struct Schlaflose;

#[async_trait]
impl RoleBehavior for Schlaflose {
    async fn after<'a>(
        &mut self,
        data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        data.dm_channels[index]
            .say(data.context, format!("Du bist jetzt {}", data.roles[index]))
            .await
            .expect("Error sending message");
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
