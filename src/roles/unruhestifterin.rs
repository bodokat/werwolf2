use super::*;

pub async fn action<'a>(
    user: &'a User,
    players: &HashMap<&'a User, Role>,
    ctx: &Context,
) -> CommandResult<Option<Swap<'a>>> {
    user.dm(ctx, |m| m.content("Welche Spieler willst du tauschen?"))
        .await?;

    let collector = ReactionCollectorBuilder::new(ctx)
        .channel_id(
            user.create_dm_channel(ctx)
                .await
                .expect("error getting dm channel")
                .id,
        )
        .removed(true)
        .await;

    let others = players.keys().filter(|&&u| u != user);

    let messages = others
        .map(move |&u| async move {
            let msg = user
                .dm(ctx, |m| {
                    m.content(u.name.clone());
                    m.1 = Some(vec!['🔁'.into()]);
                    m
                })
                .await
                .expect("error sending message");

            (msg.id, u)
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<HashMap<_, _>>()
        .await;

    let mut collector =
        collector.filter(|r| ready(messages.contains_key(&r.as_inner_ref().message_id)));

    let mut to_swap = None::<&User>;

    while let Some(x) = collector.next().await {
        if let Some(&target) = messages.get(&x.as_inner_ref().message_id) {
            match *x {
                serenity::collector::ReactionAction::Added(_) => match to_swap {
                    Some(to_swap) if to_swap != target => return Ok(Some(Swap(target, to_swap))),
                    None => to_swap = Some(target),
                    _ => (),
                },
                serenity::collector::ReactionAction::Removed(_) => match to_swap {
                    Some(t) if t == target => to_swap = None,
                    _ => (),
                },
            }
        }
    }

    Ok(None)
}
