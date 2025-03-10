use crate::{
    model::member::MEMBERSMANAGER,
    prelude::*,
    shop::{ShopData, SHOPMANAGER},
};
use serenity::{builder::CreateButton, model::application::ButtonStyle};

pub async fn shop_commands(ctx: &Context, guild: GuildId) {
    #[slash_command([])]
    async fn shop(ctx: &Context, inter: CommandInteraction) {
        let shop_man = SHOPMANAGER.read().await;
        let mut mem_man = MEMBERSMANAGER.write().await;

        let member = mem_man.get_mut(inter.user.id.clone()).await.unwrap();
        member.shop_data = ShopData {
            current_page: 0,
            pages: shop_man
                .get_pages(&ctx, &member.member().await.unwrap())
                .await,
        };

        if let Some(page) = member.shop_data.pages.first() {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(page.to_embed(&member, member.shop_data.pages.len() as i32))
                            .ephemeral(true)
                            .button(
                                CreateButton::new("previous")
                                    .emoji('◀')
                                    .style(ButtonStyle::Secondary),
                            )
                            .button(
                                CreateButton::new("buy")
                                    .emoji('🛒')
                                    .style(ButtonStyle::Success),
                            )
                            .button(
                                CreateButton::new("next")
                                    .emoji('▶')
                                    .style(ButtonStyle::Secondary),
                            ),
                    ),
                )
                .await
                .unwrap();
        } else {
            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(loc!("shop-command-no-pages-response"))
                            .ephemeral(true),
                    ),
                )
                .await
                .unwrap();
        }
    }
}
