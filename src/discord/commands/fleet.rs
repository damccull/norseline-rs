use crate::{database, discord::api::DiscordApiError};
use sqlx::PgPool;
use twilight_interactions::command::{CommandInputData, CommandModel, CreateCommand, ResolvedUser};
use twilight_model::application::{
    callback::{Autocomplete, CallbackData, InteractionResponse},
    command::CommandOptionChoice,
    interaction::ApplicationCommand,
};

#[allow(clippy::large_enum_variant)]
#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "fleet", desc = "Manage or view your fleet, or show it off.")]
pub enum FleetCommand {
    #[command(name = "add")]
    Add(AddCommand),
    #[command(name = "list")]
    List(ListCommand),
    #[command(name = "remove")]
    Remove(RemoveCommand),
    #[command(name = "rename")]
    Rename(RenameCommand),
    #[command(name = "show")]
    Show(ShowCommand),
}
impl FleetCommand {
    pub const NAME: &'static str = "fleet";
}

impl FleetCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET", skip(cmd))]
    pub async fn handler(
        cmd: &ApplicationCommand,
        pool: &PgPool,
    ) -> Result<InteractionResponse, DiscordApiError> {
        let x: CommandInputData = cmd.data.clone().into();
        match FleetCommand::from_interaction(x) {
            Ok(subcommand) => match subcommand {
                FleetCommand::Add(add_command) => add_command.handle(cmd, pool).await,
                FleetCommand::List(_) => todo!(),
                FleetCommand::Remove(_) => todo!(),
                FleetCommand::Rename(_) => todo!(),
                FleetCommand::Show(show_command) => show_command.handle(cmd).await,
            },
            Err(e) => {
                return Err(DiscordApiError::UnsupportedCommand(format!(
                    "Something went wrong parsing the interaction: {}",
                    e
                )));
            }
        }
    }

    #[tracing::instrument(name = "Discord Interaction - FLEET ADD AUTOCOMPLETE", skip(cmd))]
    pub async fn autocomplete_handler(
        cmd: &ApplicationCommand,
        pool: &PgPool,
    ) -> Result<InteractionResponse, DiscordApiError> {
        let x: CommandInputData = cmd.data.clone().into();
        match FleetCommandPartial::from_interaction(x) {
            Ok(subcommand) => match subcommand {
                FleetCommandPartial::Add(add_command) => add_command.handle(cmd, pool).await,
                // _ => return Err(DiscordApiError::AutocompleteUnsupported),
            },
            Err(e) => {
                return Err(DiscordApiError::UnsupportedCommand(format!(
                    "Something went wrong parsing the interaction: {}",
                    e
                )));
            }
        }
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "add", desc = "Add a ship to your fleet.")]
pub struct AddCommand {
    /// The model of ship you want to add.
    #[command(rename = "model", autocomplete = true)]
    pub ship_model: String,
    /// The name of the ship. (Optional)
    #[command(rename = "name")]
    pub ship_name: Option<String>,
}

impl AddCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET ADD", skip(self))]
    async fn handle(
        &self,
        cmd: &ApplicationCommand,
        pool: &PgPool,
    ) -> Result<InteractionResponse, DiscordApiError> {
        let model = match database::get_ship_by_id(pool, self.ship_model.to_owned()).await {
            Ok(x) => x,
            Err(e) => {
                return Err(DiscordApiError::UnexpectedError(anyhow::anyhow!(
                    "Unable to parse given string as UUID: {:?}",
                    e
                )))
            }
        };
        let ship_name = match self.ship_name.to_owned() {
            Some(name) => format!(" named _{}_", name),
            None => "".into(),
        };

        match model {
            Some(model) => {
                unsafe {
                    FAKEDB.push(Ship {
                        model: model.name.to_owned(),
                        name: self.ship_name.clone(),
                    });
                }
                Ok(InteractionResponse::ChannelMessageWithSource(
                    CallbackData {
                        allowed_mentions: None,
                        flags: None,
                        tts: None,
                        content: Some(format!(
                            "Adding a {}{} to the fleet.",
                            model.name, ship_name
                        )),
                        embeds: Default::default(),
                        components: Default::default(),
                    },
                ))
            }
            None => Ok(InteractionResponse::ChannelMessageWithSource(
                CallbackData {
                    allowed_mentions: None,
                    flags: None,
                    tts: None,
                    content: Some(format!("`{}` is not a valid ship model.", self.ship_model,)),
                    embeds: Default::default(),
                    components: Default::default(),
                },
            )),
        }
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(
    name = "list",
    desc = "Privately list the ships in your, or the specified user's, fleet."
)]
pub struct ListCommand {
    /// The user who's fleet you'd like to see. (Optional)
    pub user: Option<ResolvedUser>,
}

