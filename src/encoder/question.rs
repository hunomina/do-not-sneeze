use crate::{common::question::Question, utils::push_u16_to_u8_vec};

use super::domain_name::encode as encode_domain_name;

pub fn encode(question: Question) -> Vec<u8> {
    let mut r = vec![];

    r.extend(encode_domain_name(question.name));

    push_u16_to_u8_vec(&mut r, question.type_.into());
    push_u16_to_u8_vec(&mut r, question.class.into());

    r
}

#[cfg(test)]
mod tests {
    use crate::common::{domain_name::DomainName, question::Type};

    use super::*;

    #[test]
    fn test_domain_name_from_buffer() {
        let question = Question {
            name: DomainName::from("abc.def.gh"),
            type_: Type::ALL,
            class: crate::common::question::Class::IN,
        };

        assert_eq!(
            [
                3, b'a', b'b', b'c', 3, b'd', b'e', b'f', 2, b'g', b'h', 0, // domain name
                0, 255, // type
                0, 1, // class
            ],
            encode(question).as_slice()
        );
    }
}
