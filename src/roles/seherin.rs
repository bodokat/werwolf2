use std::iter::once;

use crate::game::choice;

use super::*;

pub async fn action<'a>(
    player: &User,
    players: &HashMap<&User, Role>,
    extra_roles: &Vec<Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    player
        .dm(ctx, |m| m.content("Wesen Rolle willst du sehen?"))
        .await?;

    let others = players.iter().filter(|(&u, _)| u != player);
    let choices = others.map(|(&u, _)| Some(u)).chain(once(None));

    let c = choice(
        ctx,
        player.create_dm_channel(ctx).await?.id,
        choices,
        |x| match x {
            Some(u) => u.name.clone(),
            None => "2 Karten aus der Mitte".to_string(),
        },
        '🔮'.into(),
    )
    .await
    .flatten();

    match c {
        Some(u) => {
            player
                .dm(ctx, |m| {
                    m.content(format!(
                        "{} hat die Rolle {}",
                        u.name,
                        players.get(u).expect("player not in map")
                    ))
                })
                .await?;
        }
        None => {
            player
                .dm(ctx, |m| {
                    m.content(format!(
                        "2 Rollen in der Mitte sind: {}, {}",
                        extra_roles[0], extra_roles[1]
                    ))
                })
                .await?;
        }
    }

    Ok(None)
}
