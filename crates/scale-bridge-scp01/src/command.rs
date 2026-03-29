use scale_bridge_core::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum NciCommand {
    Weight,
    Status,
    Zero,
    HighResolution,
    Units,
    Metrology,
    Tare,
    About,
    Diagnostic,
}

impl Command for NciCommand {
    fn command_byte(&self) -> u8 {
        match self {
            NciCommand::Weight => b'W',
            NciCommand::Status => b'S',
            NciCommand::Zero => b'Z',
            NciCommand::HighResolution => b'H',
            NciCommand::Units => b'U',
            NciCommand::Metrology => b'M',
            NciCommand::Tare => b'T',
            NciCommand::About => b'A',
            NciCommand::Diagnostic => b'D',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commands_have_correct_bytes() {
        assert_eq!(NciCommand::Weight.command_byte(), b'W');
        assert_eq!(NciCommand::Status.command_byte(), b'S');
        assert_eq!(NciCommand::Zero.command_byte(), b'Z');
        assert_eq!(NciCommand::HighResolution.command_byte(), b'H');
        assert_eq!(NciCommand::Units.command_byte(), b'U');
        assert_eq!(NciCommand::Metrology.command_byte(), b'M');
        assert_eq!(NciCommand::Tare.command_byte(), b'T');
        assert_eq!(NciCommand::About.command_byte(), b'A');
        assert_eq!(NciCommand::Diagnostic.command_byte(), b'D');
    }
}
