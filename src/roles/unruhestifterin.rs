use crate::game::choice;

use super::*;

#[derive(Clone)]
pub struct Unruhestifterin;

impl Display for Unruhestifterin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}

impl Role for Unruhestifterin {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(UnruhestifterinData { to_swap: None })
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

#[derive(Clone)]
pub struct UnruhestifterinData {
    to_swap: Option<(usize, usize)>,
}

#[async_trait]
impl RoleBehavior for UnruhestifterinData {
    async fn ask<'a>(
        &mut self,
        data: &GameData<'a>,
        reactions: &mut ReceiverStream<Interaction>,
        index: usize,
    ) {
        data.dm_channels[index]
            .say(data.context, "Du darfst nun zwei Spieler vertauschen")
            .await
            .expect("Error sending message");

        let others = data.users.iter().enumerate().filter(|&(i, _)| i != index);

        let first = choice(
            data.context,
            reactions,
            data.dm_channels[index].id,
            "Wähle den ersten Spieler",
            others.clone(),
            |(_, u)| u.name.clone(),
        )
        .await;

        let second = choice(
            data.context,
            reactions,
            data.dm_channels[index].id,
            "Wähle den zweiten Spieler",
            others,
            |(_, u)| u.name.clone(),
        )
        .await;

        if let (Some((first, u1)), Some((second, u2))) = (first, second) {
            data.dm_channels[index]
                .say(
                    data.context,
                    format!(
                        "Es werden nun {} und {} vertauscht",
                        u1.name.clone(),
                        u2.name.clone()
                    ),
                )
                .await
                .expect("Error sending message");
            self.to_swap = Some((first, second));
        }
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, _index: usize) {
        if let Some((x, y)) = self.to_swap {
            data.roles.swap(x, y);
        }
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for UnruhestifterinData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}
