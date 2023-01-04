use crate::utils::concat_two_u8s;

const ALIAS_FLAG: u8 = 0b11000000;

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

pub type Label = String;

impl DomainName {
    pub fn from_buffer<'a>(bytes: &'a [u8], source: &'a [u8]) -> (Self, &'a [u8]) {
        let mut buffer = bytes.into_iter();
        let mut labels = vec![];

        while let Some(x) = buffer.next() {
            if x == &0 {
                labels.push(Label::new()); // this allows us to add the ending . in the name
                break;
            }

            if x & ALIAS_FLAG == ALIAS_FLAG {
                // take next byte, concat with current and counter detection flag on result to get 14th last bits
                let alias_position_in_source =
                    concat_two_u8s(*x, *buffer.next().unwrap()) & !((ALIAS_FLAG as u16) << 8);

                // read the alias from the source buffer and add its labels to the current label list
                labels.append(
                    &mut DomainName::from_source(source, alias_position_in_source as usize).labels,
                );
                return (DomainName { labels }, buffer.as_slice());
            }

            let label = (0..*x).map(|_| *buffer.next().unwrap() as char).collect();
            labels.push(label);
        }
        (DomainName { labels }, buffer.as_slice())
    }

    fn from_source(source: &[u8], position: usize) -> Self {
        let trimmed_source = &<&[u8]>::clone(&source).to_vec()[position..];
        DomainName::from_buffer(trimmed_source, source).0
    }
}

impl From<&str> for DomainName {
    fn from(s: &str) -> Self {
        let mut labels: Vec<String> = s.split('.').into_iter().map(|s| s.into()).collect();
        if labels.last().unwrap().ne("") {
            labels.push(Label::new());
        }
        DomainName { labels }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_name_from_buffer() {
        const BUFFER: &[u8] = &[3, b'a', b'b', b'c', 3, b'd', b'e', b'f', 2, b'g', b'h', 0];
        let (domain_name, rest) = DomainName::from_buffer(BUFFER, BUFFER);
        assert_eq!("abc.def.gh.", domain_name.to_string(),);
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

        let (domain_name, rest) = DomainName::from_buffer(BUFFER, BUFFER);

        assert_eq!("wpad.numericable.fr.", domain_name.to_string());
        assert_eq!(
            [
                0, 1, // question type
                0, 1, // question class
            ],
            rest
        );
    }

    #[test]
    fn test_read_alias() {
        let buffer_containing_alias: &[u8] = &[
            226, 44, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            192, 12, // pointer to question name
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        let (original_name, _) = DomainName::from_buffer(
            &buffer_containing_alias[12..], // 12 is the position at which the original name is defined in the buffer
            buffer_containing_alias,
        );

        let (aliased_name, _) = DomainName::from_buffer(
            &buffer_containing_alias[28..], // 28 is the position at which the alias to the original name is
            buffer_containing_alias,
        );

        assert_eq!(original_name, aliased_name);
        assert_eq!(DomainName::from("google.com"), aliased_name);
    }

    #[test]
    fn test_read_alias_with_odd_lenght_prefix() {
        let buffer_containing_alias: &[u8] = &[
            226, 44, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            3, b'w', b'w', b'w', // alias prefix
            192, 12, // pointer to question name
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        let (aliased_name, _) = DomainName::from_buffer(
            &buffer_containing_alias[28..], // 28 is the position at which the alias to the original name is
            buffer_containing_alias,
        );

        assert_eq!(DomainName::from("www.google.com"), aliased_name);
    }

    #[test]
    fn test_read_alias_with_even_lenght_prefix() {
        let buffer_containing_alias: &[u8] = &[
            226, 44, 129, 128, 0, 1, 0, 1, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            4, b'i', b'n', b'f', b'o', // alias prefix
            192, 12, // pointer to question name
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        let (aliased_name, _) = DomainName::from_buffer(
            &buffer_containing_alias[28..], // 28 is the position at which the alias to the original name is
            buffer_containing_alias,
        );

        assert_eq!(DomainName::from("info.google.com"), aliased_name);
    }

    #[test]
    fn test_read_double_alias() {
        let buffer_containing_alias: &[u8] = &[
            226, 44, 129, 128, 0, 1, 0, 2, 0, 0, 0, 0, // header
            6, b'g', b'o', b'o', b'g', b'l', b'e', 3, b'c', b'o', b'm',
            0, // question domain name
            0, 1, // question type
            0, 1, // question class
            4, b'i', b'n', b'f', b'o', // first alias prefix
            192, 12, // pointer to question name
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
            3, b'w', b'w', b'w', // second alias prefix
            192, 28, // pointer to first alias
            0, 1, // response type
            0, 1, // response class
            0, 0, 0, 244, //ttl
            0, 4, // response data length
            216, 58, 214, 174, // response data
        ];

        let (first_alias, _) = DomainName::from_buffer(
            &buffer_containing_alias[28..], // 28 is the position at which the alias to the original name is
            buffer_containing_alias,
        );

        let (second_alias, _) = DomainName::from_buffer(
            &buffer_containing_alias[49..], // 28 is the position at which the alias to the original name is
            buffer_containing_alias,
        );

        assert_eq!(DomainName::from("info.google.com"), first_alias);
        assert_eq!(DomainName::from("www.info.google.com"), second_alias);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(
            String::from("abc.def.gh."),
            DomainName {
                labels: vec![
                    Label::from("abc"),
                    Label::from("def"),
                    Label::from("gh"),
                    Label::new()
                ]
            }
            .to_string(),
        )
    }

    #[test]
    fn test_from_string_with_ending_dot() {
        assert_eq!(
            DomainName {
                labels: vec![
                    Label::from("abc"),
                    Label::from("def"),
                    Label::from("gh"),
                    Label::new()
                ]
            },
            DomainName::from("abc.def.gh.")
        )
    }

    #[test]
    fn test_from_string_without_ending_dot() {
        assert_eq!(
            DomainName {
                labels: vec![
                    Label::from("abc"),
                    Label::from("def"),
                    Label::from("gh"),
                    Label::new()
                ]
            },
            DomainName::from("abc.def.gh")
        )
    }
}
