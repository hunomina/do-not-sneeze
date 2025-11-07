use std::fmt::Display;

pub type Label = String;

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct DomainName {
    pub labels: Vec<Label>,
}

impl Display for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.labels.join("."))
    }
}

impl std::fmt::Debug for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl From<&str> for DomainName {
    fn from(s: &str) -> Self {
        let mut labels: Vec<String> = s.split('.').map(|s| s.into()).collect();
        if !labels.last().unwrap().is_empty() {
            labels.push(Label::new());
        }
        DomainName { labels }
    }
}
