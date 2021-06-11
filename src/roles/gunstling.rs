use std::iter;

use super::*;

#[derive(Clone, Default)]
pub struct Gunstling;

#[async_trait]
impl RoleBehavior for Gunstling {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        let mut wolves = data
            .roles
            .iter()
            .enumerate()
            .filter(|(_, role)| role.group() == Group::Wolf);

        let content = match wolves.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.users[x].name.clone())
                    .chain(wolves.map(|(u, _)| data.users[u].name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        data.dm_channels[index]
            .say(data.context, content)
            .await
            .expect("error sending message");
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Gunstling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Günstling")
    }
}
