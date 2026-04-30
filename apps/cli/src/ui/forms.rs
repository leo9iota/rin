use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedField {
    RpcUrl,
    Contract,
    Event,
    StartBlock,
}

impl FocusedField {
    pub fn next(self) -> Self {
        match self {
            Self::RpcUrl => Self::Contract,
            Self::Contract => Self::Event,
            Self::Event => Self::StartBlock,
            Self::StartBlock => Self::RpcUrl,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::RpcUrl => Self::StartBlock,
            Self::Contract => Self::RpcUrl,
            Self::Event => Self::Contract,
            Self::StartBlock => Self::Event,
        }
    }
}

#[derive(Debug)]
pub struct SetupFormState {
    pub rpc_url: Input,
    pub contract: Input,
    pub event: Input,
    pub start_block: Input,
    pub focused: FocusedField,
}

impl Default for SetupFormState {
    fn default() -> Self {
        Self {
            rpc_url: Input::default().with_value("ws://127.0.0.1:8545".into()),
            contract: Input::default(),
            event: Input::default().with_value("Transfer(address,address,uint256)".into()),
            start_block: Input::default().with_value("latest".into()),
            focused: FocusedField::RpcUrl,
        }
    }
}

impl SetupFormState {
    pub fn active_input_mut(&mut self) -> &mut Input {
        match self.focused {
            FocusedField::RpcUrl => &mut self.rpc_url,
            FocusedField::Contract => &mut self.contract,
            FocusedField::Event => &mut self.event,
            FocusedField::StartBlock => &mut self.start_block,
        }
    }

    pub fn active_input(&self) -> &Input {
        match self.focused {
            FocusedField::RpcUrl => &self.rpc_url,
            FocusedField::Contract => &self.contract,
            FocusedField::Event => &self.event,
            FocusedField::StartBlock => &self.start_block,
        }
    }
}
