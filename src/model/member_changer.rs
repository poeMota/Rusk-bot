use std::collections::HashMap;

use crate::prelude::*;
use serenity::{builder::CreateEmbed, model::Colour};

impl ProjectMember {
    pub async fn main_changer(&self, ctx: &Context) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new()
            .embeds(Vec::from([
                self.to_embed(ctx, true).await,
                CreateEmbed::new()
                    .title(get_string("member-changer-embed-title", None))
                    .description(get_string(
                        "member-changer-embed-description",
                        Some(HashMap::from([(
                            "member_id",
                            self.id.get().to_string().as_str(),
                        )])),
                    ))
                    .color(Colour::BLUE),
            ]))
            .components(get_params_buttons(
                "member-changer",
                vec!["score", "own-folder", "tasks", "notes", "warns"],
            ))
            .ephemeral(true)
    }
}
