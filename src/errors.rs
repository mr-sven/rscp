macro_rules! error_code_ext {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        /// Error code result
        $(#[$($attrs)*])*
        pub enum $name {
            $($vn = $v),+
        }

        impl Into<u32> for $name {
            fn into(self) -> u32 {
                self as u32
            }
        }

        impl From<u32> for $name {
            fn from(orig: u32) -> Self {
                match orig {
                    $(x if x == $name::$vn as u32 => $name::$vn,)*
                    _ => $name::Unknown
                }
            }
        }
    }
}

error_code_ext! {
    #[derive(Copy, Clone)]
    #[derive(PartialEq, Debug)]
    #[repr(u32)]
    pub enum ErrorCode {
        NotHandled = 0x01,
        AccessDenied = 0x02,
        Format = 0x03,
        Again = 0x04,
        OutOfBounds = 0x05,
        NotAvailable = 0x06,
        UnknownTag = 0x07,
        AlreadyInUse = 0x08,
        Unknown = 0xff
    }
}

/// Errors pubished by the package.
#[derive(Debug)] // Allow the use of "{:?}" format specifier
pub enum Errors {
    /// Error from Frame parser.
    Parse(String),
    /// If nothing is received.
    ReceiveNothing,
    /// Authentication failed.
    AuthFailed,
    /// If not connected.
    NotConnected,
}

impl std::error::Error for Errors {}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::Parse(ref msg) => write!(f, "Frame parse error: {}", msg),
            Errors::ReceiveNothing => write!(f, "Receive nothing"),
            Errors::AuthFailed => write!(f, "Authentication failed"),
            Errors::NotConnected => write!(f, "Not Connected"),
        }
    }
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_error_display_impl() {
    assert_eq!(
        format!("{}", Errors::Parse("test".to_string())),
        "Frame parse error: test"
    );
    assert_eq!(format!("{}", Errors::ReceiveNothing), "Receive nothing");
    assert_eq!(format!("{}", Errors::AuthFailed), "Authentication failed");
    assert_eq!(format!("{}", Errors::NotConnected), "Not Connected");
}

#[test]
fn test_error_code() {
    assert_eq!(
        ErrorCode::from(0x01u32),
        ErrorCode::NotHandled,
        "Test From<u32>"
    );
    assert_eq!(
        Into::<u32>::into(ErrorCode::NotHandled),
        0x01u32,
        "Test Into<u32>"
    );
    assert_eq!(
        ErrorCode::from(0xffffu32),
        ErrorCode::Unknown,
        "Test From Unknown<u32>"
    );

    let error_code = ErrorCode::from(0x01u32);

    let error_code_copy = error_code;
    assert_eq!(error_code_copy, ErrorCode::NotHandled, "Test copy");

    let error_code_clone = error_code.clone();
    assert_eq!(error_code_clone, ErrorCode::NotHandled, "Test clone");
}
