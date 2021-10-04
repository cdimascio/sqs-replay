use regex::Regex;
use sqs::model::Message;
use std::collections::HashSet;


#[derive(Debug)]
pub(crate) struct MessageFilterStats {
    pub total: i32,
    pub deduped: i32,
}


pub(crate) struct MessageFilter {
    seen: HashSet<String>,
    re: Option<Regex>,
    pub results: Vec<Message>,
    pub skipped: Vec<Message>,
    pub stats: MessageFilterStats,
}

impl MessageFilter {
    pub(crate) fn new(re: Option<Regex>) -> Self {
        MessageFilter {
            seen: HashSet::new(),
            results: Vec::new(),
            skipped: Vec::new(),
            re,
            stats: MessageFilterStats {
                total: 0,
                deduped: 0,
            },
        }
    }

    pub fn add(&mut self, mut messages: Vec<Message>) {
        if self.re.is_none() {
            self.results.append(&mut messages);
        }

        let empty = &String::from("");
        let re = self.re.as_ref().unwrap();
        for m in messages {
            let body = m.body.as_ref().unwrap_or(empty);
            match re.captures(body) {
                Some(c) => {
                    let key = String::from(c.get(1).unwrap().as_str());
                    if !self.seen.contains(&key) {
                        self.seen.insert(key);
                        self.stats.total += 1;
                        self.results.push(m);
                    } else {
                        self.skipped.push(m);
                        self.stats.deduped += 1;
                    }
                }
                None => {
                    self.stats.total += 1;
                    self.results.push(m);
                },
            }
        }
    }

    pub fn clear(&mut self) {
        self.skipped.clear();
        self.results.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::MessageFilter;
    use sqs::model::Message;
    use regex::Regex;

    #[test]
    fn filter_by_selector() {
        let m1 = Message::builder().body("a").build();
        let m2 = Message::builder().body("b").build();
        let m3 = Message::builder().body("test a").build();
        let m4 = Message::builder().body("test b").build();
        let m5 = Message::builder().body("test c").build();
        let messages = vec![m1, m2, m3, m4, m5];
        let re = Regex::new("(test)").unwrap();
        let mut mf = MessageFilter::new(Some(re));

        mf.add(messages);
        assert_eq!(mf.results.len(), 3);
    }
}
