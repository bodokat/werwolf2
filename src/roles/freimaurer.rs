use std::iter;

use super::*;

#[derive(Clone, Default)]
pub struct Freimaurer;

impl Display for Freimaurer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Freimaurer")
    }
}

#[async_trait]
impl RoleBehavior for Freimaurer {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        _reactions: &mut ReceiverStream<ReactionAction>,
        index: usize,
    ) {
        let mut others = data
            .roles
            .iter()
            .enumerate()
            .filter(|&(_, r)| r.group() == Group::Freimaurer);

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.users[x].name.clone())
                    .chain(others.map(|(u, _)| data.users[u].name.clone()))
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
        Group::Wolf
    }
}
