use std::iter;

use super::*;

use rand::prelude::{thread_rng, IteratorRandom};

#[derive(Clone, Default)]
pub struct Werwolf;

impl Display for Werwolf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Werwolf")
    }
}

#[async_trait]
impl RoleBehavior for Werwolf {
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
            .filter(|&(i, r)| r.group() == Group::Freimaurer && i != index);

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Werwölfe sind: {}",
                iter::once(data.users[x].name.clone())
                    .chain(others.map(|(u, _)| data.users[u].name.clone()))
                    .format(", ")
            ),
            None => match data
                .extra_roles
                .iter()
                .filter(|r| r.group() != Group::Wolf)
                .choose(&mut thread_rng())
            {
                Some(x) => format!("Du bist alleine. Eine Karte aus der Mitte ist: {}", x),
                None => "Du bist alleine. Es sind nur Werwölfe in der Mitte".to_string(),
            },
        };

        data.dm_channels[index]
            .say(data.context, content)
            .await
            .unwrap();
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }
}
