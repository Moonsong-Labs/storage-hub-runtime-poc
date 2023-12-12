use clap::ValueEnum;

pub(crate) type ChainPrefix = u16;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub(crate) enum SupportedRuntime {
	/// Localhost
	Local,
	/// Docker Compose
	Compose,
}

impl SupportedRuntime {
	pub(crate) fn ws_address(&self) -> String {
		match self {
			Self::Local => "ws://127.0.0.1:9944".to_string(),
			Self::Compose => "ws://storagehub:9944".to_string(),
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
