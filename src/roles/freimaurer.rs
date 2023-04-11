use itertools::Itertools;

use super::{Data, Display, Group, Role, RoleBehavior, Team, async_trait};
use std::any::Any;
use std::iter;

#[derive(Clone, Default)]
pub struct Freimaurer;

impl Display for Freimaurer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Freimaurer")
    }
}

impl Role for Freimaurer {
    fn build(&self) -> Box<dyn RoleBehavior> {
        Box::new(Freimaurer)
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }

    fn name(&self) -> String {
        "Freimaurer".into()
    }
}

#[async_trait]
impl RoleBehavior for Freimaurer {
    async fn ask<'a>(&mut self, data: &Data<'a>, index: usize) {
        let mut others = data
            .roles
            .iter()
            .enumerate()
            .filter(|&(_, &r)| (*r).type_id() == Freimaurer.type_id());

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(data.players[x].name.clone())
                    .chain(others.map(|(u, _)| data.players[u].name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        data.players[index].say(content);
    }
}
