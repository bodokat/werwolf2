use std::iter;

use super::*;

#[derive(Clone, Default)]
pub struct Gunstling;

#[async_trait]
impl RoleData for Gunstling {
    async fn ask(
        &mut self,
        player: &User,
        players: &HashMap<&User, Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
        _receiver: &mut ReceiverStream<ReactionAction>,
    ) {
        let mut wolves = players
            .iter()
            .filter(|(_, role)| role.group() == Group::Wolf);

        let content = match wolves.next() {
            Some((x, _)) => format!(
                "Die Werwölfe sind: {}",
                iter::once(x.name.clone())
                    .chain(wolves.map(|(u, _)| u.name.clone()))
                    .format(", ")
            ),
            None => "Es gibt keine Werwölfe.".to_string(),
        };

        player.dm(ctx, |m| m.content(content)).await.unwrap();
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
