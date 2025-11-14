use crate::{
    common::question::{Class, Question, Type},
    decoder::DecodingError,
    utils::extract_next_sixteen_bits_from_buffer,
};

use super::domain_name::decode as decode_domain_name;

pub fn decode<'a>(
    buffer: &'a [u8],
    source: &'a [u8],
) -> Result<(Question, &'a [u8]), DecodingError> {
    let (name, buffer) = decode_domain_name(buffer, source);
    let (type_bytes, buffer) = extract_next_sixteen_bits_from_buffer(buffer);
    let (class_bytes, buffer) = extract_next_sixteen_bits_from_buffer(buffer);

    Ok((
        Question {
            name,
            type_: Type::try_from(type_bytes).map_err(DecodingError::InvalidQuestionType)?,
            class: Class::try_from(class_bytes).map_err(DecodingError::InvalidQuestionClass)?,
        },
        buffer,
    ))
}

#[cfg(test)]
mod tests {
    use crate::common::{domain_name::DomainName, resource_record::Type as RRType};

    use super::*;

    #[test]
    fn test_question_from_buffer() {
        let buffer = &[
            4, 119, 112, 97, 100, 11, 110, 117, 109, 101, 114, 105, 99, 97, 98, 108, 101, 2, 102,
            114, 0, // domain name
            0, 1, // question type
            0, 1, // question class
        ];
        let (question, rest) = decode(buffer, buffer).unwrap();
        let expected_question = Question {
            name: DomainName {
                labels: vec!["wpad".into(), "numericable".into(), "fr".into(), "".into()],
            },
            type_: Type::RRType(RRType::A),
            class: Class::IN,
        };

        assert_eq!(expected_question, question);
        assert_eq!(rest, &[]);
    }
}
