use std::iter;

use super::*;

use itertools::Itertools;
use rand::prelude::{thread_rng, IteratorRandom};

#[derive(Clone, Default)]
pub struct Werwolf;

impl Role for Werwolf {
    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }

    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Werwolf)
    }

    fn name(&self) -> String {
        "Werwolf".into()
    }
}

impl Display for Werwolf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Werwolf")
    }
}

#[async_trait]
impl RoleBehavior for Werwolf {
    async fn ask<'a>(&mut self, data: &GameData<'a>, index: usize) {
        let mut others = data
            .roles
            .iter()
            .enumerate()
            .filter(|&(i, &r)| (*r).type_id() == Werwolf.type_id() && i != index);

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Werwölfe sind: {}",
                iter::once(data.players[x].name.clone())
                    .chain(others.map(|(u, _)| data.players[u].name.clone()))
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

        data.players[index].say(content);
    }
}
