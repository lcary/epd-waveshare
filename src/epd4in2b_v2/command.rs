//! SPI Commands for the Waveshare 4.2" E-Ink Display Module B v2.2
use crate::traits;

/// EPD4IN2B_V2 commands
///
/// Should rarely (never?) be needed directly.
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    /// Set Resolution, LUT selection, BWR pixels, gate scan direction, source shift direction, booster switch, soft reset
    PanelSetting = 0x00,
    /// selecting internal and external power
    PowerSetting = 0x01,
    /// After the Power Off command, the driver will power off following the Power Off Sequence
    PowerOff = 0x02,
    /// Setting Power OFF sequence
    PowerOffSequenceSetting = 0x03,
    /// Turning On the Power
    PowerOn = 0x04,
    /// This command enables the internal bandgap, which will be cleared by the next POF
    PowerOnMeasure = 0x05,
    /// Starting data transmission
    BoosterSoftStart = 0x06,
    /// After this command is transmitted, the chip would enter the deep-sleep mode to save power
    DeepSleep = 0x07,
    /// This command starts transmitting data and write them into SRAM (OLD data)
    DataStartTransmission1 = 0x10,
    /// Used in Python init with value 0x03 (Data Entry Mode Setting)
    DataEntryMode = 0x11,
    /// While user sent this command, driver will refresh display (data/VCOM) according to SRAM data and LUT
    DisplayRefresh = 0x12,
    /// This command starts transmitting data and write them into SRAM (NEW data)  
    DataStartTransmission2 = 0x13,
    /// XON (?) - used in Python init with value 0x80
    Xon = 0x18,
    /// Master activation - used in TurnOnDisplay for flag=1
    MasterActivation = 0x20,
    /// Display update control 2 - used in TurnOnDisplay for flag=1 with 0xF7
    DisplayUpdateControl2 = 0x22,
    /// Write RAM for Black/White data (used for flag=1)
    WriteRamBw = 0x24,
    /// Red transmission (for flag=1) - sends red data
    RedDataTransmission = 0x26,
    /// Read chip ID / status
    ReadId = 0x2F,
    /// Border waveform control
    BorderWaveformControl = 0x3C,
    /// Set RAM X address start/end position
    SetRamXAddressStartEndPosition = 0x44,
    /// Set RAM Y address start/end position
    SetRamYAddressStartEndPosition = 0x45,
    /// Set RAM X address counter
    SetRamXAddressCounter = 0x4E,
    /// Set RAM Y address counter
    SetRamYAddressCounter = 0x4F,
    /// VCOM and data interval setting
    VcomAndDataIntervalSetting = 0x50,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Command as CommandTrait;

    #[test]
    fn command_addr() {
        assert_eq!(Command::DeepSleep.address(), 0x07);
        assert_eq!(Command::PanelSetting.address(), 0x00);
        assert_eq!(Command::DisplayRefresh.address(), 0x12);
    }
}
