use crate::game::choice;

use super::*;

pub async fn action<'a>(
    player: &'a User,
    players: &HashMap<&'a User, Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    player
        .dm(ctx, |m| m.content("Mit wem willst du tauschen?"))
        .await
        .expect("error sending message");

    let others = players.iter().filter(|(&u, _)| u != player);

    let to_swap: Option<(&&User, &Role)> = choice(
        ctx,
        player.create_dm_channel(ctx).await?.id,
        others,
        |(u, _)| u.name.clone(),
        '🤚'.into(),
    )
    .await;

    if let Some((u, r)) = to_swap {
        // println!("sende dieb message");
        player
            .dm(ctx, |m| m.content(format!("{} war {}", u.name, r)))
            .await?;
    }

    Ok(to_swap.map(|(u, _)| Swap(player, u)))
}
