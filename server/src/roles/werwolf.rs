use std::iter;

use super::{async_trait, Data, Display, Group, Role, RoleBehavior, Team};

use itertools::Itertools;
use rand::prelude::{thread_rng, IteratorRandom};

pub static Werwolf: &'static dyn Role = &WerwolfImpl;

#[derive(Clone, Default)]
struct WerwolfImpl;

impl Role for WerwolfImpl {
    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }

    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(WerwolfImpl)
    }

    fn name(&self) -> String {
        "Werwolf".into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for WerwolfImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Werwolf")
    }
}

#[async_trait]
impl RoleBehavior for WerwolfImpl {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let mut others = data
            .roles
            .iter()
            .enumerate()
            .filter(|&(i, &r)| r.group() == Group::Wolf && i != index);

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
                Some(x) => format!(
                    "Du bist alleine. Eine Karte aus der Mitte ist: {}",
                    x.name()
                ),
                None => "Du bist alleine. Es sind nur Werwölfe in der Mitte".to_string(),
            },
        };

        data.players[index].say(content);
    }
}
