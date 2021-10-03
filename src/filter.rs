use sqs::model::Message;
use std::collections::HashSet;

struct MessageFilter {
    set: HashSet<String>,
}

impl MessageFilter {
    fn new() -> Self {
        return MessageFilter {
            set: HashSet::new(),
        };
    }
    fn filter(&mut self, messages: Vec<Message>, selector: Option<String>) -> Vec<Message> {
        if selector.is_none() {
            return messages;
        }
        let key = selector.as_ref().unwrap();
        let empty = &String::from("");
        let r = messages
            .iter()
            .filter(|m| {
                let body_ref = m.body.as_ref().unwrap_or(empty);
                let keep = !body_ref.contains(key);
                println!("--->{:?} {}", m.body, key);
                self.set.insert(key.clone());
                keep
            })
            .cloned()
            .collect();
        r
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
        let m3 = Message::builder().body("test").build();
        let m4 = Message::builder().body("test2").build();
        let messages = vec![m1, m2, m3, m4];
        let mut mf = MessageFilter::new();

        let filtered = mf.filter(messages, Option::Some(String::from("test")));
        println!("{:?}", filtered);
        assert_eq!(filtered.len(), 2);
    }
}

// fn filter_messages(
//     &self,
//     results: Vec<Result<Vec<Message>>>,
//     selector: Option<String>,
// ) -> Result<Vec<Message>> {
//     let mut seen = HashSet::new();
//     let empty = &String::from("");
//     if selector.is_some() {
//         // if selector is present, use key to proactively delete previously played messages
//         let mut filtered_messages = Vec::new();
//         let sel = selector.unwrap().clone();
//         let mut skipped_count = 0;
//         for rm in results {
//             match rm {
//                 Ok(messages) => {
//                     for m in messages {
//                         let body = m.body.as_ref().unwrap_or(empty);
//                         let re = Regex::new(sel.as_str()).unwrap();
//                         match re.captures(body) {
//                             Some(c) => {
//                                 let key = String::from(c.get(1).unwrap().as_str());
//                                 if !seen.contains(&key) {
//                                     seen.insert(key);
//                                     filtered_messages.push(m);
//                                 } else {
//                                     skipped_count += 1;
//                                 }
//                             },
//                             None => {}
//                         };
//                     }
//                 }
//                 Err(e) => println!("[error] {}", e),
//             }
//         }
//         Ok(filtered_messages)
//     } else {
//         let mut filtered_messages = Vec::new();
//         for r in results {
//             match r {
//                 Ok(messages) => {
//                     for m in messages {
//                         filtered_messages.push(m)
//                     }
//                 }
//                 Err(e) => println!("[error] {}", e),
//             }
//         }
//         Ok(filtered_messages)
//     }
// }
// }
