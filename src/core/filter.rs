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
                    // use last match capture group as the key
                    let mut last_match = "";
                    for sc in c.iter() {
                        if sc.is_some() {
                            last_match = sc.unwrap().as_str();
                        }
                    }
                    let key = String::from(last_match);
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
                }
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
    use crate::core::filter::MessageFilter;
    use regex::Regex;
    use sqs::model::Message;

    #[test]
    fn multiple_capture_groups() {
        let s1 = r#"{"Message" : "{\"payload\":{\"personId\":\"amzn1.actor.person.oid.A1EMM9UP3PZP0I\",\\"createdAt\":\"2022-02-23T18:22:22.86119594Z\",\"eventType\":\"insurance_profile_added\",\"requestId\":\"7722675b-9e5f-4293-9956-7953e3f159f1\",\"clientId\":[\"arn:aws:sts::961062956876:assumed-role/apex-webapp-test/ClientSession\"],\"operationId\":\"setInsuranceOnProfile\""#;
        let s2 = r#"{"Message" : "{\"payload\":{\"personId\":\"amzn1.actor.person.oid.AY3TVJTLVCF6X\",\\"createdAt\":\"2022-02-23T18:22:22.86119594Z\",\"eventType\":\"insurance_profile_added\",\"requestId\":\"7722675b-9e5f-4293-9956-7953e3f159f1\",\"clientId\":[\"arn:aws:sts::961062956876:assumed-role/apex-webapp-test/ClientSession\"],\"operationId\":\"setInsuranceOnProfile\""#;
        let s3 = r#"{"eventType":"s3_evaluation","runId":"backfill/2022-01-21.csv","row":16403,"personId":"amzn1.actor.person.oid.AY3TVJTLVCF6X","bin":"004915","pcn":"","group":""}"#;
        let s4 = r#"{"eventType":"s3_evaluation","runId":"backfill/2022-01-21.csv","row":16403,"personId":"amzn1.actor.person.oid.A1EMM9UP3PZP0I","bin":"004915","pcn":"","group":""}"#;
        let s5 = r#"something that won't match"#;

        let m1 = Message::builder().body(s1).build();
        let m2 = Message::builder().body(s2).build();
        let m3 = Message::builder().body(s3).build();
        let m4 = Message::builder().body(s4).build();
        let m5 = Message::builder().body(s5).build();

        let messages = vec![m1, m2, m3, m4, m5];
        // matches escaped double quoted person id or unescaped double quoted person id
        let re = Regex::new(r#"(personId\\":\\"(.*?)\\"|personId":"(.*?)")"#).unwrap();

        let mut mf = MessageFilter::new(Some(re));
        mf.add(messages);
        assert_eq!(mf.results.len(), 3);
    }

    #[test]
    fn one_capture_group() {
        let s1 = r#"{"Message" : "{\"payload\":{\"personId\":\"amzn1.actor.person.oid.A1EMM9UP3PZP0I\",\\"createdAt\":\"2022-02-23T18:22:22.86119594Z\",\"eventType\":\"insurance_profile_added\",\"requestId\":\"7722675b-9e5f-4293-9956-7953e3f159f1\",\"clientId\":[\"arn:aws:sts::961062956876:assumed-role/apex-webapp-test/ClientSession\"],\"operationId\":\"setInsuranceOnProfile\""#;
        let s2 = r#"{"Message" : "{\"payload\":{\"personId\":\"amzn1.actor.person.oid.A1EMM9UP3PZP0I\",\\"createdAt\":\"2022-02-23T18:22:22.86119594Z\",\"eventType\":\"insurance_profile_added\",\"requestId\":\"7722675b-9e5f-4293-9956-7953e3f159f1\",\"clientId\":[\"arn:aws:sts::961062956876:assumed-role/apex-webapp-test/ClientSession\"],\"operationId\":\"setInsuranceOnProfile\""#;
        let s3 = r#"{"Message" : "{\"payload\":{\"personId\":\"amzn1.actor.person.oid.AY3TVJTLVCF6X\",\\"createdAt\":\"2022-02-23T18:22:22.86119594Z\",\"eventType\":\"insurance_profile_added\",\"requestId\":\"7722675b-9e5f-4293-9956-7953e3f159f1\",\"clientId\":[\"arn:aws:sts::961062956876:assumed-role/apex-webapp-test/ClientSession\"],\"operationId\":\"setInsuranceOnProfile\""#;
        let s4 = r#"{"eventType":"s3_evaluation","runId":"backfill/2022-01-21.csv","row":16403,"personId":"amzn1.actor.person.oid.AY3TVJTLVCF6X","bin":"004915","pcn":"","group":""}"#;
        let s5 = r#"{"eventType":"s3_evaluation","runId":"backfill/2022-01-21.csv","row":16403,"personId":"amzn1.actor.person.oid.A1EMM9UP3PZP0I","bin":"004915","pcn":"","group":""}"#;

        let m1 = Message::builder().body(s1).build();
        let m2 = Message::builder().body(s2).build();
        let m3 = Message::builder().body(s3).build();
        let m4 = Message::builder().body(s4).build();
        let m5 = Message::builder().body(s5).build();

        let messages = vec![m1, m2, m3, m4, m5];
        // matches escaped double quoted person id e.g. \"personId\": \"amzn1.actor.person.oid.A1EMM9UP3PZP0I\"
        let re = Regex::new(r#"personId\\":\\"(.*?)\\""#).unwrap();

        let mut mf = MessageFilter::new(Some(re));
        mf.add(messages);
        assert_eq!(mf.results.len(), 4);
    }

    #[test]
    fn one_capture_match() {
        let m1 = Message::builder().body("a").build();
        let m2 = Message::builder().body("b").build();
        let m3 = Message::builder().body("test a").build();
        let m4 = Message::builder().body("test b").build();
        let m5 = Message::builder().body("test c").build();
        let m6 = Message::builder().body("bbb c").build();
        let m7 = Message::builder().body("a bbb c").build();

        let messages = vec![m1, m2, m3, m4, m5, m6, m7];
        let re = Regex::new(r#"(test|bbb)"#).unwrap();

        let mut mf = MessageFilter::new(Some(re));
        mf.add(messages);
        assert_eq!(mf.results.len(), 4);
    }

    #[test]
    fn one_capture_match_2() {
        let m1 = Message::builder().body("a").build();
        let m2 = Message::builder().body("b").build();
        let m3 = Message::builder().body("test a").build();
        let m4 = Message::builder().body("test b").build();
        let m5 = Message::builder().body("test c").build();
        let m6 = Message::builder().body("bbb c").build();
        let m7 = Message::builder().body("a bbb c").build();

        let messages = vec![m1, m2, m3, m4, m5, m6, m7];
        let re = Regex::new(r#"(test)"#).unwrap();

        let mut mf = MessageFilter::new(Some(re));
        mf.add(messages);
        assert_eq!(mf.results.len(), 5);
    }
}
