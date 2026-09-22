#[derive(Default)]
pub struct PlayCore {}

impl PlayCore {
    pub fn receive_event(&mut self, event: PlayEvent) {
        log::info!("Received event: {:?}", event);
        // Handle the received event here
    }
}

#[derive(Debug)]
pub enum PlayEvent {
    KeyPressed(KeyPressed),
}

#[derive(Debug)]
pub enum KeyPressed {
    Key1,
}
