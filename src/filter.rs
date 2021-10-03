use crate::error::ReplayError;
use regex::Regex;
use sqs::model::Message;
use std::collections::HashSet;

pub(crate) struct MessageFilter {
    set: HashSet<String>,
}

impl MessageFilter {
    pub(crate) fn new() -> Self {
        return MessageFilter {
            set: HashSet::new(),
        };
    }
    pub fn filter(
        &mut self,
        messages: Vec<Message>,
        selector: Option<String>,
    ) -> Result<Vec<Message>, ReplayError> {
        if selector.is_none() {
            return Ok(messages);
        }
        let re = Regex::new(selector.unwrap().as_str()).unwrap();
        if re.captures_len() != 2 {
            return Err(ReplayError::BadSelector);
        }

        let empty = &String::from("");
        let r = messages
            .iter()
            .filter(|m| {
                let body = m.body.as_ref().unwrap_or(empty);
                match re.captures(body) {
                    Some(c) => {
                        let key = String::from(c.get(1).unwrap().as_str());
                        if !self.set.contains(&key) {
                            self.set.insert(key);
                            true
                        } else {
                            false
                        }
                    }
                    None => true,
                }
            })
            .cloned()
            .collect();
        println!("===>{:?}", r);
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::MessageFilter;
    use sqs::model::Message;
    use std::collections::HashSet;

    #[test]
    fn it_works() {
        let m1 = Message::builder().body("a").build();
        let m2 = Message::builder().body("b").build();
        let m3 = Message::builder().body("test a").build();
        let m4 = Message::builder().body("test b").build();
        let m4 = Message::builder().body("test c").build();
        let messages = vec![m1, m2, m3, m4];
        let mut mf = MessageFilter::new();

        let filtered = mf.filter(messages, Option::Some(String::from("(test)")));
        assert_eq!(filtered.unwrap().len(), 3);
    }
}
