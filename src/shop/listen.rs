use crate::{model::member::MEMBERSMANAGER, prelude::*};
use serenity::{
    builder::{CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    model::application::{ButtonStyle, ComponentInteractionDataKind},
};

pub async fn shop_component_listeners() {
    #[listen_component("previous")]
    async fn previous(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::Button = inter.data.kind {
            let mut mem_man = match MEMBERSMANAGER.try_write() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.previous",
                        "error while try_write MEMBERSMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    MEMBERSMANAGER.write().await
                }
            };

            let member = mem_man.get_mut(inter.user.id.clone()).await.unwrap();

            let shop_man = match SHOPMANAGER.try_read() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.previous",
                        "error while try_read SHOPMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    SHOPMANAGER.read().await
                }
            };

            member.shop_data.pages = shop_man
                .get_pages(&ctx, &member.member().await.unwrap())
                .await;

            member.shop_data.current_page -= 1;
            if member.shop_data.current_page < 0 {
                member.shop_data.current_page = member.shop_data.pages.len() as i32 - 1;
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .embed(
                                member
                                    .shop_data
                                    .pages
                                    .get(member.shop_data.current_page as usize)
                                    .unwrap()
                                    .to_embed(&member, member.shop_data.pages.len() as i32),
                            )
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
        }
    }

    #[listen_component("next")]
    async fn next(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::Button = inter.data.kind {
            let mut mem_man = match MEMBERSMANAGER.try_write() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.next",
                        "error while try_write MEMBERSMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    MEMBERSMANAGER.write().await
                }
            };

            let member = mem_man.get_mut(inter.user.id.clone()).await.unwrap();

            let shop_man = match SHOPMANAGER.try_read() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.next",
                        "error while try_read SHOPMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    SHOPMANAGER.read().await
                }
            };

            member.shop_data.pages = shop_man
                .get_pages(&ctx, &member.member().await.unwrap())
                .await;

            member.shop_data.current_page += 1;
            if member.shop_data.current_page > member.shop_data.pages.len() as i32 - 1 {
                member.shop_data.current_page = 0;
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .embed(
                                member
                                    .shop_data
                                    .pages
                                    .get(member.shop_data.current_page as usize)
                                    .unwrap()
                                    .to_embed(&member, member.shop_data.pages.len() as i32),
                            )
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
        }
    }

    #[listen_component("buy")]
    async fn buy(ctx: &Context, inter: ComponentInteraction) {
        if let ComponentInteractionDataKind::Button = inter.data.kind {
            let mut mem_man = match MEMBERSMANAGER.try_write() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.buy",
                        "error while try_write MEMBERSMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    MEMBERSMANAGER.write().await
                }
            };

            let member = mem_man.get_mut(inter.user.id.clone()).await.unwrap();

            let shop_man = match SHOPMANAGER.try_read() {
                Ok(man) => man,
                Err(_) => {
                    Logger::error(
                        "shop.listen.buy",
                        "error while try_read SHOPMANAGER, maybe deadlock, trying await...",
                    )
                    .await;
                    SHOPMANAGER.read().await
                }
            };

            member.shop_data.pages = shop_man
                .get_pages(&ctx, &member.member().await.unwrap())
                .await;

            if let Some(page) = member
                .shop_data
                .pages
                .get(member.shop_data.current_page as usize)
                .cloned()
            {
                page.buy(&inter, member).await;
            }

            inter
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .embed(
                                member
                                    .shop_data
                                    .pages
                                    .get(member.shop_data.current_page as usize)
                                    .unwrap()
                                    .to_embed(&member, member.shop_data.pages.len() as i32),
                            )
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
        }
    }
}
