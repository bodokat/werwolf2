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
impl RoleData for Freimaurer {
    async fn ask(
        &mut self,
        player: &User,
        players: &HashMap<&User, Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        let mut others = players.iter().filter(|(&other_user, role)| {
            role.group() == Group::Freimaurer && other_user != player
        });

        let content = match others.next() {
            Some((x, _)) => format!(
                "Die anderen Freimaurer sind: {}",
                iter::once(x.name.clone())
                    .chain(others.map(|(u, _)| u.name.clone()))
                    .format(", ")
            ),
            None => "Du bist alleine.".to_string(),
        };

        player.dm(ctx, |m| m.content(content)).await.unwrap();
    }

    fn team(&self) -> Team {
        Team::Wolf
    }

    fn group(&self) -> Group {
        Group::Wolf
    }
}
