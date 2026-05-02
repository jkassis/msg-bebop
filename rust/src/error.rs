#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Retryable,
    Terminal,
    CallerBug,
}

impl ErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Retryable => "retryable",
            ErrorCategory::Terminal => "terminal",
            ErrorCategory::CallerBug => "caller_bug",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourierError {
    pub category: ErrorCategory,
    pub message: String,
}

impl CourierError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Retryable,
            message: message.into(),
        }
    }

    pub fn terminal(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Terminal,
            message: message.into(),
        }
    }

    pub fn caller_bug(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::CallerBug,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CourierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.category.as_str(), self.message)
    }
}

impl std::error::Error for CourierError {}
