macro_rules! user_level_ext {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        /// Level of user
        $(#[$($attrs)*])*
        pub enum $name {
            $($vn = $v),+
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl From<u8> for $name {
            fn from(orig: u8) -> Self {
                match orig {
                    $(x if x == $name::$vn as u8 => $name::$vn,)*
                    _ => $name::Unknown
                }
            }
        }
    }
}

user_level_ext! {
    #[derive(Copy, Clone)]
    #[derive(PartialEq, Debug)]
    #[repr(u8)]
    pub enum UserLevel {
        NotAuthorized = 0,
        User = 10,
        Installer = 20,
        Service = 30,
        Admin = 40,
        E3dc = 50,
        E3dcRoot = 60,
        Unknown = 0xff
    }
}

/// ################################################
///      TEST TEST TEST
/// ################################################

#[test]
fn test_user_level() {
    assert_eq!(UserLevel::from(10), UserLevel::User, "Test From<u8>");
    assert_eq!(Into::<u8>::into(UserLevel::User), 10, "Test Into<u8>");
    assert_eq!(
        UserLevel::from(0xfe),
        UserLevel::Unknown,
        "Test From Unknown<u8>"
    );

    let user_level = UserLevel::from(10);

    let user_level_copy = user_level;
    assert_eq!(user_level_copy, UserLevel::User, "Test copy");

    let user_level_clone = user_level.clone();
    assert_eq!(user_level_clone, UserLevel::User, "Test clone");
}
