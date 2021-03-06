use rustc_serialize::json::{self, DecoderError};

#[derive(RustcDecodable, Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub members: Option<Vec<String>>, //wth can be missing?
    pub is_member: bool,
    pub is_general: bool
}

pub fn new_from_str(json: &str) -> Result<Channel, DecoderError>{
    json::decode::<Channel>(json)
}

// Members or properties changing doesn't make it a different channel.
// impl PartialEq for Channel {
//     fn eq(&self, other: &Channel) -> bool {
//         self.id == other.id
//     }
// }

// impl Eq for Channel {}

#[cfg(test)]
mod test {
    use super::new_from_str;

    #[test]
    fn test_decode_from_str() {
        let json = "{
            \"id\": \"banana\",
            \"name\": \"banter\",
            \"members\": [\"Timon\"],
            \"is_member\": false,
            \"is_general\": false

        }";
        let channel = new_from_str(json).unwrap();
        assert_eq!(channel.id, "banana");
        assert_eq!(channel.name, "banter");
        assert_eq!(channel.members.unwrap(), vec!["Timon"]);
        assert_eq!(channel.is_member, false);
    }
}

