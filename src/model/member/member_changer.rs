use crate::{model::member::ProjectMember, prelude::*};
use serenity::{builder::CreateEmbed, model::Colour};

impl ProjectMember {
    pub async fn main_changer(&self) -> CreateInteractionResponseMessage {
        CreateInteractionResponseMessage::new()
            .embed(
                CreateEmbed::new()
                    .title(loc!("member-changer-embed-title"))
                    .description(loc!(
                        "member-changer-embed-description",
                        "member_id" = self.id.get()
                    ))
                    .color(Colour::BLUE),
            )
            .components(get_params_buttons(
                "member-changer",
                vec!["score", "own-folder", "tasks", "notes", "warns"],
            ))
            .ephemeral(true)
    }
}
