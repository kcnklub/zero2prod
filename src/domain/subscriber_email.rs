use validator::validate_email;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(input: String) -> Result<SubscriberEmail, String> {
        if validate_email(&input) {
            Ok(Self(input))
        } else {
            Err(format!("{input} is not a valid email!"))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::{faker::internet::en::SafeEmail, Fake};

    use super::SubscriberEmail;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let input = "".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let input = "testtest.com".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let input = "@test.com".to_string();
        assert_err!(SubscriberEmail::parse(input));
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
