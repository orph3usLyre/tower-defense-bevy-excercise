use crate::communication::*;
use crate::resources::GameCommandChannel;
use crate::AppState;
use crate::TDCommand;
use bevy::prelude::*;
use tracing::event;
use tracing::Level;

pub fn event_dispatch(
    mut command_send: EventWriter<TDCommand>,
    channel: ResMut<GameCommandChannel>,
) {
    if channel.0.is_empty() {
        return;
    }
    while let Ok(ins) = channel.0.try_recv() {
        event!(Level::INFO, "Command sent (event_dispatch): {:#?}", ins);
        command_send.send(ins);
    }
}

pub fn execute_outside_commands(
    mut command_channel: EventReader<TDCommand>,
    mut restart_channel: EventWriter<Restart>,
    mut toggle_tiles: EventWriter<ToggleTile>,
    mut create_tower: EventWriter<CreateTower>,
) {
    if command_channel.is_empty() {
        return;
    }
    for command in command_channel.iter() {
        event!(Level::INFO, "Received command: {:#?}", command);
        match command {
            TDCommand::Toggle(toggle) => {
                event!(Level::INFO, "matched toggle tile on {:?}", toggle.hex_pos);
                toggle_tiles.send(*toggle);
            }
            TDCommand::Restart(_) => {
                event!(Level::INFO, "matched reset, restarting...");
                restart_channel.send(Restart);
                return;
            }
            TDCommand::Tower(tower) => {
                event!(
                    Level::INFO,
                    "matched create tower {:?} on {:?}",
                    tower.tower_type,
                    tower.hex_pos
                );
                create_tower.send(*tower);
            }
        }
    }
}

pub fn receive_restart_command(
    mut restart_reader: EventReader<Restart>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if restart_reader.iter().last().is_some() {
        next_state.set(AppState::Restart);
    }
}
