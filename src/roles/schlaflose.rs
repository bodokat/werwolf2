use super::*;

#[derive(Clone, Default)]
pub struct Schlaflose;

#[async_trait]
impl RoleData for Schlaflose {
    fn after(
        &self,
        player: &User,
        player_roles: &mut HashMap<&User, &Box<dyn Role>>,
        _extra_roles: &[Box<dyn Role>],
        ctx: &Context,
    ) {
        let role = player_roles.get(player).unwrap().to_string();
        let ctx = ctx.clone();
        let player_id = player.id;
        tokio::spawn(async move {
            let _ = player_id
                .create_dm_channel(&ctx)
                .await
                .unwrap()
                .say(&ctx, format!("Du bist jetzt {}", role))
                .await;
        });
    }

    fn team(&self) -> Team {
        Team::Dorf
    }

    fn group(&self) -> Group {
        Group::Mensch
    }
}

impl Display for Schlaflose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schlaflose")
    }
}
