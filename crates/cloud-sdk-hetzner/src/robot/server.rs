//! Source-locked Robot server list, get, and rename operations.

mod identity;
mod request;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod duplicates;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod protected;
#[cfg(feature = "serde")]
mod protected_parse;

pub use identity::{RobotServerNumber, RobotServerNumberError};
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_SERVER_ADDRESSES, MAX_ROBOT_SERVER_LIST_ITEMS, RobotServer, RobotServerList,
    RobotServerSummary,
};
#[cfg(feature = "serde")]
pub use protected::{
    ProtectedIpAddr, RobotServerCapabilities, RobotServerDate, RobotServerStatus,
    RobotServerSubnet, RobotStorageBoxNumber,
};
pub use request::{
    MAX_ROBOT_SERVER_NAME_BYTES, RobotServerGetRequest, RobotServerListRequest, RobotServerName,
    RobotServerRequestError, RobotServerUpdateIntent, RobotServerUpdateRequest,
};

#[cfg(feature = "serde")]
pub use decode::{RobotServerDecodeError, decode_robot_server, decode_robot_server_list};

#[cfg(all(test, feature = "serde"))]
mod tests;
