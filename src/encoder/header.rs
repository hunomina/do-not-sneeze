use crate::{
    common::header::{
        Header, IS_AUTHORITATIVE_ANSWER_BIT_MASK, IS_RECURSION_AVAILABLE_BIT_MASK,
        IS_RECUSTION_DESIRED_BIT_MASK, IS_TRUNCATED_BIT_MASK, MESSAGE_TYPE_BIT_MASK, MessageType,
        QUERY_TYPE_INVERSE_BIT_VALUE, QUERY_TYPE_SERVER_STATUS_REQUEST_BIT_VALUE,
        QUERY_TYPE_STANDARD_BIT_VALUE, QueryType,
    },
    utils::push_u16_to_u8_vec,
};

pub fn encode(header: Header) -> Vec<u8> {
    let mut e = vec![];

    push_u16_to_u8_vec(&mut e, header.id);

    let mut second_line = 0u16;

    if header.qr == MessageType::Response {
        second_line |= MESSAGE_TYPE_BIT_MASK;
    }

    second_line |= match header.opcode {
        QueryType::Standard => QUERY_TYPE_STANDARD_BIT_VALUE,
        QueryType::Inverse => QUERY_TYPE_INVERSE_BIT_VALUE,
        QueryType::ServerStatusRequest => QUERY_TYPE_SERVER_STATUS_REQUEST_BIT_VALUE,
    };

    if header.authoritative_answer {
        second_line |= IS_AUTHORITATIVE_ANSWER_BIT_MASK
    }

    if header.truncated {
        second_line |= IS_TRUNCATED_BIT_MASK;
    }

    if header.recursion_desired {
        second_line |= IS_RECUSTION_DESIRED_BIT_MASK;
    }

    if header.recursion_available {
        second_line |= IS_RECURSION_AVAILABLE_BIT_MASK;
    }

    second_line |= header.response_code.value();

    push_u16_to_u8_vec(&mut e, second_line);
    push_u16_to_u8_vec(&mut e, header.questions_count);
    push_u16_to_u8_vec(&mut e, header.answers_count);
    push_u16_to_u8_vec(&mut e, header.authority_count);
    push_u16_to_u8_vec(&mut e, header.additional_count);

    e
}

#[cfg(test)]
mod tests {
    use crate::common::header::ResponseCode;

    use super::*;

    #[test]
    fn encode_header() {
        let header = Header {
            id: 57900,
            qr: MessageType::Query,
            opcode: QueryType::Standard,
            authoritative_answer: false,
            truncated: false,
            recursion_desired: true,
            recursion_available: true,
            reserved: false,
            response_code: ResponseCode::FormatError,
            questions_count: 1,
            answers_count: 0,
            authority_count: 12,
            additional_count: 0,
        };

        let encoded = encode(header);

        assert_eq!(
            [226, 44, 1, 129, 0, 1, 0, 0, 0, 12, 0, 0],
            encoded.as_slice()
        );
    }
}
