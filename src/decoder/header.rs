use crate::{
    common::header::{
        Header, IS_AUTHORITATIVE_ANSWER_BIT_MASK, IS_RECURSION_AVAILABLE_BIT_MASK,
        IS_RECUSTION_DESIRED_BIT_MASK, IS_TRUNCATED_BIT_MASK, MessageType, QueryType, ResponseCode,
    },
    decoder::DecodingError,
    utils::{concat_two_u8s, extract_next_sixteen_bits_from_buffer},
};

pub fn decode(buffer: &[u8; 12]) -> Result<Header, DecodingError> {
    let (id, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 10]

    let (next_sixteen_bits, buffer) = buffer.split_at(2); // now buffer is [u8; 8]
    let next_sixteen_bits = concat_two_u8s(next_sixteen_bits[0], next_sixteen_bits[1]);

    let qr = MessageType::from(next_sixteen_bits);
    let opcode =
        QueryType::try_from(next_sixteen_bits).map_err(DecodingError::InvalidHeaderQueryType)?;
    let response_code = ResponseCode::try_from(next_sixteen_bits)
        .map_err(DecodingError::InvalidHeaderResponseCode)?;

    let authoritative_answer = next_sixteen_bits & IS_AUTHORITATIVE_ANSWER_BIT_MASK != 0;
    let truncated = next_sixteen_bits & IS_TRUNCATED_BIT_MASK != 0;
    let recursion_desired = next_sixteen_bits & IS_RECUSTION_DESIRED_BIT_MASK != 0;
    let recursion_available = next_sixteen_bits & IS_RECURSION_AVAILABLE_BIT_MASK != 0;

    let (questions_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 6]
    let (answers_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 4]
    let (authority_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 2]
    let (additional_count, buffer) = extract_next_sixteen_bits_from_buffer(buffer); // now buffer is [u8; 0]

    // buffer should be empty since we've read 12 u8 => 96 bits == header size
    assert!(buffer.is_empty());

    Ok(Header {
        id,
        qr,
        opcode,
        authoritative_answer,
        truncated,
        recursion_desired,
        recursion_available,
        reserved: false,
        response_code,
        questions_count,
        answers_count,
        authority_count,
        additional_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUFFER: &[u8; 12] = &[226, 44, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0];

    #[test]
    fn test() {
        let header = decode(BUFFER).unwrap();

        let expected_header = Header {
            id: 57900,
            qr: MessageType::Query,
            opcode: QueryType::Standard,
            authoritative_answer: false,
            truncated: false,
            recursion_desired: true,
            recursion_available: false,
            reserved: false,
            response_code: ResponseCode::NoError,
            questions_count: 1,
            answers_count: 0,
            authority_count: 0,
            additional_count: 0,
        };

        assert!(header == expected_header);
    }
}
