use anyhow::Result;

use crate::config::PeripheralsConfig;
use crate::tools::Tool;

pub fn create_peripheral_tools(config: &PeripheralsConfig) -> Result<Vec<Box<dyn Tool>>> {
    if !config.enabled || config.boards.is_empty() {
        return Ok(Vec::new());
    }

    let board_names: Vec<String> = config.boards.iter().map(|b| b.board.clone()).collect();

    #[cfg(feature = "hardware")]
    {
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(crate::tools::HardwareBoardInfoTool::new(
                board_names.clone(),
            )),
            Box::new(crate::tools::HardwareMemoryMapTool::new(
                board_names.clone(),
            )),
            Box::new(crate::tools::HardwareMemoryReadTool::new(board_names)),
        ];
        return Ok(tools);
    }

    #[cfg(not(feature = "hardware"))]
    {
        let _ = board_names;
        Ok(Vec::new())
    }
}

pub fn handle_command(
    _command: crate::PeripheralCommands,
    _config: &crate::config::Config,
) -> Result<()> {
    println!("Peripheral command surface is scaffolded in this cut.");
    Ok(())
}