impl ListCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET")]
    async fn handler(_cmd: &ApplicationCommand) -> Result<InteractionResponse, DiscordApiError> {
        Ok(InteractionResponse::ChannelMessageWithSource(
            CallbackData {
                allowed_mentions: None,
                flags: None,
                tts: None,
                content: Some("Privately perusing the fleet.".into()),
                embeds: Default::default(),
                components: Default::default(),
            },
        ))
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "remove", desc = "Remove a ship from your fleet.")]
pub struct RemoveCommand {}

impl RemoveCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET")]
    async fn handler(_cmd: &ApplicationCommand) -> Result<InteractionResponse, DiscordApiError> {
        Ok(InteractionResponse::ChannelMessageWithSource(
            CallbackData {
                allowed_mentions: None,
                flags: None,
                tts: None,
                content: Some("Removing a ship from the fleet.".into()),
                embeds: Default::default(),
                components: Default::default(),
            },
        ))
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "rename", desc = "Remame a ship in your fleet.")]
pub struct RenameCommand {}

impl RenameCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET")]
    async fn handler(_cmd: &ApplicationCommand) -> Result<InteractionResponse, DiscordApiError> {
        Ok(InteractionResponse::ChannelMessageWithSource(
            CallbackData {
                allowed_mentions: None,
                flags: None,
                tts: None,
                content: Some("Renaming a ship in the fleet.".into()),
                embeds: Default::default(),
                components: Default::default(),
            },
        ))
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "show", desc = "Show your fleet to the channel.")]
pub struct ShowCommand {
    /// This is a dummy option. Set it true or false. It's just here temporarily due to a bug.
    _dummy: bool,
}

impl ShowCommand {
    #[tracing::instrument(name = "Discord Interaction - FLEET SHOW")]
    async fn handle(
        &self,
        _cmd: &ApplicationCommand,
    ) -> Result<InteractionResponse, DiscordApiError> {
        unsafe {
            Ok(InteractionResponse::ChannelMessageWithSource(
                CallbackData {
                    allowed_mentions: None,
                    flags: None,
                    tts: None,
                    content: Some(format!("Showing off the fleet.\n```\n{:?}\n```", FAKEDB)),
                    embeds: Default::default(),
                    components: Default::default(),
                },
            ))
        }
    }
}

// AUTOCOMPLETE command models

#[derive(CommandModel, Debug)]
#[command(partial = true)]
pub enum FleetCommandPartial {
    #[command(name = "add")]
    Add(AddCommandPartial),
}

#[derive(CommandModel, Debug)]
#[command(partial = true)]
pub struct AddCommandPartial {
    /// The model of ship you want to add.
    #[command(rename = "model")]
    pub ship_model: String,
}

impl AddCommandPartial {
    #[tracing::instrument(name = "Discord Autocomplete Handler - AddCommandPartial")]
    async fn handle(
        &self,
        _cmd: &ApplicationCommand,
        pool: &PgPool,
    ) -> Result<InteractionResponse, DiscordApiError> {
        let user_query = self.ship_model.to_lowercase();
        let choices = match database::all_ship_models(pool).await {
            Ok(x) => x
                .into_iter()
                .filter(|s| s.name.to_lowercase().contains(&user_query))
                .take(25)
                .collect::<Vec<_>>()
                .iter()
                .map(|s| CommandOptionChoice::String {
                    name: s.name.to_string(),
                    value: s.id.to_string(),
                })
                .collect::<Vec<_>>(),
            Err(e) => {
                return Err(DiscordApiError::UnexpectedError(anyhow::anyhow!(
                    "Error querying database: {:?}",
                    e
                )))
            }
        };

        Ok(InteractionResponse::Autocomplete(Autocomplete { choices }))
    }
}

//TODO: Get rid of this when testing is done and a real database is in use
#[derive(Debug)]
#[allow(dead_code)]
struct Ship {
    pub model: String,
    pub name: Option<String>,
}

static mut FAKEDB: Vec<Ship> = Vec::new();
