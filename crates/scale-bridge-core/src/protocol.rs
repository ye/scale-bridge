use crate::ScaleError;

pub trait Command {
    fn command_byte(&self) -> u8;
}

pub trait Protocol {
    type Command: Command;
    type Response;
    fn encode_command(&self, cmd: &Self::Command) -> Vec<u8>;
    fn decode_response(
        &self,
        cmd: &Self::Command,
        frame: &[u8],
    ) -> Result<Self::Response, ScaleError>;
}
