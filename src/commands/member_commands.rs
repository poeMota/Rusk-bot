use crate::{model::member::MEMBERSMANAGER, prelude::*};
use serenity;

pub async fn member_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn my_statistics(ctx: &Context, inter: CommandInteraction) {
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().embed(member.to_embed(&ctx, false).await),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn member_statistics(ctx: &Context, inter: CommandInteraction, dismember: User) {
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
        let member = mem_man.get(dismember.id).await.unwrap();

        inter.defer_ephemeral(&ctx.http).await.unwrap();

        inter
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().embed(member.to_embed(&ctx, true).await),
            )
            .await
            .unwrap();
    }

    #[slash_command([])]
    async fn change_member(ctx: &Context, inter: CommandInteraction) {
        let mut mem_man = MEMBERSMANAGER.try_write().unwrap();
        let member = mem_man.get(inter.user.id).await.unwrap();

        inter
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(member.main_changer(&ctx).await),
            )
            .await
            .unwrap();
    }
}
