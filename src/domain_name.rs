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
                labels.push(Label::new());
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
        let (domain_name, bytes) = DomainName::from_buffer(&[
            0x03, b'a', b'b', b'c', 0x03, b'd', b'e', b'f', 0x02, b'g', b'h', 0,
        ]);
        assert_eq!(
            domain_name.labels,
            vec![
                Label::from("abc".to_string()),
                Label::from("def".to_string()),
                Label::from("gh".to_string()),
                Label::new()
            ]
        );
        assert_eq!(bytes, &[]);
    }

    #[test]
    fn test_parse_and_return_rest() {
        const BUFFER: &[u8] = &[
            4, 119, 112, 97, 100, 11, 110, 117, 109, 101, 114, 105, 99, 97, 98, 108, 101, 2, 102,
            114, 0, // question
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
