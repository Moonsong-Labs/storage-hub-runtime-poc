use clap::ValueEnum;

pub type ChainPrefix = u16;
pub type ChainTokenSymbol = String;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum SupportedRuntime {
	Local,
	Compose,
}

impl SupportedRuntime {
	pub fn ws_address(&self) -> String {
		match self {
			Self::Local => "ws://127.0.0.1:9944".to_string(),
			Self::Compose => "ws://storagehub:9944".to_string(),
		}
	}
}

impl From<ChainPrefix> for SupportedRuntime {
	fn from(v: ChainPrefix) -> Self {
		match v {
			0 => Self::Local,
			_ => unimplemented!("Chain prefix not supported"),
		}
	}
}

impl From<ChainTokenSymbol> for SupportedRuntime {
	fn from(v: ChainTokenSymbol) -> Self {
		match v.as_str() {
			"shDOT" => Self::Local,
			_ => unimplemented!("Chain unit not supported"),
		}
	}
}

impl std::fmt::Display for SupportedRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Local => write!(f, "Local"),
			Self::Compose => write!(f, "Compose"),
		}
	}
}
