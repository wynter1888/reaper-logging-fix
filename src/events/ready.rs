use serenity::{all::Command, gateway::ActivityData, model::prelude::Ready, prelude::Context};
use tracing::{debug, error, info};

use crate::{
    commands::get_command_list,
    events::expire::{expire_actions, expire_giveaways},
    models::handler::Handler,
};

impl Handler {
    pub async fn on_ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);

        ctx.set_activity(Some(ActivityData::playing(
            "with users' emotions (but faster)",
        )));

        debug!("Starting action expiration loop");
        tokio::spawn(expire_actions(self.clone(), ctx.clone()));
        debug!("Starting giveaway expiration loop");
        tokio::spawn(expire_giveaways(self.clone()));

        debug!("Adding current commands to slash commands list");
        let mut successful_commands = vec![];
        for command in get_command_list() {
            match Command::create_global_command(&ctx.http, command.register()).await {
                Ok(_) => successful_commands.push(command.name()),
                Err(e) => error!(
                    "Attempted to register command {} but failed with error: {}",
                    command.name(),
                    e
                ),
            }
        }
        info!(
            "Successfully registered commands: {}. {} is ready!",
            successful_commands.join(", "),
            ready.user.name
        );
    }
}
