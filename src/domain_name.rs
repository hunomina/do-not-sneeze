#[derive(PartialEq)]
pub struct DomainName {
    pub labels: Vec<Label>,
}

impl DomainName {
    pub fn to_string(&self) -> String {
        self.labels.join(".")
    }
}

impl std::fmt::Debug for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

type Label = String;

impl DomainName {
    pub fn from_buffer(bytes: &[u8]) -> (Self, &[u8]) {
        let mut buffer = bytes.into_iter();
        let mut labels = vec![];
        while let Some(x) = buffer.next() {
            if x == &0 {
                labels.push(Label::new()); // this allows us to add the ending . in the name
                break;
            }

            let label = (0..*x).map(|_| *buffer.next().unwrap() as char).collect();
            labels.push(label);
        }
        (DomainName { labels }, buffer.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_name_from_buffer() {
        let (domain_name, rest) =
            DomainName::from_buffer(&[3, b'a', b'b', b'c', 3, b'd', b'e', b'f', 2, b'g', b'h', 0]);
        assert_eq!(
            domain_name.labels,
            vec![
                Label::from("abc".to_string()),
                Label::from("def".to_string()),
                Label::from("gh".to_string()),
                Label::new()
            ]
        );
        assert_eq!(rest, &[]);
    }

    #[test]
    fn test_parse_and_return_rest() {
        const BUFFER: &[u8] = &[
            4, b'w', b'p', b'a', b'd', 11, b'n', b'u', b'm', b'e', b'r', b'i', b'c', b'a', b'b',
            b'l', b'e', 2, b'f', b'r', 0, // question domain name
            0, 1, // question type
            0, 1, // question class
        ];

        let (domain_name, rest) = DomainName::from_buffer(BUFFER);

        assert_eq!("wpad.numericable.fr.", domain_name.to_string());
        assert_eq!(
            [
                0, 1, // question type
                0, 1, // question class
            ],
            rest
        );
    }
}
