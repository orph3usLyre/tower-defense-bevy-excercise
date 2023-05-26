use std::thread;
use tower_defense_bevy_excercise::setup_tower_defense;
use tower_defense_bevy_excercise::TDCommand;

fn main() {
    let (mut app, _tx) = setup_tower_defense();

    let _command_channel_handle = thread::spawn(move || {
        let _result = _tx.send(TDCommand::Toggle);
        // TODO: Add parsing and error handling
    });
    app.run();
    _command_channel_handle.join().unwrap();
}
