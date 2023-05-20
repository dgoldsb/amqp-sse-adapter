use std::convert::{TryFrom, TryInto};

pub type RoutingKey = [char; 255];

pub struct MyString(pub String);

impl TryFrom<MyString> for RoutingKey {
    type Error = &'static str;

    fn try_from(value: MyString) -> Result<Self, Self::Error> {
        let mut routing_key: RoutingKey = [' '; 255];

        let chars: Vec<char> = value.0.chars().collect();
        if chars.len() > routing_key.len() {
            return Err("String is too long to convert to RoutingKey");
        }

        for (i, &ch) in chars.iter().enumerate() {
            routing_key[i] = ch;
        }

        Ok(routing_key)
    }
}

impl TryInto<MyString> for RoutingKey {
    type Error = &'static str;

    fn try_into(self) -> Result<MyString, Self::Error> {
        let routing_key_string: String = self.iter().filter(|c| **c != ' ').collect();
        Ok(MyString(routing_key_string))
    }
}
