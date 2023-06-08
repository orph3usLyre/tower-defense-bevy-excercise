use crossbeam_channel::Sender;
use std::{io::stdin, thread, time::Duration};
use tower_defense_bevy_excercise::{parse_command, setup_tower_defense, TDCommand};
use tracing::{event, span, Level};

fn main() {
    // setup app and sender
    let (mut app, tx) = setup_tower_defense();
    // spawn thread with command receive loop
    let _command_channel_handle = thread::spawn(move || command_receive_loop(tx));
    // run app
    app.run();
    _command_channel_handle.join().unwrap();
}

fn command_receive_loop(tx: Sender<TDCommand>) {
    let span = span!(Level::INFO, "TDCommand channel");
    let _guard = span.enter();

    loop {
        let stdin = stdin();
        let mut buffer = String::new();
        if stdin.read_line(&mut buffer).is_ok() {
        } else {
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        event!(Level::DEBUG, "Read input: {buffer}");
        match parse_command(buffer.trim_end()) {
            Some(td_command) => {
                if tx.try_send(td_command).is_ok() {
                    event!(Level::DEBUG, "Command sent (main)");
                    continue;
                } else {
                    event!(Level::DEBUG, "Unable to send command to receiver (main)");
                }
            }
            None => event!(Level::DEBUG, "Unable to parse command"),
        }
    }
}
