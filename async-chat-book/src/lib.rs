use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod utils;

// TODO(elsuizo:2021-11-12): no podemos reemplazar a los types Post y Message por un type solo que
// sea mas generico y que tenga un builder???

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromClient {
    Join {
        group_name: Arc<String>,
    },
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}
//-------------------------------------------------------------------------
//                        testing
//-------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    // testeamos que la serializacion funciona bien en ambos sentidos
    #[test]
    fn test_from_client_json() {
        use std::sync::Arc;

        let from_client = FromClient::Post {
            group_name: Arc::new("Dogs".to_string()),
            message: Arc::new("Samoyeds rock!!!".to_string()),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"Post":{"group_name":"Dogs","message":"Samoyeds rock!!!"}}"#
        );

        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }
}
