use std::iter;

use itertools::Itertools;

use super::{Data, Display, Group, Role, RoleBehavior, Team, async_trait};

#[derive(Clone, Default)]
pub struct Gunstling;

impl Role for Gunstling {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Gunstling)
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn name(&self) -> String {
        "Günstling".into()
    }
}

#[async_trait]
impl RoleBehavior for Gunstling {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let mut wolves = data
            .roles
            .iter()
            .enumerate()
            .filter(|(_, role)| role.group() == Group::Wolf);

        let content = match wolves.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.players[x].name.clone())
                    .chain(wolves.map(|(u, _)| data.players[u].name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        data.players[index].say(content);
    }
}

impl Display for Gunstling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Günstling")
    }
}
