use crate::common::domain_name::DomainName;

pub fn encode(domain_name: DomainName) -> Vec<u8> {
    let mut e = vec![];

    for label in domain_name.labels.iter() {
        e.push(label.len() as u8);
        for c in label.chars() {
            e.push(c as u8);
        }
    }

    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_domain_name() {
        let origin_domain = DomainName::from("google.com");

        let encoded_domain = encode(origin_domain.clone());
        let slice = encoded_domain.as_slice();

        assert!(
            slice
                == [
                    6, // 6 letters for google
                    b'g', b'o', b'o', b'g', b'l', b'e', // google
                    3,    // 3 letters for com
                    b'c', b'o', b'm', // com
                    0
                ]
        );
    }
}
