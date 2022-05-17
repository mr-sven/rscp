use std::fmt::{Display, Formatter, Result};

/// This macro is used in the tag group list enum to extend it with the tags function and from u8.
macro_rules! group_list {
    (
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        $(#[$($attrs)*])*
        pub enum $name {
            $($vn = $v),+
        }

        /// Returns the name of the selected enum by id
        /// 
        /// # Arguments
        /// 
        /// * `id` - the id of the tag
        /// 
        /// # Examples
        /// 
        /// ```
        /// let rscp_group = TagGroup::from(0x00);
        /// println!("{}", rscp_group.tags(0x00000004)); // USER_LEVEL
        /// ``` 
        impl $name {
            pub fn tags(&self, id: u32) -> String {
                match self {
                    $($name::$vn => $vn::from(id).to_string()),*
                }
            }
        }

        impl From<u8> for $name {
            fn from(orig: u8) -> Self {
                match orig {
                    $(x if x == $name::$vn as u8 => $name::$vn,)*
                    _ => $name::UNKNOWN
                }
            }
        }
    }
}

/// This macro is used to extend the tag group enum with to_string, from u32 and also adds the group
/// identifier to each enum value. Example, see enum `UNKNOWN`
macro_rules! group {
    (
        ($grp:expr),
        then $cb:tt,
        $(#[$($attrs:tt)*])*
        pub enum $name:ident { $($vn:ident = $v:tt),+ }
    ) => {
        macro_attr_callback! {
            $cb,
            $(#[$($attrs)*])*
            pub enum $name {
                $($vn = ($grp as u32) << 24 | $v),+
            }

            impl Display for $name {
                fn fmt(&self, f: &mut Formatter) -> Result {
                    match self {
                        $($name::$vn => write!(f, concat!(stringify!($name), "_", stringify!($vn)))),*
                    }
                }
            }

            impl From<u32> for $name {
                fn from(orig: u32) -> Self {
                    match orig {
                        $(x if x == $name::$vn as u32 => $name::$vn,)*
                        _ => $name::GENERAL_ERROR
                    }
                }
            }

            impl Into<u32> for $name {
                fn into(self) -> u32 {
                    self as u32
                }
            }
        }
    };
}

group_list! {
    /// List of all tag groups
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum TagGroup {
        RSCP = 0x00,
        EMS = 0x01,
        PVI = 0x02,
        BAT = 0x03,
        DCDC = 0x04,
        PM = 0x05,
        DB = 0x06,

        SRV = 0x08,
        HA = 0x09,
        INFO = 0x0a,
        EP = 0x0b,
        SYS = 0x0c,
        UM = 0x0d,
        WB = 0x0e,
        PTDB = 0x0f,
        LED = 0x10,
        DIAG = 0x11,
        SGR = 0x12,
        MBS = 0x13,
        EH = 0x14,
        UPNPC = 0x15,
        KNX = 0x16,
        EMSHB = 0x17,
        MYPV = 0x18,
        GPIO = 0x19,
        FARM = 0x1a,
        SE = 0x1b,
        QPI = 0x1c,
        GAPP = 0x1d,
        EMSPR = 0x1e,

        WBD = 0x20,
        REFU = 0x21,
        OVP = 0x22,

        SERVER = 0xf8,
        GROUP = 0xfc,       

        UNKNOWN = 0xff
    }
}

macro_attr! {
    /// Group of unknown results
    #[group!(TagGroup::UNKNOWN)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum UNKNOWN {
        GENERAL_ERROR = 0x7fffff
    }
}


macro_attr! {
    #[group!(TagGroup::RSCP)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum RSCP {
        AUTHENTICATION = 0x000001,
        AUTHENTICATION_USER = 0x000002,
        AUTHENTICATION_PASSWORD = 0x000003,
        USER_LEVEL = 0x000004,
        SET_ENCRYPTION_PASSPHRASE = 0x000005,
        AUTH_CHALLENGE = 0x000006,
        AUTH_CHALLENGE_INDEX = 0x000007,
        AUTH_CHALLENGE_DATA = 0x000008,
        SET_PROTOCOL_VERSION = 0x000009,
        SUPPORTED_PROTOCOL_VERSIONS = 0x00000a,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::EMS)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum EMS {
        POWER_PV = 0x000001,
        POWER_BAT = 0x000002,
        POWER_HOME = 0x000003,
        POWER_GRID = 0x000004,
        POWER_ADD = 0x000005,
        AUTARKY = 0x000006,
        SELF_CONSUMPTION = 0x000007,
        BAT_SOC = 0x000008,
        COUPLING_MODE = 0x000009,
        STORED_ERRORS = 0x00000a,
        ERROR_CONTAINER = 0x00000b,
        ERROR_TYPE = 0x00000c,
        ERROR_SOURCE = 0x00000d,
        ERROR_MESSAGE = 0x00000e,
        ERROR_CODE = 0x00000f,
        ERROR_TIMESTAMP = 0x000010,
        MODE = 0x000011,
        BALANCED_PHASES = 0x000012,
        INSTALLED_PEAK_POWER = 0x000013,
        DERATE_AT_PERCENT_VALUE = 0x000014,
        DERATE_AT_POWER_VALUE = 0x000015,
        ERROR_BUZZER_ENABLED = 0x000016,
        SET_BALANCED_PHASES = 0x000017,
        SET_INSTALLED_PEAK_POWER = 0x000018,
        SET_DERATE_PERCENT = 0x000019,
        SET_ERROR_BUZZER_ENABLED = 0x00001a,
        START_ADJUST_BATTERY_VOLTAGE = 0x00001b,
        CANCEL_ADJUST_BATTERY_VOLTAGE = 0x00001c,
        ADJUST_BATTERY_VOLTAGE_STATUS = 0x00001d,
        CONFIRM_ERRORS = 0x00001e,
        POWER_WB_ALL = 0x00001f,
        POWER_WB_SOLAR = 0x000020,
        EXT_SRC_AVAILABLE = 0x000021,
        RESCUE_BAT_MODE = 0x000022,
        REQ_SET_RESCUE_BAT_MODE = 0x000023,
        SET_POWER = 0x000030,
        REQ_SET_POWER_MODE = 0x000031,
        REQ_SET_POWER_VALUE = 0x000032,
        STATUS = 0x000040,
        USED_CHARGE_LIMIT = 0x000041,
        BAT_CHARGE_LIMIT = 0x000042,
        DCDC_CHARGE_LIMIT = 0x000043,
        USER_CHARGE_LIMIT = 0x000044,
        USED_DISCHARGE_LIMIT = 0x000045,
        BAT_DISCHARGE_LIMIT = 0x000046,
        DCDC_DISCHARGE_LIMIT = 0x000047,
        USER_DISCHARGE_LIMIT = 0x000048,
        SET_POWER_CONTROL_OFFSET = 0x000060,
        REMAINING_BAT_CHARGE_POWER = 0x000071,
        REMAINING_BAT_DISCHARGE_POWER = 0x000072,
        EMERGENCY_POWER_STATUS = 0x000073,
        SET_EMERGENCY_POWER = 0x000074,
        SET_OVERRIDE_AVAILABLE_POWER = 0x000075,
        SET_BATTERY_TO_CAR_MODE = 0x000076,
        BATTERY_TO_CAR_MODE = 0x000077,
        SET_BATTERY_BEFORE_CAR_MODE = 0x000078,
        BATTERY_BEFORE_CAR_MODE = 0x000079,
        GET_IDLE_PERIODS = 0x000080,
        SET_IDLE_PERIODS = 0x000081,
        IDLE_PERIOD = 0x000082,
        IDLE_PERIOD_TYPE = 0x000083,
        IDLE_PERIOD_DAY = 0x000084,
        IDLE_PERIOD_START = 0x000085,
        IDLE_PERIOD_END = 0x000086,
        IDLE_PERIOD_HOUR = 0x000087,
        IDLE_PERIOD_MINUTE = 0x000088,
        IDLE_PERIOD_ACTIVE = 0x000089,
        IDLE_PERIOD_CHANGE_MARKER = 0x00008a,
        GET_POWER_SETTINGS = 0x00008b,
        SET_POWER_SETTINGS = 0x00008c,
        SETTINGS_CHANGE_MARKER = 0x00008d,
        GET_MANUAL_CHARGE = 0x00008e,
        START_MANUAL_CHARGE = 0x00008f,
        START_EMERGENCYPOWER_TEST = 0x000090,
        GET_GENERATOR_STATE = 0x000091,
        SET_GENERATOR_MODE = 0x000092,
        EMERGENCYPOWER_TEST_STATUS = 0x000093,
        EPTEST_NEXT_TESTSTART = 0x000094,
        EPTEST_START_COUNTER = 0x000095,
        EPTEST_RUNNING = 0x000096,
        REQ_GET_SYS_SPECS = 0x000097,
        GET_SYS_SPECS = 0x000098,
        SYS_SPEC = 0x000099,
        SYS_SPEC_INDEX = 0x00009a,
        SYS_SPEC_NAME = 0x00009b,
        SYS_SPEC_VALUE_INT = 0x00009c,
        SYS_SPEC_VALUE_STRING = 0x00009d,
        SYS_STATUS = 0x00009e,
        POWER_LIMITS_USED = 0x000100,
        MAX_CHARGE_POWER = 0x000101,
        MAX_DISCHARGE_POWER = 0x000102,
        DISCHARGE_START_POWER = 0x000103,
        POWERSAVE_ENABLED = 0x000104,
        WEATHER_REGULATED_CHARGE_ENABLED = 0x000105,
        WEATHER_FORECAST_MODE = 0x000106,
        MANUAL_CHARGE_START_COUNTER = 0x000150,
        MANUAL_CHARGE_ACTIVE = 0x000151,
        MANUAL_CHARGE_ENERGY_COUNTER = 0x000152,
        MANUAL_CHARGE_LASTSTART = 0x000153,
        REMOTE_CONTROL = 0x000200,
        DEACTIVATE_REMOTE_CONTROL = 0x000201,
        IP_REMOTE_CONTROL = 0x000202,
        EP_DELAY = 0x000203,
        SET_EP_DELAY = 0x000204,
        REMOTE_CONTROL_STATUS = 0x000205,
        IDLE_PERIOD_MIN_SOC_UCB = 0x000206,
        IDLE_PERIOD_MAX_SOC_UCB = 0x000207,
        SET_IDLE_PERIOD_MIN_SOC_UCB = 0x000208,
        SET_IDLE_PERIOD_MAX_SOC_UCB = 0x000209,
        REGULATOR_MODE = 0x000210,
        SET_REGULATOR_MODE = 0x000211,
        SUPPORTED_REGULATOR_MODES = 0x000212,
        EMERGENCY_POWER_OVERLOAD_STATUS = 0x000213,
        EMERGENCY_POWER_RETRY = 0x000214,
        DETECT_PHASE_OFFSET = 0x000217,
        PHASE_DETECTION_STATUS = 0x000218,
        PHASE_OFFSET = 0x000219,
        ABORT_PHASE_DETECTION = 0x000220,
        REGULATOR_STRATEGY = 0x000221,
        SET_REGULATOR_STRATEGY = 0x000222,
        POWER_PV_AC_OUT = 0x000223,
        PV_ENERGY = 0x000224,
        PARAM_AC_ENERGY_OUT = 0x000225,
        PARAM_AC_ENERGY_IN = 0x000226,
        PARAM_DC_IN = 0x000227,
        ENERGY_STORAGE_MODEL = 0x000228,
        PARAM_CURR_CHARGED_ENERGY = 0x000229,
        PARAM_FULL_CHARGED_ENERGY_EP_RESERVE = 0x000230,
        PARAM_DESIGN_ENERGY = 0x000231,
        PARAM_FULL_CHARGED_ENERGY = 0x000232,
        PARAM_USED_CAPACITY = 0x000233,
        SPECIFICATION_VALUES = 0x000234,
        PARAM_MAX_CHARGE_POWER = 0x000235,
        PARAM_MAX_DISCHARGE_POWER = 0x000236,
        PARAM_MAX_PV_POWER = 0x000237,
        PARAM_MAX_AC_POWER = 0x000238,
        PARAM_INSTALLED_BAT_CAP = 0x000239,
        PARAM_HYBRIT_SUPPORTED = 0x000240,
        PARAM_INIT_STATUS = 0x000241,
        EP_RESERVE = 0x000242,
        SEC_LIMITS = 0x000243,
        PARAM_SEL_TOTAL_MAX = 0x000244,
        PARAM_SEL_TOTAL_MIN = 0x000245,
        PARAM_SEL_PHASE_MAX_L1 = 0x000246,
        PARAM_SEL_PHASE_MAX_L2 = 0x000247,
        PARAM_SEL_PHASE_MAX_L3 = 0x000248,
        PARAM_SEL_PHASE_MIN_L1 = 0x000249,
        PARAM_SEL_PHASE_MIN_L2 = 0x000250,
        PARAM_SEL_PHASE_MIN_L3 = 0x000251,
        SEC_DEVICE_STATUS = 0x000252,
        PARAM_PVI_1 = 0x000253,
        PARAM_PVI_2 = 0x000254,
        PARAM_PVI_3 = 0x000255,
        PARAM_DCDC = 0x000256,
        PARAM_BAT = 0x000257,
        BAT_CURRENT_IN = 0x000258,
        BAT_CURRENT_OUT = 0x000259,
        MAX_DC_POWER = 0x000260,
        AC_REACTIVE_POWER = 0x000261,
        REQ_SET_EP_PARTIAL_GRID = 0x000262,
        GET_PARTIAL_GRID = 0x000263,
        ESTIMATED_POWER_LIMITS = 0x000264,
        DESIGN_POWER_LIMITS = 0x000265,
        SET_CAN_ID_FEED_IN_REDUCTION = 0x000266,
        CAN_ID_FEED_IN_REDUCTION = 0x000267,
        SET_CAN_ID_UNBALANCED_LOAD = 0x000268,
        CAN_ID_UNBALANCED_LOAD = 0x000269,
        SET_WALLBOX_MODE = 0x000270,
        GET_WALLBOX_MODE = 0x000271,
        SET_MAX_FUSE_POWER = 0x000272,
        GET_MAX_FUSE_POWER = 0x000273,
        SET_CONNECTED_POWER = 0x000274,
        GET_CONNECTED_POWER = 0x000275,
        DERATE_AT_CONNECTED_POWER = 0x000276,
        SET_DERATE_AT_CONNECTED_POWER = 0x000277,
        WB_AVAILABLE = 0x000280,
        WB_PREFERRED_CHARGE_POWER = 0x000281,
        SET_PEAK_SHAVING_POWER = 0x000282,
        GET_PEAK_SHAVING_POWER = 0x000283,
        GET_RUNSCREENVALUES = 0x000284,
        SET_PEAK_SHAVING_TIMES = 0x000286,
        GET_PEAK_SHAVING_TIMES = 0x000287,
        SET_LIST_ACTOR = 0x000288,
        GET_LIST_ACTOR = 0x000289,
        ACTOR_ITEM = 0x000290,
        ACTOR_ID = 0x000291,
        ACTOR_NAME = 0x000292,
        ACTOR_PRIORITY = 0x000293,
        PERIOD_ITEM = 0x000300,
        PERIOD_ACTIVE = 0x000301,
        PERIOD_NAME = 0x000302,
        PERIOD_WEEKDAYS = 0x000303,
        PERIOD_START = 0x000304,
        PERIOD_STOP = 0x000305,
        PERIOD_POWER = 0x000306,
        PARAM_DERATE_POWER_VALUE = 0x040001,
        PARAM_AVAILABLE_POWER = 0x040002,
        PARAM_IP_REMOTE_CONTROL = 0x040004,
        PARAM_POWEROFFSET_VALUE = 0x040005,
        PARAM_POWER_VALUE_L1 = 0x040006,
        PARAM_POWER_VALUE_L2 = 0x040007,
        PARAM_POWER_VALUE_L3 = 0x040008,
        PARAM_SET_POINT = 0x040009,
        PARAM_DERATE_POWER_VALUE_L1 = 0x040010,
        PARAM_DERATE_POWER_VALUE_L2 = 0x040011,
        PARAM_DERATE_POWER_VALUE_L3 = 0x040012,
        PARAM_REMOTE_CONTROL_ACTIVE = 0x040013,
        PARAM_TIME_TO_RETRY = 0x040014,
        PARAM_NO_REMAINING_RETRY = 0x040015,
        PARAM_INDEX = 0x040016,
        PARAM_WALLBOX_SETPOINT_L1 = 0x040017,
        PARAM_WALLBOX_SETPOINT_L2 = 0x040018,
        PARAM_WALLBOX_SETPOINT_L3 = 0x040019,
        PARAM_DEACTIVATE_SURPLUS_ACTOR = 0x040115,
        ALIVE = 0x050000,
        PARAM_LIMITS_TOTAL_MAX = 0x400265,
        PARAM_LIMITS_TOTAL_MIN = 0x400266,
        PARAM_LIMITS_PHASE_MAX_L1 = 0x400267,
        PARAM_LIMITS_PHASE_MAX_L2 = 0x400268,
        PARAM_LIMITS_PHASE_MAX_L3 = 0x400269,
        PARAM_LIMITS_PHASE_MIN_L1 = 0x400270,
        PARAM_LIMITS_PHASE_MIN_L2 = 0x400271,
        PARAM_LIMITS_PHASE_MIN_L3 = 0x400272,
        PARAM_CURR_CHARGED_ENERGY_EP_RESERVE = 0x400278,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::PVI)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum PVI {
        ON_GRID = 0x000001,
        STATE = 0x000002,
        LAST_ERROR = 0x000003,
        IS_FLASHING = 0x000004,
        REQ_START_FLASHING = 0x000005,
        FLASH_FILE_LIST = 0x000006,
        FLASH_FILE = 0x000007,
        SERVICE_PROGRESS_STATE = 0x000008,
        TYPE = 0x000009,
        LAND_CODE = 0x000010,
        LAND_CODE_LIST = 0x000011,
        REQ_SET_LAND_CODE = 0x000012,
        ERROR_STRING = 0x000013,
        ERROR_LIST = 0x000014,
        STATUS_STRING = 0x000015,
        STATUS_LIST = 0x000016,
        REQ_SET_DEVICE_SILENCE = 0x000017,
        DEVICE_SILENCE = 0x000018,
        SELF_TEST = 0x000019,
        UZK_VOLTAGE = 0x000050,
        COS_PHI = 0x000060,
        REQ_SET_COS_PHI = 0x000061,
        COS_PHI_VALUE = 0x000062,
        COS_PHI_IS_AKTIV = 0x000063,
        COS_PHI_EXCITED = 0x000064,
        VOLTAGE_MONITORING = 0x000070,
        REQ_SET_VOLTAGE_MONITORING = 0x000071,
        VOLTAGE_MONITORING_THRESHOLD_TOP = 0x000072,
        VOLTAGE_MONITORING_THRESHOLD_BOTTOM = 0x000073,
        VOLTAGE_MONITORING_SLOPE_UP = 0x000074,
        VOLTAGE_MONITORING_SLOPE_DOWN = 0x000075,
        FREQUENCY_UNDER_OVER = 0x000080,
        SET_FREQUENCY_UNDER_OVER = 0x000081,
        FREQUENCY_UNDER = 0x000082,
        FREQUENCY_OVER = 0x000083,
        SET_SYSTEM_MODE = 0x000084,
        SYSTEM_MODE = 0x000085,
        SET_POWER_MODE = 0x000086,
        POWER_MODE = 0x000087,
        USED_STRING_COUNT = 0x000090,
        REQ_SET_USED_STRING_COUNT = 0x000091,
        DERATE_TO_POWER = 0x000092,
        TEMPERATURE = 0x000100,
        TEMPERATURE_COUNT = 0x000101,
        MAX_TEMPERATURE = 0x000102,
        MIN_TEMPERATURE = 0x000103,
        CT_TAR_USV_BOX = 0x000104,
        DATA = 0x040000,
        INDEX = 0x040001,
        VALUE = 0x040005,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        SERIAL_NUMBER = 0x0abc01,
        VERSION = 0x0abc02,
        VERSION_MAIN = 0x0abc03,
        VERSION_PIC = 0x0abc04,
        VERSION_ALL = 0x0abc05,
        REQ_SET_RESET_GPIO = 0x0abc50,
        REQ_SET_POWEROFF_GPIO = 0x0abc51,
        REQ_SET_NIGHTSWITCH_GPIO = 0x0abc52,
        REQ_SET_PRE_GRID_CHARGE = 0x0abc60,
        AC_MAX_PHASE_COUNT = 0x0ac000,
        AC_POWER = 0x0ac001,
        AC_VOLTAGE = 0x0ac002,
        AC_CURRENT = 0x0ac003,
        AC_APPARENTPOWER = 0x0ac004,
        AC_REACTIVEPOWER = 0x0ac005,
        AC_ENERGY_ALL = 0x0ac006,
        AC_MAX_APPARENTPOWER = 0x0ac007,
        AC_ENERGY_DAY = 0x0ac008,
        AC_ENERGY_GRID_CONSUMPTION = 0x0ac009,
        DC_MAX_STRING_COUNT = 0x0dc000,
        DC_POWER = 0x0dc001,
        DC_VOLTAGE = 0x0dc002,
        DC_CURRENT = 0x0dc003,
        DC_MAX_POWER = 0x0dc004,
        DC_MAX_VOLTAGE = 0x0dc005,
        DC_MIN_VOLTAGE = 0x0dc006,
        DC_MAX_CURRENT = 0x0dc007,
        DC_MIN_CURRENT = 0x0dc008,
        DC_STRING_ENERGY_ALL = 0x0dc009,
        AC_ENERGY_PRODUCED_L1 = 0x0dc00a,
        AC_ENERGY_PRODUCED_L2 = 0x0dc00b,
        AC_ENERGY_PRODUCED_L3 = 0x0dc00c,
        AC_ENERGY_CONSUMED_L1 = 0x0dc00d,
        AC_ENERGY_CONSUMED_L2 = 0x0dc00e,
        AC_ENERGY_CONSUMED_L3 = 0x0dc00f,
        REQ_ENABLE_FAN_TEST = 0x0dc010,
        REQ_DISABLE_FAN_TEST = 0x0dc011,
        RESET_LAND_NORM = 0x0dc012,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::BAT)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum BAT {
        RSOC = 0x000001,
        MODULE_VOLTAGE = 0x000002,
        CURRENT = 0x000003,
        MAX_BAT_VOLTAGE = 0x000004,
        MAX_CHARGE_CURRENT = 0x000005,
        EOD_VOLTAGE = 0x000006,
        MAX_DISCHARGE_CURRENT = 0x000007,
        CHARGE_CYCLES = 0x000008,
        TERMINAL_VOLTAGE = 0x000009,
        STATUS_CODE = 0x00000a,
        ERROR_CODE = 0x00000b,
        DEVICE_NAME = 0x00000c,
        DCB_COUNT = 0x00000d,
        RSOC_REAL = 0x00000e,
        ASOC = 0x00000f,
        FCC = 0x000010,
        RC = 0x000011,
        MAX_DCB_CELL_CURRENT = 0x000012,
        MIN_DCB_CELL_CURRENT = 0x000013,
        MAX_DCB_CELL_VOLTAGE = 0x000014,
        MIN_DCB_CELL_VOLTAGE = 0x000015,
        MAX_DCB_CELL_TEMPERATURE = 0x000016,
        MIN_DCB_CELL_TEMPERATURE = 0x000017,
        DCB_ALL_CELL_TEMPERATURES = 0x000018,
        DCB_CELL_TEMPERATURE = 0x000019,
        DCB_ALL_CELL_VOLTAGES = 0x00001a,
        DCB_CELL_VOLTAGE = 0x00001b,
        OPEN_BREAKER = 0x00001c,
        OPEN_BREAKER_CONFIRM = 0x00001d,
        READY_FOR_SHUTDOWN = 0x00001e,
        FIRMWARE_VERSION = 0x00001f,
        INFO = 0x000020,
        TRAINING_MODE = 0x000021,
        UPDATE_STATUS = 0x000022,
        REQ_SET_TRAINING_MODE = 0x000023,
        TIME_LAST_RESPONSE = 0x000024,
        MANUFACTURER_NAME = 0x000025,
        USABLE_CAPACITY = 0x000026,
        USABLE_REMAINING_CAPACITY = 0x000027,
        SET_A1_DATA = 0x000028,
        REQ_SET_A1_MODE = 0x000029,
        REQ_SET_A1_VOLTAGE = 0x000030,
        REQ_SET_A1_CURRENT = 0x000031,
        CONTROL_CODE = 0x000032,
        BPM_STATUS = 0x000033,
        DCB_ERROR_LIST = 0x000034,
        DCB_INFO = 0x000042,
        SPECIFICATION = 0x000043,
        INTERNALS = 0x000044,
        DESIGN_CAPACITY = 0x000045,
        DESIGN_VOLTAGE = 0x000046,
        CHARGE_HIGH_TEMP = 0x000047,
        CHARGE_LOW_TEMP = 0x000048,
        MANUFACTURE_DATE = 0x000049,
        SERIALNO = 0x000050,
        DATA_TABLE_VERSION = 0x000051,
        PROTOCOL_VERSION = 0x000052,
        PCB_VERSION = 0x000053,
        TOTAL_USE_TIME = 0x000054,
        TOTAL_DISCHARGE_TIME = 0x000055,
        REQ_AVAILABLE_BATTERIES = 0x000056,
        AVAILABLE_BATTERIES = 0x000057,
        BATTERY_SPEC = 0x000058,
        INSTANCE_DESCRIPTOR = 0x000059,
        REQ_OPEN_FET = 0x000060,
        FET_STATE = 0x000061,
        BATTERY_SOFT_ON = 0x000062,
        SET_BAT_VOLT_ADJUSTMENT = 0x000063,
        BAT_VOLT_ADJUSTMENT = 0x000064,
        BAT_VOLT_ADJ_READY_INDEX = 0x000065,
        DISCHARGE_UNTIL_EMPTY_STATE = 0x000092,
        SET_DISCHARGE_UNTIL_EMPTY = 0x000094,
        CONTROL_STATE = 0x000095,
        INTERNAL_STATE = 0x000096,
        IS_BREAKER_OPEN = 0x000097,
        CLOSE_BREAKER = 0x000098,
        DCB_INDEX = 0x000100,
        DCB_LAST_MESSAGE_TIMESTAMP = 0x000101,
        DCB_MAX_CHARGE_VOLTAGE = 0x000102,
        DCB_MAX_CHARGE_CURRENT = 0x000103,
        DCB_END_OF_DISCHARGE = 0x000104,
        DCB_MAX_DISCHARGE_CURRENT = 0x000105,
        DCB_FULL_CHARGE_CAPACITY = 0x000106,
        DCB_REMAINING_CAPACITY = 0x000107,
        DCB_SOC = 0x000108,
        DCB_SOH = 0x000109,
        DCB_CYCLE_COUNT = 0x000110,
        DCB_CURRENT = 0x000111,
        DCB_VOLTAGE = 0x000112,
        DCB_CURRENT_AVG_30S = 0x000113,
        DCB_VOLTAGE_AVG_30S = 0x000114,
        DCB_DESIGN_CAPACITY = 0x000115,
        DCB_DESIGN_VOLTAGE = 0x000116,
        DCB_CHARGE_LOW_TEMPERATURE = 0x000117,
        DCB_CHARGE_HIGH_TEMPERATURE = 0x000118,
        DCB_MANUFACTURE_DATE = 0x000119,
        DCB_SERIALNO = 0x000120,
        DCB_PROTOCOL_VERSION = 0x000121,
        DCB_FW_VERSION = 0x000122,
        DCB_DATA_TABLE_VERSION = 0x000123,
        DCB_PCB_VERSION = 0x000124,
        SPECIFIED_CAPACITY = 0x000125,
        SPECIFIED_DSCHARGE_POWER = 0x000126,
        SPECIFIED_CHARGE_POWER = 0x000127,
        SPECIFIED_MAX_DCB_COUNT = 0x000128,
        ROLE = 0x000129,
        INTERNAL_CURRENT_AVG30 = 0x000130,
        INTERNAL_MTV_AVG30 = 0x000131,
        INTERNAL_MAX_CHARGE_CURRENT = 0x000132,
        INTERNAL_MAX_DISCHARGE_CURRENT = 0x000133,
        INTERNAL_MAX_CHARGE_CURR_PER_DCB = 0x000134,
        INTERNAL_MAX_DISCHARGE_CURR_PER_DCB = 0x000135,
        INTERNAL_MAX_CHARGE_CURR_DATA_LOG = 0x000136,
        INTERNAL_MAX_DISCHARGE_CURR_DATA_LOG = 0x000137,
        DCB_NR_SERIES_CELL = 0x000300,
        DCB_NR_PARALLEL_CELL = 0x000301,
        DCB_MANUFACTURE_NAME = 0x000302,
        DCB_DEVICE_NAME = 0x000303,
        DCB_SERIALCODE = 0x000304,
        DCB_NR_SENSOR = 0x000305,
        DCB_STATUS = 0x000306,
        DCB_WARNING = 0x000307,
        DCB_ALARM = 0x000308,
        DCB_ERROR = 0x000309,
        DATA = 0x040000,
        INDEX = 0x040001,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        PARAM_BAT_VOLT_STATUS = 0x400001,
        PARAM_BAT_VOLT_TARGET_VALUE = 0x400002,
        PARAM_BAT_VOLT_MIN_VOLTAGE = 0x400003,
        PARAM_BAT_VOLT_MAX_VOLTAGE = 0x400004,
        PARAM_BAT_VOLT_ENABLED = 0x400005,
        PARAM_BAT_NUMBER = 0x400006,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::DCDC)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum DCDC {
        I_BAT = 0x000001,
        U_BAT = 0x000002,
        P_BAT = 0x000003,
        I_DCL = 0x000004,
        U_DCL = 0x000005,
        P_DCL = 0x000006,
        SELF_TEST = 0x000007,
        FIRMWARE_VERSION = 0x000008,
        FPGA_FIRMWARE = 0x000009,
        SERIAL_NUMBER = 0x00000a,
        BOARD_VERSION = 0x00000b,
        FLASH_FILE_LIST = 0x00000c,
        FLASH_FILE = 0x00000d,
        IS_FLASHING = 0x00000e,
        FLASH = 0x00000f,
        STATUS = 0x000010,
        STATE = 0x000011,
        SUBSTATE = 0x000012,
        STATUS_AS_STRING = 0x000013,
        STATE_AS_STRING = 0x000014,
        SUBSTATE_AS_STRING = 0x000015,
        VERIFY_CORTEX = 0x000021,
        FLASH_FPGA = 0x000022,
        FLASH_FPGA_FILE_LIST = 0x000023,
        SELF_TEST_RESULT = 0x000024,
        FLASH_STATUS = 0x000025,
        GET_PARAMETER = 0x000026,
        SET_PARAMETER = 0x000027,
        REQ_SET_PID_DEBUG = 0x000028,
        COPY_RING_BUFFER = 0x000029,
        RING_BUFFER = 0x000030,
        FREED_RING_BUFFER = 0x000031,
        REQ_DCL_OPERATION_VOLTAGE = 0x000050,
        DCL_OPERATION_VOLTAGE = 0x000051,
        SET_POWER = 0x000071,
        SET_IDLE = 0x000072,
        HANDLE_ERRORS = 0x000073,
        CLEAR_ERRORS = 0x000074,
        SEND_COMMAND = 0x000075,
        BROADCAST_COMMAND = 0x000076,
        ERROR_PENDING = 0x000077,
        REQ_SET_PVI_TYPE = 0x000078,
        PVI_TYPE = 0x000079,
        ON_GRID = 0x000080,
        REQ_SET_ON_GRID = 0x000081,
        NEXT_SLAVE_STATE = 0x000082,
        REQ_ENABLE_NEXT_SLAVE = 0x000083,
        DCDC_TYPE = 0x000084,
        SEND_KICKSTART = 0x000085,
        DATA = 0x040000,
        INDEX = 0x040001,
        PARAM_FLASH_PROGRESS = 0x040010,
        PARAM_FLASH_TYPE = 0x040011,
        PARAM_FLASHING_ACTIVE = 0x040012,
        PARAM_FLASH_MODE = 0x040013,
        PARAM_FLASH_FILE = 0x040014,
        PARAM_CRC = 0x040015,
        PARAM_PARAMETER_BLOCK = 0x040016,
        PARAM_PARAMETER_INDEX_FROM = 0x040017,
        PARAM_PARAMETER_INDEX_UNTIL = 0x040018,
        PARAM_PARAMETER_VALUE = 0x040019,
        PARAM_RING_BUFFER_ELEMENT = 0x040030,
        PARAM_RB_ID = 0x040031,
        PARAM_RB_TIME = 0x040032,
        PARAM_RB_I_BAT = 0x040033,
        PARAM_RB_U_BAT = 0x040034,
        PARAM_RB_I_DCL = 0x040035,
        PARAM_RB_U_DCL = 0x040036,
        PARAM_RB_MODE = 0x040037,
        PARAM_RB_SUBSTATE = 0x040038,
        PARAM_RB_SETPOINT = 0x040039,
        PARAM_RB_INDEX_DCDC = 0x040040,
        PARAM_RB_INDEX_FROM = 0x040041,
        PARAM_RB_INDEX_UNTIL = 0x040042,
        PARAM_DCL_OV_UPPER_VOLTAGE = 0x040050,
        PARAM_DCL_OV_LOWER_VOLTAGE = 0x040051,
        PARAM_DCL_OV_INDEX = 0x040052,
        COUNT_HW_CONTROLLER = 0x040060,
        REQ_ENABLE_FAN_TEST = 0x040061,
        REQ_DISABLE_FAN_TEST = 0x040062,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::PM)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum PM {
        POWER_L1 = 0x000001,
        POWER_L2 = 0x000002,
        POWER_L3 = 0x000003,
        ACTIVE_PHASES = 0x000004,
        MODE = 0x000005,
        ENERGY_L1 = 0x000006,
        ENERGY_L2 = 0x000007,
        ENERGY_L3 = 0x000008,
        DEVICE_ID = 0x000009,
        ERROR_CODE = 0x00000a,
        SET_PHASE_ELIMINATION = 0x00000b,
        FIRMWARE_VERSION = 0x00000c,
        SET_FOR_EMERGENCY_TEST = 0x00000d,
        IS_CAN_SILENCE = 0x00000e,
        MAX_PHASE_POWER = 0x00000f,
        VOLTAGE_L1 = 0x000011,
        VOLTAGE_L2 = 0x000012,
        VOLTAGE_L3 = 0x000013,
        TYPE = 0x000014,
        REQ_SET_TYPE = 0x000015,
        GET_PHASE_ELIMINATION = 0x000018,
        COMM_STATE = 0x000050,
        CS_START_TIME = 0x000051,
        CS_LAST_TIME = 0x000052,
        CS_SUCC_FRAMES_ALL = 0x000053,
        CS_SUCC_FRAMES_100 = 0x000054,
        CS_EXP_FRAMES_ALL = 0x000055,
        CS_EXP_FRAMES_100 = 0x000056,
        CS_ERR_FRAMES_ALL = 0x000057,
        CS_ERR_FRAMES_100 = 0x000058,
        CS_UNK_FRAMES = 0x000059,
        CS_ERR_FRAME = 0x00005a,
        CONNECTED_DEVICES = 0x0000a0,
        CONNECTED_DEVICE = 0x0000a1,
        REQ_SET_DEVICE_NAME = 0x0000b0,
        DEVICE_NAME = 0x0000b1,
        SET_EXTERN_LOAD = 0x0000b3,
        INJECT_DATA = 0x0000b4,
        DATA = 0x040000,
        INDEX = 0x040001,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::DB)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum DB {
        GRAPH_INDEX = 0x000001,
        BAT_POWER_IN = 0x000002,
        BAT_POWER_OUT = 0x000003,
        DC_POWER = 0x000004,
        GRID_POWER_IN = 0x000005,
        GRID_POWER_OUT = 0x000006,
        CONSUMPTION = 0x000007,
        PM_0_POWER = 0x000008,
        PM_1_POWER = 0x000009,
        BAT_CHARGE_LEVEL = 0x00000a,
        BAT_CYCLE_COUNT = 0x00000b,
        CONSUMED_PRODUCTION = 0x00000c,
        AUTARKY = 0x00000d,
        PRODUCTION_POWER = 0x00000e,
        SUM_CONTAINER = 0x000010,
        VALUE_CONTAINER = 0x000020,
        HISTORY_DATA_DAY = 0x000100,
        SET_IDLE = 0x000101,
        IS_IDLE = 0x000102,
        ENERGY_COUNTERS = 0x000103,
        REQ_HISTORY_UTC_TIME_START = 0x000104,
        HISTORY_DATA_WEEK = 0x000200,
        HISTORY_DATA_MONTH = 0x000300,
        HISTORY_DATA_YEAR = 0x000400,
        SYNC_HIST = 0x000500,
        VACUUM_HIST = 0x000501,
        SYNC_BPU = 0x000502,
        VACUUM_BPU = 0x000503,
        SYNC_DCB = 0x000504,
        VACUUM_DCB = 0x000505,
        SYNC_BPU_CONF = 0x000506,
        VACUUM_BPU_CONF = 0x000507,
        SYNC_DCB_CONF = 0x000508,
        VACUUM_DCB_CONF = 0x000509,
        SYNC_WALLBOX = 0x00050a,
        VACUUM_WALLBOX = 0x00050b,
        SYNC_PV_DEBUG = 0x00050c,
        VACUUM_PV_DEBUG = 0x00050d,
        SYNC_CONFIG = 0x00050e,
        VACUUM_CONFIG = 0x00050f,
        SET_SYNC_TIME = 0x000510,
        PVI_DIAL_RECORDINGS = 0x000511,
        SYNC_BAT_DIAGNOSE = 0x000512,
        VACUUM_BAT_DIAGNOSE = 0x000513,
        SYNC_EXT_LG = 0x000514,
        VACUUM_EXT_LG = 0x000515,
        REQ_CLEAN_DATABASE = 0x000516,
        SYNC_ALL = 0x000517,
        PAR_TIME_MIN = 0x300000,
        PAR_TIME_MAX = 0x300001,
        PARAM_ROW = 0x300002,
        PARAM_COLUMN = 0x300003,
        PARAM_INDEX = 0x300004,
        PARAM_VALUE = 0x300005,
        PARAM_MAX_ROWS = 0x300006,
        PARAM_TIME = 0x300007,
        PARAM_VERSION = 0x300008,
        PARAM_HEADER = 0x300009,
        PARAM_PRODUCTION_L1 = 0x300010,
        PARAM_PRODUCTION_L2 = 0x300011,
        PARAM_PRODUCTION_L3 = 0x300012,
        PARAM_DC_POWER_S1 = 0x300013,
        PARAM_DC_POWER_S2 = 0x300014,
        PARAM_DC_POWER_S3 = 0x300015,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::SRV)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum SRV {
        IS_ONLINE = 0x000001,
        ADD_USER = 0x000002,
        SET_LOCAL_USER = 0x000003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::HA)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum HA {
        DATAPOINT_LIST = 0x000001,
        DATAPOINT = 0x000002,
        DATAPOINT_INDEX = 0x000003,
        DATAPOINT_TYPE = 0x000004,
        DATAPOINT_NAME = 0x000005,
        DATAPOINT_DESCRIPTIONS = 0x000006,
        DATAPOINT_DESCRIPTION = 0x000007,
        DATAPOINT_DESCRIPTION_NAME = 0x000008,
        DATAPOINT_DESCRIPTION_VALUE = 0x000009,
        ACTUATOR_STATES = 0x000010,
        DATAPOINT_STATE = 0x000011,
        DATAPOINT_MODE = 0x000012,
        DATAPOINT_STATE_TIMESTAMP = 0x000013,
        DATAPOINT_STATE_VALUE = 0x000014,
        DATAPOINT_SUPPLY_QUALITY = 0x000015,
        DATAPOINT_SIGNAL_QUALITY = 0x000016,
        ADD_ACTUATOR = 0x000020,
        REMOVE_ACTUATOR = 0x000030,
        COMMAND_ACTUATOR = 0x000040,
        REQ_COMMAND = 0x000041,
        DESCRIPTIONS_CHANGE = 0x000050,
        CONFIGURATION_CHANGE_COUNTER = 0x000060,
        POSSIBLE_POWER_METERS = 0x000070,
        POSSIBLE_POWER_METER = 0x000071,
        POSSIBLE_ANALOG_MODES = 0x000080,
        POSSIBLE_ANALOG_MODE = 0x000081,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::INFO)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum INFO {
        SERIAL_NUMBER = 0x000001,
        PRODUCTION_DATE = 0x000002,
        MODULES_SW_VERSIONS = 0x000003,
        MODULE_SW_VERSION = 0x000004,
        MODULE = 0x000005,
        VERSION = 0x000006,
        A35_SERIAL_NUMBER = 0x000007,
        IP_ADDRESS = 0x000008,
        SUBNET_MASK = 0x000009,
        MAC_ADDRESS = 0x00000a,
        GATEWAY = 0x00000b,
        DNS = 0x00000c,
        DHCP_STATUS = 0x00000d,
        TIME = 0x00000e,
        UTC_TIME = 0x00000f,
        TIME_ZONE = 0x000010,
        INFO = 0x000011,
        SET_IP_ADDRESS = 0x000012,
        SET_SUBNET_MASK = 0x000013,
        SET_DHCP_STATUS = 0x000014,
        SET_GATEWAY = 0x000015,
        SET_DNS = 0x000016,
        SET_TIME = 0x000017,
        SET_TIME_ZONE = 0x000018,
        SW_RELEASE = 0x000019,
        SET_GUI_TARGET = 0x00001a,
        GUI_TARGET = 0x00001b,
        PLATFORM_TYPE = 0x00001c,
        IS_CALIBRATED = 0x00001d,
        CALIBRATION_CHECK = 0x00001e,
        RESET_CALIBRATION = 0x00001f,
        HW_TIME = 0x000020,
        SET_TIME_UTC = 0x000021,
        SET_HW_TIME = 0x000022,
        SET_FACILITY = 0x000023,
        GET_FACILITY = 0x000024,
        NAME = 0x000025,
        STREET = 0x000026,
        STREET_NO = 0x000027,
        POSTCODE = 0x000028,
        CITY = 0x000029,
        FON = 0x00002a,
        E_MAIL = 0x00002b,
        COUNTRY = 0x00002c,
        GET_FS_USAGE = 0x00002d,
        FS_SIZE = 0x00002e,
        FS_USED = 0x00002f,
        FS_AVAILABLE = 0x000030,
        FS_USE_PERCENT = 0x000031,
        INODES = 0x000032,
        INODES_USED = 0x000033,
        INODES_AVAILABLE = 0x000034,
        INODES_USE_PERCENT = 0x000035,
        SURNAME = 0x000036,
        UPNP_STATUS = 0x000037,
        SET_UPNP_STATUS = 0x000038,
        IS_OVP_POSSIBLE = 0x000039,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::EP)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum EP {
        SWITCH_TO_GRID = 0x000001,
        SWITCH_TO_ISLAND = 0x000002,
        IS_READY_FOR_SWITCH = 0x000003,
        IS_GRID_CONNECTED = 0x000004,
        IS_ISLAND_GRID = 0x000005,
        IS_INVALID_STATE = 0x000006,
        IS_POSSIBLE = 0x000007,
        LEAVE_INVALID_STATE_TO_ISLAND = 0x000008,
        LEAVE_INVALID_STATE_TO_GRID = 0x000009,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::SYS)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum SYS {
        SYSTEM_REBOOT = 0x000001,
        IS_SYSTEM_REBOOTING = 0x000002,
        RESTART_APPLICATION = 0x000003,
        SCRIPT_FILE_LIST = 0x000010,
        SCRIPT_FILE = 0x000011,
        EXECUTE_SCRIPT = 0x000015,
        REQ_SYSTEM_SHUTDOWN = 0x000016,
        IS_SYSTEM_SHUTING_DOWN = 0x000017,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::UM)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum UM {
        UPDATE_STATUS = 0x000001,
        UPDATE_DCDC = 0x000002,
        CHECK_FOR_UPDATES = 0x000003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::WB)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum WB {
        ENERGY_ALL = 0x000001,
        ENERGY_SOLAR = 0x000002,
        SOC = 0x000003,
        STATUS = 0x000004,
        ERROR_CODE = 0x000005,
        MODE = 0x000006,
        APP_SOFTWARE = 0x000007,
        BOOTLOADER_SOFTWARE = 0x000008,
        HW_VERSION = 0x000009,
        FLASH_VERSION = 0x00000a,
        DEVICE_ID = 0x00000b,
        PM_POWER_L1 = 0x00000c,
        PM_POWER_L2 = 0x00000d,
        PM_POWER_L3 = 0x00000e,
        PM_ACTIVE_PHASES = 0x00000f,
        PM_MODE = 0x000011,
        PM_ENERGY_L1 = 0x000012,
        PM_ENERGY_L2 = 0x000013,
        PM_ENERGY_L3 = 0x000014,
        PM_DEVICE_ID = 0x000015,
        PM_ERROR_CODE = 0x000016,
        PM_FIRMWARE_VERSION = 0x000017,
        DIAG_DEVICE_ID = 0x000018,
        DIAG_BAT_CAPACITY = 0x000019,
        DIAG_USER_PARAM = 0x00001a,
        DIAG_MAX_CURRENT = 0x00001b,
        DIAG_PHASE_VOLTAGE = 0x00001c,
        DIAG_DISPLAY_SPEECH = 0x00001d,
        DIAG_DESIGN = 0x00001e,
        DIAG_INFOS = 0x00001f,
        DIAG_WARNINGS = 0x000020,
        DIAG_ERRORS = 0x000021,
        DIAG_TEMP_1 = 0x000022,
        DIAG_TEMP_2 = 0x000023,
        DIAG_CP_PEGEL = 0x000024,
        DIAG_PP_IN_A = 0x000025,
        DIAG_STATUS_DIODE = 0x000026,
        DIAG_DIG_IN_1 = 0x000027,
        DIAG_DIG_IN_2 = 0x000028,
        PM_DEVICE_STATE = 0x000029,
        PM_DEVICE_STATE_CONNECTED = 0x000030,
        SET_MODE = 0x000031,
        PM_DEVICE_STATE_IN_SERVICE = 0x000032,
        PM_MAX_PHASE_POWER = 0x000040,
        REQ_SET_DEVICE_NAME = 0x000041,
        DEVICE_NAME = 0x000042,
        DATA = 0x040000,
        INDEX = 0x040001,
        MODE_PARAM_MODE = 0x040031,
        MODE_PARAM_MAX_CURRENT = 0x040032,
        AVAILABLE_SOLAR_POWER = 0x041000,
        POWER = 0x041001,
        STATUS_BIT = 0x041002,
        SET_EXTERN = 0x041010,
        EXTERN_DATA_SUN = 0x041011,
        EXTERN_DATA_NET = 0x041012,
        EXTERN_DATA_ALL = 0x041013,
        EXTERN_DATA_ALG = 0x041014,
        SET_BAT_CAPACITY = 0x041015,
        SET_ENERGY_ALL = 0x041016,
        SET_ENERGY_SOLAR = 0x041017,
        SET_PARAM_1 = 0x041018,
        SET_PARAM_2 = 0x041019,
        RSP_PARAM_2 = 0x04101a,
        RSP_PARAM_1 = 0x04101b,
        CONNECTED_DEVICES = 0x04101c,
        SET_SOC = 0x04101d,
        STATION_AVAILABLE = 0x04101e,
        SET_STATION_AVAILABLE = 0x04101f,
        SET_PW = 0x041020,
        SET_STATION_ENABLED = 0x041021,
        MAC_ADDRESS = 0x041022,
        PROXIMITY_PLUG = 0x041023,
        REQ_PREFERRED_CHARGE_POWER = 0x041024,
        CHARGE_FULL = 0x041025,
        SET_CHARGE_FULL = 0x041026,
        ACTIVE_CHARGE_STRATEGY = 0x041027,
        SET_ACTIVE_CHARGE_STRAGETY = 0x041028,
        PARAMETER_LIST = 0x041029,
        STATION_ENABLED = 0x04102a,
        SET_PARAMETER_LIST = 0x041030,
        GATEWAY = 0x041031,
        SUBNET_MASK = 0x041032,
        IP_ADDR = 0x041033,
        DHCP_ENABLED = 0x041034,
        SET_DHCP_ENABLED = 0x041035,
        WALLBOX_TYPE = 0x041036,
        UPDATE_NETWORK_CONFIG = 0x041037,
        SUN_MODE_ACTIVE = 0x041038,
        SET_SUN_MODE_ACTIVE = 0x041039,
        NUMBER = 0x04103a,
        NUMBER_PHASES = 0x04103b,
        SET_NUMBER_PHASES = 0x04103c,
        ABORT_CHARGING = 0x04103d,
        REQ_SET_ABORT_CHARGING = 0x04103e,
        SET_ABORT_CHARGING = 0x04103f,
        SHUKO_AVAILABLE = 0x041040,
        IS_SHUKO_LOCKED = 0x041041,
        SET_SHUKO_LOCKED = 0x041042,
        MAX_POWER_PER_PHASE = 0x041043,
        MIN_POWER_PER_PHASE = 0x041044,
        UPPER_CURRENT_LITMIT = 0x041045,
        LOWER_CURRENT_LITMIT = 0x041046,
        MAX_CHARGE_CURRENT = 0x041047,
        MIN_CHARGE_CURRENT = 0x041048,
        SET_MAX_CHARGE_CURRENT = 0x041049,
        SET_MIN_CHARGE_CURRENT = 0x04104a,
        PARAM_INDEX = 0x04104b,
        CHARGE_STOP_HYSTERESIS = 0x04104c,
        SET_CHARGE_STOP_HYSTERESIS = 0x04104d,
        GET_KEY_LOCK_MODE = 0x04104e,
        SET_KEY_LOCK_MODE = 0x04104f,
        KEY_STATE = 0x041050,
        SERIAL = 0x041051,
        MAX_CHARGE_POWER = 0x041052,
        MIN_CHARGE_POWER = 0x041053,
        EXTERN_DATA = 0x042010,
        EXTERN_DATA_LEN = 0x042011,
        PARAM_USR = 0x042012,
        PARAM_PW = 0x042013,
        DEVICE_STATE = 0x060000,
        DEVICE_CONNECTED = 0x060001,
        DEVICE_WORKING = 0x060002,
        DEVICE_IN_SERVICE = 0x060003,
        SET_BIC_MODE = 0x0f0001,
        GET_BIC_MODE = 0x0f0002,
        GET_CHARGE_PLAN_TEXT = 0x0f0003,
        STRING_PARAMETER = 0x440010,
        PREFERRED_CHARGE_POWER = 0x741024,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::PTDB)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum PTDB {
        SET_STD_PROPS = 0x000001,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::LED)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum LED {
        SET_BAR_SWITCHED_ON_STATE = 0x000001,
        BAR_SWITCHED_ON_STATE = 0x000002,
        INTENSITY = 0x000003,
        SET_INTENSITY = 0x000004,
        COLOR = 0x000005,
        SET_COLOR = 0x000006,
        HW_INFO = 0x000007,
        REQ_STORE_CONFIG = 0x000008,
        CONFIG_STORED = 0x000009,
        DEVICE_STATE = 0x060000,
        INDEX = 0x060001,
        RED = 0x060002,
        GREEN = 0x060003,
        BLUE = 0x060004,
        FW_VERSION = 0x060005,
        BL_VERSION = 0x060006,
        DEVICE_CONNECTED = 0x060007,
        DEVICE_WORKING = 0x060008,
        DEVICE_IN_SERVICE = 0x060009,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::DIAG)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum DIAG {
        CURRENT_ISSUES = 0x000000,
        REPORTED_ISSUES = 0x000001,
        ISSUE = 0x060000,
        ERR_CODE = 0x060001,
        ENDURE_TIME = 0x060002,
        TIME_ARISED = 0x060003,
        ERR_MSG = 0x060004,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::SGR)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum SGR {
        STATE = 0x000001,
        READY_TO_USE = 0x000002,
        HW_PROVIDER_LIST = 0x000003,
        HW_PROVIDER = 0x000004,
        NAME = 0x000005,
        AKTIV = 0x000006,
        REQ_SET_COOLDOWN_START = 0x000007,
        COOLDOWN_END = 0x000008,
        USED_POWER = 0x000009,
        REQ_USED_POWER = 0x000010,
        REQ_SET_STATE = 0x000011,
        DATA = 0x040000,
        INDEX = 0x040001,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::MBS)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum MBS {
        MODBUS_ENABLED = 0x000001,
        MODBUS_CONNECTORS = 0x000002,
        REQ_ENABLE_CONNECTOR = 0x000003,
        REQ_DISABLE_CONNECTOR = 0x000004,
        CHANGE_SETTING = 0x000005,
        REQ_CHANGE_SETTING_ERR = 0x000006,
        MODBUS_CONNECTOR_CONTAINER = 0x010002,
        MODBUS_CONNECTOR_NAME = 0x010003,
        MODBUS_CONNECTOR_ID = 0x010004,
        MODBUS_CONNECTOR_ENABLED = 0x010005,
        MODBUS_CONNECTOR_SETUP = 0x010006,
        MODBUS_SETUP_NAME = 0x010007,
        MODBUS_SETUP_TYPE = 0x010008,
        MODBUS_SETUP_VALUE = 0x010009,
        MODBUS_SETUP_VALUES = 0x01000a,
        MODBUS_SETUP_VALUE_STRING = 0x01000b,
        REQ_SET_MODBUS_ENABLED = 0x700001,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::EH)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum EH {
        UNREPORTED_ERRORS = 0x000001,
        MARKED_REPORTED = 0x000002,
        PARAM_ROW = 0x040000,
        PARAM_ROW_ID = 0x040001,
        PARAM_ROW_TIME = 0x040002,
        PARAM_ROW_CODE = 0x040003,
        PARAM_ROW_TYPE = 0x040004,
        PARAM_ROW_CLEARED = 0x040005,
        PARAM_ROW_ERR_SRC = 0x040006,
        PARAM_ROW_MSG = 0x040007,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::UPNPC)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum UPNPC {
        DEFAULT_LIST = 0x000001,
        SET_DEFAULT_LIST = 0x000002,
        SERVICE_LIST = 0x000003,
        DEFAULT_LIST_REV = 0x000006,
        SERVICE_LIST_REV = 0x000007,
        PARAM_DEVICE_ENTRY = 0x040000,
        PARAM_SERIALNO = 0x040001,
        PARAM_IP_ADR = 0x040002,
        PARAM_PORT = 0x040003,
        PARAM_NAME = 0x040004,
        PARAM_LOCATION = 0x040005,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::KNX)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum KNX {
        RSP_SET = 0x000001,
        MAC = 0x000002,
        IP = 0x000003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::EMSHB)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum EMSHB {
        HB_DATA = 0x000001,
        PARAM_VERSION = 0x040000,
        PARAM_BAT_S1 = 0x040001,
        PARAM_BAT_S2 = 0x040002,
        PARAM_BAT_S3 = 0x040003,
        PARAM_LM1 = 0x040004,
        PARAM_LM2 = 0x040005,
        PARAM_LM3 = 0x040006,
        PARAM_AC_L1 = 0x040007,
        PARAM_AC_L2 = 0x040008,
        PARAM_AC_L3 = 0x040009,
        PARAM_C_L1 = 0x040010,
        PARAM_C_L2 = 0x040011,
        PARAM_C_L3 = 0x040012,
        PARAM_SOC = 0x040013,
        PARAM_SYS_STATUS = 0x040014,
        PARAM_WB = 0x040015,
        PARAM_WB_INDEX = 0x040016,
        PARAM_WB_L1 = 0x040017,
        PARAM_WB_L2 = 0x040018,
        PARAM_WB_L3 = 0x040019,
        PARAM_WB_L1_Active = 0x040020,
        PARAM_WB_L2_Active = 0x040021,
        PARAM_WB_L3_Active = 0x040022,
        PARAM_PV_S1 = 0x040023,
        PARAM_PV_S2 = 0x040024,
        PARAM_PV_S3 = 0x040025,
        PARAM_LM = 0x040026,
        PARAM_ID = 0x040027,
        PARAM_L1 = 0x040028,
        PARAM_L2 = 0x040029,
        PARAM_L3 = 0x040030,
        PARAM_LM_ALIVE_FLAG = 0x040031,
        PARAM_WB_ALIVE_FLAG = 0x040032,
        PARAM_WB_SOLAR_L1 = 0x040033,
        PARAM_WB_SOLAR_L2 = 0x040034,
        PARAM_WB_SOLAR_L3 = 0x040035,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::MYPV)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum MYPV {
        RSP_FIND_DEVICES = 0x000003,
        RSP_REMOVE_DEVICES = 0x000006,
        RSP_INSTANT_BOOST = 0x000007,
        DEVICE = 0x000100,
        DEVICE_SERIAL = 0x000101,
        DEVICE_ENABLED = 0x000102,
        DEVICE_IP = 0x000103,
        DEVICE_TEMPERATURE_CURRENT = 0x000104,
        DEVICE_TEMPERATURE_MAXIMUM = 0x000105,
        DEVICE_POWER = 0x000106,
        DEVICE_STATUS = 0x000107,
        DEVICE_CONTROL_MODE = 0x000108,
        DEVICE_TYPE = 0x000109,
        DEVICE_TIMESPAN_IBOOST = 0x000110,
        DEVICE_BOOST_LIST = 0x000200,
        DEVICE_BOOST_ITEM = 0x000300,
        DEVICE_BOOST_START = 0x000301,
        DEVICE_BOOST_STOP = 0x000302,
        DEVICE_BOOST_TEMPERATURE = 0x000303,
        DEVICE_BOOST_ACTIVE = 0x000304,
        DEVICE_BOOST_WEEKDAYS = 0x000305,
        DEVICE_BOOST_NAME = 0x000306,
        RSP_LIST_DEVICES = 0x200004,
        RSP_WRITE_DEVICES = 0x300004,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::GPIO)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum GPIO {
        RSP_SET = 0x000001,
        RSP_GET = 0x000002,
        RSP_LIST = 0x000003,
        TUPEL = 0x060001,
        NUMBER = 0x060002,
        NAME = 0x060003,
        VALUE = 0x060004,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::FARM)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum FARM {
        CONNECTED_DEVICES = 0x000001,
        REQ_CONNECTED_DEVICES_REV = 0x000003,
        PARAM_DEVICE = 0x040000,
        PARAM_SERIALNO = 0x040001,
        PARAM_CNAME = 0x040002,
        CONNECTED_DEVICES_REV = 0x040003,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::SE)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum SE {
        SE_COUNT = 0x000001,
        SE_DATA = 0x000002,
        SET_POWER = 0x000003,
        SET_DERATE = 0x000004,
        SET_COUPLE_MODE = 0x000005,
        COUPLE_MODE = 0x000006,
        EP_RESERVE = 0x000009,
        REQ_SET_EP_RESERVE = 0x000010,
        GET_ESTIMATED_POWER_LIMIT = 0x000011,
        DESIGN_LIMIT = 0x000012,
        REQ_RESET_POWERSAVE_TIMEOUT = 0x000027,
        EMERGENCY_POWER_OVERLOAD_STATUS = 0x000028,
        EMERGENCY_POWER_RETRY = 0x000029,
        IS_EMERGENCYPOWER_POSSIBLE = 0x000030,
        PARAM_INDEX = 0x040000,
        PARAM_DCDC_STATUS = 0x040001,
        PARAM_BAT_STATUS = 0x040002,
        PARAM_CTRL_STATE = 0x040003,
        PARAM_PvPower = 0x040004,
        PARAM_PvEnergy = 0x040005,
        PARAM_BatteryPower = 0x040006,
        PARAM_BatCapacity = 0x040007,
        PARAM_Limits = 0x040008,
        PARAM_DesiredPower = 0x040009,
        PARAM_DesiredDerating = 0x040010,
        PARAM_INT = 0x040011,
        PARAM_UINT = 0x040012,
        PARAM_FLOAT = 0x040013,
        PARAM_EmergencyMode = 0x040014,
        PARAM_PVI1_STATUS = 0x040020,
        PARAM_PVI2_STATUS = 0x040021,
        PARAM_PVI3_STATUS = 0x040022,
        PARAM_EP_RESERVE = 0x040023,
        PARAM_TIME_LAST_FULL = 0x040024,
        PARAM_TIME_LAST_EMPTY = 0x040025,
        PARAM_LAST_SOC = 0x040026,
        PARAM_TIME_TO_RETRY = 0x040030,
        PARAM_NO_REMAINING_RETRY = 0x040031,
        PARAM_EP_RESERVE_W = 0x040033,
        PARAM_EP_RESERVE_MAX_W = 0x040034,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::QPI)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum QPI {
        INVERTER_COUNT = 0x000001,
        INVERTER_DATA = 0x000002,
        UPDATE_FIRMWARE = 0x000003,
        UPDATE_STATUS = 0x000004,
        INVERTER_SET_VALUES = 0x000005,
        RESET_STATE_1_COUNTER = 0x000006,
        STATE_1_COUNTER = 0x000007,
        INVERTER_SET_POWER = 0x000008,
        SET_BAT_INFO = 0x000009,
        GET_PARAM = 0x00000a,
        SET_PARAM = 0x00000b,
        ERR_LIST = 0x00000c,
        ERR_LIST_4105 = 0x00000d,
        CLEAR_ERR_HIST = 0x00000e,
        STATE_0 = 0x00000f,
        DEBUG_DATA = 0x000010,
        INVERTER_COUNT_DETAIL = 0x000011,
        SELECTED_INVERTER_DATA = 0x000012,
        ERR_HIST = 0x000013,
        HW_INFO = 0x000014,
        VERSION_RESET = 0x000015,
        SET_DESIRED_CURRENT_FOR_EP = 0x000016,
        GET_DESIRED_CURRENT_FOR_EP = 0x000017,
        SETTINGS_EP_ENABLED = 0x000030,
        SET_SETTINGS_EP_ENABLED = 0x00003a,
        SETTINGS_VDE_2510_ENABLED = 0x00003b,
        SET_SETTINGS_VDE_2510_ENABLED = 0x00003c,
        PARAM_INDEX = 0x040000,
        PARAM_U_Bat = 0x040001,
        PARAM_I_Bat = 0x040002,
        PARAM_U_AC = 0x040003,
        PARAM_I_AC = 0x040004,
        PARAM_PHI = 0x040005,
        PARAM_POWER = 0x040006,
        PARAM_APP_POWER = 0x040007,
        PARAM_REA_POWER = 0x040008,
        PARAM_FILE_NAME = 0x040009,
        PARAM_PROGRESS = 0x040010,
        PARAM_CHILD = 0x040011,
        PARAM_POWER_L1 = 0x040012,
        PARAM_POWER_L2 = 0x040013,
        PARAM_POWER_L3 = 0x040014,
        PARAM_TIME = 0x040015,
        PARAM_STATE_1_COUNT = 0x040016,
        PARAM_CMD = 0x040017,
        PARAM_STATE_1_PS = 0x400018,
        PARAM_U_MAX = 0x400019,
        PARAM_U_MIN = 0x400020,
        PARAM_I_MAX = 0x400021,
        PARAM_I_MIN = 0x400022,
        PARAM_BLOCK = 0x400023,
        PARAM_ITEM = 0x400024,
        PARAM_VALUE = 0x400025,
        PARAM_ERR = 0x400030,
        PARAM_ERR_STR = 0x400031,
        PARAM_ERR_DATE = 0x400032,
        PARAM_ERR_CODE = 0x400033,
        PARAM_ERR_F_MIN = 0x400034,
        PARAM_ERR_F_MAX = 0x400035,
        PARAM_ERR_U_MIN = 0x400036,
        PARAM_ERR_U_MAX = 0x400037,
        PARAM_ERR_U_L1_PHI = 0x400038,
        PARAM_ERR_U_L1_RMS = 0x400039,
        PARAM_ERR_U_L2_PHI = 0x400040,
        PARAM_ERR_U_L2_RMS = 0x400041,
        PARAM_ERR_U_L3_PHI = 0x400042,
        PARAM_ERR_U_L3_RMS = 0x400043,
        PARAM_ERR_U_INV_PHI = 0x400044,
        PARAM_ERR_U_INV_RMS = 0x400045,
        PARAM_ERR_I_DCL_RMS = 0x400046,
        PARAM_ERR_U_DCL = 0x400047,
        PARAM_ERR_I_LOAD_PHI = 0x400048,
        PARAM_ERR_I_LOAD_RMS = 0x400049,
        PARAM_ERR_REGULATOR_OUT = 0x400050,
        PARAM_ERR_STATE_MASHINE_E_STATE = 0x400051,
        PARAM_ERR_STATE_ACT_REG = 0x400052,
        PARAM_ERR_TMP_0 = 0x400053,
        PARAM_ERR_TMP_1 = 0x400054,
        PARAM_ERR_TMP_2 = 0x400055,
        PARAM_ERR_TMP_3 = 0x400056,
        PARAM_ERR_CO_PRO = 0x400057,
        PARAM_ERR_TREATMENT = 0x400058,
        PARAM_ERR_I_LOAD_T0 = 0x400059,
        PARAM_ERR_U_INV_TO = 0x400060,
        PARAM_U_AC_L1 = 0x400061,
        PARAM_U_AC_L2 = 0x400062,
        PARAM_U_AC_L3 = 0x400063,
        PARAM_I_AC_L1 = 0x400064,
        PARAM_I_AC_L2 = 0x400065,
        PARAM_I_AC_L3 = 0x400066,
        PARAM_APP_POWER_L1 = 0x400067,
        PARAM_APP_POWER_L2 = 0x400068,
        PARAM_APP_POWER_L3 = 0x400069,
        PARAM_REA_POWER_L1 = 0x400070,
        PARAM_REA_POWER_L2 = 0x400071,
        PARAM_REA_POWER_L3 = 0x400072,
        PARAM_STATE_0_STATE = 0x400073,
        PARAM_STATE_0_ERR_COUNT_ALL = 0x400074,
        PARAM_STATE_0_ERR_COUNT_ACTIVE = 0x400075,
        PARAM_STATE_0_OP_STATE = 0x400076,
        PARAM_CONF_STATE = 0x400079,
        PARAM_ACTIVATED = 0x400080,
        PARAM_INVERTER_GROUP = 0x400081,
        PARAM_DEBUG_SM = 0x400083,
        PARAM_DEBUG_ACTUAL_REG = 0x400084,
        PARAM_DEBUG_U_DCL = 0x400085,
        PARAM_DEBUG_I_DCL_RMS = 0x400086,
        PARAM_DEBUG_I_LOAD_RMS = 0x400087,
        PARAM_DEBUG_I_LOAD_T0 = 0x400088,
        PARAM_DEBUG_U_INV_RMS = 0x400089,
        PARAM_DEBUG_U_INV_T0 = 0x400090,
        PARAM_DEBUG_U_L1_RMS = 0x400091,
        PARAM_DEBUG_U_L2_RMS = 0x400092,
        PARAM_DEBUG_U_L3_RMS = 0x400093,
        PARAM_DEBUG_U_L1_T0 = 0x400094,
        PARAM_DEBUG_U_L2_T0 = 0x400095,
        PARAM_DEBUG_U_L3_T0 = 0x400096,
        PARAM_DEBUG_TMP_0 = 0x400097,
        PARAM_DEBUG_TMP_1 = 0x400098,
        PARAM_DEBUG_TMP_2 = 0x400099,
        PARAM_DEBUG_TMP_3 = 0x400100,
        PARAM_DEBUG_F_LINE = 0x400101,
        PARAM_DEBUG_I_DCL_AVG = 0x400102,
        PARAM_DEBUG_U_L1_PHI = 0x400103,
        PARAM_DEBUG_U_L2_PHI = 0x400104,
        PARAM_DEBUG_U_L3_PHI = 0x400105,
        PARAM_DEBUG_INV_PHI = 0x400106,
        PARAM_DEBUG_I_LOAD_PHI = 0x400107,
        PARAM_NUMBER_CHILDS = 0x400108,
        PARAM_COUNT_DETAIL = 0x400109,
        PARAM_DEBUG_U_L1_RMS_COPRO = 0x40010a,
        PARAM_DEBUG_U_L2_RMS_COPRO = 0x40010b,
        PARAM_DEBUG_U_L3_RMS_COPRO = 0x40010c,
        PARAM_DEBUG_F_LINE_COPRO = 0x40010d,
        PARAM_SW_VERSION_DATE = 0x40010e,
        PARAM_SW_VERSION = 0x40010f,
        PARAM_SW_SVN = 0x400110,
        PARAM_COPRO_SW_VERSION_DATE = 0x400111,
        PARAM_COPRO_SW_VERSION = 0x400112,
        PARAM_COPRO_SW_SVN = 0x400113,
        PARAM_HW_VERSION_MAIN = 0x400114,
        PARAM_HW_VERSION_COPRO = 0x400115,
        PARAM_HW_VERSION_PCB_CODE = 0x400116,
        PARAM_BOARD_SERIAL = 0x400117,
        PARAM_MODULE_SERIAL = 0x400118,
        PARAM_ERR_F_LINE = 0x400119,
        PARAM_ERR_OPT_STATE = 0x40011a,
        PARAM_RT_RESULT = 0x40011b,
        PARAM_DOOR_SW_OPEN = 0x40011c,
        PARAM_FAN_REQESTED = 0x40011d,
        PARAM_DEBUG_COPRO_STATE = 0x40011e,
        PARAM_ERR_I_DCL_T0 = 0x40011f,
        PARAM_ERR_COPRO_U_L1_RMS = 0x400120,
        PARAM_ERR_COPRO_U_L2_RMS = 0x400121,
        PARAM_ERR_COPRO_U_L3_RMS = 0x400122,
        PARAM_ERR_COPRO_U_INV_RMS = 0x400123,
        PARAM_ERR_COPRO_REL_STATE = 0x400124,
        PARAM_ERR_COPRO_MODE = 0x400125,
        PARAM_ERR_COPRO_F_LINE = 0x400126,
        PARAM_ERR_COPRO_PEN_TV_STATE = 0x400127,
        PARAM_ERR_COPRO_ERR_DATA = 0x400128,
        PARAM_ERR_COPRO_TRIPP_LN_OUT = 0x400129,
        PARAM_ERR_COPRO_TRIPP_LN_IN = 0x40012a,
        PARAM_ERR_COPRO_U_PEN_AVG = 0x40012b,
        PARAM_ERR_I_PRIM = 0x40012c,
        PARAM_SW_COUNTRY = 0x40012d,
        PARAM_SELECTEC_COUNTRY = 0x40012e,
        PARAM_COPRO_SW_COUNTRY = 0x40012f,
        PARAM_COPRO_SELECTEC_COUNTRY = 0x400130,
        PARAM_MAX_AC_APPARENT_POWER = 0x400131,
        PARAM_DEBUG_U_INV_RMS_COPRO = 0x400132,
        PARAM_DEBUG_I_PRIM_RMS = 0x400133,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::GAPP)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum GAPP {
        DEV_COUNT = 0x000001,
        SERIALNO = 0x000002,
        SUPPORTED_REACTIVE_POWER_FUNCTIONS = 0x000003,
        ENABLED_REACTIVE_POWER_FUNCTIONS = 0x000004,
        SET_REACTIVE_POWER_FUNCTIONS = 0x000005,
        SET_REACTIVE_POWER_FUNCTIONS_PARAMETER = 0x000006,
        REACTIVE_POWER_FUNCTIONS_PARAMETER = 0x000007,
        SUPPORTED_ACTIVE_POWER_FUNCTIONS = 0x000008,
        ENABLED_ACTIVE_POWER_FUNCTIONS_PU = 0x000009,
        SET_ACTIVE_POWER_FUNCTIONS_PU = 0x00000a,
        SET_ACTIVE_POWER_FUNCTIONS_PU_PARAMETER = 0x00000b,
        ACTIVE_POWER_FUNCTION_PU_PARAMETER = 0x00000c,
        REACTIVE_POWER_SETTINGS_EQUAL = 0x00000d,
        ACTIVE_POWER_SETTINGS_EQUAL = 0x00000e,
        PARAM_INDEX = 0x040000,
        PARAM_SERIALNO = 0x040001,
        PARAM_REACTIVE_POWER_FUNCTION = 0x040002,
        PARAM_ACTIVE_POWER_FUNCTION_PU = 0x040003,
        PARAM_GAPP_PARAMETER = 0x040004,
        PARAM_GAPP_PARAMETER_FUNCTION = 0x040005,
        PARAM_GAPP_PARAMETER_VALUE_LIST = 0x040006,
        PARAM_GAPP_PARAMETER_VALUE_LIST_ENTRY = 0x040007,
        PARAM_GAPP_PARAMETER_SCALE_FACTOR_X = 0x040008,
        PARAM_GAPP_PARAMETER_SCALE_FACTOR_Y = 0x040009,
        PARAM_GAPP_PARAMETER_VALUE_MAX = 0x040010,
        PARAM_GAPP_PARAMETER_VALUE_MIN = 0x040011,
        PARAM_GAPP_PARAMETER_VALUE = 0x040012,
        PARAM_SUPPORTED_REACTIVE_POWER_FUNCTIONS = 0x040014,
        PARAM_SUPPORTED_ACTIVE_POWER_FUNCTIONS = 0x040015,
        PARAM_SUCCESS = 0x040016,
        PARAM_GAPP_PARAMETER_HAS_Y = 0x040017,
        PARAM_GAPP_PARAMETER_COUNT_MIN = 0x040018,
        PARAM_GAPP_PARAMETER_COUNT_MAX = 0x040019,
        PARAM_GAPP_PARAMETER_COUNT_USED = 0x040020,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::EMSPR)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum EMSPR {
        RSP_GET_EQUATIONS = 0x000001,
        RSP_SET_EQUATIONS = 0x000002,
        RSP_GET_ACTIVE = 0x000003,
        RSP_GET_PINCOUNT = 0x000004,
        RSP_SET_PINCOUNT = 0x000005,
        RSP_CHANGECOUNTER = 0x000006,
        RSP_GET_INVERTERENABLING = 0x000007,
        RSP_SET_INVERTERENABLING = 0x000008,
        RSP_GET_INVERTERENABLEWAIT = 0x00000b,
        RSP_SET_INVERTERENABLEWAIT = 0x00000c,
        EQUATION = 0x060001,
        INPUT = 0x060002,
        MASK = 0x060003,
        INVALID = 0x060004,
        OUTPUT = 0x060005,
        ISACTIVE = 0x060006,
        FAILURESTATE = 0x060007,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::WBD)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum WBD {
        START_SCAN = 0x000001,
        IS_SCANNING = 0x000002,
        CREATE_WB = 0x000003,
        CANCEL_CAN = 0x000004,
        DELETE_WALL_BOX = 0x000005,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::REFU)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum REFU {
        START_SCAN = 0x000001,
        IS_SCANNING = 0x000002,
        CREATE_INV = 0x000003,
        CANCEL_CAN = 0x000004,
        DELETE_INVERTER = 0x000005,
        NO_INVERTERS = 0x000006,
        CONNECTED_DEVICES = 0x000008,
        PARAM_MAC = 0x400001,
        PARAM_IP = 0x400002,
        PARAM_ALIVE = 0x400003,
        PARAM_INDEX = 0x400004,
        PARAM_DHCP = 0x400005,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::OVP)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum OVP {
        STATUS = 0x000001,
        RESET = 0x000002,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::SERVER)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum SERVER {
        REGISTER_CONNECTION = 0x00a001,
        UNREGISTER_CONNECTION = 0x00a002,
        REQ_RSCP_CMD = 0x00a003,
        REQ_PING = 0x00a004,
        REQ_NEW_VIRTUAL_CONNECTION = 0x00a005,
        CONNECTION_ID = 0x00b001,
        AUTH_LEVEL = 0x00b002,
        STATUS = 0x00b003,
        RSCP_DATA_LEN = 0x00b004,
        RSCP_DATA = 0x00b005,
        TYPE = 0x00b006,
        HASH_CODE = 0x00b007,
        USER = 0x00b008,
        PASSWD = 0x00b009,
        IDENTIFIER = 0x00b010,
        CONNECTION_REGISTERED = 0x08a001,
        CONNECTION_UNREGISTERED = 0x08a002,
        RSCP_CMD_RESP = 0x08a003,
        PING = 0x08a004,
        GENERAL_ERROR = 0x7fffff
    }
}

macro_attr! {
    #[group!(TagGroup::GROUP)]
    #[allow(non_camel_case_types, dead_code)]
    #[repr(u32)]
    pub enum GROUP {
        CTRL_REQ_STATUS = 0x000001,
        CTRL_GROUP_ID = 0x000002,
        CTRL_READY = 0x000003,
        CTRL_P_OPERATION_POINT = 0x000004,
        CTRL_P_ACTUAL = 0x000005,
        CTRL_FORECAST_60MINUTES = 0x000006,
        CTRL_REQ_CONTROL = 0x000007,
        CTRL_P_TARGET = 0x000008,
        CTRL_ACTIVE = 0x000009,
        CTRL_AWARD = 0x00000a,
        GENERAL_ERROR = 0x7fffff
    }
}