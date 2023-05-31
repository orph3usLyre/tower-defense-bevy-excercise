use std::{io::stdin, thread, time::Duration};
use tower_defense_bevy_excercise::{parse_command, setup_tower_defense};
use tracing::{event, span, Level};

fn main() {
    let (mut app, _tx) = setup_tower_defense();

    let _command_channel_handle = thread::spawn(move || {
        let span = span!(Level::INFO, "TDCommand channel");
        let _guard = span.enter();

        loop {
            let stdin = stdin();
            let mut buffer = String::new();
            if stdin.read_line(&mut buffer).is_ok() {
            } else {
                thread::sleep(Duration::from_secs(1));
                continue;
            }

            event!(Level::DEBUG, "Read input: {buffer}");
            match parse_command(buffer.trim_end()) {
                Ok((_, td_command)) => {
                    if _tx.try_send(td_command).is_ok() {
                        event!(Level::DEBUG, "Command sent (main)");
                        continue;
                    } else {
                        event!(Level::DEBUG, "Unable to send command to receiver (main)");
                    }
                }
                Err(_) => event!(Level::DEBUG, "Unable to parse command"),
            }
        }
    });
    app.run();
    _command_channel_handle.join().unwrap();
}
