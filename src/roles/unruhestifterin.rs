use itertools::Itertools;

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

    fn name(&self) -> String {
        "Unruhestifterin".into()
    }
}

#[derive(Clone)]
pub struct UnruhestifterinData {
    to_swap: Option<(usize, usize)>,
}

#[async_trait]
impl RoleBehavior for UnruhestifterinData {
    async fn ask<'a>(&mut self, data: &GameData<'a>, index: usize) {
        data.players[index].say("Du darfst nun zwei Spieler vertauschen".into());

        let others = data
            .players
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .collect_vec();

        let first = data.players[index]
            .choice(
                "Wähle den ersten Spieler".into(),
                others.iter().map(|(_, u)| u.name.clone()).collect(),
            )
            .await;
        let first = others[first];

        let others = others
            .into_iter()
            .filter(|&(i, _)| i != first.0)
            .collect::<Vec<_>>();

        let second = data.players[index]
            .choice(
                "Wähle den zweiten Spieler".into(),
                others.iter().map(|(_, u)| u.name.clone()).collect(),
            )
            .await;
        let second = others[second];

        data.players[index].say(format!(
            "Es werden nun {} und {} vertauscht",
            first.1.name.clone(),
            second.1.name.clone()
        ));

        self.to_swap = Some((first.0, second.0));
    }

    fn action<'a>(&mut self, data: &mut GameData<'a>, _index: usize) {
        if let Some((x, y)) = self.to_swap {
            data.roles.swap(x, y);
        }
    }
}

impl Display for UnruhestifterinData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unruhestifterin")
    }
}
