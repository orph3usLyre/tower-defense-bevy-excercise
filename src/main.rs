use std::{io::stdin, thread};
use tower_defense_bevy_excercise::{parse_command, setup_tower_defense};
use tracing::{event, span, Level};

fn main() {
    let (mut app, _tx) = setup_tower_defense();

    let _command_channel_handle = thread::spawn(move || {
        let span = span!(Level::INFO, "TDCommand channel");
        let _guard = span.enter();
        loop {
            let mut buffer = String::new();

            stdin()
                .read_line(&mut buffer)
                .expect("Couldn't read input from terminal");
            event!(Level::DEBUG, "Read input: {buffer}");
            match parse_command(&buffer.trim_end()) {
                Ok((_, td_command)) => {
                    if let Ok(_) = _tx.send(td_command) {
                        event!(Level::DEBUG, "Command sent (main)");
                        continue;
                    } else {
                        event!(Level::DEBUG, "Unable to send command to receiver (main)");
                        break;
                    }
                }
                Err(_) => event!(Level::DEBUG, "Unable to parse command"),
            }
        }
    });
    app.run();
    _command_channel_handle.join().unwrap();
}
